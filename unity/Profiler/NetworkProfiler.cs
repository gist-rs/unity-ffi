using System;
using System.Runtime.InteropServices;
using UnityNetwork;

namespace Unity.Profiler
{
    /// <summary>
    /// Wrapper class for Network Profiler FFI bridge
    /// Provides safe managed interface to Rust network profiler
    /// </summary>
    public unsafe class NetworkProfiler : IDisposable
    {
        #region P/Invoke Declarations

        private const string DLL_NAME = "mmorpg_profiler";

        /// <summary>
        /// Initialize the network profiler
        /// </summary>
        /// <param name="max_completed">Maximum number of completed requests to track</param>
        /// <param name="context">Profiler context (0=Unity, 1=Rust, 2=Total)</param>
        /// <returns>Pointer to tracker state, or null on failure</returns>
        [DllImport(DLL_NAME, CallingConvention = CallingConvention.Cdecl)]
        private static extern void* profiler_network_init(ulong max_completed, uint context);

        /// <summary>
        /// Start tracking a new network request
        /// </summary>
        /// <param name="state">Pointer to tracker state</param>
        /// <param name="request_type">Type of request</param>
        /// <param name="uuid_out">Output buffer for request UUID (16 bytes)</param>
        /// <returns>0=Success, -1=Invalid state, -2=Rate limit, -3=Operation failed, -99=Panic</returns>
        [DllImport(DLL_NAME, CallingConvention = CallingConvention.Cdecl)]
        private static extern int profiler_network_start_request(
            void* state,
            RequestType request_type,
            byte* uuid_out
        );

        /// <summary>
        /// Record the start of a request stage
        /// </summary>
        /// <param name="state">Pointer to tracker state</param>
        /// <param name="uuid">Request UUID (16 bytes)</param>
        /// <param name="stage">Stage identifier</param>
        /// <returns>0=Success, -1=Invalid state, -2=Request not found, -3=Stage not started, -99=Panic</returns>
        [DllImport(DLL_NAME, CallingConvention = CallingConvention.Cdecl)]
        private static extern int profiler_network_record_stage_start(
            void* state,
            byte* uuid,
            RequestStage stage
        );

        /// <summary>
        /// Record the end of a request stage
        /// </summary>
        /// <param name="state">Pointer to tracker state</param>
        /// <param name="uuid">Request UUID (16 bytes)</param>
        /// <param name="stage">Stage identifier</param>
        /// <returns>0=Success, -1=Invalid state, -2=Request not found, -3=Stage not started, -99=Panic</returns>
        [DllImport(DLL_NAME, CallingConvention = CallingConvention.Cdecl)]
        private static extern int profiler_network_record_stage_end(
            void* state,
            byte* uuid,
            RequestStage stage
        );

        /// <summary>
        /// Complete a network request
        /// </summary>
        /// <param name="state">Pointer to tracker state</param>
        /// <param name="uuid">Request UUID (16 bytes)</param>
        /// <param name="status">Final request status</param>
        /// <returns>0=Success, -1=Invalid state, -2=Request not found, -3=Operation failed, -99=Panic</returns>
        [DllImport(DLL_NAME, CallingConvention = CallingConvention.Cdecl)]
        private static extern int profiler_network_complete_request(
            void* state,
            byte* uuid,
            RequestStatus status
        );

        /// <summary>
        /// Get waterfall data for completed requests
        /// </summary>
        /// <param name="state">Pointer to tracker state</param>
        /// <param name="context">Profiler context (0=Unity, 1=Rust, 2=Total)</param>
        /// <param name="data_out">Pointer to WaterfallDataFfi struct with pre-allocated buffer</param>
        /// <returns>0=Success, -1=Invalid state, -2=Invalid context, -3=Insufficient capacity, -99=Panic</returns>
        [DllImport(DLL_NAME, CallingConvention = CallingConvention.Cdecl)]
        private static extern int profiler_network_get_waterfall(
            void* state,
            uint context,
            WaterfallData* data_out
        );

        /// <summary>
        /// Shutdown the network profiler and free resources
        /// </summary>
        /// <param name="state">Pointer to tracker state</param>
        /// <returns>0=Success, -1=Invalid state, -99=Panic</returns>
        [DllImport(DLL_NAME, CallingConvention = CallingConvention.Cdecl)]
        private static extern int profiler_network_shutdown(void* state);

        #endregion

        #region Fields

        private void* state;
        private bool disposed;
        private readonly uint maxCompletedRequests;
        private readonly ProfilerContext context;

        #endregion

        #region Constructor

