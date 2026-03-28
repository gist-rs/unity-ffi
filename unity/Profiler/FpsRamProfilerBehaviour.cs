using UnityEngine;
using Unity.Profiler;

namespace Unity.Profiler
{
    /// <summary>
    /// MonoBehaviour integration for FPS/RAM Profiler
    /// Provides automatic lifecycle management and Unity-specific profiling hooks
    /// Adapted from Graphy's profiling pattern
    /// </summary>
    [AddComponentMenu("Profiler/FPS RAM Profiler Behaviour")]
    [DisallowMultipleComponent]
    public class FpsRamProfilerBehaviour : MonoBehaviour
    {
        #region Settings

        [Header("Profiler Settings")]
        [Tooltip("Enable frame time recording")]
        [SerializeField]
        private bool enableFrameRecording = true;

        [Tooltip("Enable Unity memory metrics submission")]
        [SerializeField]
        private bool enableMemoryTracking = true;

        [Tooltip("Memory update interval in seconds (default: 0.5s)")]
        [SerializeField]
        [Range(0.1f, 5.0f)]
        private float memoryUpdateInterval = 0.5f;

        [Tooltip("Profiler context (Unity, Rust, or Total)")]
        [SerializeField]
        private ProfilerContext profilerContext = ProfilerContext.Total;

        [Header("Debug Settings")]
        [Tooltip("Enable debug logging for profiler operations")]
        [SerializeField]
        private bool enableDebugLogging = false;

        [Tooltip("Show profiler information in console")]
        [SerializeField]
        private bool showProfilerInfo = true;

        [Header("Hotkeys")]
        [Tooltip("Key to toggle profiler visibility (default: F5)")]
        [SerializeField]
        private KeyCode toggleKey = KeyCode.F5;

        [Tooltip("Enable hotkey for toggling profiler")]
        [SerializeField]
        private bool enableHotkey = true;

        #endregion

        #region Static Access

        /// <summary>
        /// Global profiler instance accessible from anywhere in the code
        /// </summary>
        public static FpsRamProfilerBehaviour Instance { get; private set; }

        /// <summary>
        /// Access to the underlying FpsRamProfiler
        /// </summary>
        public static FpsRamProfiler Profiler => Instance?.profiler;

        #endregion

        #region Fields

        private FpsRamProfiler profiler;
        private float lastMemoryUpdateTime;
        private float lastFrameTime;
        private int frameCount;
        private bool isProfilerVisible = true;

        #endregion

        #region Properties

        /// <summary>
        /// Check if the profiler is currently visible
        /// </summary>
        public bool IsVisible => isProfilerVisible;

        /// <summary>
        /// Get the current profiler context
        /// </summary>
        public ProfilerContext Context => profilerContext;

        #endregion

        #region Unity Lifecycle

        /// <summary>
        /// Initialize the profiler on awake
        /// </summary>
        private void Awake()
        {
            // Singleton pattern
            if (Instance != null && Instance != this)
            {
                Debug.LogWarning("[FpsRamProfiler] Multiple FpsRamProfilerBehaviour instances detected. Destroying duplicate.");
                Destroy(gameObject);
                return;
            }

            Instance = this;

            // Optional: don't destroy on load to persist profiler data across scenes
            // DontDestroyOnLoad(gameObject);

            InitializeProfiler();
        }

        /// <summary>
        /// Start tracking Unity-specific metrics
        /// </summary>
        private void Start()
        {
            if (showProfilerInfo && enableDebugLogging)
            {
                Debug.Log($"[FpsRamProfiler] Initialized with context: {profilerContext}");
                Debug.Log($"[FpsRamProfiler] Frame recording: {enableFrameRecording}, Memory tracking: {enableMemoryTracking}");
            }

            // Set initial visibility
            if (profiler != null)
            {
                profiler.SetVisibility(isProfilerVisible);
            }
        }

        /// <summary>
        /// Record frame time every frame
        /// </summary>
        private void Update()
        {
            // Check for hotkey
            if (enableHotkey && Input.GetKeyDown(toggleKey))
            {
                ToggleVisibility();
            }

            // Skip if profiler is not initialized
            if (profiler == null || !profiler.IsInitialized)
            {
                return;
            }

            // Record frame time
            if (enableFrameRecording)
            {
                float deltaTimeMs = Time.unscaledDeltaTime * 1000.0f;
                profiler.RecordFrame(deltaTimeMs);

                // Track frame time for debugging
                lastFrameTime = deltaTimeMs;
                frameCount++;
            }

            // Submit Unity memory metrics periodically
            if (enableMemoryTracking)
            {
                float currentTime = Time.unscaledTime;
                if (currentTime - lastMemoryUpdateTime >= memoryUpdateInterval)
                {
                    SubmitUnityMemory();
                    lastMemoryUpdateTime = currentTime;
                }
            }
        }

