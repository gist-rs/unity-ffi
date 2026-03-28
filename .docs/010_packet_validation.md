# Packet Validation and Magic Byte

## Overview

This document explains the packet validation mechanism used in the Unity FFI project, with a focus on the magic byte (`0xCC`) that serves as the protocol signature.

## The Magic Byte

### What is it?

The magic byte is a **single-byte protocol signature** (`0xCC`) included in every packet sent between Unity and Rust. It serves as a unique identifier for our binary protocol.

### Value

```rust
const MAGIC: u8 = 0xCC;
```

### Location in Packet Structure

Every packet starts with the following fields:

```rust
#[repr(C)]
pub struct PacketHeader {
    pub packet_type: u8,        // 1 byte - Packet type identifier
    pub magic: u8,              // 1 byte - 0xCC (magic signature)
    pub request_uuid: [u8; 16], // 16 bytes - UUID v7
}
// Total header: 18 bytes
```

## Purpose of Magic Byte

### 1. Protocol Validation

Ensures received data is actually from our protocol and not garbage:

**✅ Valid Packet:**
```
[0x03][0xCC][UUID bytes...]
 ↑      ↑
 type   magic = 0xCC ✓
```

**❌ Invalid Packet:**
```
[0xFF][0x00][random data...]
 ↑      ↑
 wrong  magic ≠ 0xCC ✗
 type
```

### 2. Data Integrity

Detects:
- **Buffer misalignment** - Reading from wrong offset
- **Memory corruption** - Data overwritten in transit
- **Protocol mismatch** - Connecting to wrong server
- **Garbage data** - Parsing uninitialized memory

### 3. Safety in Zero-Copy FFI

In unsafe Rust and C# code with raw pointers, the magic byte prevents:

```rust
// DANGEROUS: Without validation, this could parse garbage as a packet
let packet = &*(ptr as *const SpriteMessage);
packet.x = 1000; // ← Could be corrupting random memory!
```

With validation:

```rust
// SAFE: Only parse if magic byte matches
let packet = &*(ptr as *const SpriteMessage);
if packet.magic != 0xCC {
    return Err(Error::InvalidPacket);
}
// Now we can trust the packet structure
```

## Validation Implementation

### Rust (Server Side)

```rust
impl SpriteMessage {
    pub fn validate(&self) -> bool {
        self.magic == PacketHeader::MAGIC 
            && self.packet_type == PacketType::SpriteMessage as u8
    }
}

// When sending packets:
pub fn broadcast_sprite(&self, message: &SpriteMessage) {
    // All packets created with magic = 0xCC by default
    let bytes = message.as_bytes();
    // ... send bytes to clients
}
```

### C# (Client Side)

```csharp
public unsafe struct SpriteMessage
{
    public byte packet_type;
    public byte magic;
    public byte operation;
    // ... other fields
    
    public bool Validate()
    {
        return magic == PacketHeader.MAGIC 
               && packet_type == (byte)PacketType.SpriteMessage;
    }
}

// When receiving packets:
private void HandleSpriteMessage(int length)
{
    if (!client.TryParseStruct<SpriteMessage>(length, out var spriteMsg))
    {
        Debug.LogWarning("Failed to parse SpriteMessage packet");
        return;
    }
    
    if (!spriteMsg.Validate())
    {
        Debug.LogWarning("Received invalid SpriteMessage packet");
        return;
    }
    
    // Safe to process packet
    HandleSpriteCreate(spriteMsg);
}
```

## Validation Flow

```
┌─────────────────────────────────────────────────────────────┐
│                     Packet Reception Flow                 │
└─────────────────────────────────────────────────────────────┘

1. Network receives raw bytes
   ↓
2. Buffer pointer cast to packet struct (unsafe!)
   ↓
3. Magic byte check: magic == 0xCC?
   ↓
   ├── NO → Reject packet, log error
   │
   └── YES → Check packet type
            ↓
        packet_type matches expected?
            ↓
            ├── NO → Wrong packet type, handle appropriately
            │
            └── YES → Packet is valid, process data
                     ↓
                 Extract fields, update game state
```