        /// <summary>
        /// Create a new NetworkProfiler instance
        /// </summary>
        /// <param name="maxCompletedRequests">Maximum number of completed requests to track (default: 100)</param>
        /// <param name="context">Profiler context (default: Total)</param>
        public NetworkProfiler(uint maxCompletedRequests = 100, ProfilerContext context = ProfilerContext.Total)
        {
            this.maxCompletedRequests = maxCompletedRequests;
            this.context = context;
            state = null;
            disposed = false;

            Initialize();
        }

        /// <summary>
        /// Initialize the profiler
        /// </summary>
        private void Initialize()
        {
            state = profiler_network_init(maxCompletedRequests, (uint)context);

            if (state == null)
            {
                throw new InvalidOperationException("Failed to initialize network profiler");
            }
        }

        #endregion

        #region Public API

        /// <summary>
        /// Check if the profiler is active
        /// </summary>
        public bool IsActive => !disposed && state != null;

        /// <summary>
        /// Get the profiler context
        /// </summary>
        public ProfilerContext Context => context;

        /// <summary>
        /// Start tracking a new network request
        /// </summary>
        /// <param name="requestType">Type of request</param>
        /// <returns>Request UUID</returns>
        /// <exception cref="ObjectDisposedException">Profiler has been disposed</exception>
        /// <exception cref="InvalidOperationException">Failed to start request (rate limit or operation failed)</exception>
        public Guid StartRequest(RequestType requestType)
        {
            ThrowIfDisposed();

            byte[] uuidBytes = new byte[16];
            fixed (byte* uuidPtr = uuidBytes)
            {
                int result = profiler_network_start_request(state, requestType, uuidPtr);

                if (result != 0)
                {
                    throw new InvalidOperationException($"Failed to start request: {GetErrorString(result)}");
                }
            }

            return new Guid(uuidBytes);
        }

        /// <summary>
        /// Record the start of a request stage
        /// </summary>
        /// <param name="requestUuid">Request UUID</param>
        /// <param name="stage">Stage identifier</param>
        /// <exception cref="ObjectDisposedException">Profiler has been disposed</exception>
        /// <exception cref="ArgumentException">Request not found or stage not started</exception>
        public void RecordStageStart(Guid requestUuid, RequestStage stage)
        {
            ThrowIfDisposed();

            byte[] uuidBytes = requestUuid.ToByteArray();
            fixed (byte* uuidPtr = uuidBytes)
            {
                int result = profiler_network_record_stage_start(state, uuidPtr, stage);

                if (result != 0)
                {
                    throw new ArgumentException($"Failed to record stage start: {GetErrorString(result)}");
                }
            }
        }

        /// <summary>
        /// Record the end of a request stage
        /// </summary>
        /// <param name="requestUuid">Request UUID</param>
        /// <param name="stage">Stage identifier</param>
        /// <exception cref="ObjectDisposedException">Profiler has been disposed</exception>
        /// <exception cref="ArgumentException">Request not found or stage not started</exception>
        public void RecordStageEnd(Guid requestUuid, RequestStage stage)
        {
            ThrowIfDisposed();

            byte[] uuidBytes = requestUuid.ToByteArray();
            fixed (byte* uuidPtr = uuidBytes)
            {
                int result = profiler_network_record_stage_end(state, uuidPtr, stage);

                if (result != 0)
                {
                    throw new ArgumentException($"Failed to record stage end: {GetErrorString(result)}");
                }
            }
        }

        /// <summary>
        /// Record both start and end of a request stage (for instant stages)
        /// </summary>
        /// <param name="requestUuid">Request UUID</param>
        /// <param name="stage">Stage identifier</param>
        /// <exception cref="ObjectDisposedException">Profiler has been disposed</exception>
        /// <exception cref="ArgumentException">Request not found or stage not started</exception>
        public void RecordStage(Guid requestUuid, RequestStage stage)
        {
            RecordStageStart(requestUuid, stage);
            RecordStageEnd(requestUuid, stage);
        }

        /// <summary>
        /// Complete a network request
        /// </summary>
        /// <param name="requestUuid">Request UUID</param>
        /// <param name="status">Final request status</param>
        /// <exception cref="ObjectDisposedException">Profiler has been disposed</exception>
        /// <exception cref="ArgumentException">Request not found or operation failed</exception>
        public void CompleteRequest(Guid requestUuid, RequestStatus status)
        {
            ThrowIfDisposed();

            byte[] uuidBytes = requestUuid.ToByteArray();
            fixed (byte* uuidPtr = uuidBytes)
            {
                int result = profiler_network_complete_request(state, uuidPtr, status);

                if (result != 0)
                {
                    throw new ArgumentException($"Failed to complete request: {GetErrorString(result)}");
                }
            }
        }

