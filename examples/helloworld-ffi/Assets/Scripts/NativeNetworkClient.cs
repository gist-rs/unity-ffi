using System;
using System.Runtime.InteropServices;
using System.Text;

namespace UnityNetwork
{
    #region Error Codes

    /// <summary>
    /// Error codes returned by FFI functions.
    /// </summary>
    public enum FfiError
    {
        Success = 0,
        InvalidPointer = -1,
        InvalidMagic = -2,
        UnknownPacketType = -3,
        BufferTooSmall = -4,
        Disconnected = -5,
        AlreadyConnected = -6,
        InvalidUrl = -7,
        CertValidationFailed = -8,
        PanicCaught = -99
    }

    #endregion

    #region Structs (Must match Rust repr(C) exactly)

    /// <summary>
    /// Common header for all packets.
    /// Must match Rust's PacketHeader exactly (bit-wise).
    /// </summary>
    [StructLayout(LayoutKind.Sequential, Pack = 1)]
    public unsafe struct PacketHeader
    {
        public byte packetType;
        public byte magic;

        /// <summary>
        /// Request UUID for end-to-end correlation (16 bytes).
        /// If all zeros, the server will generate a new UUID.
        /// </summary>
        public fixed byte request_uuid[16];

        public const byte MAGIC = 0xCC;

        /// <summary>
        /// Create a new packet header (UUID defaults to zeros for backward compatibility).
        /// </summary>
        public PacketHeader(byte packetType)
        {
            this.packetType = packetType;
            this.magic = MAGIC;

            // Initialize UUID to zeros (fixed buffer in unsafe struct doesn't need fixed statement)
            for (int i = 0; i < 16; i++)
            {
                request_uuid[i] = 0;
            }
        }

        /// <summary>
        /// Create a new packet header with a request UUID.
        /// </summary>
        [Obsolete("Use PacketBuilder.CreatePlayerPos() instead. Rust now handles UUID generation internally.", true)]
        public PacketHeader(byte packetType, Guid uuid)
        {
            this.packetType = packetType;
            this.magic = MAGIC;
            SetUuid(uuid);
        }

        /// <summary>
        /// Check if header is valid (magic number matches).
        /// </summary>
        public bool IsValid()
        {
            return magic == MAGIC;
        }

        /// <summary>
        /// Get request UUID as a Guid.
        /// Returns Guid.Empty if UUID is all zeros (backward compatibility mode).
        /// </summary>
        [Obsolete("Use PacketBuilder instead. Rust now handles UUID generation internally.", true)]
        public Guid GetUuid()
        {
            byte[] uuidBytes = new byte[16];
            bool allZeros = true;
            for (int i = 0; i < 16; i++)
            {
                uuidBytes[i] = request_uuid[i];
                if (request_uuid[i] != 0)
                {
                    allZeros = false;
                }
            }
            return allZeros ? Guid.Empty : new Guid(uuidBytes);
        }

        /// <summary>
        /// Set request UUID.
        /// </summary>
        [Obsolete("Use PacketBuilder instead. Rust now handles UUID generation internally.", true)]
        public void SetUuid(Guid uuid)
        {
            byte[] uuidBytes = uuid.ToByteArray();
            for (int i = 0; i < 16; i++)
            {
                request_uuid[i] = uuidBytes[i];
            }
        }

        /// <summary>
        /// Check if this header has a valid request UUID (not all zeros).
        /// </summary>
        [Obsolete("Use PacketBuilder instead. Rust now handles UUID generation internally.", true)]
        public bool HasUuid()
        {
            for (int i = 0; i < 16; i++)
            {
                if (request_uuid[i] != 0)
                {
                    return true;
                }
            }
            return false;
        }

        /// <summary>
        /// Serialize header to bytes (18 bytes: packetType + magic + request_uuid[16]).
        /// </summary>
        [Obsolete("Use PacketBuilder instead. Rust now handles serialization internally.", true)]
        public byte[] ToBytes()
        {
            byte[] bytes = new byte[18];
            bytes[0] = packetType;
            bytes[1] = magic;
            for (int i = 0; i < 16; i++)
            {
                bytes[2 + i] = request_uuid[i];
            }
            return bytes;
        }

