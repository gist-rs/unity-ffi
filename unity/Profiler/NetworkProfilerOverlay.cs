using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using UnityEngine;
using UnityEngine.UI;
using Unity.Profiler;

namespace Unity.Profiler
{
    /// <summary>
    /// Graphy-style Network Profiler overlay UI
    /// Provides Chrome DevTools-like waterfall visualization for network requests
    /// Adapted from Graphy (MIT licensed by TalesFromScript)
    /// </summary>
    [AddComponentMenu("Profiler/Network Profiler Overlay")]
    [DisallowMultipleComponent]
    public class NetworkProfilerOverlay : MonoBehaviour
    {
        #region Tabs

        /// <summary>
        /// Profiler tabs for different contexts
        /// </summary>
        public enum ProfilerTab
        {
            /// <summary>Unity engine metrics only</summary>
            Unity = 0,
            /// <summary>Rust backend metrics only</summary>
            Rust = 1,
            /// <summary>Combined metrics (Unity + Rust)</summary>
            Total = 2
        }

        /// <summary>
        /// Sort type for request table
        /// </summary>
        public enum SortType
        {
            /// <summary>Sort by timestamp (newest first)</summary>
            Timestamp = 0,
            /// <summary>Sort by duration (longest first)</summary>
            Duration = 1,
            /// <summary>Sort by request type</summary>
            RequestType = 2,
            /// <summary>Sort by status</summary>
            Status = 3
        }

        #endregion

        #region Settings

        [Header("Display Settings")]
        [Tooltip("Enable waterfall graph display")]
        [SerializeField]
        private bool showWaterfallGraph = true;

        [Tooltip("Enable request table display")]
        [SerializeField]
        private bool showRequestTable = true;

        [Tooltip("Enable request details panel")]
        [SerializeField]
        private bool showDetailsPanel = true;

        [Header("Update Rate Throttling")]
        [Tooltip("Request table update rate (times per second)")]
        [SerializeField]
        [Range(1, 30)]
        private int tableUpdateRate = 3;

        [Tooltip("Waterfall graph update rate (times per second)")]
        [SerializeField]
        [Range(10, 60)]
        private int graphUpdateRate = 30;

        [Header("Waterfall Display")]
        [Tooltip("Maximum number of requests to display")]
        [SerializeField]
        private int maxRequestsToDisplay = 100;

        [Tooltip("Show request UUIDs in table")]
        [SerializeField]
        private bool showUuids = false;

        [Tooltip("Show stage details in waterfall")]
        [SerializeField]
        private bool showStageDetails = true;

        [Header("Hotkeys")]
        [Tooltip("Key to toggle profiler visibility")]
        [SerializeField]
        private KeyCode toggleKey = KeyCode.F6;

        [Tooltip("Enable hotkey for toggling profiler")]
        [SerializeField]
        private bool enableHotkey = true;

        #endregion

        #region UI References

        [Header("UI References")]
        [Tooltip("Main profiler panel")]
        [SerializeField]
        private GameObject profilerPanel;

        [Header("Tab Buttons")]
        [Tooltip("Unity tab button")]
        [SerializeField]
        private Button unityTabButton;

        [Tooltip("Rust tab button")]
        [SerializeField]
        private Button rustTabButton;

        [Tooltip("Total tab button")]
        [SerializeField]
        private Button totalTabButton;

        [Header("Waterfall Graph")]
        [Tooltip("Waterfall graph image")]
        [SerializeField]
        private RawImage waterfallGraphImage;

        [Tooltip("Waterfall graph texture")]
        [SerializeField]
        private Texture2D waterfallTexture;

        [Header("Request Table")]
        [Tooltip("Request table content rect")]
        [SerializeField]
        private RectTransform requestTableContent;

        [Tooltip("Request table entry prefab")]
        [SerializeField]
        private GameObject requestEntryPrefab;

        [Header("Details Panel")]
        [Tooltip("Request details panel")]
        [SerializeField]
        private GameObject detailsPanel;

        [Tooltip("Request UUID text")]
        [SerializeField]
        private Text requestUuidText;

        [Tooltip("Request type text")]
        [SerializeField]
        private Text requestTypeText;

        [Tooltip("Request status text")]
        [SerializeField]
        private Text requestStatusText;

        [Tooltip("Request duration text")]
        [SerializeField]
        private Text requestDurationText;

