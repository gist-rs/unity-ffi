using System;
using System.Runtime.InteropServices;

namespace Unity.Profiler
{
    /// <summary>
    /// Profiler context for metrics collection (FFI-compatible)
    /// </summary>
    [StructLayout(LayoutKind.Sequential, Pack = 1)]
    public enum ProfilerContext : uint
    {
        /// <summary>Unity engine metrics only</summary>
        Unity = 0,
        /// <summary>Rust backend metrics only</summary>
        Rust = 1,
        /// <summary>Combined metrics (Unity + Rust)</summary>
        Total = 2
    }

    /// <summary>
    /// FPS metrics matching Graphy's G_FpsMonitor properties (FFI-compatible)
    /// </summary>
    [StructLayout(LayoutKind.Sequential, Pack = 1)]
    public struct FpsMetricsGraphy
    {
        /// <summary>Current FPS (frames per second)</summary>
        public float current_fps;

        /// <summary>Average FPS over the rolling window</summary>
        public float avg_fps;

        /// <summary>1% low FPS (worst 1% of frames) - Graphy's OnePercentFPS</summary>
        public float one_percent_low;

        /// <summary>0.1% low FPS (worst 0.1% of frames) - Graphy's Zero1PercentFps</summary>
        public float zero1_percent_low;
    }

    /// <summary>
    /// Memory usage metrics matching Graphy's G_RamMonitor properties (FFI-compatible)
    /// </summary>
    [StructLayout(LayoutKind.Sequential, Pack = 1)]
    public struct MemoryMetricsGraphy
    {
        /// <summary>Allocated memory in MB</summary>
        public float allocated_mb;

        /// <summary>Reserved memory in MB</summary>
        public float reserved_mb;

        /// <summary>Mono heap size in MB</summary>
        public float mono_mb;
    }

    /// <summary>
    /// Graph data for frame timing visualization (FFI-compatible)
    /// Contains 512-element array for shader rendering
    /// </summary>
    [StructLayout(LayoutKind.Sequential, Pack = 1)]
    public unsafe struct ProfilerGraphData
    {
        /// <summary>Pointer to float array of frame times (allocated by caller)</summary>
        public float* values;

        /// <summary>Capacity of the values array</summary>
        public uint length;

        /// <summary>Average frame time in milliseconds</summary>
        public float average;

        /// <summary>Good threshold in milliseconds (< 16.6ms for 60 FPS)</summary>
        public float good_threshold;

        /// <summary>Caution threshold in milliseconds (< 33.3ms for 30 FPS)</summary>
        public float caution_threshold;

        /// <summary>Get values as a managed array</summary>
        public float[] GetValues()
        {
            float[] result = new float[length];
            for (uint i = 0; i < length; i++)
            {
                result[i] = values[i];
            }
            return result;
        }
    }

    /// <summary>
    /// Frame timing information for spike detection (FFI-compatible)
    /// </summary>
    [StructLayout(LayoutKind.Sequential, Pack = 1)]
    public struct FrameTimingGraphy
    {
        /// <summary>Frame number</summary>
        public ulong frame_number;

        /// <summary>Duration in milliseconds</summary>
        public float duration_ms;

        /// <summary>True if this frame is a spike (> 2x average)</summary>
        [MarshalAs(UnmanagedType.I1)]
        public bool is_spike;
    }

    /// <summary>
    /// Complete profiler snapshot (FFI-compatible)
    /// Contains all metrics for a given context at a point in time
    /// </summary>
    [StructLayout(LayoutKind.Sequential, Pack = 1)]
    public struct ProfilerSnapshot
    {
        /// <summary>Profiler context (Unity, Rust, or Total)</summary>
        public ProfilerContext context;

        /// <summary>Timestamp in nanoseconds since Unix epoch</summary>
        public ulong timestamp_ns;

        /// <summary>FPS metrics</summary>
        public FpsMetricsGraphy fps;

        /// <summary>Memory metrics</summary>
        public MemoryMetricsGraphy memory;

        /// <summary>Number of frame timings in the buffer</summary>
        public uint frame_timing_count;
    }

    /// <summary>
    /// Request for profiler data (FFI-compatible)
    /// </summary>
    [StructLayout(LayoutKind.Sequential, Pack = 1)]
    public struct ProfilerDataRequest
    {
        /// <summary>Profiler context to query</summary>
        public ProfilerContext context;

        /// <summary>Maximum number of frame timings to retrieve</summary>
        public uint max_frame_timings;
    }

    /// <summary>
    /// Response from profiler data request (FFI-compatible)
    /// </summary>
    [StructLayout(LayoutKind.Sequential, Pack = 1)]
    public struct ProfilerDataResponse
    {
        /// <summary>Profiler snapshot with all metrics</summary>
        public ProfilerSnapshot snapshot;

        /// <summary>Status code (0 = success, non-zero = error)</summary>
        public int status;
    }

    /// <summary>
    /// Unity FPS data for FFI submission (FFI-compatible)
    /// </summary>
    [StructLayout(LayoutKind.Sequential, Pack = 1)]
    public struct UnityFpsDataGraphy
    {
        /// <summary>Frame time in milliseconds</summary>
        public float frame_time_ms;

        /// <summary>Allocated memory in MB</summary>
        public float allocated_mb;

        /// <summary>Reserved memory in MB</summary>
        public float reserved_mb;

        /// <summary>Mono heap size in MB</summary>
        public float mono_mb;
    }
}