        /// <summary>
        /// Get waterfall data for completed requests
        /// </summary>
        /// <param name="context">Profiler context (default: Total)</param>
        /// <returns>Waterfall data with entries</returns>
        /// <exception cref="ObjectDisposedException">Profiler has been disposed</exception>
        /// <exception cref="InvalidOperationException">Failed to get waterfall data</exception>
        public WaterfallEntry[] GetWaterfall(ProfilerContext context = ProfilerContext.Total)
        {
            ThrowIfDisposed();

            // Allocate managed memory for entries
            const uint maxEntries = 1000;
            WaterfallEntry[] entriesArray = new WaterfallEntry[maxEntries];

            // Pin the array and get a pointer
            GCHandle handle = GCHandle.Alloc(entriesArray, GCHandleType.Pinned);
            try
            {
                WaterfallData data;
                data.entries = (WaterfallEntry*)handle.AddrOfPinnedObject().ToPointer();
                data.capacity = maxEntries;
                data.length = 0;
                data.max_duration_ms = 0.0f;
                data.context = context;

                int result = profiler_network_get_waterfall(state, (uint)context, &data);

                if (result != 0)
                {
                    if (result == -3)
                    {
                        // Insufficient capacity - return as much as we got
                        WaterfallEntry[] partialResult = new WaterfallEntry[data.length];
                        Array.Copy(entriesArray, partialResult, (int)data.length);
                        return partialResult;
                    }

                    throw new InvalidOperationException($"Failed to get waterfall data: {GetErrorString(result)}");
                }

                // Copy only the actual entries
                WaterfallEntry[] resultArray = new WaterfallEntry[data.length];
                Array.Copy(entriesArray, resultArray, (int)data.length);
                return resultArray;
            }
            finally
            {
                handle.Free();
            }
        }

        /// <summary>
        /// High-level method to track a complete request with automatic stage management
        /// </summary>
        /// <param name="requestType">Type of request</param>
        /// <param name="action">Action to execute (will be timed)</param>
        /// <returns>Request UUID</returns>
        public Guid TrackRequest(RequestType requestType, Action<Guid> action)
        {
            Guid requestId = StartRequest(requestType);

            try
            {
                RecordStageStart(requestId, RequestStage.UserInput);
                RecordStageEnd(requestId, RequestStage.UserInput);

                RecordStageStart(requestId, RequestStage.UnityProcess);

                // Execute the user action
                action(requestId);

                RecordStageEnd(requestId, RequestStage.UnityProcess);

                CompleteRequest(requestId, RequestStatus.Completed);
            }
            catch
            {
                CompleteRequest(requestId, RequestStatus.Failed);
                throw;
            }

            return requestId;
        }

        #endregion

        #region IDisposable

        /// <summary>
        /// Dispose the profiler and release native resources
        /// </summary>
        public void Dispose()
        {
            Dispose(true);
            GC.SuppressFinalize(this);
        }

        /// <summary>
        /// Dispose pattern implementation
        /// </summary>
        protected virtual void Dispose(bool disposing)
        {
            if (!disposed)
            {
                if (state != null)
                {
                    profiler_network_shutdown(state);
                    state = null;
                }

                disposed = true;
            }
        }

        /// <summary>
        /// Finalizer
        /// </summary>
        ~NetworkProfiler()
        {
            Dispose(false);
        }

        #endregion

        #region Network Integration Helpers

        /// <summary>
        /// Create a packet header with a new UUID v7 for request tracking.
        /// This method generates a UUID v7 and sets it in the packet header,
        /// then starts tracking the request in the profiler.
        /// </summary>
        /// <param name="packetType">Type of the packet (e.g., PlayerPos, SpriteMessage)</param>
        /// <param name="requestType">Profiler request type to track</param>
        /// <returns>Tuple of (packet header, request UUID) for tracking</returns>
        /// <exception cref="ObjectDisposedException">Profiler has been disposed</exception>
        /// <exception cref="InvalidOperationException">Failed to start request tracking</exception>
        public unsafe (PacketHeader header, Guid uuid) CreateTrackedPacketHeader(
            byte packetType,
            RequestType requestType)
        {
            ThrowIfDisposed();

            // Generate UUID v7 for this request
            Guid requestUuid = UuidV7.NewUuid();

            // Start tracking this request in the profiler
            RecordStageStart(requestUuid, RequestStage.UserInput);
            RecordStageEnd(requestUuid, RequestStage.UserInput);

            // Create packet header with the UUID
            PacketHeader header = new PacketHeader(packetType, requestUuid);

            return (header, requestUuid);
        }

