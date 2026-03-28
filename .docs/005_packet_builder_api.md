# PacketBuilder API Documentation

## Overview

PacketBuilder is a high-level API for creating network packets in Unity applications. It delegates all protocol logic to Rust FFI, including UUID v7 generation and packet serialization. Unity only provides high-level intent (business data) and receives complete packets ready to send.

### Key Features

- **Auto-generated UUID v7**: All packets include time-ordered UUID for end-to-end correlation
- **Type-safe**: Compile-time type checking in both Rust and C#
- **Zero-copy**: Direct struct serialization without intermediate allocation
- **Thread-safe**: All FFI functions are safe to call from multiple threads
- **Error handling**: Comprehensive error codes and error string retrieval
- **Performance**: < 1 μs per packet after warmup

### Architecture

```
Unity (C#)                      Rust FFI                          Server
───────────────────────────────────────────────────────────────────────────
PacketBuilder.CreatePlayerPos()
    │
    ├─ Call packet_builder_create_player_pos() FFI
    │                                  │
    │                                  ├─ Generate UUID v7
    │                                  ├─ Construct PacketHeader
    │                                  ├─ Construct PlayerPos struct
    │                                  ├─ Serialize to bytes
    │                                  └─ Return byte array
    │
    └─ Receive complete packet bytes
         │
         └─ Send to server
```

### Unity View-Only Principle

**Unity Responsibility (View Layer)**
- Display data to user
- Receive user input
- Call high-level PacketBuilder methods
- Send/receive packets

**Rust Responsibility (Logic Layer)**
- Generate UUID v7
- Construct packets
- Serialize/deserialize
- Protocol implementation

## Rust FFI API

### PacketBuilderError

Error codes returned by PacketBuilder FFI functions.

```rust
#[repr(C)]
pub enum PacketBuilderError {
    Success = 0,
    InvalidPointer = -1,
    BufferTooSmall = -2,
    InvalidPacketType = -3,
    PanicCaught = -99,
}
```

| Error Code | Value | Description |
|------------|-------|-------------|
| Success | 0 | Operation completed successfully |
| InvalidPointer | -1 | Null pointer passed to FFI function |
| BufferTooSmall | -2 | Output buffer too small for packet |
| InvalidPacketType | -3 | Invalid packet type requested |
| PanicCaught | -99 | Panic caught in Rust code (internal error) |

### packet_builder_create_player_pos

Create a PlayerPos packet with auto-generated UUID v7.

#### Function Signature

```rust
pub unsafe extern "C" fn packet_builder_create_player_pos(
    id_ptr: *const u8,      // Player UUID as 16 bytes (little-endian)
    x: i32,                  // X position
    y: i32,                  // Y position
    out_ptr: *mut u8,        // Output buffer pointer
    capacity: usize,          // Output buffer capacity
) -> i32                    // Returns bytes written or error code
```

#### Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| id_ptr | *const u8 | Player/character UUID as 16 bytes (little-endian) |
| x | i32 | X coordinate |
| y | i32 | Y coordinate |
| out_ptr | *mut u8 | Output buffer pointer (caller-allocated) |
| capacity | usize | Output buffer capacity (must be ≥ 44 bytes) |

#### Return Value

| Value | Description |
|-------|-------------|
| > 0 | Number of bytes written (should be 44) |
| -1 | Invalid pointer (null pointer passed) |
| -2 | Buffer too small (capacity < 44 bytes) |
| -99 | Panic caught in Rust code |

#### Packet Layout

```
Offset  Size    Field            Description
─────────────────────────────────────────────────────────
0x00    1       packet_type      PacketType::PlayerPos (0x02)
0x01    1       magic           0xCC
0x02    16      request_uuid     UUID v7 (auto-generated)
0x12    2       padding         Alignment padding
0x14    16      id              Player UUID (from id_ptr)
0x24    4       x               X position
0x28    4       y               Y position
─────────────────────────────────────────────────────────
Total:  44 bytes
```

#### Example

```csharp
// C# wrapper usage
Guid playerId = Guid.NewGuid();
int x = 100;
int y = 200;

byte[] packet = PacketBuilder.CreatePlayerPos(playerId, x, y);
// packet is 44 bytes, ready to send
```

### packet_builder_create_game_state

Create a GameState packet with auto-generated UUID v7.

#### Function Signature

