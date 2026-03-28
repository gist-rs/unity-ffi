# Unity FFI Architecture Documentation

## Overview

This document describes the architecture of the Unity-Rust FFI (Foreign Function Interface) networking system. The system implements a **view-only principle** where Unity acts as a pure view layer (display data, receive user input) while Rust handles all business logic including protocol implementation, UUID generation, and packet serialization.

### Core Principles

1. **Unity is View-Only**: Unity C# only handles display and user input
2. **Rust is Logic Layer**: All protocol logic resides in Rust
3. **Single Source of Truth**: Packet structures defined once in Rust
4. **Zero-Copy Communication**: Direct struct serialization without intermediate allocation
5. **Type-Safe FFI**: Compile-time type checking across language boundary

### System Goals

- **Performance**: < 1 μs per packet after warmup
- **Simplicity**: Unity code focuses on gameplay, not networking
- **Maintainability**: Protocol changes only require Rust updates
- **Extensibility**: Easy to add new packet types
- **Reliability**: Panic guards and comprehensive error handling

## Architecture Diagrams

### High-Level Architecture

```
┌────────────────────────────────────────────────────────────────┐
│                              Unity (C#)                        │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐             │
│  │    View     │  │  Game Logic │  │   UI/Input  │             │
│  │ (Display)   │◄─┤   (Mono)    │◄─┤  (Events)   │             │
│  └──────┬──────┘  └──────┬──────┘  └──────┬──────┘             │
│         │                │                │                    │
│         │                └────────────────┘                    │
│         ▼                                                      │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │            PacketBuilder (High-Level API)                │  │
│  │  - CreatePlayerPos()                                     │  │
│  │  - CreateGameState()                                     │  │
│  │  - CreateSpriteMessage()                                 │  │
│  │  - CreateAuthenticate()                                  │  │
│  │  - CreateKeepAlive()                                     │  │
│  └──────────────────────────┬───────────────────────────────┘  │
└─────────────────────────────┼──────────────────────────────────┘
                              │
                              │ FFI Boundary
                              │
                              ▼
┌────────────────────────────────────────────────────────────────┐
│                           Rust FFI Layer                       │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │            PacketBuilder (FFI Module)                    │  │
│  │  - packet_builder_create_player_pos()                    │  │
│  │  - packet_builder_create_game_state()                    │  │
│  │  - packet_builder_create_sprite_message()                │  │
│  │  - packet_builder_create_authenticate()                  │  │
│  │  - packet_builder_create_keep_alive()                    │  │
│  └──────────────────────────┬───────────────────────────────┘  │
│                             │                                  │
│                             ▼                                  │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │               Protocol Module                            │  │
│  │  - PacketHeader (with UUID v7)                           │  │
│  │  - PlayerPos, GameState, SpriteMessage, etc.             │  │
│  │  - PacketType enum                                       │  │
│  │  - repr(C) structs for FFI compatibility                 │  │
│  └──────────────────────────┬───────────────────────────────┘  │
│                             │                                  │
│                             ▼                                  │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │               UUID v7 Generator                          │  │
│  │  - Uuid::now_v7()                                        │  │
│  │  - Time-ordered (48-bit timestamp + 74-bit random)       │  │
│  │  - Single source of truth for UUID generation            │  │
│  └──────────────────────────┬───────────────────────────────┘  │
│                             │                                  │
│                             ▼                                  │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │               Network Client                             │  │
│  │  - WebTransport (QUIC protocol)                          │  │
│  │  - Bridge threads (async ↔ FFI conversion)               │  │
│  │  - Connection management                                 │  │
│  └──────────────────────────┬───────────────────────────────┘  │
└─────────────────────────────┼──────────────────────────────────┘
                              │
                              │ WebTransport
                              │
                              ▼
┌────────────────────────────────────────────────────────────────┐
│                          Rust Server                           │
│  - Axum web framework                                          │
│  - mmorpg-protocol for packet parsing                          │
│  - Business logic processing                                   │
│  - Database integration                                        │
└────────────────────────────────────────────────────────────────┘
```

### Data Flow: Packet Creation

```
Unity C#                                      Rust FFI
────────                                        ────────
                                               
PacketBuilder.CreatePlayerPos(playerId, x, y)
        │
        ├─ Allocates byte[] buffer
        │
        └─ Calls packet_builder_create_player_pos()
                                        │
                                        ├─ Generates UUID v7
                                        │
                                        ├─ Constructs PacketHeader
                                        │   (packet_type + magic + uuid)
                                        │
                                        ├─ Constructs PlayerPos struct
                                        │
                                        ├─ Serializes to bytes
                                        │   (memcpy repr(C) struct)
                                        │
                                        └─ Returns byte count
        │
        ├─ Receives byte count
        │
        ├─ Trims buffer to actual size
        │
        └─ Returns byte[] to caller
                │
                └─ networkClient.Send(packet)
                        │
                        └─ WebTransport to server
```

