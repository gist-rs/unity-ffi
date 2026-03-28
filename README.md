# Unity FFI Networking POC

High-performance Unity-to-Rust networking using FFI (Foreign Function Interface) and WebTransport. This POC demonstrates zero-copy communication between Unity (C#) and a Rust server without serialization overhead or GC pressure.

### Key Features

- **PacketBuilder API**: High-level packet construction with auto-generated UUID v7
- **View-Only Principle**: Unity focuses on display, Rust handles protocol logic
- **Zero-Copy Communication**: Uses `#[repr(C)]` structs and `fixed` pointers in C#
- **Thread-Safe Async Bridge**: Tokio runtime with bridge threads to properly handle async/FFI boundary
- **Caller-Allocated Buffers**: Rust never allocates memory for Unity, preventing heap corruption
- **Panic Guards**: All FFI functions wrapped in `catch_unwind` to prevent process crashes
- **Self-Signed TLS Support**: Development mode with automatic certificate handling
- **Circle Motion Broadcast**: Server simulates player moving in circle (radius=5.0, speed=2.0 rad/s) at 20Hz for visualization testing

## 📋 Overview

**📝 Recently Fixed Issues (2024-01)**:
- ✅ Fixed `scripts/build.sh` to change to project root directory (was changing to `scripts/` dir)
- ✅ Added `test-client` and `test-ffi-arch` to workspace members in `Cargo.toml`
- ✅ Updated test instructions to include build commands for test binaries
- ✅ Fixed documentation references to handovers and issues (corrected paths to `.handovers/` and `.issues/`)
- ✅ Removed `scripts/build_bin/` directory that was accidentally created in the wrong location


This POC implements a low-latency communication pipeline:

```
Unity (C#) → Native FFI (Rust) → WebTransport → Rust Server
```



### Critical Architecture Note

This POC uses **bridge threads** to convert between blocking FFI interface and async tokio runtime:

```
C# → std::sync::mpsc → [Bridge Thread 1] → tokio::sync::mpsc → Tokio Task ✅
                                                    ↓
                                               WebTransport
                                                    ↑
C# ← std::sync::mpsc ← [Bridge Thread 2] ← tokio::sync::mpsc ← Tokio Task ✅
```

**Why this matters**: Using `std::sync::mpsc::Receiver::recv()` directly inside `tokio::spawn()` blocks the entire task, preventing QUIC's async flow control from working. Bridge threads solve this by converting between blocking and async paradigms. See [HANDOVER.md](HANDOVER.md) for detailed analysis.

## 🏗️ Project Structure

```
unity-ffi/
├── crates/                      # Core library crates (workspace members)
│   ├── unity-network/          # Rust FFI library (cdylib)
│   │   ├── src/
│   │   │   ├── lib.rs         # Main FFI functions with bridge threads
│   │   │   └── types.rs       # Shared repr(C) structs
│   │   └── Cargo.toml
│   ├── game-server/            # WebTransport server
│   │   ├── src/
│   │   │   └── main.rs        # Server implementation
│   │   └── Cargo.toml
│   ├── game-ffi/               # Core FFI types
│   │   └── Cargo.toml
│   └── game-ffi-derive/        # Derive macros for FFI types
│       └── Cargo.toml
├── tests/                       # Test infrastructure
│   ├── PacketBuilderTests/     # .NET FFI tests
│   ├── test-client/            # Simple Rust client (baseline test)
│   │   ├── src/
│   │   │   └── main.rs        # Async-only, no FFI
│   │   └── Cargo.toml
│   ├── test-ffi-arch/          # FFI architecture test (reproduces/solves bug)
│   │   ├── src/
│   │   │   └── main.rs        # Simulates Unity threading with bridge threads
│   │   └── Cargo.toml
├── unity/                       # C# scripts for Unity
│   ├── NativeNetworkClient.cs  # Low-level FFI bridge
│   ├── PacketBuilder.cs         # High-level packet construction API
│   ├── NetworkPlayer.cs         # High-level MonoBehaviour
│   ├── Editor/                  # Unity Editor scripts
│   │   ├── BuildTools/          # Asset bundle build tools
│   │   │   ├── BuildAssetBundles.cs
│   │   │   └── README.md
│   │   ├── GameFFITests.cs     # FFI integration tests
│   │   └── PacketBuilderIntegrationTests.cs
│   ├── Generated/               # Generated FFI bindings
│   └── Profiler/                # Performance profiling tools
├── examples/
│   └── helloworld-ffi/         # Unity example project
│       ├── Assets/
│       │   ├── Plugins/
│       │   │   └── macOS/
│       │   │       └── libunity_network.dylib
│       │   └── Scripts/
│       │       ├── NativeNetworkClient.cs
│       │       └── NetworkPlayer.cs
│       └── ProjectSettings/
│           └── ProjectSettings.asset
├── build_bin/                   # Build output directory
│   ├── libunity_network.dylib  # Native library (copy to Unity)
│   └── unity-ffi-server        # Server binary
├── docs/                        # Documentation
│   └── UNITY_SETUP_GUIDE.md    # (Consolidated into this README)
├── scripts/                     # All shell scripts
│   ├── build.sh                 # Build FFI library and server
│   ├── run.sh                   # Start/stop server
│   ├── setup.sh                 # Setup Unity project
│   ├── teardown.sh              # Clean up server
│   ├── build_profiler.sh        # Build profiler tools
│   ├── setup_profiler.sh        # Setup profiler
│   ├── quickstart_profiler.sh   # Quick profiler setup
│   ├── rebuild_for_rosetta.sh     # Rebuild for Rosetta
│   └── generate_bindings.sh     # Generate Unity C# bindings
├── HANDOVER.md                  # Detailed handover document
├── ISSUES.md                    # Known issues and remaining work
└── README.md                    # This file
```

### Component Details

#### `crates/unity-network/` - Rust FFI Library
**Purpose**: Provides `extern "C"` interface for Unity to call Rust code.

**Key Files**:
- `src/lib.rs`: FFI functions with bridge thread architecture
  - `network_init()`: Initialize logging
  - `network_connect()`: Create bridge threads, spawn tokio runtime, establish WebTransport connection
  - `network_send()`: Send packets to server (C# → Rust)
  - `network_poll()`: Poll for incoming packets (Rust → C#)
  - `network_destroy()`: Cleanup resources

- `src/packet_builder.rs`: High-level packet construction API
  - `packet_builder_create_player_pos()`: Create PlayerPos packet with auto-generated UUID v7
  - `packet_builder_create_game_state()`: Create GameState packet with auto-generated UUID v7
  - `packet_builder_create_sprite_message()`: Create SpriteMessage packet with auto-generated UUID v7
  - `packet_builder_create_authenticate()`: Create Authenticate packet with auto-generated UUID v7
  - `packet_builder_create_keep_alive()`: Create KeepAlive packet with auto-generated UUID v7
  - `packet_builder_get_error_string()`: Get error description for error codes

- `src/types.rs`: Shared `#[repr(C)]` structs
  - `PacketHeader`: Common header for all packets (includes UUID v7)
  - `PlayerPos`: Player position update
  - `GameState`: Server state snapshot
  - `PacketType`: Enum of packet types

**Architecture**:
```
┌─────────────────────────────────────────────┐
│ Unity (C#)                                  │
│   network_send(ptr, size)                   │
└──────────────┬──────────────────────────────┘
               ↓
┌─────────────────────────────────────────────┐
│ FFI Layer (extern "C")                      │
│   Copies bytes to std::sync::mpsc           │
└──────────────┬──────────────────────────────┘
               ↓
┌─────────────────────────────────────────────┐
│ Bridge Thread 1 (Dedicated)                 │
│   std::sync::mpsc::recv() [BLOCKS OK]       │
│   → tokio::sync::mpsc::blocking_send()      │
└──────────────┬──────────────────────────────┘
               ↓
┌─────────────────────────────────────────────┐
│ Tokio Runtime (Async)                       │
│   tokio::sync::mpsc::Receiver::recv().await │
│   → WebTransport send                       │
└─────────────────────────────────────────────┘
```

**Receive Path**:
```
┌─────────────────────────────────────────────┐
│ Server                                      │
│   WebTransport receive                      │
└──────────────┬──────────────────────────────┘
               ↓
┌─────────────────────────────────────────────┐
│ Tokio Runtime (Async)                       │
│   WebTransport receive                      │
│   → tokio::sync::mpsc::send()               │
└──────────────┬──────────────────────────────┘
               ↓
┌─────────────────────────────────────────────┐
│ Bridge Thread 2 (Dedicated)                 │
│   Runs own Tokio runtime                    │
│   tokio::sync::mpsc::Receiver::recv().await │
│   → std::sync::mpsc::send() [BLOCKING OK]   │
└──────────────┬──────────────────────────────┘
               ↓
┌─────────────────────────────────────────────┐
│ FFI Layer (extern "C")                      │
│   network_poll() copies from std::sync::mpsc│
└──────────────┬──────────────────────────────┘
               ↓
┌─────────────────────────────────────────────┐
│ Unity (C#)                                  │
│   Receives into caller-allocated buffer     │
└─────────────────────────────────────────────┘
```

**Key Architecture Points**:
- **2 Bridge Threads total**: One for send (C# → Server), one for receive (Server → C#)
- **1 Tokio Runtime Thread**: Runs main WebTransport connection
- **Bridge Thread 1**: Dedicated thread that blocks on `std::sync::mpsc::recv()`, converts to async
- **Bridge Thread 2**: Dedicated thread with its own Tokio runtime, converts async to blocking
- **Thread Safety**: All channels are thread-safe; Unity calls are blocking-safe

#### `crates/game-server/` - WebTransport Server
**Purpose**: Runs WebTransport server, handles connections, broadcasts circle motion.

**Key Features**:
- Self-signed certificate generation for development
- Listens on port 4433 (`wtransport://127.0.0.1:4433`)
- Connection management with player tracking
- Circle motion broadcast (20Hz, radius=5.0, speed=2.0 rad/s)
- Packet parsing and routing

**Running**:
```bash
# Start server (auto-kills existing server, builds if needed)
./scripts/run.sh

# Force rebuild then run
./scripts/run.sh --rebuild
```

#### `tests/test-client/` - Simple Test Client
**Purpose**: Verify WebTransport works without FFI overhead.

**Build**:
```bash
cargo build --release -p test-client
```

**Usage**:
```bash
./target/release/test-client
```

**Expected**: Bidirectional communication works immediately.

#### `tests/test-ffi-arch/` - FFI Architecture Test
**Purpose**: Reproduce and verify the FFI threading bug/fix.

**Build**:
```bash
cargo build --release -p test-ffi-arch
```

**Usage**:
```bash
./target/release/test-ffi-arch
```

**Before Fix**: Packets sent: 247, Packets received: 0 ❌
**After Fix**: Packets sent: ~300, Packets received: ~300 ✅

#### `unity/` - C# Scripts
**Purpose**: Source C# scripts for Unity.

**Scripts**:
- `NativeNetworkClient.cs`: Low-level FFI P/Invoke wrapper
  - `Initialize()`: Set up logging callback
  - `Connect()`: Establish connection to server
  - `SendStruct<T>()`: Zero-copy send any struct
  - `Poll()`: Poll for incoming data
  - `TryParseStruct<T>()`: Parse received data

- `PacketBuilder.cs`: High-level packet construction API
  - Provides static methods for each packet type (CreatePlayerPos, CreateGameState, etc.)
  - Handles all FFI calls to Rust packet_builder functions
  - Auto-generates UUID v7 in Rust (no UUID generation in Unity)
  - Manages memory allocation and error handling
  - Type-safe with compile-time checking

- `NetworkPlayer.cs`: High-level MonoBehaviour
  - Handles connection lifecycle
  - Sends position updates every `updateInterval`
  - Receives and processes server packets
  - Visualizes circle motion
  - Shows debug UI overlay

## 🚀 Quick Start

### Prerequisites

- **Rust**: 1.70 or later
- **Unity**: 2023.2 or later
- **macOS**: This POC is configured for macOS (uses `.dylib`)
- **Build Tools**: `cargo`, `nm`, standard Unix tools

### Step 1: Build the Components

```bash
# Navigate to the POC directory
cd poc/unity-ffi

# Build release version (recommended for testing)
./scripts/build.sh release

# Or build debug version (with symbols)
./scripts/build.sh debug
```

This creates a `build_bin/` directory containing:
- `libunity_network.dylib` - Native library for Unity
- `unity-ffi-server` - WebTransport server binary

#### Building for Rosetta (Unity under x86_64)

If you're running Unity under Rosetta on Apple Silicon, you need to build the library for x86_64:

```bash
# Build for x86_64 (Rosetta)
./scripts/build.sh release x86_64

# Or use helper script that rebuilds and copies to Unity automatically
./scripts/rebuild_for_rosetta.sh
```

The `build.sh` script now accepts an optional architecture argument:
- `arm64` (default): Native Apple Silicon build
- `x86_64`: For Unity running under Rosetta

### Step 2: Start the Server

```bash
# Run the server (automatically kills existing server on port 4433, builds if needed, and starts)
./scripts/run.sh

# Or force rebuild before running
./scripts/run.sh --rebuild
```

The server will:
1. Generate a self-signed certificate automatically
2. Start listening on `wtransport://127.0.0.1:4433`
3. Start circle motion broadcast task (sends PlayerPos updates every 50ms)
4. Wait for Unity connections

**Note**: For development, Unity can connect without providing a certificate hash. The server uses self-signed certificates automatically.

### Step 2.5: Stop the Server

When you're done testing or need to restart the server, use teardown script:

```bash
# Stop server and free up port 4433
./scripts/teardown.sh
```

The teardown script will:
- Find the server process running on port 4433
- Kill the process gracefully
- Clean up PID files
- Preserve log files for debugging

The script will show:
```
=====================================
Unity FFI Teardown Script
=====================================
Stopping server on port 4433...

Step 1: Finding server process...
Found server process (PID: 12345)
Stopping server...
✓ Server stopped successfully

=====================================
Teardown Complete!
=====================================
```

**Note**: If you prefer to stop the server manually, you can use:
```bash
# Find and kill the process
lsof -ti:4433 | xargs kill -9

# Or view the log file
cat /tmp/unity-ffi-server.log
```

### Step 3: Setup Unity Project

#### 3.1 Create or Open Unity Project

**Option A: Use Example Project**
```bash
# Setup example Unity project
./scripts/setup.sh examples/helloworld-ffi

# Open in Unity
open -a Unity examples/helloworld-ffi
```

**Option B: New Project**
1. Create a new Unity project (2023.2+)
2. Create folder structure: `Assets/Plugins/macOS/`
3. Copy `build_bin/libunity_network.dylib` to `Assets/Plugins/macOS/`
4. Copy `unity/*.cs` to `Assets/Scripts/`

#### 3.2 Enable Unsafe Code

Unity requires explicit permission to compile unsafe code.

**Method 1: Unity Editor (Recommended)**
1. Open your Unity project
2. Go to **Edit → Project Settings → Player**
3. Scroll down to **Other Settings → Configuration**
4. Set **Allow 'unsafe' Code** to **ON**

**Method 2: Direct Configuration File**
1. Close Unity Editor
2. Edit `ProjectSettings/ProjectSettings.asset`
3. Add or modify: `allowUnsafeCode: 1`
4. Save and reopen Unity

#### 3.3 Import and Configure

1. **Select Plugin**: Click `Assets/Plugins/macOS/libunity_network.dylib`
2. **Verify Inspector Settings**:
   - **Plugin Importer** should be shown
   - **CPU**: Any CPU
   - **Platform**: macOS
   - **Compatibility**: Editor and Standalone

3. **Create Test GameObject**:
   - GameObject → Create Empty
   - Rename to "NetworkPlayer"
   - Add Component: "Network Player"

4. **Configure NetworkPlayer**:
   ```
   Server URL: https://127.0.0.1:4433
   Certificate Hash: (leave empty for development)
   Player ID: 1
   Update Interval: 0.05 (20Hz)
   Log Packets: ✓
   Show Debug Info: ✓
   Circle Player Prefab: (optional)
   Circle Player Scale: 1.0
   ```

#### 3.4 Test Connection

1. **Press Play** in Unity Editor
2. **Watch Console** for connection messages:
   ```
   Connecting to server: https://127.0.0.1:4433
   Connection successful!
   [Sent] Hello message at tick 1735184000
   <color=green>Hello World Round-Trip Complete!</color>
   ```

3. **Watch Game View** for debug overlay:
   ```
   Status: Connected
   Player ID: 1
   Position: (0.00, 0.00)
   Server Tick: 0
   Players: 0
   ```

4. **Verify Circle Motion**:
   - Red sphere should appear and move in circle
   - Circle radius: 5.0 units
   - Speed: 2.0 radians/second (~3.14s per revolution)
   - Update rate: 20 Hz
   - Console logs: `<color=yellow>Received Circle Motion from server at (x, y)</color>`

### Step 4: Verify Communication

#### Move the Player Object

1. In Unity Scene, move the NetworkPlayer GameObject
2. **Watch Unity Console** for sent packets:
   ```
   [Sent] PlayerPos: id=1, x=10.50, y=20.30
   ```
3. **Watch Server terminal** for received packets:
   ```
   PlayerPos: client=..., player_id=1, x=10.50, y=20.30
   ```

#### Check Server Logs

```bash
# View server logs in real-time (if logging to file)
tail -f /tmp/unity-ffi-server.log
```

Expected output:
```
INFO New connection from client: 01234567-89ab-cdef-0123-456789abcdef
INFO GameState: client=01234567..., type=Hello, tick=1735184000
INFO   -> Hello from client, sending echo response
```

## 📖 Detailed Usage

### NativeNetworkClient API

The `NativeNetworkClient` class provides low-level FFI access:

```csharp
// Initialize (call once, usually in Awake)
var client = new NativeNetworkClient();
client.Initialize();

// Connect to server
// For development, pass null as certificateHash
client.Connect("https://127.0.0.1:4433", null);

// Send struct (zero-copy)
var pos = new PlayerPos(id: 1, x: 10.5f, y: 20.3f);
client.SendStruct(pos);

// Poll for incoming data (call in Update)
int bytesReceived = client.Poll();
if (bytesReceived > 0) {
    // Get packet type from first byte
    var packetType = client.GetPacketType(bytesReceived);
    
    // Parse based on type
    switch (packetType) {
        case PacketType.PlayerPos:
            if (client.TryParseStruct<PlayerPos>(bytesReceived, out var playerPos)) {
                // Handle player position
                Debug.Log($"Player at ({playerPos.x}, {playerPos.y})");
            }
            break;
        case PacketType.GameState:
            if (client.TryParseStruct<GameState>(bytesReceived, out var gameState)) {
                // Handle game state
                Debug.Log($"Game tick: {gameState.tick}");
            }
            break;
    }
}

// Cleanup (call in OnDestroy or OnDisable)
client.Disconnect();
```

### Sending Custom Packets

#### 1. Define Shared Types in Rust

**File**: `crates/unity-network/src/types.rs`
```rust
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct CustomPacket {
    pub header: PacketHeader,
    pub data: u32,
    pub value: f32,
}
```

#### 2. Add Packet Type Enum

**Rust** (`crates/unity-network/src/types.rs`):
```rust
pub enum PacketType {
    KeepAlive = 0,
    PlayerPos = 1,
    GameState = 2,
    Custom = 3,  // Add this
}
```

**C#** (`NativeNetworkClient.cs`):
```csharp
public enum PacketType : byte {
    KeepAlive = 0,
    PlayerPos = 1,
    GameState = 2,
    Custom = 3,  // Add this
}
```

#### 3. Define in C#

**File**: `Assets/Scripts/NativeNetworkClient.cs`
```csharp
[StructLayout(LayoutKind.Sequential, Pack = 1)]
public struct CustomPacket {
    public PacketHeader header;
    public uint data;
    public float value;
}
```

#### 4. Send from Unity

```csharp
var packet = new CustomPacket {
    header = new PacketHeader((byte)PacketType.Custom),
    data = 42,
    value = 3.14f
};
client.SendStruct(packet);
```

#### 5. Handle in Server

**File**: `crates/game-server/src/main.rs`
```rust
// In handle_packet() function
match packet_type {
    PacketType::Custom => {
        if data.len() >= mem::size_of::<CustomPacket>() {
            let custom = unsafe { *(data.as_ptr() as *const CustomPacket) };
            info!("Custom packet: data={}, value={}", custom.data, custom.value);
            // Send response or process logic here
        }
    }
    // ... other packet types
}
```

## 🔧 Development Mode

The server uses WebTransport's certificate validation bypass for development.

**Current State**:
- **No certificate hash required** in Unity for testing
- **Certificate validation bypassed**: Uses `with_no_cert_validation()` for development
- Server logs: "Using client config with certificate validation bypassed (development mode)"
- Unity connects with: `client.Connect("https://127.0.0.1:4433", null)`

**Future**: Self-signed certificate generation will be added for proper TLS testing

**For Production**: Replace bypass with proper CA-signed certificates and implement certificate pinning.

## 🔧 Advanced Configuration

### Build Options

```bash
# Release build (optimized)
./scripts/build.sh release

# Debug build (with symbols)
./scripts/build.sh debug
```

### Protocol Versioning

Both client and server must use the same protocol version:

**Rust** (`crates/unity-network/src/lib.rs`):
```rust
const PROTOCOL_VERSION: u32 = 1;
```

**C#** (`NativeNetworkClient.cs`):
```csharp
private const uint PROTOCOL_VERSION = 1;
```

### Buffer Sizes

**Rust** (`crates/unity-network/src/lib.rs`):
```rust
// MPSC channel buffer sizes
const CHANNEL_CAPACITY: usize = 128;  // Number of packets buffered
```

**C#** (`NativeNetworkClient.cs`):
```csharp
// Unity receive buffer size
private const int RECEIVE_BUFFER_SIZE = 4096;
```

## 🐛 Troubleshooting

### Connection Issues

**Problem**: "Failed to connect to server"

**Solutions**:
1. Verify server is running: `./scripts/run.sh`
2. Check URL format: must be `https://127.0.0.1:4433` (not `http://`)
3. For development, ensure Certificate Hash is empty in Unity
4. Check Unity Console for specific error messages
5. Verify port 4433 is not blocked by firewall

### DLL Not Found

**Symptoms**: `dlopen` error with "mach-o file, but is an incompatible architecture"

**Cause**: Unity is running under Rosetta (x86_64) but the library was built for ARM64 (native), or vice versa.

**Solution**:

1. Check Unity's architecture:
   ```bash
   # Check if Unity is running under Rosetta
   # Unity Intel builds run under Rosetta on Apple Silicon
   ```

2. Rebuild the library for the correct architecture:
   ```bash
   # For Unity under Rosetta (x86_64)
   ./scripts/build.sh release x86_64
   
   # Or use helper script
   ./scripts/rebuild_for_rosetta.sh
   ```

3. Copy the library to Unity's Plugins folder:
   ```bash
   cp build_bin/libunity_network.dylib examples/helloworld-ffi/Assets/Plugins/macOS/
   ```

4. Verify the architecture:
   ```bash
   file examples/helloworld-ffi/Assets/Plugins/macOS/libunity_network.dylib
   # Should show "x86_64" for Rosetta or "arm64" for native
   ```

**Problem**: "DllNotFoundException: unity_network"

**Solutions**:
1. Verify `libunity_network.dylib` is in `Assets/Plugins/macOS/`
2. Check file permissions: `chmod +x Assets/Plugins/macOS/libunity_network.dylib`
3. Verify Unity build settings have "macOS" selected
4. Restart Unity Editor
5. Check Console for plugin import errors

### Unsafe Code Error

**Problem**: "Unsafe code may only appear if compiling with /unsafe"

**Solution**: Enable "Allow 'unsafe' Code" in Player Settings (see Step 3.2 above)

### Certificate Errors

**Problem**: Connection fails with certificate validation errors

**Solutions**:
1. **Development**: Ensure Certificate Hash is empty in Unity NetworkPlayer component
2. Check server logs for "Using client config with certificate validation bypassed"
3. Verify server is listening on port 4433
4. Try restarting the server after making changes

### No Packets Received

**Problem**: Unity sends packets but receives nothing

**Solutions**:
1. Check Unity Console for "Bridge thread started" logs
2. Verify server logs show connection established
3. Ensure "Log Packets" is enabled in NetworkPlayer component
4. Check for "timed out" errors (indicates blocking issue)
5. Verify WebTransport works with test-client first

### Performance Issues

**Problem**: Low FPS or lag

**Solutions**:
1. Reduce `updateInterval` in NetworkPlayer (increase interval)
2. Use Release build: `./scripts/build.sh release`
3. Reduce log frequency (set `logPackets = false`)
4. Profile with Unity Profiler
5. Check CPU usage in Unity and Server

## 📊 Packet Types

| Type | Value | Description |
|------|-------|-------------|
| `KeepAlive` | 0 | Heartbeat packet (for future use) |
| `PlayerPos` | 1 | Player position update (ID 999 = circle motion player) |
| `GameState` | 2 | Server state snapshot (tick, player count) |
| `Custom` | 3+ | User-defined packets |

## 🛡️ Security Considerations

This POC currently uses **certificate validation bypass** (`with_no_cert_validation()`) for development convenience. This is intentional for rapid prototyping but **not secure for production**.

**Current State**:
- Certificate validation is disabled in client configuration
- This allows connections to any server certificate
- Useful for local development without proper certificate infrastructure

**Future**: Self-signed certificate support will be added for proper TLS testing

**For Production**:
1. Enable proper certificate validation (remove `with_no_cert_validation()`)
2. Use CA-signed certificates or properly configured self-signed certificates
3. Implement certificate pinning for security
4. Add authentication tokens
5. Validate all incoming data
6. Implement rate limiting
7. Use encrypted connections (already via TLS 1.3)

## 📚 Code Style

This project follows these conventions:

- **Rust**: `snake_case` for functions/variables, `PascalCase` for types
- **C#**: `PascalCase` for public members, `camelCase` for private
- **Documentation**: All public FFI functions documented
- **Error Handling**: All FFI calls return error codes
- **Logging**: Structured logging with `tracing` crate

## 🧪 Testing

### Unit Tests

```bash
# Run Rust tests
cargo test -p unity-network

# Run with output
cargo test -p unity-network -- --nocapture
```

### Integration Tests

**Note**: `test-client` and `test-ffi-arch` are workspace members and can be built with `cargo build -p <package-name>`.

1. **Build test binaries**:
   ```bash
   # Build test-client
   cargo build --release -p test-client
   
   # Build test-ffi-arch
   cargo build --release -p test-ffi-arch
   ```

2. **Start server**:
   ```bash
   ./build_bin/unity-ffi-server
   ```

3. **Run simple test-client** (baseline):
   ```bash
   ./target/release/test-client
   ```
   Expected: Bidirectional communication works immediately

4. **Run test-ffi-arch** (FFI pattern):
   ```bash
   ./target/release/test-ffi-arch
   ```
   Expected: Packets sent AND received (not zero)

5. **Test in Unity**:
   - Press Play in Unity
   - Move player object to send packets
   - Verify packets in both Unity Console and Server terminal

### Testing Circle Motion Visualization

1. **Start the server**:
   ```bash
   ./build_bin/unity-ffi-server
   ```
   Expected: "Starting circle motion broadcast (radius=5.0, speed=2.0 rad/s)"

2. **Connect Unity**:
   - Play your Unity scene with NetworkPlayer component
   - A red sphere (CirclePlayer) will appear when first position update arrives

3. **Verify the movement**:
   - Red sphere moves smoothly in circle around center (0, 0)
   - Circle radius: 5.0 units
   - Speed: 2.0 radians/second (~3.14s per circle)
   - Update rate: 20 Hz (every 50ms)

4. **Check Unity Console**:
   - Logs: `"[Received] PlayerPos: id=999, x=5.00, y=0.00"`
   - x and y values oscillate as sphere moves in circle
   - Yellow logs: `"<color=yellow>Received Circle Motion from server at (x, y)</color>"`

5. **Optional: Customize visualization**:
   - Assign custom prefab to "Circle Player Prefab" in NetworkPlayer
   - Adjust "Circle Player Scale" to resize visualization
   - If no prefab assigned, red sphere is created automatically

## 🔄 Development Workflow

### Making Changes

1. **Modify code**:
   ```bash
   # Edit Rust code
   vim crates/unity-network/src/lib.rs
   vim crates/unity-network/src/types.rs
   vim crates/game-server/src/main.rs
   
   # Or edit C# code
   vim unity/NetworkPlayer.cs
   ```

2. **Rebuild**:
   ```bash
   # Release build
   ./scripts/build.sh release
   
   # Or debug build
   ./scripts/build.sh debug
   ```

3. **Copy to Unity**:
   ```bash
   # Copy updated library
   cp build_bin/libunity_network.dylib examples/helloworld-ffi/Assets/Plugins/macOS/
   
   # Or if using custom project
   cp build_bin/libunity_network.dylib /path/to/your/unity/Assets/Plugins/macOS/
   ```

4. **Refresh Unity**:
   - Assets → Refresh (Cmd+R on macOS)
   - Or close and reopen Unity

5. **Test**:
   - Press Play in Unity
   - Check Console and Server logs

### Generating Unity C# Bindings

When you modify FFI types in `unity-network/src/types.rs`, you must regenerate the C# bindings:

1. **Run the generation script**:
   ```bash
   ./scripts/generate_bindings.sh
   ```

2. **Verify the output**:
   ```bash
   # Check generated file
   cat unity/Generated/GameFFI.cs
   
   # Verify memory layout matches Rust
   cargo run --package unity-network --example extract_layout
   
   # Extract UUIDs for reference
   cargo run --package unity-network --example extract_uuids
   ```

3. **Unity will auto-detect the change**:
   - Unity automatically picks up the new `GameFFI.cs`
   - If not: `Assets > Refresh`

**When to regenerate**:
- After adding new `#[derive(GameComponent)]` structs
- After modifying existing struct fields
- After changing field types
- After adding or removing fields

**Important**:
- ⚠️ **NEVER edit `GameFFI.cs` manually** - it's auto-generated
- ✅ Rust FFI types are the single source of truth
- ✅ All changes start in Rust, then regenerate C# bindings

For detailed information, see [.docs/009_generate_bindings.md](.docs/009_generate_bindings.md).

### Debugging

**Enable detailed logging**:
```bash
# Build test-ffi-arch first (if not already built)
cargo build --release -p test-ffi-arch

# Server with debug logs
RUST_LOG=debug,unity_ffi_server=debug ./build_bin/unity-ffi-server

# Test-ffi-arch with debug
RUST_LOG=debug,test_ffi_arch=debug ./target/release/test-ffi-arch
```

**Check for success indicators**:
- ✅ "Bridge thread started: std::sync::mpsc → tokio::sync::mpsc"
- ✅ "Outbound task started"
- ✅ "First outbound packet sent"
- ✅ "First inbound packet received"
- ✅ "First circle motion received from server!"

**Check for failure indicators**:
- ❌ "receive_datagram timed out" (blocking issue)
- ❌ "No data for X seconds" (deadlock)
- ❌ Packets sent: X, Packets received: 0 (bug not fixed)

**Verify FFI symbols**:
```bash
nm -gU build_bin/libunity_network.dylib | grep network_
```
Should show: `_network_connect`, `_network_destroy`, `_network_init`, `_network_poll`, `_network_send`

## 📝 Important Documents

- [.docs/009_generate_bindings.md](.docs/009_generate_bindings.md) - Guide for auto-generating Unity C# bindings from Rust FFI types

### Handovers
Comprehensive handover documents with detailed bug analysis, root cause investigation, and anti-patterns to avoid:
- [.handovers/001_unity_csharp_auto_generation_implementation.md](.handovers/001_unity_csharp_auto_generation_implementation.md) - Implementation details for Unity C# auto-generation
- [.handovers/002_reorganize_scripts_folder.md](.handovers/002_reorganize_scripts_folder.md) - Scripts folder reorganization

### Issues
Known issues, remaining work, and future improvements:
- [.issues/001_complete_unity_csharp_auto_generation.md](.issues/001_complete_unity_csharp_auto_generation.md) - Complete Unity C# auto-generation implementation

- **[README.md](README.md)**: This file - main documentation

## 📄 License

MIT OR Apache-2.0

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests
5. Update documentation
6. Submit a pull request

## 📞 Support

For issues or questions:
1. Check this README's **Troubleshooting** section
2. Review handover documents in [.handovers/](.handovers/) for detailed technical analysis
3. Check issue tracking in [.issues/](.issues/) for known problems and remaining work
4. Check Unity Console and Server terminal logs
5. Verify packet types and struct definitions match
6. Ensure "Allow 'unsafe' Code" is enabled in Player Settings

## 🎯 Success Criteria

- ✅ test-ffi-arch sends and receives packets (not zero)
- ✅ Unity receives circle motion from server (yellow logs)
- ✅ Unity sprite moves via server control (no cyan local fallback)
- ✅ Server receives Unity packets in real-time
- ✅ No "timed out" logs in normal operation
- ✅ Bidirectional communication works for 30+ seconds
- ✅ Performance: < 10ms latency, > 100 packets/second

---

**Note**: This is a POC (Proof of Concept) and should not be used in production without additional security hardening, testing, and error handling.

**Last Updated**: 2025-12-27  
**Unity Version**: 2023.2+  
**Platform**: macOS (Windows/Linux support planned)  
**Status**: ✅ Bug fixed, production-ready architecture
