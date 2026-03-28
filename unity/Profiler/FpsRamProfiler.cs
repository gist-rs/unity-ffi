using System;
using System.Runtime.InteropServices;
using UnityEngine;

namespace Unity.Profiler
{
    /// <summary>
    /// Wrapper class for FPS/RAM Profiler FFI bridge
    /// Provides safe managed interface to Rust FPS/RAM profiler (Graphy-style)
    /// </summary>
    public unsafe class FpsRamProfiler : IDisposable
    {
        #region P/Invoke Declarations

        private const string DLL_NAME = "mmorpg_profiler";

        /// <summary>
        /// Initialize the FPS/RAM profiler with Graphy-style settings
        /// </summary>
        /// <returns>Pointer to profiler state, or IntPtr.Zero on failure</returns>
        [DllImport(DLL_NAME, CallingConvention = CallingConvention.Cdecl)]
        private static extern void* profiler_init_graphy_style();

        /// <summary>
        /// Record a frame with timing information
        /// </summary>
        /// <param name="state">Pointer to profiler state</param>
        /// <param name="delta_time_ms">Frame delta time in milliseconds</param>
        [DllImport(DLL_NAME, CallingConvention = CallingConvention.Cdecl)]
        private static extern void profiler_record_frame_graphy(void* state, double delta_time_ms);

        /// <summary>
        /// Submit Unity memory metrics
        /// </summary>
        /// <param name="state">Pointer to profiler state</param>
        /// <param name="allocated_mb">Allocated memory in MB</param>
        /// <param name="reserved_mb">Reserved memory in MB</param>
        /// <param name="mono_mb">Mono heap size in MB</param>
        [DllImport(DLL_NAME, CallingConvention = CallingConvention.Cdecl)]
        private static extern void profiler_submit_unity_memory(
            void* state,
            float allocated_mb,
            float reserved_mb,
            float mono_mb
        );

        /// <summary>
        /// Get FPS metrics for a specific context
        /// </summary>
        /// <param name="state">Pointer to profiler state</param>
        /// <param name="context">Profiler context (0=Unity, 1=Rust, 2=Total)</param>
        /// <param name="metrics_out">Output FpsMetricsGraphy struct</param>
        /// <returns>0=Success, -1=Invalid state, -2=Invalid context, -99=Panic</returns>
        [DllImport(DLL_NAME, CallingConvention = CallingConvention.Cdecl)]
        private static extern int profiler_get_fps_metrics(
            void* state,
            uint context,
            FpsMetricsGraphy* metrics_out
        );

        /// <summary>
        /// Get memory metrics for a specific context
        /// </summary>
        /// <param name="state">Pointer to profiler state</param>
        /// <param name="context">Profiler context (0=Unity, 1=Rust, 2=Total)</param>
        /// <param name="metrics_out">Output MemoryMetricsGraphy struct</param>
        /// <returns>0=Success, -1=Invalid state, -2=Invalid context, -99=Panic</returns>
        [DllImport(DLL_NAME, CallingConvention = CallingConvention.Cdecl)]
        private static extern int profiler_get_memory_metrics(
            void* state,
            uint context,
            MemoryMetricsGraphy* metrics_out
        );

        /// <summary>
        /// Get graph data for frame timing visualization
        /// </summary>
        /// <param name="state">Pointer to profiler state</param>
        /// <param name="context">Profiler context (0=Unity, 1=Rust, 2=Total)</param>
        /// <param name="data_out">Pointer to ProfilerGraphData struct with pre-allocated buffer</param>
        /// <returns>0=Success, -1=Invalid state, -2=Invalid context, -3=Insufficient capacity, -99=Panic</returns>
        [DllImport(DLL_NAME, CallingConvention = CallingConvention.Cdecl)]
        private static extern int profiler_get_graph_data(
            void* state,
            uint context,
            ProfilerGraphData* data_out
        );