```rust
pub unsafe extern "C" fn packet_builder_create_game_state(
    tick: u32,               // Server tick or timestamp
    player_count: u32,        // Number of players or message type
    out_ptr: *mut u8,         // Output buffer pointer
    capacity: usize,           // Output buffer capacity
) -> i32                     // Returns bytes written or error code
```

#### Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| tick | u32 | Server tick number or timestamp |
| player_count | u32 | Number of players or message type |
| out_ptr | *mut u8 | Output buffer pointer (caller-allocated) |
| capacity | usize | Output buffer capacity (must be ≥ 36 bytes) |

#### Return Value

| Value | Description |
|-------|-------------|
| > 0 | Number of bytes written (should be 36) |
| -1 | Invalid pointer (null pointer passed) |
| -2 | Buffer too small (capacity < 36 bytes) |
| -99 | Panic caught in Rust code |

#### Packet Layout

```
Offset  Size    Field            Description
─────────────────────────────────────────────────────────
0x00    1       packet_type      PacketType::GameState (0x03)
0x01    1       magic           0xCC
0x02    16      request_uuid     UUID v7 (auto-generated)
0x12    2       padding         Alignment padding
0x14    4       tick            Server tick
0x18    4       player_count    Number of players
0x1C    8       reserved        Reserved for future use
─────────────────────────────────────────────────────────
Total:  36 bytes
```

### packet_builder_create_sprite_message

Create a SpriteMessage packet with auto-generated UUID v7.

#### Function Signature

```rust
pub unsafe extern "C" fn packet_builder_create_sprite_message(
    operation: u8,            // Sprite operation type
    sprite_type: u8,          // Sprite type
    id_ptr: *const u8,       // Sprite UUID as 16 bytes
    x: i16,                  // X position
    y: i16,                  // Y position
    out_ptr: *mut u8,         // Output buffer pointer
    capacity: usize,          // Output buffer capacity
) -> i32                     // Returns bytes written or error code
```

#### Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| operation | u8 | Sprite operation: 0=Create, 1=Update, 2=Delete, 3=Snapshot |
| sprite_type | u8 | Sprite type: 0=Serrif |
| id_ptr | *const u8 | Sprite UUID as 16 bytes (little-endian) |
| x | i16 | X coordinate |
| y | i16 | Y coordinate |
| out_ptr | *mut u8 | Output buffer pointer (caller-allocated) |
| capacity | usize | Output buffer capacity (must be ≥ 46 bytes) |

#### Return Value

| Value | Description |
|-------|-------------|
| > 0 | Number of bytes written (should be 46) |
| -1 | Invalid pointer (null pointer passed) |
| -2 | Buffer too small (capacity < 46 bytes) |
| -99 | Panic caught in Rust code |

#### Packet Layout

```
Offset  Size    Field            Description
─────────────────────────────────────────────────────────
0x00    1       packet_type      PacketType::SpriteMessage (0x04)
0x01    1       magic           0xCC
0x02    16      request_uuid     UUID v7 (auto-generated)
0x12    1       operation       Sprite operation
0x13    1       padding1        Alignment padding
0x14    1       sprite_type     Sprite type
0x15    3       padding2        Alignment padding
0x18    16      id              Sprite UUID (from id_ptr)
0x28    2       x               X position
0x2A    2       y               Y position
0x2C    2       padding3        Alignment padding
─────────────────────────────────────────────────────────
Total:  46 bytes
```

### packet_builder_create_authenticate

Create an Authenticate packet with auto-generated UUID v7.

#### Function Signature

```rust
pub unsafe extern "C" fn packet_builder_create_authenticate(
    user_id_ptr: *const u8,   // User UUID as 16 bytes
    out_ptr: *mut u8,          // Output buffer pointer
    capacity: usize,            // Output buffer capacity
) -> i32                      // Returns bytes written or error code
```

#### Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| user_id_ptr | *const u8 | User UUID as 16 bytes (little-endian) |
| out_ptr | *mut u8 | Output buffer pointer (caller-allocated) |
| capacity | usize | Output buffer capacity (must be ≥ 34 bytes) |

#### Return Value

| Value | Description |
|-------|-------------|
| > 0 | Number of bytes written (should be 34) |
| -1 | Invalid pointer (null pointer passed) |
| -2 | Buffer too small (capacity < 34 bytes) |
| -99 | Panic caught in Rust code |

#### Packet Layout

```
Offset  Size    Field            Description
─────────────────────────────────────────────────────────
0x00    1       packet_type      PacketType::Authenticate (0x01)
0x01    1       magic           0xCC
0x02    16      request_uuid     UUID v7 (auto-generated)
0x12    16      user_id         User UUID (from user_id_ptr)
─────────────────────────────────────────────────────────
Total:  34 bytes
```

