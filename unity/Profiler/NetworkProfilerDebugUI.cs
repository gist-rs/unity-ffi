using UnityEngine;
using Unity.Profiler;
using System;
using System.Text;

namespace Unity.Profiler
{
    /// <summary>
    /// Debug UI for Network Profiler waterfall visualization
    /// Provides on-screen display of request metrics and waterfall data
    /// </summary>
    [AddComponentMenu("Profiler/Network Profiler Debug UI")]
    public class NetworkProfilerDebugUI : MonoBehaviour
    {
        #region Settings

        [Header("UI Settings")]
        [Tooltip("Width of the debug window")]
        [SerializeField]
        private int windowWidth = 600;

        [Tooltip("Height of the debug window")]
        [SerializeField]
        private int windowHeight = 400;

        [Tooltip("Show UI on startup")]
        [SerializeField]
        private bool showOnStartup = true;

        [Tooltip("Auto-refresh interval in seconds")]
        [SerializeField]
        private float refreshInterval = 0.5f;

        [Header("Display Settings")]
        [Tooltip("Show request UUIDs")]
        [SerializeField]
        private bool showUuids = true;

        [Tooltip("Maximum number of requests to display")]
        [SerializeField]
        private int maxRequestsToDisplay = 50;

        [Tooltip("Show statistics")]
        [SerializeField]
        private bool showStatistics = true;

        #endregion

        #region Fields

        private bool showUI;
        private float lastRefreshTime;
        private WaterfallEntry[] cachedEntries;
        private Vector2 scrollPosition;
        private string selectedContext = "Total";

        // Cached GUI styles for runtime performance
        private GUIStyle toolbarStyle;
        private GUIStyle labelStyle;
        private GUIStyle boldLabelStyle;
        private GUIStyle helpBoxStyle;

        #endregion

        #region Unity Lifecycle

        /// <summary>
        /// Initialize the debug UI
        /// </summary>
        private void Awake()
        {
            showUI = showOnStartup;
            cachedEntries = Array.Empty<WaterfallEntry>();
            InitializeStyles();
        }

        /// <summary>
        /// Render the debug UI
        /// </summary>
        private void OnGUI()
        {
            // Draw toggle button in top-right corner
            Rect toggleRect = new Rect(Screen.width - 120, 10, 110, 30);
            bool newShowUI = GUI.Toggle(toggleRect, showUI, "Profiler");
            if (newShowUI != showUI)
            {
                showUI = newShowUI;
                if (showUI)
                {
                    RefreshWaterfallData();
                }
            }

            if (!showUI) return;

            // Draw main window
            Rect windowRect = new Rect(10, 50, windowWidth, windowHeight);
            windowRect = GUILayout.Window(12345, windowRect, DrawWindow, "Network Profiler");
        }

        /// <summary>
        /// Auto-refresh data periodically
        /// </summary>
        private void Update()
        {
            if (!showUI) return;

            if (Time.realtimeSinceStartup - lastRefreshTime > refreshInterval)
            {
                RefreshWaterfallData();
                lastRefreshTime = Time.realtimeSinceStartup;
            }
        }

        #endregion

        #region UI Drawing

        /// <summary>
        /// Draw the main debug window
        /// </summary>
        private void DrawWindow(int windowID)
        {
            GUILayout.BeginVertical();

            // Header section
            DrawHeader();

            GUILayout.Space(10);

            // Statistics section
            if (showStatistics)
            {
                DrawStatistics();
                GUILayout.Space(10);
            }

            // Context selection
            DrawContextSelector();
            GUILayout.Space(5);

            // Request list
            DrawRequestList();

            GUILayout.EndVertical();

            // Allow window to be dragged
            GUI.DragWindow();
        }

        /// <summary>
        /// Draw the header section
        /// </summary>
        private void DrawHeader()
        {
            GUILayout.BeginHorizontal();

            // Refresh button
            if (GUILayout.Button("Refresh", GUILayout.Width(80)))
            {
                RefreshWaterfallData();
            }

            // Clear button
            if (GUILayout.Button("Clear", GUILayout.Width(80)))
            {
                cachedEntries = Array.Empty<WaterfallEntry>();
            }

            // Auto-refresh toggle
            bool autoRefresh = refreshInterval > 0;
            bool newAutoRefresh = GUILayout.Toggle(autoRefresh, "Auto-refresh");
            if (newAutoRefresh != autoRefresh)
            {
                refreshInterval = newAutoRefresh ? 0.5f : 0;
            }

            GUILayout.FlexibleSpace();

            // Close button
            if (GUILayout.Button("Close", GUILayout.Width(60)))
            {
                showUI = false;
            }

            GUILayout.EndHorizontal();
        }