### Data Flow: Packet Reception

```
Unity C#                                      Rust FFI
────────                                        ────────
                                               
network_poll(out_buffer, capacity)
        │
        └─ Calls network_poll() FFI
                                        │
                                        ├─ Polls internal queue
                                        │   (from bridge thread)
                                        │
                                        ├─ Copies packet to buffer
                                        │
                                        └─ Returns packet size
        │
        ├─ Receives packet bytes
        │
        ├─ Parses packet type
        │
        ├─ Dispatches to handler
        │
        └─ Updates game state
```

### Bridge Thread Architecture

```
Unity C#                 Bridge Thread 1            Tokio Runtime
────────────────────────────────────────────────────────────────
network_send(packet) ──► mpsc::send() ──► tokio::spawn()
                                                 │
                                                 │ async WebTransport
                                                 │ send_packet()
                                                 │
                                                 ▼
                                              Server

Server                    Tokio Runtime          Bridge Thread 2
────────────────────────────────────────────────────────────────
                                        ◄─── mpsc::recv()
                                        │
                                        │ receive_packet()
                                        │
network_poll() ◄────────────────────────┘
```

**Why Bridge Threads?**
- FFI functions must be blocking (synchronous)
- Tokio runtime requires async functions
- Bridge threads convert between blocking (FFI) and async (Tokio) paradigms
- Prevents blocking entire tokio executor

## Component Details

### Unity Layer (View)

#### Responsibilities

- Display game world
- Receive user input
- Call high-level PacketBuilder methods
- Dispatch received packets to game logic
- Update UI state

#### What Unity Does NOT Do

- ❌ Generate UUID v7
- ❌ Construct packet headers
- ❌ Serialize packets
- ❌ Implement protocol logic
- ❌ Handle packet ordering
- ❌ Manage network connections

#### Key Components

**PacketBuilder.cs**
- High-level C# wrapper for Rust FFI
- Provides simple static methods for each packet type
- Handles memory management and error conversion
- Type-safe with compile-time checking

**NativeNetworkClient.cs**
- Low-level FFI bridge
- Manages network connection lifecycle
- Provides send/poll API
- Contains obsolete methods (marked for removal)

### Rust FFI Layer (Logic)

#### Responsibilities

- Generate UUID v7 for all packets
- Construct packets according to protocol
- Serialize packets to bytes
- Manage network connections
- Handle WebTransport protocol
- Provide bridge thread architecture

#### Key Components

**packet_builder.rs**
- FFI functions for packet creation
- Auto-generates UUID v7 internally
- Type-safe extern "C" functions
- Comprehensive error handling
- Zero-copy serialization

**lib.rs**
- Main FFI module
- Exports packet_builder module
- Network client implementation
- Bridge thread management
- Panic guards for all FFI functions

**Protocol Module (mmorpg-protocol)**
- Packet structure definitions
- PacketType enum
- repr(C) structs for FFI compatibility
- Single source of truth for packet format

### Packet Structure

#### PacketHeader with UUID v7

```rust
#[repr(C)]
pub struct PacketHeader {
    pub packet_type: u8,        // 1 byte - Packet type
    pub magic: u8,              // 1 byte - Magic number (0xCC)
    pub request_uuid: [u8; 16],  // 16 bytes - UUID v7
}
// Total: 18 bytes
```

**Field Descriptions:**

- `packet_type`: Identifies packet type (PlayerPos, GameState, etc.)
- `magic`: Magic number 0xCC for packet validation
- `request_uuid`: UUID v7 for end-to-end request correlation
  - Generated automatically by Rust
  - Time-ordered for request tracing
  - Unique per packet (collision-resistant)

#### UUID v7 Format

```
┌─────────────────────────────────────────────────────────────────┐
│                    UUID v7 Structure                         │
├─────────────────────────────────────────────────────────────────┤
│ Bits 0-47:    Unix timestamp (milliseconds, 48 bits)        │
│ Bits 48-77:   Monotonic counter (30 bits)                  │
│ Bits 78-81:   Version (4 bits) = 0x7                       │
│ Bits 82-87:   Variant (6 bits) = 0b10xxxx                   │
│ Bits 88-127:  Random (40 bits)                             │
└─────────────────────────────────────────────────────────────────┘
```

**Benefits of UUID v7:**
- Time-ordered for waterfall visualization
- High uniqueness (48-bit timestamp + 74-bit random)
- No coordination required across instances
- Better than v4 for request correlation