### packet_builder_create_keep_alive

Create a KeepAlive packet with auto-generated UUID v7.

#### Function Signature

```rust
pub unsafe extern "C" fn packet_builder_create_keep_alive(
    out_ptr: *mut u8,          // Output buffer pointer
    capacity: usize,            // Output buffer capacity
) -> i32                      // Returns bytes written or error code
```

#### Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| out_ptr | *mut u8 | Output buffer pointer (caller-allocated) |
| capacity | usize | Output buffer capacity (must be ≥ 18 bytes) |

#### Return Value

| Value | Description |
|-------|-------------|
| > 0 | Number of bytes written (should be 18) |
| -1 | Invalid pointer (null pointer passed) |
| -2 | Buffer too small (capacity < 18 bytes) |
| -99 | Panic caught in Rust code |

#### Packet Layout

```
Offset  Size    Field            Description
─────────────────────────────────────────────────────────
0x00    1       packet_type      PacketType::KeepAlive (0x00)
0x01    1       magic           0xCC
0x02    16      request_uuid     UUID v7 (auto-generated)
─────────────────────────────────────────────────────────
Total:  18 bytes (header only)
```

### packet_builder_get_error_string

Get human-readable error description for error code.

#### Function Signature

```rust
pub extern "C" fn packet_builder_get_error_string(
    error_code: i32            // Error code from PacketBuilder function
) -> *const u8               // Returns pointer to null-terminated UTF-8 string
```

#### Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| error_code | i32 | Error code returned by PacketBuilder function |

#### Return Value

| Value | Description |
|-------|-------------|
| *const u8 | Pointer to null-terminated UTF-8 error string |
| "Unknown error code: X" | If error code not recognized |

#### Example

```csharp
int result = packet_builder_create_player_pos(...);
if (result < 0) {
    IntPtr errorPtr = packet_builder_get_error_string(result);
    string error = Marshal.PtrToStringUTF8(errorPtr);
    Debug.LogError($"Failed to create packet: {error}");
}
```

## C# API

### PacketBuilder Class

High-level C# wrapper for PacketBuilder FFI functions.

#### Namespace

```csharp
namespace Unity.Network
```

#### DLL Name

```csharp
private const string DLL_NAME = "unity_network";
```

### Public Methods

#### CreatePlayerPos

Create a PlayerPos packet with auto-generated UUID v7.

```csharp
public static byte[] CreatePlayerPos(Guid playerUuid, int x, int y)
```

**Parameters:**
- `playerUuid` (Guid): Player UUID
- `x` (int): X position
- `y` (int): Y position

**Returns:** Complete packet as byte array (44 bytes)

**Throws:** `InvalidOperationException` if packet creation fails

**Example:**

```csharp
Guid playerId = Guid.Parse("12345678-1234-1234-1234-123456789abc");
int x = 100;
int y = 200;

try {
    byte[] packet = PacketBuilder.CreatePlayerPos(playerId, x, y);
    networkClient.Send(packet);
    Debug.Log($"Sent PlayerPos packet: {packet.Length} bytes");
} catch (InvalidOperationException ex) {
    Debug.LogError($"Failed to create packet: {ex.Message}");
}
```

#### CreateGameState

Create a GameState packet with auto-generated UUID v7.

```csharp
public static byte[] CreateGameState(uint tick, uint playerCount)
```

**Parameters:**
- `tick` (uint): Server tick or timestamp
- `playerCount` (uint): Number of players or message type

**Returns:** Complete packet as byte array (36 bytes)

**Throws:** `InvalidOperationException` if packet creation fails

**Example:**

```csharp
uint serverTick = 999999;
uint playerCount = 42;

byte[] packet = PacketBuilder.CreateGameState(serverTick, playerCount);
networkClient.Send(packet);
```

#### CreateSpriteMessage

Create a SpriteMessage packet with auto-generated UUID v7.

```csharp
public static byte[] CreateSpriteMessage(
    SpriteOperation operation, 
    SpriteType spriteType, 
    Guid spriteUuid, 
    short x, 
    short y
)
```

**Parameters:**
- `operation` (SpriteOperation): Sprite operation type
- `spriteType` (SpriteType): Sprite type
- `spriteUuid` (Guid): Sprite UUID
- `x` (short): X position
- `y` (short): Y position

**Returns:** Complete packet as byte array (46 bytes)