        /// <summary>
        /// Draw statistics section
        /// </summary>
        private void DrawStatistics()
        {
            if (cachedEntries.Length == 0)
            {
                GUILayout.Label("No requests tracked", boldLabelStyle);
                return;
            }

            // Calculate statistics
            int totalRequests = cachedEntries.Length;
            int completedCount = 0;
            int failedCount = 0;
            int pendingCount = 0;
            float totalDuration = 0;
            float maxDuration = 0;
            float minDuration = float.MaxValue;

            foreach (var entry in cachedEntries)
            {
                switch (entry.status)
                {
                    case RequestStatus.Completed:
                        completedCount++;
                        break;
                    case RequestStatus.Failed:
                        failedCount++;
                        break;
                    case RequestStatus.Pending:
                        pendingCount++;
                        break;
                }

                if (entry.status == RequestStatus.Completed || entry.status == RequestStatus.Failed)
                {
                    totalDuration += entry.total_duration_ms;
                    maxDuration = Mathf.Max(maxDuration, entry.total_duration_ms);
                    minDuration = Mathf.Min(minDuration, entry.total_duration_ms);
                }
            }

            float avgDuration = completedCount > 0 ? totalDuration / completedCount : 0;

            // Display statistics
            GUILayout.BeginVertical(helpBoxStyle);

            GUILayout.BeginHorizontal();
            GUILayout.Label($"Total Requests: {totalRequests}", boldLabelStyle);
            GUILayout.Label($"Completed: {completedCount}", GetStyleForStatus(RequestStatus.Completed));
            GUILayout.Label($"Failed: {failedCount}", GetStyleForStatus(RequestStatus.Failed));
            GUILayout.Label($"Pending: {pendingCount}", GetStyleForStatus(RequestStatus.Pending));
            GUILayout.EndHorizontal();

            GUILayout.BeginHorizontal();
            GUILayout.Label($"Avg Duration: {avgDuration:F2} ms");
            GUILayout.Label($"Max Duration: {maxDuration:F2} ms");
            GUILayout.Label($"Min Duration: {(minDuration < float.MaxValue ? minDuration.ToString("F2") : "N / A")} ms");
            GUILayout.EndHorizontal();

            GUILayout.EndVertical();
        }

        /// <summary>
        /// Draw context selector
        /// </summary>
        private void DrawContextSelector()
        {
            GUILayout.BeginHorizontal();
            GUILayout.Label("Context:", GUILayout.Width(60));

            string[] contexts = { "Unity", "Rust", "Total" };
            int selectedIndex = Array.IndexOf(contexts, selectedContext);

            if (selectedIndex == -1)
            {
                selectedIndex = 2; // Default to Total
            }

            selectedIndex = GUILayout.SelectionGrid(selectedIndex, contexts, 3);

            if (contexts[selectedIndex] != selectedContext)
            {
                selectedContext = contexts[selectedIndex];
                RefreshWaterfallData();
            }

            GUILayout.EndHorizontal();
        }

