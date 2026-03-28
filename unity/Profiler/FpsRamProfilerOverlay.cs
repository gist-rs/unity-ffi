using System;
using System.Runtime.InteropServices;
using UnityEngine;
using UnityEngine.UI;
using Unity.Profiler;

namespace Unity.Profiler
{
    /// <summary>
    /// Graphy-style FPS/RAM profiler overlay UI
    /// Provides real-time visualization of FPS, memory, and frame timing data
    /// Adapted from Graphy (MIT licensed by TalesFromScript)
    /// </summary>
    [AddComponentMenu("Profiler/FPS RAM Profiler Overlay")]
    [DisallowMultipleComponent]
    public class FpsRamProfilerOverlay : MonoBehaviour
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

        #endregion

        #region Settings

        [Header("Display Settings")]
        [Tooltip("Enable FPS display")]
        [SerializeField]
        private bool showFpsDisplay = true;

        [Tooltip("Enable memory display")]
        [SerializeField]
        private bool showMemoryDisplay = true;

        [Tooltip("Enable frame timing graph")]
        [SerializeField]
        private bool showFrameTimingGraph = true;

        [Header("Update Rate Throttling")]
        [Tooltip("FPS text update rate (times per second)")]
        [SerializeField]
        [Range(1, 30)]
        private int fpsTextUpdateRate = 3;

        [Tooltip("Frame timing graph update rate (times per second)")]
        [SerializeField]
        [Range(10, 60)]
        private int graphUpdateRate = 30;

        [Header("Memory Display")]
        [Tooltip("Maximum memory for bar scaling (MB)")]
        [SerializeField]
        private float maxMemoryMb = 2048.0f;

        [Header("Frame Timing Graph")]
        [Tooltip("Good frame time threshold (ms) - green color")]
        [SerializeField]
        private float goodThresholdMs = 16.67f;

        [Tooltip("Caution frame time threshold (ms) - yellow color")]
        [SerializeField]
        private float cautionThresholdMs = 33.33f;

        [Header("Hotkeys")]
        [Tooltip("Key to toggle profiler visibility")]
        [SerializeField]
        private KeyCode toggleKey = KeyCode.F5;

        [Tooltip("Enable hotkey for toggling profiler")]
        [SerializeField]
        private bool enableHotkey = true;

        #endregion

        #region UI References

        [Header("UI References")]
        [Tooltip("Main profiler panel")]
        [SerializeField]
        private GameObject profilerPanel;

        [Header("FPS Display")]
        [Tooltip("FPS text display")]
        [SerializeField]
        private Text fpsText;

        [Tooltip("Frame time text display")]
        [SerializeField]
        private Text frameTimeText;

        [Header("Memory Display")]
        [Tooltip("Memory text display")]
        [SerializeField]
        private Text memoryText;

        [Tooltip("Reserved memory bar")]
        [SerializeField]
        private GameObject memoryBarReserved;

        [Tooltip("Allocated memory bar")]
        [SerializeField]
        private GameObject memoryBarAllocated;

        [Tooltip("Mono memory bar")]
        [SerializeField]
        private GameObject memoryBarMono;

        [Header("Frame Timing Graph")]
        [Tooltip("Frame timing graph texture")]
        [SerializeField]
        private RawImage frameTimingGraph;

        [Tooltip("Graph data buffer size (must match FFI)")]
        [SerializeField]
        private int graphBufferSize = 512;

        [Header("Tabs")]
        [Tooltip("Tab buttons for context switching")]
        [SerializeField]
        private Button[] tabButtons;

        #endregion

        #region Fields

        private ProfilerTab currentTab = ProfilerTab.Total;
        private bool isVisible = true;
        private FpsRamProfiler profiler;
        private float lastFpsTextUpdateTime;
        private float lastGraphUpdateTime;
        private Texture2D graphTexture;
        private Color[] graphColors;
        private float[] graphBuffer;

        // Graphy colors
        private readonly Color goodColor = new Color(0.0f, 1.0f, 0.0f, 1.0f);      // Green
        private readonly Color cautionColor = new Color(1.0f, 1.0f, 0.0f, 1.0f);   // Yellow
        private readonly Color criticalColor = new Color(1.0f, 0.0f, 0.0f, 1.0f);  // Red
        private readonly Color averageLineColor = new Color(1.0f, 1.0f, 1.0f, 0.5f); // Semi-transparent white