        /// <summary>
        /// Toggle profiler visibility state
        /// </summary>
        /// <param name="state">Pointer to profiler state</param>
        /// <param name="visible">True to enable profiler, False to disable</param>
        [DllImport(DLL_NAME, CallingConvention = CallingConvention.Cdecl)]
        private static extern void profiler_toggle_visibility(void* state, bool visible);

        /// <summary>
        /// Reset all profiler metrics
        /// </summary>
        /// <param name="state">Pointer to profiler state</param>
        [DllImport(DLL_NAME, CallingConvention = CallingConvention.Cdecl)]
        private static extern void profiler_reset(void* state);

        /// <summary>
        /// Shutdown the profiler and free all resources
        /// </summary>
        /// <param name="state">Pointer to profiler state</param>
        [DllImport(DLL_NAME, CallingConvention = CallingConvention.Cdecl)]
        private static extern void profiler_shutdown(void* state);

        #endregion

        #region Fields

        private void* state;
        private bool disposed;

        #endregion

        #region Constructor

        /// <summary>
        /// Create a new FpsRamProfiler instance
        /// </summary>
        public FpsRamProfiler()
        {
            state = null;
            disposed = false;

            Initialize();
        }

        /// <summary>
        /// Initialize the profiler
        /// </summary>
        private void Initialize()
        {
            state = profiler_init_graphy_style();

            if (state == null)
            {
                throw new InvalidOperationException("Failed to initialize FPS/RAM profiler");
            }

            // Enable profiler by default
            profiler_toggle_visibility(state, true);
        }

        #endregion

        #region Properties

        /// <summary>
        /// Check if the profiler is properly initialized
        /// </summary>
        public bool IsInitialized => state != null;

        /// <summary>
        /// Check if the profiler has been disposed
        /// </summary>
        public bool IsDisposed => disposed;

        #endregion

        #region Public API - Frame Recording

        /// <summary>
        /// Record a frame with timing information
        /// </summary>
        /// <param name="deltaTimeMs">Frame delta time in milliseconds</param>
        /// <returns>True if successful, False if profiler is disposed</returns>
        public bool RecordFrame(double deltaTimeMs)
        {
            if (disposed || state == null)
            {
                return false;
            }

            try
            {
                profiler_record_frame_graphy(state, deltaTimeMs);
                return true;
            }
            catch (Exception ex)
            {
                Debug.LogError($"[FpsRamProfiler] Failed to record frame: {ex.Message}");
                return false;
            }
        }

        /// <summary>
        /// Submit Unity memory metrics
        /// </summary>
        /// <param name="allocatedMb">Allocated memory in MB</param>
        /// <param name="reservedMb">Reserved memory in MB</param>
        /// <param name="monoMb">Mono heap size in MB</param>
        /// <returns>True if successful, False if profiler is disposed</returns>
        public bool SubmitUnityMemory(float allocatedMb, float reservedMb, float monoMb)
        {
            if (disposed || state == null)
            {
                return false;
            }

            try
            {
                profiler_submit_unity_memory(state, allocatedMb, reservedMb, monoMb);
                return true;
            }
            catch (Exception ex)
            {
                Debug.LogError($"[FpsRamProfiler] Failed to submit memory metrics: {ex.Message}");
                return false;
            }
        }

        #endregion

        #region Public API - Metrics Retrieval

        /// <summary>
        /// Get FPS metrics for a specific context
        /// </summary>
        /// <param name="context">Profiler context</param>
        /// <param name="metrics">Output FPS metrics</param>
        /// <returns>True if successful, False on error</returns>
        public bool GetFpsMetrics(ProfilerContext context, out FpsMetricsGraphy metrics)
        {
            metrics = default;

            if (disposed || state == null)
            {
                return false;
            }

            try
            {
                int result = profiler_get_fps_metrics(state, (uint)context, &metrics);

                if (result != 0)
                {
                    Debug.LogWarning($"[FpsRamProfiler] GetFpsMetrics failed with code: {result}");
                    return false;
                }

                return true;
            }
            catch (Exception ex)
            {
                Debug.LogError($"[FpsRamProfiler] Failed to get FPS metrics: {ex.Message}");
                return false;
            }
        }