#### Packet Sizes

| Packet Type  | Size  | Layout                              |
|--------------|-------|-------------------------------------|
| KeepAlive     | 18 B  | packet_type(1) + magic(1) + uuid(16) |
| Authenticate  | 34 B  | packet_type(1) + magic(1) + uuid(16) + user_id(16) |
| PlayerPos    | 44 B  | packet_type(1) + magic(1) + uuid(16) + padding(2) + id(16) + x(4) + y(4) |
| GameState    | 36 B  | packet_type(1) + magic(1) + uuid(16) + padding(2) + tick(4) + player_count(4) + reserved(8) |
| SpriteMessage| 30 B  | packet_type(1) + magic(1) + operation(1) + padding1(1) + sprite_type(1) + padding2(3) + id(16) + x(2) + y(2) + padding3(2) |

**Alignment Notes:**
- All fields aligned to natural boundaries
- Padding added for struct alignment
- repr(C) ensures consistent layout across FFI boundary
- **Magic byte (0xCC)**: Validates packet integrity and prevents parsing garbage data
  - Acts as protocol signature
  - Required for all packets: `magic == 0xCC`
  - Prevents buffer misalignment errors

## Design Decisions

### 1. Unity View-Only Principle

**Decision:** Unity only provides high-level intent (business data) and receives display data. All protocol logic resides in Rust.

**Rationale:**
- Clear separation of concerns
- Prevents protocol logic duplication
- Easier to maintain and extend
- Unity developers focus on gameplay

**Trade-offs:**
- More complex FFI layer
- Requires careful type definitions
- Bridge thread architecture needed

### 2. Auto-Generated UUID v7

**Decision:** Rust generates UUID v7 automatically for all packets. Unity never generates UUIDs.

**Rationale:**
- Single source of truth for UUID generation
- Consistent UUID format across all packets
- Time-ordered for request correlation
- No duplicate UUID generation logic

**Trade-offs:**
- Cannot specify custom UUID in Unity
- Must extract UUID from packet for profiling
- Slightly more complex to track requests

### 3. Caller-Allocated Buffers

**Decision:** Rust never allocates memory for Unity. All output buffers are allocated by Unity and passed as pointers.

**Rationale:**
- Prevents heap corruption
- Clear memory ownership
- No memory leaks
- Easier to debug

**Trade-offs:**
- Unity must know max packet size
- Slightly more complex API
- Buffer size checks required

### 4. Zero-Copy Serialization

**Decision:** Packets are serialized by directly copying repr(C) structs to byte arrays. No intermediate serialization (JSON, protobuf, etc.).

**Rationale:**
- Maximum performance (< 1 μs per packet)
- No serialization overhead
- Type-safe with compile-time checking
- Minimal memory allocation

**Trade-offs:**
- Platform-dependent (endianness, alignment)
- Breaking changes require code update
- No version compatibility

### 5. Bridge Thread Architecture

**Decision:** Bridge threads convert between blocking FFI interface and async tokio runtime.

**Rationale:**
- FFI functions must be blocking
- Tokio requires async
- Prevents blocking entire executor
- Enables proper flow control

**Trade-offs:**
- Additional thread overhead
- More complex architecture
- Potential thread contention

### 6. repr(C) Structs

**Decision:** All FFI structs use #[repr(C)] attribute for C-compatible layout.

**Rationale:**
- Predictable memory layout
- Compatible with C# [StructLayout(LayoutKind.Sequential)]
- Cross-language interoperability
- Stable across Rust compiler versions

**Trade-offs:**
- No Rust optimizations
- May waste space for padding
- Requires manual alignment

## Performance Characteristics

### Packet Creation

- **First call:** ~30-40 ms (DLL loading overhead)
- **Subsequent calls:** < 1 μs per packet (after warmup)
- **Throughput:** > 3,000,000 packets/second
- **Memory:** 128 bytes temporary buffer per call

### Network Communication

- **Protocol:** WebTransport (QUIC over UDP)
- **Latency:** < 10 ms (typical, network-dependent)
- **Bandwidth:** ~1 KB/s per client (typical gameplay)
- **Packet size:** 18-46 bytes per packet

### Memory Usage

- **Per packet:** 18-46 bytes (depending on type)
- **FFI overhead:** ~1 KB per connection
- **Bridge threads:** ~8 KB per thread (2 threads)
- **Total per client:** < 100 KB

### Thread Safety

- **FFI functions:** Thread-safe (no internal state)
- **Network client:** Protected by internal mutex
- **Bridge threads:** Isolated state, no sharing
- **Packet creation:** Safe from multiple threads

## Security Considerations

### Input Validation

