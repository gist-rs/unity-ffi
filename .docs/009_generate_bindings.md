# 009 Generating Unity C# Bindings Guide

## Document Purpose

This document (#009) provides a comprehensive guide for auto-generating Unity C# bindings from Rust FFI types using the `#[derive(GameComponent)]` macro system. It follows:

- **001_architecture.md** - System architecture and design principles
- **002_profiler_overview.md** - Profiler system overview
- **003_profiler_quickstart.md** - Quick start guide for profiler
- **004_before_after_comparison.md** - Comparison of manual vs auto-generated approaches
- **005_packet_builder_api.md** - Packet Builder API documentation
- **006_migration_guide.md** - Migration guide from manual to auto-generated
- **007_packet_builder_migration.md** - Packet Builder specific migration
- **008_unity_sprite_setup.md** - Unity sprite system setup
- **009_generate_bindings.md** - This document - Generating Unity C# bindings

## Overview

This guide explains how to auto-generate Unity C# bindings from Rust FFI types using `#[derive(GameComponent)]` macro system. The system ensures **0 lines of manual C# code** with complete automation and a single source of truth (Rust).

## What Gets Generated

For each `#[derive(GameComponent)]` struct in `unity-network/src/types.rs`, the system generates:

### Complete C# Struct Definition
```csharp
[StructLayout(LayoutKind.Sequential, Pack = 1)]
public struct PlayerPos
{
    public byte packet_type;
    public byte magic;
    public Guid request_uuid;
    public ulong player_id;
    public float x;
    public float y;
    [MarshalAs(UnmanagedType.ByValArray, SizeConst = 6)]
    private byte[] _padding;

    public static readonly int Size = 40;
    public static readonly Guid UUID = Guid.Parse("02e02ca7-c4c7-7871-b515-2ae36f6a9cd8");
    
    public bool validate() { ... }
    public byte[] AsBytes() { ... }
    public static PlayerPos FromBytes(byte[] data) { ... }
    public override string ToString() { ... }
}
```

### Generated Features
- ✅ **StructLayout attribute** - Ensures memory layout matches Rust exactly
- ✅ **Field mapping** - Rust types automatically mapped to C# types
- ✅ **Padding fields** - Auto-generated with proper `[MarshalAs]` attributes
- ✅ **Size constant** - Exact byte count for validation
- ✅ **UUID constant** - Auto-generated UUID v7 for type identification
- ✅ **Validation methods** - Magic number checking
- ✅ **Zero-copy methods** - `AsBytes()` and `FromBytes()` for direct memory access
- ✅ **ToString()** - Debug-friendly output

## Quick Start

### 1. Generate Bindings
```bash
cd unity-ffi
./scripts/generate_bindings.sh
```

This will:
- Run the generation example
- Save output to `unity/Generated/GameFFI.cs`
- Validate the file was created
- Show summary of generated structs and enums

### 2. Verify Generation
```bash
# Check generated file
cat unity/Generated/GameFFI.cs

# Verify memory layout matches Rust
cargo run --package unity-network --example extract_layout

# List all generated UUIDs
cargo run --package unity-network --example extract_uuids
```

### 3. Use in Unity
```csharp
using GameFFI;

// Parse received packet
PlayerPos pos = PlayerPos.FromBytes(packetBytes);

// Validate packet
if (pos.validate()) {
    // Use the data
    Debug.Log($"Player {pos.player_id} at ({pos.x}, {pos.y})");
}
```

## Development Workflow

### Adding New FFI Types

#### Step 1: Define Rust Struct
```rust
// unity-network/src/types.rs

#[repr(C)]
#[derive(GameComponent, Debug, Clone, Copy)]
pub struct NewPacket {
    pub packet_type: u8,
    pub magic: u8,
    pub entity_id: u64,
    pub health: f32,
    pub stamina: f32,
}
```

#### Step 2: Update PacketType Enum
```rust
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PacketType {
    KeepAlive = 0,
    PlayerPos = 1,
    GameState = 2,
    SpriteMessage = 3,
    NewPacket = 4,  // Add new type
}
```

#### Step 3: Generate Bindings
```bash
./scripts/generate_bindings.sh
```

#### Step 4: Update Generation Script (if needed)
If you added a new struct, update `unity-network/examples/generate_unity_cs.rs`:
```rust
use unity_network::{GameState, PacketHeader, PlayerPos, SpriteMessage, NewPacket};

fn main() {
    // ... existing code ...
    
    // Add your new struct
    extract_struct_from_generated_code(NewPacket::generate_unity_cs());
    
    // ... rest of code ...
}
```

#### Step 5: Test in Unity
- Unity will auto-detect the file change
- If not: `Assets > Refresh`
- Test packet serialization/deserialization

### Modifying Existing Types

When you change a struct:
- Add/remove field
- Change field type
- Reorder fields

**Always regenerate bindings:**
```bash
./scripts/generate_bindings.sh
```

The UUID will automatically change if you make a breaking change, which helps clients detect protocol mismatches.

## Generated Structs Reference

### Currently Generated Types

| Struct | Size | Description |
|--------|------|-------------|
| `PacketHeader` | 2 bytes | Common packet header with type and magic |
| `PlayerPos` | 40 bytes | Player position update with UUID |
| `GameState` | 20 bytes | Game state snapshot |
| `SpriteMessage` | 30 bytes | Sprite operation message |

### Enums

| Enum | Values | Description |
|------|--------|-------------|
| `PacketType` | KeepAlive, PlayerPos, GameState, SpriteMessage | Packet type discriminator |
| `SpriteOp` | Create, Update, Delete, Snapshot | Sprite operation types |
| `SpriteType` | Serrif | Sprite type enumeration |
| `FfiError` | Success, InvalidPointer, etc. | FFI error codes |

## Type Mapping

### Rust to C# Type Mapping

| Rust Type | C# Type | Notes |
|-----------|---------|-------|
| `u8` | `byte` | |
| `i8` | `sbyte` | |
| `u16` | `ushort` | |
| `i16` | `short` | |
| `u32` | `uint` | |
| `i32` | `int` | |
| `u64` | `ulong` | |
| `i64` | `long` | |
| `f32` | `float` | |
| `f64` | `double` | |
| `bool` | `bool` | |
| `[u8; N]` | `byte[]` | With `MarshalAs` attribute |
| `[T; N]` | `T[]` | With `MarshalAs` attribute |
| `uuid::Uuid` | `Guid` | |

## Memory Layout

### Understanding Padding

Rust automatically adds padding for alignment. The system generates matching padding in C#:

```rust
// Rust
#[repr(C)]
#[derive(GameComponent)]
pub struct PlayerPos {
    pub packet_type: u8,  // offset 0
    pub magic: u8,        // offset 1
    pub request_uuid: uuid::Uuid,  // offset 2 (16 bytes)
    pub player_id: u64,   // offset 18
    pub x: f32,           // offset 26
    pub y: f32,           // offset 30
    // implicit padding: offset 34-39 (6 bytes) for alignment
}
```

```csharp
// C#
public struct PlayerPos {
    public byte packet_type;       // offset 0
    public byte magic;             // offset 1
    public Guid request_uuid;      // offset 2 (16 bytes)
    public ulong player_id;        // offset 18
    public float x;                // offset 26
    public float y;                // offset 30
    [MarshalAs(UnmanagedType.ByValArray, SizeConst = 6)]
    private byte[] _padding;       // offset 34-39 (6 bytes)
}
```

### Verify Layout

Always verify memory layout after changes:
```bash
cargo run --package unity-network --example extract_layout
```

This shows:
- Total size in bytes
- Field offsets
- Padding locations
- Alignment requirements

## UUID System

### What Are UUIDs?

Each `#[derive(GameComponent)]` struct gets a deterministic UUID v7:
- Generated from struct signature
- Same struct = same UUID
- Breaking change = new UUID
- Detects protocol version mismatches

### UUID Format

```rust
// Example UUID
UUID = "02e02ca7-c4c7-7871-b515-2ae36f6a9cd8"
```

Version: 7 (time-ordered, sortable)
Variant: RFC4122

### When UUID Changes

UUID changes when you:
- Add/remove field
- Change field type
- Reorder fields
- Add breaking attributes (`#[field(skip)]`, etc.)

UUID does NOT change when you:
- Change field name
- Add non-breaking attributes (`#[field(min)]`, `#[field(max)]`)
- Use `#[hash = "name"]` mode

### Extract UUIDs

```bash
cargo run --package unity-network --example extract_uuids
```

## Troubleshooting

### Problem: Generated C# Doesn't Match Rust

**Symptoms**: Serialization/deserialization fails, data corruption

**Solution**:
```bash
# Check memory layout
cargo run --package unity-network --example extract_layout

# Compare Rust and C# offsets
# Ensure padding fields match exactly
```

### Problem: Protocol Version Mismatch Errors

**Symptoms**: "Protocol version mismatch" in logs

**Solution**:
```bash
# Extract UUIDs to see current protocol version
cargo run --package unity-network --example extract_uuids

# Compare client and server UUIDs
# If different, regenerate bindings on client side
```

### Problem: Struct Size Mismatch

**Symptoms**: `BufferTooSmall` errors

**Solution**:
```bash
# Check actual sizes
cargo run --package unity-network --example extract_layout

# Verify Size constant in generated C# matches Rust size
# Check for missing padding fields
```

### Problem: Unity Can't Find Generated Types

**Symptoms**: `CS0246: The type or namespace name 'PlayerPos' could not be found`

**Solution**:
```bash
# Verify file exists
ls -la unity/Generated/GameFFI.cs

# Check namespace is correct
grep "namespace GameFFI" unity/Generated/GameFFI.cs

# Refresh Unity
# In Unity: Assets > Refresh
```

### Problem: Generation Script Fails

**Symptoms**: `cargo run` fails with errors

**Solution**:
```bash
# Check Rust compiles
cargo build --package unity-network

# Check example compiles
cargo build --package unity-network --example generate_unity_cs

# Try running manually
cargo run --package unity-network --example generate_unity_cs
```

## Advanced Usage

### Custom Unity Names

```rust
#[repr(C)]
#[derive(GameComponent)]
#[unity(name = "PlayerPositionUnity")]
pub struct PlayerPos {
    // fields...
}
```

Generates `public struct PlayerPositionUnity` in C#.

### Loose Hash Mode

For fast iteration where you want UUID to stay stable:
```rust
#[repr(C)]
#[derive(GameComponent)]
#[hash = "name"]
pub struct ProtoState {
    pub data: Vec<u8>,
    // Can add/remove fields without changing UUID
}
```

### Manual UUID Assignment

For legacy code requiring stable UUID:
```rust
#[repr(C)]
#[derive(GameComponent)]
#[uuid = "fc8bd668-fc0a-4ab7-8b3d-f0f22bb539e2"]
pub struct LegacyComponent {
    // fields...
}
```

### Skipping Fields from Public API

```rust
#[repr(C)]
#[derive(GameComponent)]
pub struct InternalPacket {
    pub visible_field: u32,
    #[field(skip)]
    pub internal_state: u64,  // Not exposed in C#
}
```

## Best Practices

### 1. Always Regenerate After Changes
```bash
# After any change to FFI types
./scripts/generate_bindings.sh
```

### 2. Verify Memory Layout
```bash
# After adding/removing fields
cargo run --package unity-network --example extract_layout
```

### 3. Check UUIDs
```bash
# When debugging protocol issues
cargo run --package unity-network --example extract_uuids
```

### 4. Unity is VIEW-ONLY
- ✅ Use `PacketBuilder.CreateXxx()` to create packets
- ✅ Use `FromBytes()` to parse received packets
- ❌ Never manually create GameFFI structs in Unity
- ❌ Never manually set request_uuid fields

### 5. Single Source of Truth
- Rust FFI types are the ONLY source of truth
- C# bindings are auto-generated from Rust
- Never edit `GameFFI.cs` manually
- All changes start in Rust, then regenerate C#

## Architecture Notes

### Zero-Copy FFI

The system enables zero-copy FFI:
- No serialization/deserialization overhead
- Direct memory access between Rust and Unity
- Exact memory layout match required

### Protocol Versioning

UUID-based protocol versioning:
- Clients detect protocol mismatches automatically
- Breaking changes generate new UUIDs
- Graceful degradation for protocol version errors

### Type Safety

Compile-time type safety:
- Rust types are the single source of truth
- C# bindings generated automatically
- Memory layout verified at compile time

## Testing

### Unit Tests
```bash
# Test FFI types
cargo test --package unity-network

# Test macro
cargo test --package game-ffi
cargo test --package game-ffi-derive
```

### Integration Tests
```bash
# Test generation
cargo run --package unity-network --example extract_bindings

# Test layout
cargo run --package unity-network --example extract_layout

# Test UUIDs
cargo run --package unity-network --example extract_uuids
```

### Manual Testing in Unity

1. Generate bindings: `./scripts/generate_bindings.sh`
2. Open Unity and let it compile
3. Create test script to send/receive packets
4. Verify data integrity round-trip
5. Check memory layout matches

## File Locations

### Core Files
- **FFI types**: `unity-network/src/types.rs`
- **Macro implementation**: `crates/game-ffi-derive/src/derive/unity/mod.rs`
- **Example generator**: `unity-network/examples/generate_unity_cs.rs`
- **Build script**: `generate_bindings.sh`
- **Generated output**: `unity/Generated/GameFFI.cs`

### Example Scripts
- **Extract bindings**: `unity-network/examples/extract_bindings.rs`
- **Extract layout**: `unity-network/examples/extract_layout.rs`
- **Extract UUIDs**: `unity-network/examples/extract_uuids.rs`
- **Basic usage**: `crates/game-ffi/examples/basic_usage.rs`

## References

### Related Documentation
- `.docs/004_before_after_comparison.md` - Original design document
- `.issues/001_complete_unity_csharp_auto_generation.md` - Issue tracking
- `.handovers/001_unity_csharp_auto_generation_implementation.md` - Implementation handover

### Code Style
- Use `#[repr(C)]` for all FFI types
- Use `#[derive(GameComponent)]` for auto-generation
- Follow Rust naming conventions
- Keep structs under 1KB for performance

## Support

### Getting Help
- Check this guide first
- Review examples in `crates/game-ffi/examples/`
- Run `extract_bindings` to see what gets generated
- Run `extract_layout` to verify memory layout

### Common Pitfalls
1. Forgetting to regenerate bindings after changes
2. Manually editing `GameFFI.cs`
3. Not checking memory layout after adding fields
4. Trying to create packets in Unity instead of using PacketBuilder
5. Using wrong field types in Rust vs C#

## Quick Reference

### Essential Commands
```bash
# Generate bindings
./scripts/generate_bindings.sh

# Extract and verify
cargo run --package unity-network --example extract_bindings
cargo run --package unity-network --example extract_layout
cargo run --package unity-network --example extract_uuids

# Run tests
cargo test --package unity-network
```

### File Paths
```bash
# Edit FFI types
vim unity-network/src/types.rs

# Regenerate
./scripts/generate_bindings.sh

# Check output
cat unity/Generated/GameFFI.cs

# Verify layout
cargo run --package unity-network --example extract_layout
```

### Generated Output Location
```bash
unity/Generated/GameFFI.cs
```

## Conclusion

The Unity C# auto-generation system provides:
- ✅ 0 lines of manual C# code
- ✅ Complete automation
- ✅ Single source of truth (Rust)
- ✅ Zero-copy FFI with automatic binding generation
- ✅ Type safety with UUID-based protocol versioning

Follow this guide to maintain FFI bindings efficiently and correctly.