        [Tooltip("Stage details text")]
        [SerializeField]
        private Text stageDetailsText;

        [Tooltip("Close details button")]
        [SerializeField]
        private Button closeDetailsButton;

        [Header("Controls")]
        [Tooltip("Filter input field")]
        [SerializeField]
        private InputField filterInputField;

        [Tooltip("Sort dropdown")]
        [SerializeField]
        private Dropdown sortDropdown;

        [Tooltip("Clear button")]
        [SerializeField]
        private Button clearButton;

        [Tooltip("Close profiler button")]
        [SerializeField]
        private Button closeButton;

        #endregion

        #region Fields

        private ProfilerTab currentTab = ProfilerTab.Total;
        private bool isVisible = true;
        private WaterfallEntry[] cachedEntries;
        private WaterfallEntry selectedEntry;
        private List<GameObject> entryObjects = new List<GameObject>();
        private float lastTableRefreshTime;
        private float lastGraphRefreshTime;
        private SortType currentSort = SortType.Timestamp;

        // StringBuilder for efficient string building
        private StringBuilder stringBuilder = new StringBuilder(512);

        #endregion

        #region Public Properties

        /// <summary>
        /// Current profiler tab
        /// </summary>
        public ProfilerTab CurrentTab
        {
            get => currentTab;
            set => SetTab(value);
        }

        /// <summary>
        /// Whether the profiler overlay is visible
        /// </summary>
        public bool IsVisible
        {
            get => isVisible;
            set => SetVisible(value);
        }

        /// <summary>
        /// Current sort type
        /// </summary>
        public SortType CurrentSort
        {
            get => currentSort;
            set => SetSort(value);
        }

        /// <summary>
        /// Current filter text
        /// </summary>
        public string FilterText => filterInputField != null ? filterInputField.text : string.Empty;

        /// <summary>
        /// Currently displayed entries
        /// </summary>
        public WaterfallEntry[] DisplayedEntries => GetFilteredEntries();

        #endregion

        #region Unity Lifecycle

        /// <summary>
        /// Initialize the overlay on awake
        /// </summary>
        private void Awake()
        {
            cachedEntries = Array.Empty<WaterfallEntry>();
            InitializeUI();
            InitializeEventListeners();
        }

        /// <summary>
        /// Start profiler overlay
        /// </summary>
        private void Start()
        {
            SetTab(currentTab);
            RefreshData();
        }

        /// <summary>
        /// Update profiler overlay
        /// </summary>
        private void Update()
        {
            HandleHotkey();
            ThrottledUpdate();
        }

        /// <summary>
        /// Cleanup on destroy
        /// </summary>
        private void OnDestroy()
        {
            CleanupEventListeners();
            CleanupEntryObjects();
        }

        #endregion

        #region Public API

        /// <summary>
        /// Toggle profiler visibility
        /// </summary>
        public void ToggleVisibility()
        {
            IsVisible = !IsVisible;
        }

        /// <summary>
        /// Set profiler visibility
        /// </summary>
        public void SetVisible(bool visible)
        {
            isVisible = visible;
            if (profilerPanel != null)
            {
                profilerPanel.SetActive(visible);
            }
        }

        /// <summary>
        /// Set current tab
        /// </summary>
        public void SetTab(ProfilerTab tab)
        {
            currentTab = tab;
            UpdateTabButtons();
            RefreshData();
        }

        /// <summary>
        /// Set sort type
        /// </summary>
        public void SetSort(SortType sort)
        {
            currentSort = sort;
            if (sortDropdown != null)
            {
                sortDropdown.value = (int)sort;
            }
            RefreshRequestTable();
        }

        /// <summary>
        /// Apply filter to entries
        /// </summary>
        public void ApplyFilter(string filter)
        {
            if (filterInputField != null)
            {
                filterInputField.text = filter;
            }
            RefreshRequestTable();
        }

        /// <summary>
        /// Clear all entries
        /// </summary>
        public void ClearEntries()
        {
            cachedEntries = Array.Empty<WaterfallEntry>();
            selectedEntry = default;
            RefreshRequestTable();
            RefreshWaterfallGraph();
            HideDetailsPanel();
        }

        /// <summary>
        /// Refresh all data
        /// </summary>
        public void RefreshData()
        {
            if (!isVisible) return;

            RefreshWaterfallData();
            RefreshRequestTable();
            RefreshWaterfallGraph();
        }

        #endregion