        /// <summary>
        /// Cleanup profiler on destroy
        /// </summary>
        private void OnDestroy()
        {
            if (Instance == this)
            {
                Instance = null;
            }

            ShutdownProfiler();
        }

        /// <summary>
        /// Handle application pause
        /// </summary>
        private void OnApplicationPause(bool pauseStatus)
        {
            if (pauseStatus && enableDebugLogging)
            {
                Debug.Log($"[FpsRamProfiler] Application paused. Frames recorded: {frameCount}");
            }
        }

        /// <summary>
        /// Handle application focus loss
        /// </summary>
        private void OnApplicationFocus(bool hasFocus)
        {
            if (!hasFocus && enableDebugLogging)
            {
                Debug.Log("[FpsRamProfiler] Application lost focus");
            }
        }

        #endregion

        #region Initialization & Shutdown

        /// <summary>
        /// Initialize the profiler
        /// </summary>
        private void InitializeProfiler()
        {
            try
            {
                profiler = new FpsRamProfiler();

                if (enableDebugLogging)
                {
                    Debug.Log("[FpsRamProfiler] Successfully initialized");
                }
            }
            catch (System.Exception ex)
            {
                Debug.LogError($"[FpsRamProfiler] Failed to initialize: {ex.Message}");
                profiler = null;
            }
        }

        /// <summary>
        /// Shutdown the profiler
        /// </summary>
        private void ShutdownProfiler()
        {
            if (profiler != null)
            {
                try
                {
                    if (enableDebugLogging)
                    {
                        Debug.Log($"[FpsRamProfiler] Shutting down. Total frames recorded: {frameCount}");
                    }

                    profiler.Dispose();
                    profiler = null;
                }
                catch (System.Exception ex)
                {
                    Debug.LogError($"[FpsRamProfiler] Error during shutdown: {ex.Message}");
                }
            }
        }

        #endregion

        #region Public API

        /// <summary>
        /// Toggle profiler visibility
        /// </summary>
        public void ToggleVisibility()
        {
            isProfilerVisible = !isProfilerVisible;

            if (profiler != null && profiler.IsInitialized)
            {
                profiler.SetVisibility(isProfilerVisible);
            }

            if (enableDebugLogging)
            {
                Debug.Log($"[FpsRamProfiler] Visibility toggled: {isProfilerVisible}");
            }
        }

        /// <summary>
        /// Set profiler visibility
        /// </summary>
        /// <param name="visible">True to show profiler, False to hide</param>
        public void SetVisibility(bool visible)
        {
            isProfilerVisible = visible;

            if (profiler != null && profiler.IsInitialized)
            {
                profiler.SetVisibility(visible);
            }
        }

        /// <summary>
        /// Reset all profiler metrics
        /// </summary>
        public void ResetMetrics()
        {
            if (profiler != null && profiler.IsInitialized)
            {
                profiler.Reset();
                frameCount = 0;

                if (enableDebugLogging)
                {
                    Debug.Log("[FpsRamProfiler] Metrics reset");
                }
            }
        }

        /// <summary>
        /// Change the profiler context
        /// </summary>
        /// <param name="newContext">New profiler context</param>
        public void SetContext(ProfilerContext newContext)
        {
            profilerContext = newContext;

            if (enableDebugLogging)
            {
                Debug.Log($"[FpsRamProfiler] Context changed to: {newContext}");
            }
        }

        /// <summary>
        /// Get FPS metrics for the current context
        /// </summary>
        /// <param name="metrics">Output FPS metrics</param>
        /// <returns>True if successful</returns>
        public bool GetFpsMetrics(out FpsMetricsGraphy metrics)
        {
            if (profiler != null && profiler.IsInitialized)
            {
                return profiler.GetFpsMetrics(profilerContext, out metrics);
            }

            metrics = default;
            return false;
        }

        /// <summary>
        /// Get memory metrics for the current context
        /// </summary>
        /// <param name="metrics">Output memory metrics</param>
        /// <returns>True if successful</returns>
        public bool GetMemoryMetrics(out MemoryMetricsGraphy metrics)
        {
            if (profiler != null && profiler.IsInitialized)
            {
                return profiler.GetMemoryMetrics(profilerContext, out metrics);
            }

            metrics = default;
            return false;
        }