        /// <summary>
        /// Create PacketHeader from bytes.
        /// Returns null if bytes length is incorrect or magic number doesn't match.
        /// </summary>
        [Obsolete("Use PacketBuilder instead. Rust now handles serialization internally.", true)]
        public static PacketHeader? FromBytes(byte[] bytes)
        {
            if (bytes.Length < 18)
            {
                return null;
            }

            PacketHeader header = new PacketHeader(bytes[0]);
            header.magic = bytes[1];

            for (int i = 0; i < 16; i++)
            {
                header.request_uuid[i] = bytes[2 + i];
            }

            if (!header.IsValid())
            {
                return null;
            }

            return header;
        }
    }

    /// <summary>
    /// Player position update packet.
    /// Layout: header (18 bytes) + padding (2 bytes) + id (4) + x (4) + y (4) = 32 bytes
    /// </summary>
    [StructLayout(LayoutKind.Sequential, Pack = 1)]
    public unsafe struct PlayerPos
    {
        public PacketHeader header;
        private ushort padding; // Padding for 4-byte alignment (Rust repr(C))
        public uint id;
        public float x;
        public float y;

        public PlayerPos(uint id, float x, float y)
        {
            this.header = new PacketHeader((byte)PacketType.PlayerPos);
            this.padding = 0; // Initialize padding to zero
            this.id = id;
            this.x = x;
            this.y = y;
        }

        public bool Validate()
        {
            return header.IsValid() && header.packetType == (byte)PacketType.PlayerPos;
        }
    }

    /// <summary>
    /// GameState snapshot packet.
    /// Layout: header (18 bytes) + padding (2 bytes) + tick (4) + playerCount (4) + reserved (8) = 36 bytes
    /// </summary>
    [StructLayout(LayoutKind.Sequential, Pack = 1)]
    public unsafe struct GameState
    {
        public PacketHeader header;
        private ushort padding; // Padding for 4-byte alignment (Rust repr(C))
        public uint tick;
        public uint playerCount;
        public fixed byte reserved[8]; // Padding for future expansion

        public GameState(uint tick, uint playerCount)
        {
            this.header = new PacketHeader((byte)PacketType.GameState);
            this.padding = 0; // Initialize padding to zero
            this.tick = tick;
            this.playerCount = playerCount;
            // reserved is initialized to zeros by default
        }

        public bool Validate()
        {
            return header.IsValid() && header.packetType == (byte)PacketType.GameState;
        }
    }

    #endregion

    #region Sprite Types

    /// <summary>
    /// Sprite operation types
    /// </summary>
    public enum SpriteOp : byte
    {
        Create = 0,
        Update = 1,
        Delete = 2,
        Snapshot = 3,
    }

    /// <summary>
    /// Sprite type enum (extensible for future types)
    /// </summary>
    public enum SpriteType : byte
    {
        Serrif = 0,
    }

    /// <summary>
    /// Sprite message packet (zero-copy, Pack=1)
    /// Layout: header(18) + operation(1) + padding1(1) + sprite_type(1) + padding2(3) + id(16) + x(2) + y(2) + padding3(2) = 46 bytes
    /// Matches Rust repr(C) exactly
    /// </summary>
    [StructLayout(LayoutKind.Sequential, Pack = 1)]
    public unsafe struct SpriteMessage
    {
        public PacketHeader header;
        public byte operation;
        private byte padding1;
        public byte sprite_type;
        private fixed byte padding2[3];
        public fixed byte id[16]; // UUID as 16 bytes
        public short x;
        public short y;
        private fixed byte padding3[2];