        #region UI Initialization

        /// <summary>
        /// Initialize UI components
        /// </summary>
        private void InitializeUI()
        {
            // Initialize sort dropdown
            if (sortDropdown != null)
            {
                sortDropdown.options.Clear();
                sortDropdown.options.Add(new Dropdown.OptionData("Timestamp"));
                sortDropdown.options.Add(new Dropdown.OptionData("Duration"));
                sortDropdown.options.Add(new Dropdown.OptionData("Request Type"));
                sortDropdown.options.Add(new Dropdown.OptionData("Status"));
                sortDropdown.value = (int)currentSort;
            }

            // Initialize tab buttons
            UpdateTabButtons();

            // Set initial visibility
            if (profilerPanel != null)
            {
                profilerPanel.SetActive(isVisible);
            }

            // Initialize waterfall texture
            if (waterfallTexture == null && waterfallGraphImage != null)
            {
                waterfallTexture = new Texture2D(512, 256, TextureFormat.RGBA32, false);
                waterfallGraphImage.texture = waterfallTexture;
            }
        }

        /// <summary>
        /// Initialize event listeners
        /// </summary>
        private void InitializeEventListeners()
        {
            if (unityTabButton != null)
            {
                unityTabButton.onClick.AddListener(() => SetTab(ProfilerTab.Unity));
            }

            if (rustTabButton != null)
            {
                rustTabButton.onClick.AddListener(() => SetTab(ProfilerTab.Rust));
            }

            if (totalTabButton != null)
            {
                totalTabButton.onClick.AddListener(() => SetTab(ProfilerTab.Total));
            }

            if (sortDropdown != null)
            {
                sortDropdown.onValueChanged.AddListener(value => SetSort((SortType)value));
            }

            if (clearButton != null)
            {
                clearButton.onClick.AddListener(ClearEntries);
            }

            if (closeButton != null)
            {
                closeButton.onClick.AddListener(() => SetVisible(false));
            }

            if (closeDetailsButton != null)
            {
                closeDetailsButton.onClick.AddListener(HideDetailsPanel);
            }
        }

        #endregion

        #region Throttled Update

        /// <summary>
        /// Throttled update based on update rates
        /// </summary>
        private void ThrottledUpdate()
        {
            if (!isVisible) return;

            float currentTime = Time.realtimeSinceStartup;

            // Update table at throttled rate
            if (currentTime - lastTableRefreshTime > 1.0f / tableUpdateRate)
            {
                RefreshWaterfallData();
                RefreshRequestTable();
                lastTableRefreshTime = currentTime;
            }

            // Update graph at throttled rate
            if (currentTime - lastGraphRefreshTime > 1.0f / graphUpdateRate)
            {
                RefreshWaterfallGraph();
                lastGraphRefreshTime = currentTime;
            }
        }

        #endregion

        #region Hotkey Handling

        /// <summary>
        /// Handle hotkey input
        /// </summary>
        private void HandleHotkey()
        {
            if (enableHotkey && Input.GetKeyDown(toggleKey))
            {
                ToggleVisibility();
            }
        }

        #endregion

        #region Data Refresh

        /// <summary>
        /// Refresh waterfall data from profiler
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

                ProfilerContext context = currentTab switch
                {
                    ProfilerTab.Unity => ProfilerContext.Unity,
                    ProfilerTab.Rust => ProfilerContext.Rust,
                    _ => ProfilerContext.Total
                };