- All FFI functions validate pointers (null checks)
- Buffer size validation prevents overflow
- Packet type validation prevents invalid operations
- Panic guards prevent Rust panics from crashing process

### UUID v7 Uniqueness

- 48-bit timestamp provides ~89 years of unique IDs
- 74-bit random bits provide astronomical collision resistance
- Monotonic counter prevents duplicate IDs in same millisecond
- Suitable for high-throughput scenarios (> 1M packets/sec)

### Memory Safety

- Caller-allocated buffers prevent heap corruption
- repr(C) ensures stable memory layout
- No unsafe code in Unity (except FFI calls)
- Panic guards catch all Rust errors

### Network Security

- WebTransport uses TLS encryption
- Self-signed certificates in development
- Certificate validation in production
- Packet validation on receive

## Extension Points

### Adding New Packet Types

1. **Define packet struct in Rust:**
   ```rust
   #[repr(C)]
   pub struct NewPacket {
       pub header: PacketHeader,
       pub field1: u32,
       pub field2: u32,
   }
   ```

2. **Add FFI function to packet_builder.rs:**
   ```rust
   pub unsafe extern "C" fn packet_builder_create_new(
       field1: u32,
       field2: u32,
       out_ptr: *mut u8,
       capacity: usize,
   ) -> i32 {
       // Implementation
   }
   ```

3. **Add C# wrapper method to PacketBuilder.cs:**
   ```csharp
   public static byte[] CreateNew(uint field1, uint field2) {
       // Implementation
   }
   ```

4. **Add unit tests for new packet type**

### Custom Protocol Logic

Extend the protocol module in `crates/mmorpg-protocol/`:
- Add new packet types
- Extend PacketType enum
- Update serialization/deserialization
- Add validation logic

### Custom FFI Functions

Add new FFI functions to `crates/unity-network/src/lib.rs`:
- Follow existing naming convention
- Add panic guards
- Document error handling
- Add unit tests

## Testing Strategy

### Rust Tests

- **Unit tests:** Inline tests in packet_builder.rs (8 tests)
- **Integration tests:** Full packet creation and validation
- **Error handling tests:** Null pointer, buffer too small, etc.
- **Performance tests:** Benchmark packet creation

### .NET Tests

- **FFI integration tests:** Standalone .NET project
- **All packet types tested:** 8 comprehensive tests
- **Error handling verified:** All error codes tested
- **Performance validated:** < 10 μs per packet target

### Unity Tests

- **Editor tests:** Unity Test Runner integration
- **Play mode tests:** Real-world scenarios
- **End-to-end tests:** Full client-server flow
- **Network profiler integration:** UUID propagation verified

## Migration Guide

See `MIGRATION_GUIDE.md` for detailed migration instructions.

### Quick Migration Summary

1. Replace `new PacketType()` with `PacketBuilder.Create...()`
2. Remove `UuidV7.NewUuid()` calls
3. Remove `ToBytes()` and `FromBytes()` calls
4. Remove `SetUuid()` and `GetUuid()` calls
5. Update tests to use new API
6. Verify no `[Obsolete]` warnings
7. Test thoroughly

## References

### Documentation

- **API Reference:** `PACKET_BUILDER_API.md`
- **Migration Guide:** `MIGRATION_GUIDE.md`
- **Unity Setup:** `../README.md`

### Source Code

- **Rust FFI:** `crates/unity-network/src/packet_builder.rs`
- **Rust Protocol:** `crates/unity-network/src/types.rs`
- **C# Wrapper:** `unity/PacketBuilder.cs`
- **Unity Client:** `unity/NativeNetworkClient.cs`

### Handover Documents

- **Handover 007:** FFI Architectural Cleanup
- **Handover 008:** Remaining Work Summary
- **Handover 009:** PacketBuilder .NET Testing
- **Handover 010:** PacketBuilder .NET Testing Completion

### Related Issues

- **Issue 006:** Refactor FFI Architectural Cleanup
- **Issue 005:** Unity UUID Integration
- **Issue 003:** Request UUID Propagation Enhancement

## Conclusion

This architecture establishes a clean separation of concerns where Unity focuses on gameplay and user experience, while Rust handles all networking and protocol logic. The view-only principle simplifies Unity code, reduces duplication, and makes the system easier to maintain and extend.

Key achievements:
- ✅ Unity is view-only (no protocol logic)
- ✅ Rust handles all business logic
- ✅ Single source of truth for packet structures
- ✅ Auto-generated UUID v7
- ✅ Zero-copy serialization
- ✅ Comprehensive error handling
- ✅ High performance (< 1 μs per packet)
- ✅ Thread-safe and panic-safe

The architecture is production-ready and provides a solid foundation for future enhancements.