        // Memory bar colors (Graphy: pink/green/blue)
        private readonly Color allocatedColor = new Color(1.0f, 0.0f, 1.0f, 1.0f);   // Pink
        private readonly Color reservedColor = new Color(0.0f, 1.0f, 0.0f, 1.0f);    // Green
        private readonly Color monoColor = new Color(0.0f, 0.5f, 1.0f, 1.0f);       // Blue

        #endregion

        #region Properties

        /// <summary>
        /// Check if the overlay is visible
        /// </summary>
        public bool IsVisible => isVisible;

        /// <summary>
        /// Get the current profiler tab
        /// </summary>
        public ProfilerTab CurrentTab => currentTab;

        #endregion

        #region Unity Lifecycle

        /// <summary>
        /// Initialize the overlay on start
        /// </summary>
        private void Start()
        {
            // Get profiler instance from behaviour
            profiler = FpsRamProfilerBehaviour.Profiler;

            if (profiler == null)
            {
                Debug.LogError("[FpsRamProfilerOverlay] FpsRamProfiler not found. Make sure FpsRamProfilerBehaviour is active.");
                enabled = false;
                return;
            }

            // Initialize tab buttons
            if (tabButtons != null)
            {
                for (int i = 0; i < tabButtons.Length; i++)
                {
                    int tabIndex = i;
                    if (tabButtons[i] != null)
                    {
                        tabButtons[i].onClick.AddListener(() => SwitchTab((ProfilerTab)tabIndex));
                    }
                }
            }

            // Initialize graph texture
            InitializeGraphTexture();

            // Set initial visibility
            SetVisibility(isVisible);
        }

        /// <summary>
        /// Update overlay every frame
        /// </summary>
        private void Update()
        {
            // Check for hotkey
            if (enableHotkey && Input.GetKeyDown(toggleKey))
            {
                ToggleVisibility();
            }

            if (!isVisible || profiler == null || !profiler.IsInitialized)
            {
                return;
            }

            float currentTime = Time.unscaledTime;

            // Update FPS text at throttled rate
            if (currentTime - lastFpsTextUpdateTime >= (1.0f / fpsTextUpdateRate))
            {
                if (showFpsDisplay)
                {
                    UpdateFpsDisplay();
                }
                lastFpsTextUpdateTime = currentTime;
            }

            // Update graph at throttled rate
            if (currentTime - lastGraphUpdateTime >= (1.0f / graphUpdateRate))
            {
                if (showFrameTimingGraph)
                {
                    UpdateFrameTimingGraph();
                }
                lastGraphUpdateTime = currentTime;
            }

            // Update memory display (every frame for smooth animation)
            if (showMemoryDisplay)
            {
                UpdateMemoryDisplay();
            }
        }

        /// <summary>
        /// Cleanup on destroy
        /// </summary>
        private void OnDestroy()
        {
            // Cleanup tab button listeners
            if (tabButtons != null)
            {
                for (int i = 0; i < tabButtons.Length; i++)
                {
                    if (tabButtons[i] != null)
                    {
                        tabButtons[i].onClick.RemoveAllListeners();
                    }
                }
            }

            // Cleanup graph texture
            if (graphTexture != null)
            {
                Destroy(graphTexture);
                graphTexture = null;
            }
        }

        #endregion

        #region Initialization

        /// <summary>
        /// Initialize the frame timing graph texture
        /// </summary>
        private void InitializeGraphTexture()
        {
            if (frameTimingGraph == null || graphBufferSize == 0)
            {
                return;
            }

            // Create texture for graph rendering
            graphTexture = new Texture2D(graphBufferSize, 1, TextureFormat.RGBA32, false);
            graphTexture.filterMode = FilterMode.Point;
            graphTexture.wrapMode = TextureWrapMode.Clamp;

            // Apply to RawImage
            frameTimingGraph.texture = graphTexture;

            // Initialize color array
            graphColors = new Color[graphBufferSize];

            // Initialize graph buffer for FFI
            graphBuffer = new float[graphBufferSize];
        }

        #endregion

        #region Tab Switching