## Common Validation Failures

### 1. Buffer Offset Error

**Problem:** Reading from wrong position in buffer

```csharp
// WRONG: Offset by 4 bytes, magic becomes garbage
byte[] packetData = buffer[4:]; // Skip 4 bytes
var packet = *(SpriteMessage*)packetData;
// packet.magic is now garbage, validation fails
```

**Fix:** Read from correct offset

```csharp
// CORRECT: Start at buffer beginning
var packet = *(SpriteMessage*)buffer;
// packet.magic == 0xCC ✓
```

### 2. Struct Layout Mismatch

**Problem:** C# and Rust have different struct layouts

```csharp
// WRONG: C# struct has extra fields
[StructLayout(LayoutKind.Sequential, Pack = 1)]
public struct SpriteMessage {
    public PacketHeader header; // ← 18 bytes
    public byte operation;      // ← 1 byte
    public byte sprite_type;    // ← 1 byte
    public Guid id;             // ← 16 bytes
    public short x;             // ← 2 bytes
    public short y;             // ← 2 bytes
    // Total: 40 bytes (wrong!)
}
```

**Fix:** Match Rust repr(C) exactly

```csharp
// CORRECT: Flat fields match Rust
[StructLayout(LayoutKind.Sequential, Pack = 1)]
public unsafe struct SpriteMessage {
    public byte packet_type;    // ← 1 byte (matches Rust)
    public byte magic;          // ← 1 byte (matches Rust)
    public byte operation;      // ← 1 byte (matches Rust)
    public byte padding1;       // ← 1 byte (matches Rust)
    public byte sprite_type;    // ← 1 byte (matches Rust)
    public fixed byte padding2[3]; // ← 3 bytes (matches Rust)
    public fixed byte id[16];   // ← 16 bytes (matches Rust)
    public short x;             // ← 2 bytes (matches Rust)
    public short y;             // ← 2 bytes (matches Rust)
    public fixed byte padding3[2]; // ← 2 bytes (matches Rust)
    // Total: 30 bytes ✓
}
```

### 3. Endianness Issues

**Problem:** Multi-byte fields have wrong byte order

```rust
// Rust: little-endian by default
pub struct PacketHeader {
    pub magic: u8 = 0xCC, // Single byte, no endianness issue
}
```

```csharp
// C# on x86/x86_64: little-endian by default
public byte magic = 0xCC; // Single byte, no endianness issue ✓
```

**Note:** The magic byte is a single byte, so endianness doesn't affect it. However, multi-byte fields (`uint`, `int16`, etc.) must match endianness between Rust and C#. Both use little-endian on standard platforms, so this typically works automatically.

## Debugging Validation Failures

### Log Invalid Packets

```csharp
if (!spriteMsg.Validate())
{
    Debug.LogWarning($"Invalid SpriteMessage: " +
                   $"magic=0x{spriteMsg.magic:X2} (expected 0xCC), " +
                   $"type={spriteMsg.packet_type}");
    return;
}
```

### Dump Packet Bytes

```rust
// Rust: Log packet bytes for debugging
pub fn log_packet_bytes(bytes: &[u8]) {
    log::debug!("Packet ({} bytes): {:02X?}", bytes.len(), bytes);
}
```

```csharp
// C#: Log packet bytes for debugging
private void LogPacketBytes(byte[] bytes, int length)
{
    StringBuilder sb = new StringBuilder();
    for (int i = 0; i < length && i < 64; i++)
    {
        sb.Append($"{bytes[i]:X2} ");
    }
    Debug.Log($"Packet bytes: {sb}");
}
```

## Packet Validation Checklist

When adding a new packet type, ensure:

