# Unity-FFI Tests

This directory contains comprehensive test suites for the Unity FFI system with UUID v7 support.

## Overview

The testing strategy uses a **standalone .NET test project** to validate Rust PacketBuilder FFI functions without requiring Unity Editor. This provides:

- **Faster iteration** - No Unity Editor startup time (~30 seconds)
- **Simplified debugging** - Can use .NET debugger without Unity
- **Isolated testing** - Test FFI layer independently
- **CI/CD friendly** - Can run in headless environments

## Test Structure

```
tests/
├── PacketBuilderTests/    # Standalone .NET test project
│   ├── Program.cs          # Main test suite (8 comprehensive tests)
│   ├── PacketBuilderTests.csproj
│   └── libunity_network.dylib  # Compiled Rust FFI library
├── test-client/           # Rust WebTransport test client
├── test-ffi-arch/         # FFI architecture tests
└── test-sprite-lifecycle/ # Sprite lifecycle integration tests
```

Note: The integration test server is located in `crates/game-server/` as a workspace member.
```

## Running Tests

### Build Rust Library

```bash
cd /path/to/unity-ffi
cargo build --release -p unity-network
```

### Build Game Server (for integration tests)

```bash
cd /path/to/unity-ffi
cargo build --release -p game-server
```

**Output**: `target/aarch64-apple-darwin/release/libunity_network.dylib`

### Copy Library to Test Directory

```bash
# macOS ARM64 (Apple Silicon)
cp target/aarch64-apple-darwin/release/libunity_network.dylib \
   tests/PacketBuilderTests/libunity_network.dylib

# macOS x86_64 (Intel)
cp target/x86_64-apple-darwin/release/libunity_network.dylib \
   tests/PacketBuilderTests/libunity_network.dylib
```

### Run .NET Tests

```bash
cd tests/PacketBuilderTests
dotnet run
```

**Expected Output**:
```
╔════════════════════════════════════════════════════════════╗
║   PacketBuilder FFI Integration Test                    ║
║   Testing Rust PacketBuilder from .NET                   ║
╚════════════════════════════════════════════════════════════╝

Test 1: PlayerPos Packet
  ✅ Success! Time: ~50 μs
  ✅ Packet size correct (44 bytes)

...

Tests Passed: 8/8
Tests Failed: 0/8

