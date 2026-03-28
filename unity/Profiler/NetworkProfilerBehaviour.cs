using UnityEngine;
using Unity.Profiler;

namespace Unity.Profiler
{
    /// <summary>
    /// MonoBehaviour integration for Network Profiler
    /// Provides automatic lifecycle management and Unity-specific profiling hooks
    /// </summary>
    [AddComponentMenu("Profiler/Network Profiler Behaviour")]
    [DisallowMultipleComponent]
    public class NetworkProfilerBehaviour : MonoBehaviour
    {
        #region Settings

        [Header("Profiler Settings")]
        [Tooltip("Maximum number of completed requests to track in the circular buffer")]
        [SerializeField]
        private uint maxCompletedRequests = 100;

        [Tooltip("Profiler context (Unity, Rust, or Total)")]
        [SerializeField]
        private ProfilerContext profilerContext = ProfilerContext.Total;

        [Header("Debug Settings")]
        [Tooltip("Enable debug logging for profiler operations")]
        [SerializeField]
        private bool enableDebugLogging = true;

        [Tooltip("Show profiler information in console")]
        [SerializeField]
        private bool showProfilerInfo = true;

        #endregion

        #region Static Access

        /// <summary>
        /// Global profiler instance accessible from anywhere in the code
        /// </summary>
        public static NetworkProfilerBehaviour Instance { get; private set; }

        /// <summary>
        /// Access to the underlying NetworkProfiler
        /// </summary>
        public static NetworkProfiler Profiler => Instance?.profiler;

        #endregion

        #region Fields