        /// <summary>
        /// Get graph data for frame timing visualization
        /// </summary>
        /// <param name="data">Output graph data with pre-allocated buffer</param>
        /// <returns>True if successful</returns>
        public bool GetGraphData(ref ProfilerGraphData data)
        {
            if (profiler != null && profiler.IsInitialized)
            {
                return profiler.GetGraphData(profilerContext, ref data);
            }

            return false;
        }

        /// <summary>
        /// Manually submit Unity memory metrics
        /// </summary>
        public void SubmitUnityMemory()
        {
            if (profiler == null || !profiler.IsInitialized)
            {
                return;
            }

            try
            {
                // Get memory metrics from Unity Profiler
                long allocatedBytes = UnityEngine.Profiling.Profiler.GetTotalAllocatedMemoryLong();
                long reservedBytes = UnityEngine.Profiling.Profiler.GetTotalReservedMemoryLong();
                long monoBytes = UnityEngine.Profiling.Profiler.GetMonoHeapSizeLong();

                // Convert to MB
                float allocatedMb = allocatedBytes / 1048576.0f;
                float reservedMb = reservedBytes / 1048576.0f;
                float monoMb = monoBytes / 1048576.0f;

                // Submit to profiler
                profiler.SubmitUnityMemory(allocatedMb, reservedMb, monoMb);

                if (enableDebugLogging && frameCount % 60 == 0) // Log every 60 frames
                {
                    Debug.Log($"[FpsRamProfiler] Memory - Reserved: {reservedMb:F1} MB, Allocated: {allocatedMb:F1} MB, Mono: {monoMb:F1} MB");
                }
            }
            catch (System.Exception ex)
            {
                Debug.LogError($"[FpsRamProfiler] Failed to submit memory metrics: {ex.Message}");
            }
        }

        /// <summary>
        /// Enable or disable frame recording
        /// </summary>
        /// <param name="enable">True to enable frame recording</param>
        public void SetFrameRecording(bool enable)
        {
            enableFrameRecording = enable;

            if (enableDebugLogging)
            {
                Debug.Log($"[FpsRamProfiler] Frame recording: {enable}");
            }
        }

        /// <summary>
        /// Enable or disable memory tracking
        /// </summary>
        /// <param name="enable">True to enable memory tracking</param>
        public void SetMemoryTracking(bool enable)
        {
            enableMemoryTracking = enable;

            if (enableDebugLogging)
            {
                Debug.Log($"[FpsRamProfiler] Memory tracking: {enable}");
            }
        }

        /// <summary>
        /// Get the last recorded frame time
        /// </summary>
        /// <returns>Last frame time in milliseconds</returns>
        public float GetLastFrameTime()
        {
            return lastFrameTime;
        }

        /// <summary>
        /// Get the total number of frames recorded
        /// </summary>
        /// <returns>Frame count</returns>
        public int GetFrameCount()
        {
            return frameCount;
        }

        #endregion

        #region Static Helper Methods

        /// <summary>
        /// Static helper to get FPS metrics for any context
        /// </summary>
        public static bool GetFpsMetricsStatic(ProfilerContext context, out FpsMetricsGraphy metrics)
        {
            if (Instance != null && Instance.profiler != null)
            {
                return Instance.profiler.GetFpsMetrics(context, out metrics);
            }

            metrics = default;
            return false;
        }

        /// <summary>
        /// Static helper to get memory metrics for any context
        /// </summary>
        public static bool GetMemoryMetricsStatic(ProfilerContext context, out MemoryMetricsGraphy metrics)
        {
            if (Instance != null && Instance.profiler != null)
            {
                return Instance.profiler.GetMemoryMetrics(context, out metrics);
            }

            metrics = default;
            return false;
        }

        /// <summary>
        /// Static helper to toggle profiler visibility
        /// </summary>
        public static void ToggleVisibilityStatic()
        {
            Instance?.ToggleVisibility();
        }

        #endregion

        #region Editor Methods

#if UNITY_EDITOR
        /// <summary>
        /// Validate inspector values
        /// </summary>
        private void OnValidate()
        {
            memoryUpdateInterval = Mathf.Clamp(memoryUpdateInterval, 0.1f, 5.0f);
        }
#endif

        #endregion
    }
}