        public SpriteMessage(byte operation, byte spriteType, Guid id, short x, short y)
        {
            this.header = new PacketHeader((byte)PacketType.SpriteMessage);
            this.operation = operation;
            this.padding1 = 0;
            this.sprite_type = spriteType;

            // Initialize padding2 to zeros
            for (int i = 0; i < 3; i++)
            {
                padding2[i] = 0;
            }

            // Convert GUID to byte array (Big Endian to match Rust)
            byte[] guidBytes = id.ToByteArray();
            for (int i = 0; i < 16; i++)
            {
                this.id[i] = guidBytes[i];
            }

            this.x = x;
            this.y = y;

            // Initialize padding3 to zeros
            for (int i = 0; i < 2; i++)
            {
                padding3[i] = 0;
            }
        }

        /// <summary>
        /// Get sprite ID as GUID
        /// </summary>
        public Guid GetId()
        {
            fixed (byte* ptr = id)
            {
                byte[] guidBytes = new byte[16];
                for (int i = 0; i < 16; i++)
                {
                    guidBytes[i] = ptr[i];
                }
                return new Guid(guidBytes);
            }
        }

        /// <summary>
        /// Get operation type
        /// </summary>
        public SpriteOp GetOperation()
        {
            return (SpriteOp)operation;
        }

        /// <summary>
        /// Get sprite type
        /// </summary>
        public SpriteType GetSpriteType()
        {
            return (SpriteType)sprite_type;
        }

        /// <summary>
        /// Create a CREATE message
        /// </summary>
        public static SpriteMessage Create(SpriteType spriteType, Guid id, short x, short y)
        {
            return new SpriteMessage((byte)SpriteOp.Create, (byte)spriteType, id, x, y);
        }

        /// <summary>
        /// Create an UPDATE message
        /// </summary>
        public static SpriteMessage Update(Guid id, short x, short y)
        {
            return new SpriteMessage((byte)SpriteOp.Update, (byte)SpriteType.Serrif, id, x, y);
        }

        /// <summary>
        /// Create a DELETE message
        /// </summary>
        public static SpriteMessage Delete(Guid id)
        {
            return new SpriteMessage((byte)SpriteOp.Delete, (byte)SpriteType.Serrif, id, 0, 0);
        }

        /// <summary>
        /// Create a SNAPSHOT message
        /// </summary>
        public static SpriteMessage Snapshot()
        {
            return new SpriteMessage((byte)SpriteOp.Snapshot, (byte)SpriteType.Serrif, Guid.Empty, 0, 0);
        }

        /// <summary>
        /// Validate the sprite message
        /// </summary>
        public bool Validate()
        {
            return header.IsValid() && header.packetType == (byte)PacketType.SpriteMessage;
        }
    }

    #endregion

    #region Packet Types

    public enum PacketType : byte
    {
        KeepAlive = 0,
        PlayerPos = 1,
        GameState = 2,
        SpriteMessage = 3,
        _Max = 255
    }

    #endregion

    /// <summary>
    /// Low-level FFI bridge for Unity-Network Rust library.
    /// Provides unsafe access to WebTransport functionality via P/Invoke.
    /// </summary>
    public unsafe class NativeNetworkClient : IDisposable
    {
        #region P/Invoke Declarations

        private const string DLL_NAME = "unity_network";

        // Log callback delegate
        [UnmanagedFunctionPointer(CallingConvention.Cdecl)]
        private delegate void LogCallback(byte* level, byte* message);

        // Import Rust FFI functions
        [DllImport(DLL_NAME, CallingConvention = CallingConvention.Cdecl)]
        private static extern int network_init(LogCallback logCallback);

        [DllImport(DLL_NAME, CallingConvention = CallingConvention.Cdecl)]
        private static extern void* network_connect(byte* url, byte* certHash, uint protocolVersion);

        [DllImport(DLL_NAME, CallingConvention = CallingConvention.Cdecl)]
        private static extern int network_send(void* ctx, byte* dataPtr, ulong dataLen);

        [DllImport(DLL_NAME, CallingConvention = CallingConvention.Cdecl)]
        private static extern int network_poll(void* ctx, byte* outPtr, ulong capacity);

        [DllImport(DLL_NAME, CallingConvention = CallingConvention.Cdecl)]
        private static extern int network_destroy(void* ctx);

        #endregion

        #region Fields