✅ All tests passed! PacketBuilder is working correctly.
```

## Test Suite

### Test 1: PlayerPos Packet
**Purpose**: Verify PlayerPos packet creation with UUID v7

**Validates**:
- Packet size: 44 bytes
- UUID v7 generation in header
- Player ID, X, Y coordinates
- Error handling

**Expected Size**: 44 bytes
```
Layout: header(18) + padding(2) + id(16) + x(4) + y(4)
```

### Test 2: GameState Packet
**Purpose**: Verify GameState packet creation with UUID v7

**Validates**:
- Packet size: 36 bytes
- UUID v7 generation in header
- Tick and player_count fields
- Error handling

**Expected Size**: 36 bytes
```
Layout: header(18) + padding(2) + tick(4) + player_count(4) + reserved(8)
```

### Test 3: SpriteMessage Packet
**Purpose**: Verify SpriteMessage packet creation with UUID v7

**Validates**:
- Packet size: 46 bytes
- UUID v7 generation in header
- Operation, sprite_type, sprite ID, position
- Error handling

**Expected Size**: 46 bytes
```
Layout: header(18) + operation(1) + padding1(1) + sprite_type(1) + padding2(3) + id(16) + x(2) + y(2) + padding3(2)
```

### Test 4: Authenticate Packet
**Purpose**: Verify Authenticate packet creation with UUID v7

**Validates**:
- Packet size: 34 bytes
- UUID v7 generation in header
- User ID (UUID)
- Error handling

**Expected Size**: 34 bytes
```
Layout: header(18) + user_id(16)
```

### Test 5: KeepAlive Packet
**Purpose**: Verify KeepAlive packet creation with UUID v7

**Validates**:
- Packet size: 18 bytes
- UUID v7 generation in header (minimal packet)
- Error handling

**Expected Size**: 18 bytes
```
Layout: header(18) only
```

### Test 6: Error Handling
**Purpose**: Verify error detection and reporting

**Validates**:
- Null pointer detection
- Buffer too small detection
- Error string retrieval
- Correct error codes (-1, -2, -3, -99)

**Error Codes**:
```
-1: InvalidPointer  (null pointer passed)
-2: BufferTooSmall (output buffer cannot hold packet)
-3: InvalidPacketType (invalid packet type)
-99: PanicCaught (panic caught in Rust code)
```

### Test 7: UUID v7 Uniqueness
**Purpose**: Verify UUID v7 generation properties

**Validates**:
- 100 UUIDs are unique
- UUIDs are time-ordered
- UUID v7 format (version 7, variant 1)

### Test 8: Performance Benchmark
**Purpose**: Benchmark packet creation performance

**Validates**:
- 10,000 packet creations
- Average time per packet < 10 μs
- Packets per second > 100,000

**Expected Results**:
```
Total time: ~3 ms
Average per packet: < 1 μs
Packets per second: > 3,000,000
```

## Protocol Structure

### PacketHeader with UUID v7

All packets include a 16-byte UUID v7 in header for end-to-end request correlation.

```rust
#[repr(C)]
pub struct PacketHeader {
    pub packet_type: u8,     // 1 byte
    pub magic: u8,            // 1 byte (0xCC)
    pub request_uuid: [u8; 16], // 16 bytes (UUID v7)
}
// Total: 18 bytes
```

### Packet Sizes (UUID-enabled)

| Packet Type  | Size | Layout |
|--------------|-------|--------|
| KeepAlive     | 18 bytes | header(18) |
| Authenticate  | 34 bytes | header(18) + user_id(16) |
| PlayerPos    | 44 bytes | header(18) + padding(2) + id(16) + x(4) + y(4) |
| GameState    | 36 bytes | header(18) + padding(2) + tick(4) + player_count(4) + reserved(8) |
| SpriteMessage| 46 bytes | header(18) + operation(1) + padding1(1) + sprite_type(1) + padding2(3) + id(16) + x(2) + y(2) + padding3(2) |

## Architecture

### UUID v7 Integration

**Auto-Generated**: Rust generates UUID v7 internally for all packets

**Time-Ordered**: UUIDs are sortable by creation time (timestamp in first 48 bits)

**End-to-End**: UUIDs correlate requests across client/server/profiler

**Collision-Resistant**: High-precision timestamp (48 bits) + random bits (74 bits)

### FFI Interface

**Type-Safe**: C# P/Invoke declarations match Rust signatures exactly

**Error Handling**: Proper error codes (-1, -2, -3, -99)

**Zero-Copy**: Direct struct serialization without intermediate serialization

**Performance**: < 1 μs per packet after warmup

### PacketBuilder vs Manual Construction

**Old Approach (Deprecated)**:
- Unity manually constructed packets byte-by-byte
- Required manual UUID v7 generation in C#
- Prone to alignment and endianness issues
- Duplicate code across client and server

**New Approach (Recommended)**:
- Rust PacketBuilder generates all packets
- Auto-generated UUID v7 in Rust
- Type-safe with compile-time checking
- Single source of truth for packet structures

## Troubleshooting

### Size Mismatch

If struct sizes don't match:

1. Verify PacketHeader is 18 bytes (includes UUID)
2. Check that Rust structs use `repr(C)`
3. Ensure C# P/Invoke signatures match Rust function signatures
4. Run struct size tests: `cargo test -p mmorpg-protocol test.*_size`

### Test Failures

If .NET tests fail:

1. Ensure Rust library is built: `cargo build --release -p unity-network`
2. Copy dylib to test directory
3. Check architecture match (ARM64 vs x86_64)
4. Verify error codes match Rust enum values

### UUID Issues

If UUIDs don't propagate:

1. Verify `PacketHeader::with_uuid()` is used in Rust
2. Check that header bytes include 16-byte UUID at offset 2
3. Ensure server reads UUID from header correctly
4. Test with end-to-end integration tests

### Performance Issues

If packets are slow:

1. First call overhead is expected (~1-2 ms for DLL loading)
2. Add warmup calls before benchmarking
3. Target: < 10 μs per packet after warmup
4. Profile with Unity Profiler or Rust benchmarks

### Architecture Mismatch

If you get architecture errors:

**macOS ARM64 (Apple Silicon)**:
```bash
cargo build --release --target aarch64-apple-darwin -p unity-network
cp target/aarch64-apple-darwin/release/libunity_network.dylib \
   tests/PacketBuilderTests/libunity_network.dylib