        /// <summary>
        /// Switch to a different profiler tab
        /// </summary>
        /// <param name="tab">Tab to switch to</param>
        public void SwitchTab(ProfilerTab tab)
        {
            currentTab = tab;

            // Update tab button states
            if (tabButtons != null)
            {
                for (int i = 0; i < tabButtons.Length; i++)
                {
                    if (tabButtons[i] != null)
                    {
                        tabButtons[i].interactable = (ProfilerTab)i != tab;

                        // Optional: Change button appearance to show active tab
                        ColorBlock colors = tabButtons[i].colors;
                        colors.normalColor = (ProfilerTab)i == tab ? Color.yellow : Color.white;
                        tabButtons[i].colors = colors;
                    }
                }
            }

            // Force immediate UI update
            ForceUIUpdate();
        }

        #endregion

        #region UI Updates

        /// <summary>
        /// Force immediate UI update
        /// </summary>
        public void ForceUIUpdate()
        {
            if (showFpsDisplay)
            {
                UpdateFpsDisplay();
            }
            if (showMemoryDisplay)
            {
                UpdateMemoryDisplay();
            }
            if (showFrameTimingGraph)
            {
                UpdateFrameTimingGraph();
            }
        }

        /// <summary>
        /// Update FPS display
        /// </summary>
        private void UpdateFpsDisplay()
        {
            if (fpsText == null || frameTimeText == null)
            {
                return;
            }

            if (!profiler.GetFpsMetrics((ProfilerContext)currentTab, out FpsMetricsGraphy fps))
            {
                return;
            }

            // Format FPS text (Graphy style)
            fpsText.text = $"FPS: {fps.current_fps:F0}\n" +
                          $"Avg: {fps.avg_fps:F0}\n" +
                          $"1%: {fps.one_percent_low:F0}\n" +
                          $"0.1%: {fps.zero1_percent_low:F0}";

            // Calculate frame time from current FPS
            float frameTimeMs = fps.current_fps > 0 ? 1000.0f / fps.current_fps : 0.0f;
            frameTimeText.text = $"{frameTimeMs:F1} ms";

            // Color code based on performance
            Color fpsColor = GetFpsColor(fps.current_fps);
            fpsText.color = fpsColor;
            frameTimeText.color = GetFrameTimeColor(frameTimeMs);
        }

        /// <summary>
        /// Update memory display
        /// </summary>
        private void UpdateMemoryDisplay()
        {
            if (memoryText == null)
            {
                return;
            }

            if (!profiler.GetMemoryMetrics((ProfilerContext)currentTab, out MemoryMetricsGraphy memory))
            {
                return;
            }

            // Format memory text (Graphy style)
            memoryText.text = $"Reserved: {memory.reserved_mb:F1} MB\n" +
                             $"Allocated: {memory.allocated_mb:F1} MB\n" +
                             $"Mono: {memory.mono_mb:F1} MB";

            // Update memory bars
            UpdateMemoryBar(memoryBarReserved, memory.reserved_mb, reservedColor);
            UpdateMemoryBar(memoryBarAllocated, memory.allocated_mb, allocatedColor);
            UpdateMemoryBar(memoryBarMono, memory.mono_mb, monoColor);
        }

        /// <summary>
        /// Update a memory bar
        /// </summary>
        /// <param name="bar">Bar GameObject</param>
        /// <param name="valueMb">Memory value in MB</param>
        /// <param name="color">Bar color</param>
        private void UpdateMemoryBar(GameObject bar, float valueMb, Color color)
        {
            if (bar == null)
            {
                return;
            }

            RectTransform rectTransform = bar.GetComponent<RectTransform>();
            if (rectTransform != null)
            {
                // Calculate percentage of max memory
                float percent = Mathf.Clamp01(valueMb / maxMemoryMb);
                rectTransform.anchorMax = new Vector2(percent, rectTransform.anchorMax.y);
            }

            Image image = bar.GetComponent<Image>();
            if (image != null)
            {
                image.color = color;
            }
        }

        /// <summary>
        /// Update frame timing graph
        /// </summary>
        private void UpdateFrameTimingGraph()
        {
            if (graphTexture == null || graphBuffer == null)
            {
                return;
            }

            // Prepare FFI data structure
            ProfilerGraphData graphData = default;
            unsafe
            {
                fixed (float* ptr = graphBuffer)
                {
                    graphData.values = ptr;
                    graphData.length = (uint)graphBufferSize;
                }
            }

            // Get graph data from profiler
            if (!profiler.GetGraphData((ProfilerContext)currentTab, ref graphData))
            {
                return;
            }

            // Render graph to texture
            RenderGraphToTexture(graphBuffer, graphData);
        }