        private void* context;
        private byte[] receiveBuffer;
        private const uint PROTOCOL_VERSION = 1;
        private const int RECEIVE_BUFFER_SIZE = 4096;
        private bool disposed;
        private LogCallback logCallback; // Keep reference to prevent GC

        #endregion

        #region Constructor

        public NativeNetworkClient()
        {
            context = null;
            receiveBuffer = new byte[RECEIVE_BUFFER_SIZE];
        }

        #endregion

        #region Initialization

        /// <summary>
        /// Initialize the FFI library with logging callback.
        /// Call this before using any other functions.
        /// </summary>
        public void Initialize()
        {
            // Store callback in field to prevent garbage collection
            logCallback = (level, message) =>
            {
                try
                {
                    string levelStr = MarshalPtrToString(level);
                    string msgStr = MarshalPtrToString(message);
                    UnityEngine.Debug.Log($"[{levelStr}] {msgStr}");
                }
                catch
                {
                    // Don't throw in logging callback
                }
            };

            int result = network_init(logCallback);
            if (result != (int)UnityNetwork.FfiError.Success)
            {
                throw new InvalidOperationException($"Failed to initialize network library. Error: {(UnityNetwork.FfiError)result}");
            }

            UnityEngine.Debug.Log("Unity-Network FFI initialized successfully");
        }

        #endregion

        #region Connection

        /// <summary>
        /// Connect to WebTransport server.
        /// </summary>
        /// <param name="url">Server URL (e.g., "https://127.0.0.1:4433")</param>
        /// <param name="certHash">SHA-256 hash of server certificate (hex string)</param>
        public void Connect(string url, string certHash = null)
        {
            if (context != null)
            {
                throw new InvalidOperationException("Already connected");
            }

            // Convert strings to UTF-8 byte arrays with null terminator for C interop
            byte[] urlBytes = Encoding.UTF8.GetBytes(url + "\0");
            byte[] certHashBytes = certHash != null ? Encoding.UTF8.GetBytes(certHash + "\0") : null;

            // Pin arrays to get pointers
            fixed (byte* urlPtr = urlBytes)
            fixed (byte* certHashPtr = certHashBytes)
            {
                context = network_connect(urlPtr, certHashPtr, PROTOCOL_VERSION);
            }

            if (context == null)
            {
                throw new InvalidOperationException("Failed to connect to server");
            }

            UnityEngine.Debug.Log($"Connected to {url}");
        }

        /// <summary>
        /// Check if connected to server.
        /// </summary>
        public bool IsConnected => context != null;

        #endregion

        #region Sending

        /// <summary>
        /// Send a raw byte array to the server.
        /// </summary>
        public void Send(byte[] data)
        {
            if (context == null)
            {
                throw new InvalidOperationException("Not connected");
            }

            if (data == null || data.Length == 0)
            {
                throw new ArgumentException("Data cannot be null or empty");
            }

            fixed (byte* dataPtr = data)
            {
                int result = network_send(context, dataPtr, (ulong)data.Length);

                if (result != (int)UnityNetwork.FfiError.Success)
                {
                    UnityNetwork.FfiError error = (UnityNetwork.FfiError)result;
                    if (error == UnityNetwork.FfiError.Disconnected)
                    {
                        context = null;
                        throw new InvalidOperationException("Disconnected from server");
                    }
                    throw new InvalidOperationException($"Send failed: {error}");
                }
            }
        }

        /// <summary>
        /// Send a struct to server using fixed memory.
        /// Zero-copy and no GC pressure.
        /// NOTE: Consider using PacketBuilder.Create*() methods instead for new code.
        /// </summary>
        [Obsolete("Consider using PacketBuilder.Create*() methods for new code.", false)]
        public void SendStruct<T>(T data) where T : unmanaged
        {
            // Use unsafe pointer without fixed since unmanaged types can be addressed directly
            SendStruct(&data, (ulong)sizeof(T));
        }