**Throws:** `InvalidOperationException` if packet creation fails

**Example:**

```csharp
byte[] packet = PacketBuilder.CreateSpriteMessage(
    SpriteOperation.Update,
    SpriteType.Serrif,
    spriteId,
    50,
    75
);
networkClient.Send(packet);
```

#### CreateAuthenticate

Create an Authenticate packet with auto-generated UUID v7.

```csharp
public static byte[] CreateAuthenticate(Guid userUuid)
```

**Parameters:**
- `userUuid` (Guid): User UUID

**Returns:** Complete packet as byte array (34 bytes)

**Throws:** `InvalidOperationException` if packet creation fails

**Example:**

```csharp
Guid userId = Guid.Parse("87654321-4321-4321-4321-cba987654321");
byte[] packet = PacketBuilder.CreateAuthenticate(userId);
networkClient.Send(packet);
```

#### CreateKeepAlive

Create a KeepAlive packet with auto-generated UUID v7.

```csharp
public static byte[] CreateKeepAlive()
```

**Parameters:** None

**Returns:** Complete packet as byte array (18 bytes)

**Throws:** `InvalidOperationException` if packet creation fails

**Example:**

```csharp
byte[] packet = PacketBuilder.CreateKeepAlive();
networkClient.Send(packet);
```

### Enums

#### SpriteOperation

Sprite operation types.

```csharp
public enum SpriteOperation
{
    Create = 0,     // Create new sprite
    Update = 1,     // Update sprite position
    Delete = 2,     // Delete sprite
    Snapshot = 3    // Snapshot of all sprites
}
```

#### SpriteType

Sprite type enum.

```csharp
public enum SpriteType
{
    Serrif = 0      // Serrif sprite
}
```

## Usage Examples

### Basic Packet Creation

```csharp
using Unity.Network;

// Create player position packet
Guid playerId = Guid.Parse("12345678-1234-1234-1234-123456789abc");
byte[] playerPosPacket = PacketBuilder.CreatePlayerPos(playerId, 100, 200);

// Create game state packet
byte[] gameStatePacket = PacketBuilder.CreateGameState(999999, 42);

// Create sprite message packet
byte[] spritePacket = PacketBuilder.CreateSpriteMessage(
    SpriteOperation.Update,
    SpriteType.Serrif,
    Guid.NewGuid(),
    50,
    75
);

// Send packets
networkClient.Send(playerPosPacket);
networkClient.Send(gameStatePacket);
networkClient.Send(spritePacket);
```

### Error Handling

```csharp
try {
    byte[] packet = PacketBuilder.CreatePlayerPos(playerId, x, y);
    networkClient.Send(packet);
} catch (InvalidOperationException ex) {
    Debug.LogError($"Packet creation failed: {ex.Message}");
    
    // Common errors:
    // - Invalid pointer (null pointer passed)
    // - Buffer too small (internal buffer insufficient)
    // - Invalid packet type
    // - Panic caught (internal error)
}
```

### Integration with NetworkProfiler

```csharp
using Unity.Network;
using Unity.Profiler;

// Generate UUID v7 in Rust (auto-generated in packet header)
Guid playerId = Guid.Parse("12345678-1234-1234-1234-123456789abc");

// Create packet (UUID v7 auto-generated by Rust)
byte[] packet = PacketBuilder.CreatePlayerPos(playerId, 100, 200);

// Extract UUID for profiling
Guid requestUuid = ExtractUuidFromPacket(packet);

// Track request with profiler
profiler.StartRequest(RequestType.PlayerPosUpdate, requestUuid);
networkClient.Send(packet);
```

### Batch Packet Creation

```csharp
// Create multiple player position packets efficiently
List<byte[]> packets = new List<byte[]>();

foreach (var player in players) {
    byte[] packet = PacketBuilder.CreatePlayerPos(
        player.Id,
        (int)player.X,
        (int)player.Y
    );
    packets.Add(packet);
}

// Send all packets
foreach (var packet in packets) {
    networkClient.Send(packet);
}
```

## Performance Characteristics

### Packet Creation Times

- **First call**: ~30-40 ms (DLL loading overhead)
- **Subsequent calls**: < 1 μs per packet (after warmup)
- **Throughput**: > 3,000,000 packets/second

### Memory Usage

- **Per packet**: 128 bytes internal buffer (temporary)
- **Output packet**: 18-46 bytes (depending on packet type)
- **Zero allocations**: After first call, no further heap allocations

### Thread Safety