                cachedEntries = NetworkProfilerBehaviour.Instance.GetWaterfall(context) ?? Array.Empty<WaterfallEntry>();
            }
            catch (Exception ex)
            {
                Debug.LogError($"[NetworkProfilerOverlay] Failed to refresh waterfall data: {ex.Message}");
                cachedEntries = Array.Empty<WaterfallEntry>();
            }
        }

        /// <summary>
        /// Refresh request table
        /// </summary>
        private void RefreshRequestTable()
        {
            if (!showRequestTable || requestTableContent == null) return;

            WaterfallEntry[] entries = GetFilteredEntries();

            // Reuse existing entry objects
            for (int i = 0; i < entryObjects.Count; i++)
            {
                if (i < entries.Length)
                {
                    UpdateEntryObject(entryObjects[i], entries[i]);
                    entryObjects[i].SetActive(true);
                }
                else
                {
                    entryObjects[i].SetActive(false);
                }
            }

            // Create new entry objects if needed
            for (int i = entryObjects.Count; i < entries.Length; i++)
            {
                if (requestEntryPrefab != null)
                {
                    GameObject entryObj = Instantiate(requestEntryPrefab, requestTableContent);
                    SetupEntryObject(entryObj, entries[i]);
                    entryObjects.Add(entryObj);
                }
            }
        }

        /// <summary>
        /// Get filtered and sorted entries
        /// </summary>
        private WaterfallEntry[] GetFilteredEntries()
        {
            string filter = FilterText.ToLower();
            WaterfallEntry[] entries = cachedEntries
                .Where(e => string.IsNullOrEmpty(filter) ||
                           e.request_type.ToString().ToLower().Contains(filter) ||
                           e.status.ToString().ToLower().Contains(filter) ||
                           e.GetUuid().ToString().ToLower().Contains(filter))
                .ToArray();

            // Apply sorting
            entries = currentSort switch
            {
                SortType.Timestamp => entries.OrderByDescending(e => e.start_ns).ToArray(),
                SortType.Duration => entries.OrderByDescending(e => e.total_duration_ms).ToArray(),
                SortType.RequestType => entries.OrderBy(e => e.request_type.ToString()).ToArray(),
                SortType.Status => entries.OrderBy(e => e.status).ToArray(),
                _ => entries
            };

            // Limit display count
            if (entries.Length > maxRequestsToDisplay)
            {
                Array.Resize(ref entries, maxRequestsToDisplay);
            }

            return entries;
        }

        #endregion

        #region Entry Objects

        /// <summary>
        /// Setup entry object with initial data
        /// </summary>
        private void SetupEntryObject(GameObject obj, WaterfallEntry entry)
        {
            Text[] texts = obj.GetComponentsInChildren<Text>();
            if (texts.Length >= 4)
            {
                texts[0].text = entry.request_type.ToString();
                texts[1].text = entry.status.ToString();
                texts[2].text = entry.total_duration_ms.ToString("F2");
                if (showUuids)
                {
                    texts[3].text = entry.GetUuid().ToString();
                }
            }

            Button button = obj.GetComponent<Button>();
            if (button != null)
            {
                button.onClick.AddListener(() => ShowDetails(entry));
            }
        }

        /// <summary>
        /// Update entry object with new data
        /// </summary>
        private void UpdateEntryObject(GameObject obj, WaterfallEntry entry)
        {
            Text[] texts = obj.GetComponentsInChildren<Text>();
            if (texts.Length >= 4)
            {
                texts[0].text = entry.request_type.ToString();
                texts[1].text = entry.status.ToString();
                texts[2].text = entry.total_duration_ms.ToString("F2");
                if (showUuids)
                {
                    texts[3].text = entry.GetUuid().ToString();
                }
            }
        }

        #endregion

        #region Waterfall Graph

        /// <summary>
        /// Refresh waterfall graph rendering
        /// </summary>
        private void RefreshWaterfallGraph()
        {
            if (!showWaterfallGraph || waterfallTexture == null) return;

            WaterfallEntry[] entries = GetFilteredEntries();
            if (entries.Length == 0)
            {
                ClearWaterfallGraph();
                return;
            }

            // Clear texture
            Color32[] colors = new Color32[512 * 256];
            for (int i = 0; i < colors.Length; i++)
            {
                colors[i] = new Color32(0, 0, 0, 0); // Transparent
            }

            // Calculate scaling
            float maxDuration = entries.Max(e => e.total_duration_ms);
            if (maxDuration == 0) maxDuration = 1.0f;
            float timeScale = 512.0f / maxDuration;

            // Draw waterfall entries
            for (int i = 0; i < entries.Length; i++)
            {
                int y = i * 2;
                if (y >= 256) break;

                Color color = GetColorForStatus(entries[i].status);
                float startX = 0;
                float width = entries[i].total_duration_ms * timeScale;

                DrawBar(colors, y, startX, width, color);
            }

            // Apply to texture
            waterfallTexture.SetPixels32(colors);
            waterfallTexture.Apply();
        }

        /// <summary>
        /// Clear waterfall graph
        /// </summary>
        private void ClearWaterfallGraph()
        {
            if (waterfallTexture == null) return;

            Color32[] colors = new Color32[512 * 256];
            for (int i = 0; i < colors.Length; i++)
            {
                colors[i] = new Color32(0, 0, 0, 0);
            }

            waterfallTexture.SetPixels32(colors);
            waterfallTexture.Apply();
        }

        /// <summary>
        /// Draw a bar on the waterfall graph
        /// </summary>
        private void DrawBar(Color32[] colors, int y, float startX, float width, Color color)
        {
            int startXInt = Mathf.Clamp((int)startX, 0, 511);
            int endXInt = Mathf.Clamp((int)(startX + width), 0, 512);

            byte r = (byte)(color.r * 255);
            byte g = (byte)(color.g * 255);
            byte b = (byte)(color.b * 255);
            byte a = (byte)(color.a * 255);

            for (int x = startXInt; x < endXInt; x++)
            {
                int index = y * 512 + x;
                if (index >= 0 && index < colors.Length)
                {
                    colors[index] = new Color32(r, g, b, a);
                    if (y + 1 < 256)
                    {
                        colors[index + 512] = new Color32(r, g, b, a);
                    }
                }
            }
        }

        #endregion

        #region Details Panel

        /// <summary>
        /// Show details for selected entry
        /// </summary>
        private void ShowDetails(WaterfallEntry entry)
        {
            selectedEntry = entry;
            if (detailsPanel == null) return;

            detailsPanel.SetActive(true);

            if (requestUuidText != null)
            {
                requestUuidText.text = entry.GetUuid().ToString();
            }

            if (requestTypeText != null)
            {
                requestTypeText.text = entry.request_type.ToString();
            }

            if (requestStatusText != null)
            {
                requestStatusText.text = entry.status.ToString();
                requestStatusText.color = GetColorForStatus(entry.status);
            }

            if (requestDurationText != null)
            {
                requestDurationText.text = $"{entry.total_duration_ms:F2} ms";
            }

            if (stageDetailsText != null)
            {
                stringBuilder.Clear();
                stringBuilder.AppendLine($"Stages: {entry.stage_count}");
                stringBuilder.AppendLine($"Start NS: {entry.start_ns}");
                stringBuilder.AppendLine($"Context: {entry.context}");
                stageDetailsText.text = stringBuilder.ToString();
            }
        }

        /// <summary>
        /// Hide details panel
        /// </summary>
        private void HideDetailsPanel()
        {
            selectedEntry = default;
            if (detailsPanel != null)
            {
                detailsPanel.SetActive(false);
            }
        }

        #endregion

        #region Tab Management

        /// <summary>
        /// Update tab button states
        /// </summary>
        private void UpdateTabButtons()
        {
            if (unityTabButton != null)
            {
                var colors = unityTabButton.colors;
                colors.normalColor = currentTab == ProfilerTab.Unity ? Color.yellow : Color.white;
                unityTabButton.colors = colors;
            }

            if (rustTabButton != null)
            {
                var colors = rustTabButton.colors;
                colors.normalColor = currentTab == ProfilerTab.Rust ? Color.yellow : Color.white;
                rustTabButton.colors = colors;
            }

            if (totalTabButton != null)
            {
                var colors = totalTabButton.colors;
                colors.normalColor = currentTab == ProfilerTab.Total ? Color.yellow : Color.white;
                totalTabButton.colors = colors;
            }
        }

        #endregion

        #region Helper Methods

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

        /// <summary>
        /// Cleanup event listeners
        /// </summary>
        private void CleanupEventListeners()
        {
            if (unityTabButton != null)
            {
                unityTabButton.onClick.RemoveAllListeners();
            }

            if (rustTabButton != null)
            {
                rustTabButton.onClick.RemoveAllListeners();
            }

            if (totalTabButton != null)
            {
                totalTabButton.onClick.RemoveAllListeners();
            }

            if (sortDropdown != null)
            {
                sortDropdown.onValueChanged.RemoveAllListeners();
            }

            if (clearButton != null)
            {
                clearButton.onClick.RemoveAllListeners();
            }

            if (closeButton != null)
            {
                closeButton.onClick.RemoveAllListeners();
            }

            if (closeDetailsButton != null)
            {
                closeDetailsButton.onClick.RemoveAllListeners();
            }
        }

        /// <summary>
        /// Cleanup entry objects
        /// </summary>
        private void CleanupEntryObjects()
        {
            foreach (var obj in entryObjects)
            {
                if (obj != null)
                {
                    Destroy(obj);
                }
            }
            entryObjects.Clear();
        }

        #endregion
    }
}