        /// <summary>
        /// Send data from a pointer with specified length.
        /// </summary>
        public void SendStruct(void* dataPtr, ulong length)
        {
            if (context == null)
            {
                throw new InvalidOperationException("Not connected");
            }

            if (dataPtr == null)
            {
                throw new ArgumentException("Data pointer cannot be null");
            }

            int result = network_send(context, (byte*)dataPtr, length);

            if (result != (int)UnityNetwork.FfiError.Success)
            {
                UnityNetwork.FfiError error = (UnityNetwork.FfiError)result;
                if (error == UnityNetwork.FfiError.Disconnected)
                {
                    context = null;
                    throw new InvalidOperationException("Disconnected from server");
                }
                throw new InvalidOperationException($"Send failed: {error}");
            }
        }

        #endregion

        #region Receiving

        /// <summary>
        /// Poll for incoming data.
        /// Returns number of bytes received, or 0 if no data available.
        /// Returns negative number on error (check FfiError enum).
        /// </summary>
        public int Poll()
        {
            if (context == null)
            {
                return (int)UnityNetwork.FfiError.Disconnected;
            }

            fixed (byte* outPtr = receiveBuffer)
            {
                int bytesReceived = network_poll(context, outPtr, (ulong)receiveBuffer.Length);

                if (bytesReceived < 0)
                {
                    UnityNetwork.FfiError error = (UnityNetwork.FfiError)bytesReceived;
                    if (error == UnityNetwork.FfiError.Disconnected)
                    {
                        context = null;
                    }
                }

                return bytesReceived;
            }
        }

        /// <summary>
        /// Get the receive buffer reference for reading polled data.
        /// Only valid after a successful Poll() call.
        /// </summary>
        public ReadOnlySpan<byte> GetReceiveBuffer(int length)
        {
            if (length < 0 || length > receiveBuffer.Length)
            {
                return ReadOnlySpan<byte>.Empty;
            }
            return new ReadOnlySpan<byte>(receiveBuffer, 0, length);
        }

        /// <summary>
        /// Parse received data as a struct.
        /// Returns true if successful, false if data is too small or invalid.
        /// </summary>
        public bool TryParseStruct<T>(int length, out T data) where T : unmanaged
        {
            data = default(T);

            if (length < sizeof(T))
            {
                return false;
            }

            fixed (byte* ptr = receiveBuffer)
            {
                data = *(T*)ptr;
                return true;
            }
        }

        /// <summary>
        /// Get packet type from received data header.
        /// Returns null if data is too small or invalid.
        /// </summary>
        public PacketType? GetPacketType(int length)
        {
            if (length < sizeof(PacketHeader))
            {
                return null;
            }

            fixed (byte* ptr = receiveBuffer)
            {
                PacketHeader* header = (PacketHeader*)ptr;
                if (!header->IsValid())
                {
                    return null;
                }
                return (PacketType)header->packetType;
            }
        }

        #endregion

        #region Cleanup

        /// <summary>
        /// Disconnect from server and cleanup resources.
        /// </summary>
        public void Disconnect()
        {
            if (context != null)
            {
                network_destroy(context);
                context = null;
                UnityEngine.Debug.Log("Disconnected from server");
            }
        }

        #endregion

        #region IDisposable

        public void Dispose()
        {
            Dispose(true);
            GC.SuppressFinalize(this);
        }

        protected virtual void Dispose(bool disposing)
        {
            if (!disposed)
            {
                if (disposing)
                {
                    Disconnect();
                    receiveBuffer = null;
                }

                disposed = true;
            }
        }

        ~NativeNetworkClient()
        {
            Dispose(false);
        }

        #endregion

        #region Helpers

        private string MarshalPtrToString(byte* ptr)
        {
            if (ptr == null)
            {
                return string.Empty;
            }

            // Find null terminator
            int length = 0;
            while (ptr[length] != 0)
            {
                length++;
            }

            if (length == 0)
            {
                return string.Empty;
            }

            return Encoding.UTF8.GetString(ptr, length);
        }

        #endregion
    }