        /// <summary>
        /// Get memory metrics for a specific context
        /// </summary>
        /// <param name="context">Profiler context</param>
        /// <param name="metrics">Output memory metrics</param>
        /// <returns>True if successful, False on error</returns>
        public bool GetMemoryMetrics(ProfilerContext context, out MemoryMetricsGraphy metrics)
        {
            metrics = default;

            if (disposed || state == null)
            {
                return false;
            }

            try
            {
                int result = profiler_get_memory_metrics(state, (uint)context, &metrics);

                if (result != 0)
                {
                    Debug.LogWarning($"[FpsRamProfiler] GetMemoryMetrics failed with code: {result}");
                    return false;
                }

                return true;
            }
            catch (Exception ex)
            {
                Debug.LogError($"[FpsRamProfiler] Failed to get memory metrics: {ex.Message}");
                return false;
            }
        }

        /// <summary>
        /// Get graph data for frame timing visualization
        /// </summary>
        /// <param name="context">Profiler context</param>
        /// <param name="data">Output graph data with pre-allocated buffer</param>
        /// <returns>True if successful, False on error</returns>
        public bool GetGraphData(ProfilerContext context, ref ProfilerGraphData data)
        {
            if (disposed || state == null)
            {
                return false;
            }

            if (data.values == null)
            {
                Debug.LogError("[FpsRamProfiler] Graph data buffer is not allocated");
                return false;
            }

            try
            {
                int result = profiler_get_graph_data(state, (uint)context, &data);

                if (result != 0)
                {
                    Debug.LogWarning($"[FpsRamProfiler] GetGraphData failed with code: {result}");
                    return false;
                }

                return true;
            }
            catch (Exception ex)
            {
                Debug.LogError($"[FpsRamProfiler] Failed to get graph data: {ex.Message}");
                return false;
            }
        }

        #endregion

        #region Public API - Control

        /// <summary>
        /// Toggle profiler visibility
        /// </summary>
        /// <param name="visible">True to enable profiler, False to disable</param>
        /// <returns>True if successful, False if profiler is disposed</returns>
        public bool SetVisibility(bool visible)
        {
            if (disposed || state == null)
            {
                return false;
            }

            try
            {
                profiler_toggle_visibility(state, visible);
                return true;
            }
            catch (Exception ex)
            {
                Debug.LogError($"[FpsRamProfiler] Failed to set visibility: {ex.Message}");
                return false;
            }
        }

        /// <summary>
        /// Reset all profiler metrics
        /// </summary>
        /// <returns>True if successful, False if profiler is disposed</returns>
        public bool Reset()
        {
            if (disposed || state == null)
            {
                return false;
            }

            try
            {
                profiler_reset(state);
                return true;
            }
            catch (Exception ex)
            {
                Debug.LogError($"[FpsRamProfiler] Failed to reset profiler: {ex.Message}");
                return false;
            }
        }

        #endregion

        #region IDisposable Implementation

        /// <summary>
        /// Cleanup profiler resources
        /// </summary>
        public void Dispose()
        {
            Dispose(true);
            GC.SuppressFinalize(this);
        }

        /// <summary>
        /// Protected dispose implementation
        /// </summary>
        protected virtual void Dispose(bool disposing)
        {
            if (disposed)
            {
                return;
            }

            if (disposing)
            {
                // Dispose managed resources
            }

            // Free unmanaged resources
            if (state != null)
            {
                try
                {
                    profiler_shutdown(state);
                    state = null;
                }
                catch (Exception ex)
                {
                    Debug.LogError($"[FpsRamProfiler] Error during shutdown: {ex.Message}");
                }
            }

            disposed = true;
        }

        /// <summary>
        /// Finalizer
        /// </summary>
        ~FpsRamProfiler()
        {
            Dispose(false);
        }

        #endregion
    }
}
