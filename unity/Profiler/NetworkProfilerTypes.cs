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
    /// Network request type (FFI-compatible)
    /// </summary>
    [StructLayout(LayoutKind.Sequential, Pack = 1)]
    public enum RequestType : uint
    {
        /// <summary>Unknown or uncategorized request</summary>
        Unknown = 0,
        /// <summary>Movement command</summary>
        MoveCommand = 1,
        /// <summary>Shop action (buy/sell)</summary>
        ShopAction = 2,
        /// <summary>Chat message</summary>
        ChatMessage = 3,
        /// <summary>Character update</summary>
        CharacterUpdate = 4,
        /// <summary>Inventory action (equip/unequip/drop)</summary>
        InventoryAction = 5,
        /// <summary>Authentication request</summary>
        Authentication = 6
    }

    /// <summary>
    /// Network request status (FFI-compatible)
    /// </summary>
    [StructLayout(LayoutKind.Sequential, Pack = 1)]
    public enum RequestStatus : uint
    {
        /// <summary>Request is pending initialization</summary>
        Pending = 0,
        /// <summary>Request is actively being processed</summary>
        InProgress = 1,
        /// <summary>Request completed successfully</summary>
        Completed = 2,
        /// <summary>Request failed with error</summary>
        Failed = 3,
        /// <summary>Request timed out</summary>
        TimedOut = 4
    }

    /// <summary>
    /// Network request processing stage (FFI-compatible)
    /// </summary>
    [StructLayout(LayoutKind.Sequential, Pack = 1)]
    public enum RequestStage : uint
    {
        /// <summary>Initial user input (keyboard/mouse)</summary>
        UserInput = 0,
        /// <summary>Unity engine processing</summary>
        UnityProcess = 1,
        /// <summary>FFI call to Rust (outbound)</summary>
        RustFFIOutbound = 2,
        /// <summary>Server processing</summary>
        Server = 3,
        /// <summary>FFI call from Rust (inbound)</summary>
        RustFFIInbound = 4,
        /// <summary>Unity rendering</summary>
        UnityRender = 5,
        /// <summary>Final output to user</summary>
        Output = 6
    }

    /// <summary>
    /// Waterfall entry representing a single request (FFI-compatible)
    /// </summary>
    [StructLayout(LayoutKind.Sequential, Pack = 1)]
    public unsafe struct WaterfallEntry
    {
        /// <summary>Request UUID (16 bytes)</summary>
        public fixed byte request_uuid[16];

        /// <summary>Type of request</summary>
        public RequestType request_type;

        /// <summary>Request status</summary>
        public RequestStatus status;

        /// <summary>Total duration in milliseconds</summary>
        public float total_duration_ms;

        /// <summary>Number of stages recorded</summary>
        public uint stage_count;

        /// <summary>Start timestamp in nanoseconds</summary>
        public ulong start_ns;

        /// <summary>Context for this request</summary>
        public ProfilerContext context;

        /// <summary>Get the request UUID as a GUID</summary>
        public Guid GetUuid()
        {
            fixed (byte* ptr = request_uuid)
            {
                return new Guid(ptr);
            }
        }

        /// <summary>Set the request UUID from a GUID</summary>
        public void SetUuid(Guid guid)
        {
            byte[] bytes = guid.ToByteArray();
            fixed (byte* ptr = request_uuid)
            {
                for (int i = 0; i < 16; i++)
                {
                    ptr[i] = bytes[i];
                }
            }
        }
    }

    /// <summary>
    /// Complete waterfall data with raw pointers (FFI-compatible)
    /// </summary>
    [StructLayout(LayoutKind.Sequential, Pack = 1)]
    public unsafe struct WaterfallData
    {
        /// <summary>Pointer to array of WaterfallEntry (allocated by caller)</summary>
        public WaterfallEntry* entries;

        /// <summary>Capacity of the entries array (maximum number of elements)</summary>
        public uint capacity;

        /// <summary>Actual number of elements written to the array</summary>
        public uint length;

        /// <summary>Maximum duration in milliseconds (for scaling)</summary>
        public float max_duration_ms;

        /// <summary>Context for this waterfall data</summary>
        public ProfilerContext context;

        /// <summary>Get entries as a managed array</summary>
        public WaterfallEntry[] GetEntries()
        {
            WaterfallEntry[] result = new WaterfallEntry[length];
            for (uint i = 0; i < length; i++)
            {
                result[i] = entries[i];
            }
            return result;
        }
    }
}