    /// <summary>
    /// Constants for interpreting GameState packet fields.
    /// This allows the same GameState struct to serve multiple purposes.
    /// </summary>
    public static class GameStateTypes
    {
        /// Special player_count values indicating the message type
        public const uint MSG_TYPE_HELLO = 0xFFFF0000;  // Hello world / connection test
        public const uint MSG_TYPE_ECHO = 0xFFFF0001;   // Echo response
        public const uint MSG_TYPE_STATE = 0xFFFF0002;  // Standard game state

        /// Helper to check if player_count is a message type
        public static bool IsMessageType(uint value)
        {
            return (value & 0xFFFF0000) != 0;
        }
    }

    /// <summary>
    /// Extension methods for GameState struct.
    /// </summary>
    public static class GameStateExtensions
    {
        /// Create a hello message for round-trip testing
        public static GameState CreateHello()
        {
            return new GameState(
                (uint)System.DateTimeOffset.UtcNow.ToUnixTimeSeconds(),
                GameStateTypes.MSG_TYPE_HELLO
            );
        }

        /// Create a standard game state update
        public static GameState CreateState(uint tick, uint playerCount)
        {
            return new GameState(tick, playerCount);
        }

        /// Check if this is a hello message
        public static bool IsHello(this GameState state)
        {
            return state.playerCount == GameStateTypes.MSG_TYPE_HELLO;
        }

        /// Check if this is an echo response
        public static bool IsEcho(this GameState state)
        {
            return state.playerCount == GameStateTypes.MSG_TYPE_ECHO;
        }

        /// Get message type description for debugging
        public static string GetTypeDescription(this GameState state)
        {
            switch (state.playerCount)
            {
                case GameStateTypes.MSG_TYPE_HELLO:
                    return "Hello";
                case GameStateTypes.MSG_TYPE_ECHO:
                    return "EchoResponse";
                case GameStateTypes.MSG_TYPE_STATE:
                    return "StateUpdate";
                default:
                    if (GameStateTypes.IsMessageType(state.playerCount))
                        return "UnknownMessage";
                    return "PlayerCount";
            }
        }
    }

    /// <summary>
    /// UUID v7 generator helper using Rust FFI.
    /// Delegates UUID generation to Rust for consistency and correctness.
    /// Format: 48-bit Unix timestamp (ms) + 74-bit random + 6-bit version/variant
    /// </summary>
    [Obsolete("Use PacketBuilder instead. Rust now handles UUID generation internally in packet builders.", true)]
    public unsafe static class UuidV7
    {
        private const string DLL_NAME = "mmorpg_profiler";

        /// <summary>
        /// FFI declaration for UUID v7 generation in Rust.
        /// Generates a time-ordered UUID compatible with Rust's uuid::now_v7().
        /// </summary>
        /// <param name="uuid_out">Pointer to 16-byte buffer for UUID bytes</param>
        /// <returns>0=Success, -1=Null pointer, -99=Panic caught</returns>
        [DllImport(DLL_NAME, CallingConvention = CallingConvention.Cdecl)]
        private static extern int profiler_generate_uuid_v7(byte* uuid_out);

        /// <summary>
        /// Generate a new UUID v7 (time-ordered UUID).
        /// Calls Rust implementation for consistency with server.
        /// </summary>
        /// <returns>A new UUID v7 as a Guid</returns>
        /// <exception cref="InvalidOperationException">Failed to generate UUID</exception>
        public static Guid NewUuid()
        {
            byte[] uuidBytes = new byte[16];
            fixed (byte* uuidPtr = uuidBytes)
            {
                int result = profiler_generate_uuid_v7(uuidPtr);

                if (result != 0)
                {
                    throw new InvalidOperationException(
                        $"Failed to generate UUID v7: {GetErrorString(result)}"
                    );
                }
            }

            return new Guid(uuidBytes);
        }

        /// <summary>
        /// Get error description from error code.
        /// </summary>
        private static string GetErrorString(int errorCode)
        {
            return errorCode switch
            {
                -1 => "Invalid pointer",
                -99 => "Panic caught in Rust code",
                _ => $"Unknown error code: {errorCode}"
            };
        }
    }
}