        private NetworkProfiler profiler;

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
                Debug.LogWarning("[NetworkProfiler] Multiple NetworkProfilerBehaviour instances detected. Destroying duplicate.");
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
            if (showProfilerInfo)
            {
                Debug.Log($"[NetworkProfiler] Initialized with context: {profilerContext}, max requests: {maxCompletedRequests}");
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

        #endregion

        #region Public API

        /// <summary>
        /// Check if the profiler is ready to use
        /// </summary>
        public bool IsProfilerReady => profiler != null && profiler.IsActive;

        /// <summary>
        /// Get the profiler context
        /// </summary>
        public ProfilerContext Context => profilerContext;

        #endregion

        #region Convenience Methods

        /// <summary>
        /// Track a complete request with automatic stage management
        /// </summary>
        /// <param name="requestType">Type of request</param>
        /// <param name="action">Action to execute (will be timed)</param>
        /// <returns>Request UUID</returns>
        public Guid TrackRequest(RequestType requestType, System.Action<Guid> action)
        {
            if (!IsProfilerReady)
            {
                Debug.LogError("[NetworkProfiler] Attempted to track request but profiler is not ready");
                return Guid.Empty;
            }

            return profiler.TrackRequest(requestType, requestId =>
            {
                try
                {
                    // Record Unity stage automatically
                    profiler.RecordStageStart(requestId, RequestStage.UnityProcess);

                    // Execute user action
                    action(requestId);

                    // Record Unity render stage
                    profiler.RecordStage(requestId, RequestStage.UnityRender);
                }
                catch (System.Exception ex)
                {
                    Debug.LogError($"[NetworkProfiler] Error tracking request: {ex.Message}");
                    throw;
                }
                finally
                {
                    profiler.RecordStageEnd(requestId, RequestStage.UnityProcess);
                }
            });
        }

        /// <summary>
        /// Start tracking a new request
        /// </summary>
        /// <param name="requestType">Type of request</param>
        /// <returns>Request UUID, or empty if profiler not ready</returns>
        public Guid StartRequest(RequestType requestType)
        {
            if (!IsProfilerReady)
            {
                Debug.LogError("[NetworkProfiler] Attempted to start request but profiler is not ready");
                return Guid.Empty;
            }

            return profiler.StartRequest(requestType);
        }

        /// <summary>
        /// Complete a request with status
        /// </summary>
        /// <param name="requestUuid">Request UUID</param>
        /// <param name="status">Final request status</param>
        public void CompleteRequest(Guid requestUuid, RequestStatus status)
        {
            if (!IsProfilerReady)
            {
                Debug.LogError("[NetworkProfiler] Attempted to complete request but profiler is not ready");
                return;
            }

            profiler.CompleteRequest(requestUuid, status);
        }

        /// <summary>
        /// Record the start of a request stage
        /// </summary>
        /// <param name="requestUuid">Request UUID</param>
        /// <param name="stage">Stage identifier</param>
        public void RecordStageStart(Guid requestUuid, RequestStage stage)
        {
            if (!IsProfilerReady) return;

            try
            {
                profiler.RecordStageStart(requestUuid, stage);
                LogDebug($"Stage start: {stage} for request {requestUuid}");
            }
            catch (System.Exception ex)
            {
                Debug.LogError($"[NetworkProfiler] Failed to record stage start: {ex.Message}");
            }
        }

        /// <summary>
        /// Record the end of a request stage
        /// </summary>
        /// <param name="requestUuid">Request UUID</param>
        /// <param name="stage">Stage identifier</param>
        public void RecordStageEnd(Guid requestUuid, RequestStage stage)
        {
            if (!IsProfilerReady) return;

            try
            {
                profiler.RecordStageEnd(requestUuid, stage);
                LogDebug($"Stage end: {stage} for request {requestUuid}");
            }
            catch (System.Exception ex)
            {
                Debug.LogError($"[NetworkProfiler] Failed to record stage end: {ex.Message}");
            }
        }

        /// <summary>
        /// Get waterfall data for completed requests
        /// </summary>
        /// <param name="context">Profiler context (default: Total)</param>
        /// <returns>Waterfall entries, or empty array if profiler not ready</returns>
        public WaterfallEntry[] GetWaterfall(ProfilerContext context = ProfilerContext.Total)
        {
            if (!IsProfilerReady)
            {
                Debug.LogError("[NetworkProfiler] Attempted to get waterfall but profiler is not ready");
                return System.Array.Empty<WaterfallEntry>();
            }

            return profiler.GetWaterfall(context);
        }

        #endregion

        #region Private Methods

        /// <summary>
        /// Initialize the profiler
        /// </summary>
        private void InitializeProfiler()
        {
            try
            {
                profiler = new NetworkProfiler(maxCompletedRequests, profilerContext);
                LogDebug($"Network profiler initialized with context: {profilerContext}");
            }
            catch (System.Exception ex)
            {
                Debug.LogError($"[NetworkProfiler] Failed to initialize profiler: {ex.Message}");
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
                    profiler.Dispose();
                    LogDebug("Network profiler shutdown");
                }
                catch (System.Exception ex)
                {
                    Debug.LogError($"[NetworkProfiler] Error during profiler shutdown: {ex.Message}");
                }
                finally
                {
                    profiler = null;
                }
            }
        }

        /// <summary>
        /// Log debug message if logging is enabled
        /// </summary>
        private void LogDebug(string message)
        {
            if (enableDebugLogging)
            {
                Debug.Log($"[NetworkProfiler] {message}");
            }
        }

        #endregion

        #region Static Helpers

        /// <summary>
        /// Static helper to track a request (convenience method)
        /// </summary>
        /// <param name="requestType">Type of request</param>
        /// <param name="action">Action to execute</param>
        /// <returns>Request UUID, or empty if profiler not ready</returns>
        public static Guid Track(RequestType requestType, System.Action<Guid> action)
        {
            return Instance?.TrackRequest(requestType, action) ?? Guid.Empty;
        }

        /// <summary>
        /// Static helper to start a request (convenience method)
        /// </summary>
        /// <param name="requestType">Type of request</param>
        /// <returns>Request UUID, or empty if profiler not ready</returns>
        public static Guid Start(RequestType requestType)
        {
            return Instance?.StartRequest(requestType) ?? Guid.Empty;
        }

        /// <summary>
        /// Static helper to complete a request (convenience method)
        /// </summary>
        /// <param name="requestUuid">Request UUID</param>
        /// <param name="status">Final request status</param>
        public static void Complete(Guid requestUuid, RequestStatus status)
        {
            Instance?.CompleteRequest(requestUuid, status);
        }

        #endregion
    }
}