- **Safe**: All FFI functions are thread-safe
- **Concurrency**: Multiple threads can call PacketBuilder simultaneously
- **No locking**: No internal synchronization required

## Best Practices

### 1. Always Use PacketBuilder

**Don't:** Manually construct packets in Unity
```csharp
// ❌ DON'T DO THIS
var header = new PacketHeader(PacketType.PlayerPos, UuidV7.NewUuid());
var packet = new PlayerPos { header = header, id = playerId, x = 100, y = 200 };
byte[] bytes = packet.ToBytes();
```

**Do:** Use PacketBuilder
```csharp
// ✅ DO THIS
byte[] packet = PacketBuilder.CreatePlayerPos(playerId, 100, 200);
```

### 2. Handle Errors Gracefully

```csharp
try {
    byte[] packet = PacketBuilder.CreatePlayerPos(playerId, x, y);
    networkClient.Send(packet);
} catch (InvalidOperationException ex) {
    Debug.LogError($"Packet creation failed: {ex.Message}");
    // Retry or fallback logic
}
```

### 3. Reuse GUIDs for Player Identity

```csharp
// Player ID is constant per player session
Guid playerId = GetPlayerId(); // Retrieved from server or session

// Use same ID for all packets
byte[] packet = PacketBuilder.CreatePlayerPos(playerId, x, y);
```

### 4. Batch Operations

```csharp
// Create multiple packets before sending
List<byte[]> packets = new List<byte[]>();

for (int i = 0; i < 100; i++) {
    byte[] packet = PacketBuilder.CreatePlayerPos(playerId, i * 10, i * 20);
    packets.Add(packet);
}

// Send all at once
foreach (var packet in packets) {
    networkClient.Send(packet);
}
```

## Migration from Old API

### Before (Deprecated)

```csharp
// Generate UUID v7 in C#
Guid requestUuid = UuidV7.NewUuid();

// Construct packet manually
var header = new PacketHeader(PacketType.PlayerPos, requestUuid);
header.SetUuid(requestUuid);

var packet = new PlayerPos {
    header = header,
    id = playerId,
    x = 100,
    y = 200
};

// Serialize to bytes
byte[] bytes = packet.ToBytes();
networkClient.Send(bytes);
```

### After (Recommended)

```csharp
// PacketBuilder handles everything
byte[] packet = PacketBuilder.CreatePlayerPos(playerId, 100, 200);
networkClient.Send(packet);
```

## Troubleshooting

### DLL Not Found

**Error:** `DllNotFoundException: Unable to load DLL 'unity_network'`

**Solution:**
1. Build Rust library: `cargo build --release -p unity-network`
2. Copy to Unity `Assets/Plugins/` directory
3. Verify architecture matches (ARM64 vs x86_64)

### Packet Size Mismatch

**Error:** `InvalidOperationException: Failed to create packet: Buffer too small`

**Solution:**
1. Verify buffer size is ≥ required packet size (18-46 bytes)
2. Check that PacketHeader is 18 bytes (includes UUID)
3. Ensure struct alignment matches Rust (`repr(C)`)

### UUID Not Propagating

**Symptom:** Server doesn't receive UUID in packet header

**Solution:**
1. Verify PacketBuilder is being used (not manual packet construction)
2. Check that bytes 2-17 of packet contain non-zero UUID
3. Verify server reads UUID from correct offset

### Performance Issues

**Symptom:** First packet creation is slow (~30-40 ms)

**Solution:**
1. This is expected (DLL loading overhead)
2. Add warmup calls before timing benchmarks
3. Subsequent calls should be < 1 μs

## References

### Source Code

- **Rust FFI:** `unity-ffi/unity-network/src/packet_builder.rs`
- **C# Wrapper:** `unity-ffi/unity/PacketBuilder.cs`
- **Protocol Types:** `crates/mmorpg-protocol/src/modules/`

### Related Documentation

- **Migration Guide:** `unity-ffi/docs/MIGRATION_GUIDE.md`
- **Architecture:** `unity-ffi/docs/ARCHITECTURE.md`
- **Unity Setup:** `unity-ffi/README.md`

### Handover Documents

- **Handover 007:** FFI Architectural Cleanup
- **Handover 008:** Remaining Work Summary
- **Handover 009:** PacketBuilder .NET Testing
- **Handover 010:** PacketBuilder .NET Testing Completion

## Support

For issues, questions, or contributions:
1. Check existing handover documents
2. Review inline code documentation
3. Run tests to verify behavior
4. Consult architecture diagrams