        /// <summary>
        /// Extract and track UUID from a received packet.
        /// This method extracts the UUID from the packet header and starts
        /// server-side tracking if a valid UUID is present.
        /// </summary>
        /// <param name="packetBytes">Raw packet bytes received from server</param>
        /// <param name="requestType">Type of request to track</param>
        /// <returns>Extracted UUID, or Guid.Empty if no UUID present</returns>
        /// <exception cref="ObjectDisposedException">Profiler has been disposed</exception>
        public unsafe Guid ExtractAndTrackPacketUuid(byte[] packetBytes, RequestType requestType)
        {
            ThrowIfDisposed();

            if (packetBytes == null || packetBytes.Length < 18)
            {
                return Guid.Empty;
            }

            // Extract UUID from bytes 2-17 (packet header format)
            byte[] uuidBytes = new byte[16];
            System.Array.Copy(packetBytes, 2, uuidBytes, 0, 16);

            // Check if UUID is all zeros (backward compatibility)
            bool allZeros = true;
            for (int i = 0; i < 16; i++)
            {
                if (uuidBytes[i] != 0)
                {
                    allZeros = false;
                    break;
                }
            }

            if (allZeros)
            {
                // No UUID in packet, server will generate one
                return Guid.Empty;
            }

            Guid requestUuid = new Guid(uuidBytes);

            // Start tracking this request with the client-provided UUID
            try
            {
                RecordStageStart(requestUuid, RequestStage.RustProcess);
            }
            catch
            {
                // If tracking fails (e.g., rate limit), continue anyway
                // The request can still be processed without tracking
            }

            return requestUuid;
        }

        /// <summary>
        /// Complete tracking for a received packet.
        /// This method records the end of server-side processing for a request.
        /// </summary>
        /// <param name="requestUuid">Request UUID to complete</param>
        /// <param name="status">Final request status</param>
        /// <exception cref="ObjectDisposedException">Profiler has been disposed</exception>
        public void CompleteReceivedPacketTracking(Guid requestUuid, RequestStatus status)
        {
            ThrowIfDisposed();

            if (requestUuid == Guid.Empty)
            {
                // No UUID to complete
                return;
            }

            try
            {
                RecordStageEnd(requestUuid, RequestStage.RustProcess);
                CompleteRequest(requestUuid, status);
            }
            catch
            {
                // If completion fails, continue anyway
                // The request has already been processed
            }
        }

        /// <summary>
        /// Track a complete request cycle: client send, server processing, client receive.
        /// This is a high-level method that handles the full request lifecycle.
        /// </summary>
        /// <typeparam name="TPacket">Type of packet struct to send</typeparam>
        /// <param name="packet">Packet to send (header will be modified to include UUID)</param>
        /// <param name="requestType">Profiler request type to track</param>
        /// <param name="sendAction">Action to send the packet</param>
        /// <param name="receiveAction">Action to wait for/parse response</param>
        /// <returns>Request UUID for correlation</returns>
        /// <exception cref="ObjectDisposedException">Profiler has been disposed</exception>
        public unsafe Guid TrackRequestCycle<TPacket>(
            ref TPacket packet,
            RequestType requestType,
            Action<TPacket> sendAction,
            Func<Guid, TPacket> receiveAction) where TPacket : unmanaged
        {
            ThrowIfDisposed();

            // Get packet type from header
            byte packetType = 0;
            unsafe
            {
                PacketHeader* header = (PacketHeader*)&packet;
                packetType = header->packetType;
            }

            // Create tracked header with UUID
            var (header, requestUuid) = CreateTrackedPacketHeader(packetType, requestType);

            // Update packet with tracked header
            unsafe
            {
                PacketHeader* packetHeader = (PacketHeader*)&packet;
                *packetHeader = header;
            }

            // Track Unity processing stage
            RecordStageStart(requestUuid, RequestStage.UnityProcess);

            try
            {
                // Send the packet
                sendAction(packet);

                RecordStageEnd(requestUuid, RequestStage.UnityProcess);

                // Receive and process response
                RecordStageStart(requestUuid, RequestStage.UnityReceive);

                TPacket response = receiveAction(requestUuid);

                RecordStageEnd(requestUuid, RequestStage.UnityReceive);

                CompleteRequest(requestUuid, RequestStatus.Completed);

                return requestUuid;
            }
            catch
            {
                CompleteRequest(requestUuid, RequestStatus.Failed);
                throw;
            }
        }

        #endregion

        #region Private Helpers

        /// <summary>
        /// Throw if profiler has been disposed
        /// </summary>
        private void ThrowIfDisposed()
        {
            if (disposed)
            {
                throw new ObjectDisposedException(nameof(NetworkProfiler));
            }
        }

        /// <summary>
        /// Get error description from error code
        /// </summary>
        private static string GetErrorString(int errorCode)
        {
            return errorCode switch
            {
                -1 => "Invalid state pointer",
                -2 => "Request not found or rate limit exceeded",
                -3 => "Operation failed",
                -4 => "Invalid context",
                -99 => "Panic caught in Rust code",
                _ => $"Unknown error code: {errorCode}"
            };
        }

        #endregion
    }
}