- [ ] Magic byte field exists and is `0xCC`
- [ ] Struct layout matches Rust `repr(C)` exactly
- [ ] `Validate()` method checks magic byte
- [ ] Packet creation sets magic byte correctly
- [ ] Unit tests for validation failures
- [ ] Integration tests with actual network data

## Examples

### Valid SpriteMessage

```
Bytes:  03 CC 01 00 00 00 00 01 9D 34 20 D1 AC 77 F0 9D 92 65 C0 C6 F0 4A 00 00 01 00 00 00
        │  │  │  │  │  └──┬───┘ └──────────────────┬─────────────────┘ └─┬─┘ └─┬─┘
        │  │  │  │  │     │                       UUID                   │     │
        │  │  │  │  │     └───── padding2                                │     │
        │  │  │  │  └─────────── sprite_type                             │     │
        │  │  │  └────────────── padding1                                │     │
        │  │  └───────────────── operation (Create)                      │     │
        │  └──────────────────── magic (0xCC ✓)                          │     │
        └─────────────────────── packet_type (SpriteMessage)             └─ x  └─ y
```

**Validation:**
```csharp
packet.magic == 0xCC ✓
packet.packet_type == 0x03 ✓
packet.operation == 0x01 ✓
```

### Invalid SpriteMessage (Corrupted Magic)

```
Bytes: 03 00 01 00 00 00 00 01 9D 34 20 D1 AC 77 F0 9D 92 65 C0 C6 F0 4A 00 00 01 00 00 00
       └┬┘ └┬┘ └┬┘ └──┬─┬─┬─┘ └────────────────┬────────────────┘ └─┬─┘ └─┬─┘
        │   │   │     │     │                   UUID                │     │
        │   │   │     │     └─ sprite_type                           │     │
        │   │   │     └────── padding2 (3 bytes)                    │     │
        │   │   └──────────── padding1                             │     │
        │   └──────────────── operation (Create)                   │     │
        │   └────────────────── magic (0x00 ✗ WRONG!)              │     │
        └───────────────────── packet_type (SpriteMessage)        └─ x  └─ y
```

**Validation:**
```csharp
packet.magic == 0x00 ✗ (should be 0xCC)
// Packet rejected
```

## Performance Impact

Validation adds minimal overhead:

```rust
pub fn validate(&self) -> bool {
    self.magic == 0xCC  // ← Single byte comparison: ~0.5ns
}
```

**Benefits far outweigh costs:**
- Prevents memory corruption (critical)
- Catches bugs early in development
- No impact on gameplay performance
- Debugging time saved > CPU cycles spent

## Best Practices

1. **Always validate packets before processing**
   ```csharp
   if (!packet.Validate()) { return; }
   ```

2. **Log validation failures with details**
   ```csharp
   Debug.LogWarning($"Invalid packet: magic=0x{packet.magic:X2}");
   ```

3. **Match Rust struct layout exactly**
   - Use `StructLayout(LayoutKind.Sequential, Pack = 1)`
   - Include padding fields explicitly
   - Match field order byte-for-byte

4. **Test with both valid and invalid data**
   ```rust
   #[test]
   fn test_magic_validation() {
       let mut packet = SpriteMessage::new(...);
       assert!(packet.validate());
       
       packet.magic = 0x00; // Corrupt magic
       assert!(!packet.validate());
   }
   ```

5. **Document packet sizes in code**
   ```csharp
   // Layout: packet_type(1) + magic(1) + ... = 30 bytes
   // Matches Rust repr(C) exactly
   ```

## Related Documentation

- [Architecture Overview](001_architecture.md) - Packet structure and design decisions
- [Packet Builder API](005_packet_builder_api.md) - Creating packets with validation
- [Migration Guide](006_migration_guide.md) - Updating structs for validation
- [Rust types.rs](../crates/unity-network/src/types.rs) - Packet definitions
- [Unity NativeNetworkClient.cs](../unity/NativeNetworkClient.cs) - FFI client implementation