        /// <summary>
        /// Draw the request list
        /// </summary>
        private void DrawRequestList()
        {
            // Table header
            GUILayout.BeginHorizontal(toolbarStyle);
            GUILayout.Label("#", GUILayout.Width(30));
            GUILayout.Label("Type", GUILayout.Width(100));
            GUILayout.Label("Status", GUILayout.Width(80));
            GUILayout.Label("Duration (ms)", GUILayout.Width(80));
            GUILayout.Label("Stages", GUILayout.Width(50));

            if (showUuids)
            {
                GUILayout.Label("UUID");
            }

            GUILayout.EndHorizontal();

            // Scrollable list
            scrollPosition = GUILayout.BeginScrollView(scrollPosition);

            int displayCount = Mathf.Min(cachedEntries.Length, maxRequestsToDisplay);

            for (int i = 0; i < displayCount; i++)
            {
                var entry = cachedEntries[i];

                GUILayout.BeginHorizontal(i % 2 == 0 ? toolbarStyle : GUIStyle.none);

                // Index
                GUILayout.Label((i + 1).ToString(), GUILayout.Width(30));

                // Request type
                GUILayout.Label(entry.request_type.ToString(), GUILayout.Width(100));

                // Status with color
                GUIStyle statusStyle = GetStyleForStatus(entry.status);
                GUILayout.Label(entry.status.ToString(), statusStyle, GUILayout.Width(80));

                // Duration
                string durationText = (entry.status == RequestStatus.Pending)
                    ? "Pending"
                    : entry.total_duration_ms.ToString("F2");
                GUILayout.Label(durationText, GUILayout.Width(80));

                // Stage count
                GUILayout.Label(entry.stage_count.ToString(), GUILayout.Width(50));

                // UUID (optional)
                if (showUuids)
                {
                    string uuidText = entry.GetUuid().ToString();
                    // Shorten UUID for display
                    if (uuidText.Length > 12)
                    {
                        uuidText = uuidText.Substring(0, 8) + "..." + uuidText.Substring(uuidText.Length - 4);
                    }
                    GUILayout.Label(uuidText);
                }

                GUILayout.EndHorizontal();

                // Draw duration bar
                if (entry.status != RequestStatus.Pending && entry.total_duration_ms > 0)
                {
                    float barWidth = Mathf.Min(entry.total_duration_ms / 10f, windowWidth - 20);
                    Rect barRect = GUILayoutUtility.GetLastRect();
                    barRect.y += barRect.height - 2;
                    barRect.height = 4;
                    barRect.width = barWidth;
                    barRect.x += 30; // Offset for index column

                    Color barColor = GetColorForStatus(entry.status);
                    DrawRect(barRect, barColor);
                }
            }

            GUILayout.EndScrollView();

            // Footer
            if (cachedEntries.Length > maxRequestsToDisplay)
            {
                GUILayout.Label($"Showing {displayCount} of {cachedEntries.Length} requests");
            }
        }

        #endregion

        #region Helpers

        /// <summary>
        /// Refresh waterfall data from the profiler
        /// </summary>
        private void RefreshWaterfallData()
        {
            try
            {
                if (NetworkProfilerBehaviour.Instance == null)
                {
                    cachedEntries = Array.Empty<WaterfallEntry>();
                    return;
                }

                ProfilerContext context = selectedContext switch
                {
                    "Unity" => ProfilerContext.Unity,
                    "Rust" => ProfilerContext.Rust,
                    _ => ProfilerContext.Total
                };

                cachedEntries = NetworkProfilerBehaviour.Instance.GetWaterfall(context) ?? Array.Empty<WaterfallEntry>();
            }
            catch (Exception ex)
            {
                Debug.LogError($"[NetworkProfilerDebugUI] Failed to refresh waterfall data: {ex.Message}");
                cachedEntries = Array.Empty<WaterfallEntry>();
            }
        }

        /// <summary>
        /// Get GUIStyle for request status
        /// </summary>
        private GUIStyle GetStyleForStatus(RequestStatus status)
        {
            GUIStyle style = new GUIStyle(labelStyle);

            style.normal.textColor = GetColorForStatus(status);
            style.fontStyle = FontStyle.Bold;

            return style;
        }

        /// <summary>
        /// Get color for request status
        /// </summary>
        private Color GetColorForStatus(RequestStatus status)
        {
            return status switch
            {
                RequestStatus.Completed => Color.green,
                RequestStatus.Failed => Color.red,
                RequestStatus.Pending => Color.yellow,
                RequestStatus.TimedOut => new Color(1.0f, 0.5f, 0.0f), // Orange
                RequestStatus.InProgress => Color.cyan,
                _ => Color.white
            };
        }

        #endregion

        #region Helper Methods

        /// <summary>
        /// Initialize GUI styles for both editor and runtime
        /// </summary>
        private void InitializeStyles()
        {
#if UNITY_EDITOR
            toolbarStyle = UnityEditor.EditorStyles.toolbar;
            labelStyle = UnityEditor.EditorStyles.label;
            boldLabelStyle = UnityEditor.EditorStyles.boldLabel;
            helpBoxStyle = UnityEditor.EditorStyles.helpBox;
#else
            // Runtime fallback styles
            toolbarStyle = new GUIStyle();
            labelStyle = new GUIStyle();
            boldLabelStyle = new GUIStyle { fontStyle = FontStyle.Bold };
            helpBoxStyle = new GUIStyle { normal = { background = Texture2D.whiteTexture } };
#endif
        }

        /// <summary>
        /// Draw a filled rectangle
        /// </summary>
        private void DrawRect(Rect position, Color color)
        {
#if UNITY_EDITOR
            UnityEditor.EditorGUI.DrawRect(position, color);
#else
            // Runtime: Use GUI.color and DrawTexture if needed
            // For simplicity, we just draw nothing in runtime builds
            // Consider implementing with GUI.DrawTexture or Unity UI for production
#endif
        }

        #endregion
    }
}