        /// <summary>
        /// Render graph data to texture
        /// </summary>
        /// <param name="values">Frame time values</param>
        /// <param name="data">Graph data with thresholds</param>
        private void RenderGraphToTexture(float[] values, ProfilerGraphData data)
        {
            if (values == null || values.Length == 0)
            {
                return;
            }

            // Find max value for scaling
            float maxValue = 0.0f;
            for (int i = 0; i < values.Length; i++)
            {
                if (values[i] > maxValue)
                {
                    maxValue = values[i];
                }
            }

            // Scale to at least caution threshold
            float maxScale = Mathf.Max(maxValue, cautionThresholdMs * 1.2f);

            // Render each pixel
            for (int i = 0; i < values.Length; i++)
            {
                float frameTime = values[i];
                Color color = GetFrameTimeColor(frameTime);

                // Scale height based on frame time
                float height = frameTime / maxScale;
                graphColors[i] = color;
            }

            // Apply colors to texture
            graphTexture.SetPixels(graphColors);
            graphTexture.Apply();
        }

        #endregion

        #region Color Helpers

        /// <summary>
        /// Get color for FPS value (Graphy style)
        /// </summary>
        /// <param name="fps">FPS value</param>
        /// <returns>Color based on FPS</returns>
        private Color GetFpsColor(float fps)
        {
            if (fps >= 60.0f)
            {
                return goodColor; // Green
            }
            else if (fps >= 30.0f)
            {
                return cautionColor; // Yellow
            }
            else
            {
                return criticalColor; // Red
            }
        }

        /// <summary>
        /// Get color for frame time (Graphy style)
        /// </summary>
        /// <param name="frameTimeMs">Frame time in milliseconds</param>
        /// <returns>Color based on frame time</returns>
        private Color GetFrameTimeColor(float frameTimeMs)
        {
            if (frameTimeMs <= goodThresholdMs)
            {
                return goodColor; // Green
            }
            else if (frameTimeMs <= cautionThresholdMs)
            {
                return cautionColor; // Yellow
            }
            else
            {
                return criticalColor; // Red
            }
        }

        #endregion

        #region Visibility Control

        /// <summary>
        /// Toggle profiler visibility
        /// </summary>
        public void ToggleVisibility()
        {
            SetVisibility(!isVisible);
        }

        /// <summary>
        /// Set profiler visibility
        /// </summary>
        /// <param name="visible">True to show, False to hide</param>
        public void SetVisibility(bool visible)
        {
            isVisible = visible;

            if (profilerPanel != null)
            {
                profilerPanel.SetActive(visible);
            }

            // Notify profiler of visibility change
            if (profiler != null)
            {
                profiler.SetVisibility(visible);
            }
        }

        #endregion

        #region Configuration

        /// <summary>
        /// Set FPS text update rate
        /// </summary>
        /// <param name="rate">Updates per second</param>
        public void SetFpsTextUpdateRate(int rate)
        {
            fpsTextUpdateRate = Mathf.Clamp(rate, 1, 30);
        }

        /// <summary>
        /// Set graph update rate
        /// </summary>
        /// <param name="rate">Updates per second</param>
        public void SetGraphUpdateRate(int rate)
        {
            graphUpdateRate = Mathf.Clamp(rate, 10, 60);
        }

        /// <summary>
        /// Set frame time thresholds
        /// </summary>
        /// <param name="good">Good threshold (ms)</param>
        /// <param name="caution">Caution threshold (ms)</param>
        public void SetThresholds(float good, float caution)
        {
            goodThresholdMs = good;
            cautionThresholdMs = caution;
        }

        /// <summary>
        /// Set maximum memory for bar scaling
        /// </summary>
        /// <param name="maxMb">Maximum memory in MB</param>
        public void SetMaxMemory(float maxMb)
        {
            maxMemoryMb = Mathf.Max(maxMb, 512.0f);
        }

        #endregion

        #region Editor Methods

#if UNITY_EDITOR
        /// <summary>
        /// Validate inspector values
        /// </summary>
        private void OnValidate()
        {
            fpsTextUpdateRate = Mathf.Clamp(fpsTextUpdateRate, 1, 30);
            graphUpdateRate = Mathf.Clamp(graphUpdateRate, 10, 60);
            maxMemoryMb = Mathf.Max(maxMemoryMb, 512.0f);
            graphBufferSize = Mathf.Clamp(graphBufferSize, 128, 1024);
        }
#endif

        #endregion
    }
}