```

**macOS x86_64 (Intel)**:
```bash
cargo build --release --target x86_64-apple-darwin -p unity-network
cp target/x86_64-apple-darwin/release/libunity_network.dylib \
   tests/PacketBuilderTests/libunity_network.dylib
```

**Check Current Architecture**:
```bash
uname -m  # Output: arm64 or x86_64
```

## Additional Testing

### Rust Protocol Tests

Verify struct sizes in Rust:

```bash
cd unity-ffi
cargo test -p unity-network test.*_size
```

**Expected Output**:
```
test test_packet_header_size ... ok
test test_player_pos_size ... ok
test test_game_state_size ... ok
test test_sprite_message_size ... ok
test test_authenticate_size ... ok
```

### Unity Editor Tests (TODO)

Unity Editor integration tests should be created in the Unity project's `Assets/Tests/` directory:

1. **PacketBuilderIntegrationTests.cs** - Test PacketBuilder in Unity Editor
2. **EndToEndUuidTests.cs** - Test UUID propagation through Unity
3. **NetworkClientTests.cs** - Test network client with real server

## References

### Handover Documents

- **Handover 008**: Remaining Work Summary - Post-FFI Architectural Cleanup
- **Handover 009**: PacketBuilder .NET Testing and Integration
- **Handover 010**: PacketBuilder .NET Testing Completion

### Source Code

- **Rust FFI**: `crates/unity-network/src/packet_builder.rs`
- **Protocol Types**: `crates/unity-network/src/types.rs`
- **.NET Tests**: `tests/PacketBuilderTests/Program.cs`
- **Unity Scripts**: `unity/PacketBuilder.cs`

### Related Work

- **Issue 006**: Refactor FFI Architectural Cleanup
- **Issue 005**: Unity UUID Integration
- **unity-network**: FFI layer for Unity integration

## Notes

- These tests are critical for ensuring FFI works correctly with UUID-enabled packets
- Memory alignment issues cause crashes or data corruption
- Always verify struct sizes after making changes to packet structures
- Use the standalone .NET tests for rapid iteration before Unity Editor testing
- Unity Editor tests will be added in future work (see Handover 008)

## Definition of Done

- [x] All 8 .NET tests pass consistently
- [x] All packet sizes match Rust structs
- [x] All FFI signatures match Rust functions
- [x] All error codes match Rust enum values
- [x] Performance meets target (< 10 μs after warmup)
- [x] UUID v7 generation verified
- [x] Error handling verified
- [x] Comprehensive documentation created
- [x] Test directory structure organized

## Future Work

- [ ] Add Unity Editor integration tests
- [ ] Add end-to-end tests with real server (use `crates/game-server`)
- [ ] Add performance regression tests
- [ ] Add memory leak detection tests
- [ ] Integrate with CI/CD pipeline
- [ ] Add test coverage reporting
- [ ] Add NUnit or xUnit framework