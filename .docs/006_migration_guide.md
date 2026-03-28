# PacketBuilder Migration Guide

## Overview

This guide helps you migrate from the old Unity packet construction API to the new PacketBuilder API. The migration moves all protocol logic (UUID generation, packet construction, serialization) from Unity C# to Rust FFI, establishing proper separation of concerns.

### Why Migrate?

**Old Approach Problems:**
- Unity manually constructing packets (business logic in view layer)
- Duplicate serialization logic in C# and Rust
- Risk of protocol desync between Unity and Rust
- Manual UUID v7 generation in C#
- Complex error-prone byte manipulation

**New Approach Benefits:**
- Rust handles all protocol logic (single source of truth)
- Auto-generated UUID v7 in Rust
- Type-safe with compile-time checking
- No duplicate code
- Easy to extend and maintain
- Better performance (< 1 μs per packet)

### What Changed?

| Aspect | Old API | New API |
|---------|----------|----------|
| UUID Generation | `UuidV7.NewUuid()` in C# | Auto-generated in Rust |
| Packet Construction | Manual `new PacketType(...)` | `PacketBuilder.Create...()` |
| Serialization | `ToBytes()` / `FromBytes()` | Handled by Rust |
| Header Management | `SetUuid()`, `GetUuid()` | Automatic in Rust |
| Error Handling | Manual checks | Built-in error handling |

## Migration Steps

### Step 1: Identify Old Code Usage

Search your Unity project for these patterns:

```csharp
// Pattern 1: UUID generation
UuidV7.NewUuid()

// Pattern 2: Packet construction
new PacketHeader(...)
new PlayerPos(...)
new GameState(...)
new SpriteMessage(...)

// Pattern 3: UUID setting
header.SetUuid(uuid)
packet.header.SetUuid(uuid)

// Pattern 4: Serialization
packet.ToBytes()
PacketHeader.FromBytes(bytes)
```

### Step 2: Replace UUID Generation

**Before:**
```csharp
// Generate UUID v7 in C#
Guid requestUuid = UuidV7.NewUuid();
var header = new PacketHeader(PacketType.PlayerPos, requestUuid);
header.SetUuid(requestUuid);
```

**After:**
```csharp
// UUID generated automatically by Rust
byte[] packet = PacketBuilder.CreatePlayerPos(playerId, x, y);
// UUID is in bytes 2-17 of packet
```

### Step 3: Replace Packet Construction

#### PlayerPos Packet

**Before:**
```csharp
Guid requestUuid = UuidV7.NewUuid();
var header = new PacketHeader(PacketType.PlayerPos, requestUuid);
header.SetUuid(requestUuid);

var packet = new PlayerPos {
    header = header,
    id = playerId,
    x = 100,
    y = 200
};

byte[] bytes = packet.ToBytes();
networkClient.Send(bytes);
```

**After:**
```csharp
byte[] packet = PacketBuilder.CreatePlayerPos(playerId, 100, 200);
networkClient.Send(packet);
```

#### GameState Packet

**Before:**
```csharp
Guid requestUuid = UuidV7.NewUuid();
var header = new PacketHeader(PacketType.GameState, requestUuid);
header.SetUuid(requestUuid);

var packet = new GameState {
    header = header,
    tick = 999999,
    player_count = 42
};

byte[] bytes = packet.ToBytes();
networkClient.Send(bytes);
```

**After:**
```csharp
byte[] packet = PacketBuilder.CreateGameState(999999, 42);
networkClient.Send(packet);
```

#### SpriteMessage Packet

**Before:**
```csharp
Guid requestUuid = UuidV7.NewUuid();
var header = new PacketHeader(PacketType.SpriteMessage, requestUuid);
header.SetUuid(requestUuid);

var packet = new SpriteMessage {
    header = header,
    operation = 1,
    sprite_type = 2,
    id = spriteId,
    x = 50,
    y = 75
};

byte[] bytes = packet.ToBytes();
networkClient.Send(bytes);
```

**After:**
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

#### Authenticate Packet

**Before:**
```csharp
Guid requestUuid = UuidV7.NewUuid();
var header = new PacketHeader(PacketType.Authenticate, requestUuid);
header.SetUuid(requestUuid);

var packet = new Authenticate {
    header = header,
    user_id = userId
};

byte[] bytes = packet.ToBytes();
networkClient.Send(bytes);
```

**After:**
```csharp
byte[] packet = PacketBuilder.CreateAuthenticate(userId);
networkClient.Send(packet);
```

#### KeepAlive Packet

**Before:**
```csharp
Guid requestUuid = UuidV7.NewUuid();
var header = new PacketHeader(PacketType.KeepAlive, requestUuid);
header.SetUuid(requestUuid);

var packet = new KeepAlive {
    header = header
};

byte[] bytes = packet.ToBytes();
networkClient.Send(bytes);
```

**After:**
```csharp
byte[] packet = PacketBuilder.CreateKeepAlive();
networkClient.Send(packet);
```

### Step 4: Remove Obsolete Code

After migrating to PacketBuilder, remove all usage of obsolete methods:

```csharp
// REMOVE THESE:
UuidV7.NewUuid()
new PacketHeader(...)
new PlayerPos(...)
new GameState(...)
new SpriteMessage(...)
new Authenticate(...)
new KeepAlive(...)
header.SetUuid(...)
header.GetUuid(...)
header.HasUuid(...)
header.ToBytes()
PacketHeader.FromBytes(...)
```

### Step 5: Verify Migration

1. **Build Project**: Ensure no compilation errors
2. **Run Tests**: Verify all existing tests pass
3. **Check Warnings**: Look for `[Obsolete]` warnings (should be zero)
4. **Test Network**: Verify packets are sent correctly
5. **Verify UUIDs**: Confirm UUIDs are propagated to server

## Common Migration Patterns

### Pattern 1: Player Movement

**Before:**
```csharp
void UpdatePlayerPosition(Guid playerId, float x, float y) {
    Guid requestUuid = UuidV7.NewUuid();
    
    var header = new PacketHeader(PacketType.PlayerPos, requestUuid);
    header.SetUuid(requestUuid);
    
    var packet = new PlayerPos {
        header = header,
        id = playerId,
        x = (int)x,
        y = (int)y
    };
    
    byte[] bytes = packet.ToBytes();
    networkClient.Send(bytes);
}
```

**After:**
```csharp
void UpdatePlayerPosition(Guid playerId, float x, float y) {
    byte[] packet = PacketBuilder.CreatePlayerPos(
        playerId,
        (int)x,
        (int)y
    );
    networkClient.Send(packet);
}
```

### Pattern 2: Batch Updates

**Before:**
```csharp
void UpdateMultiplePlayers(Dictionary<Guid, Vector2> positions) {
    foreach (var kvp in positions) {
        Guid requestUuid = UuidV7.NewUuid();
        
        var header = new PacketHeader(PacketType.PlayerPos, requestUuid);
        header.SetUuid(requestUuid);
        
        var packet = new PlayerPos {
            header = header,
            id = kvp.Key,
            x = (int)kvp.Value.x,
            y = (int)kvp.Value.y
        };
        
        byte[] bytes = packet.ToBytes();
        networkClient.Send(bytes);
    }
}
```

**After:**
```csharp
void UpdateMultiplePlayers(Dictionary<Guid, Vector2> positions) {
    foreach (var kvp in positions) {
        byte[] packet = PacketBuilder.CreatePlayerPos(
            kvp.Key,
            (int)kvp.Value.x,
            (int)kvp.Value.y
        );
        networkClient.Send(packet);
    }
}
```

### Pattern 3: Error Handling

**Before:**
```csharp
void SendPlayerPos(Guid playerId, int x, int y) {
    try {
        Guid requestUuid = UuidV7.NewUuid();
        
        var header = new PacketHeader(PacketType.PlayerPos, requestUuid);
        header.SetUuid(requestUuid);
        
        var packet = new PlayerPos {
            header = header,
            id = playerId,
            x = x,
            y = y
        };
        
        byte[] bytes = packet.ToBytes();
        
        if (bytes == null || bytes.Length == 0) {
            Debug.LogError("Failed to serialize packet");
            return;
        }
        
        networkClient.Send(bytes);
    } catch (Exception ex) {
        Debug.LogError($"Packet creation failed: {ex.Message}");
    }
}
```

**After:**
```csharp
void SendPlayerPos(Guid playerId, int x, int y) {
    try {
        byte[] packet = PacketBuilder.CreatePlayerPos(playerId, x, y);
        networkClient.Send(packet);
    } catch (InvalidOperationException ex) {
        Debug.LogError($"Packet creation failed: {ex.Message}");
    }
}
```

### Pattern 4: NetworkProfiler Integration

**Before:**
```csharp
void TrackedPlayerUpdate(Guid playerId, int x, int y) {
    Guid requestUuid = UuidV7.NewUuid();
    
    var header = new PacketHeader(PacketType.PlayerPos, requestUuid);
    header.SetUuid(requestUuid);
    
    profiler.StartRequest(RequestType.PlayerPosUpdate, requestUuid);
    
    var packet = new PlayerPos {
        header = header,
        id = playerId,
        x = x,
        y = y
    };
    
    byte[] bytes = packet.ToBytes();
    networkClient.Send(bytes);
    
    profiler.CompleteRequest(requestUuid, RequestStatus.Success);
}
```

**After:**
```csharp
void TrackedPlayerUpdate(Guid playerId, int x, int y) {
    byte[] packet = PacketBuilder.CreatePlayerPos(playerId, x, y);
    
    // Extract UUID from packet for profiling
    Guid requestUuid = ExtractUuidFromPacket(packet);
    
    profiler.StartRequest(RequestType.PlayerPosUpdate, requestUuid);
    networkClient.Send(packet);
    profiler.CompleteRequest(requestUuid, RequestStatus.Success);
}

Guid ExtractUuidFromPacket(byte[] packet) {
    if (packet.Length < 18) return Guid.Empty;
    
    byte[] uuidBytes = new byte[16];
    Array.Copy(packet, 2, uuidBytes, 0, 16);
    return new Guid(uuidBytes);
}
```

## Breaking Changes

### 1. UUID Generation Removed

**Impact:** `UuidV7.NewUuid()` no longer needed

**Migration:** Remove all calls to `UuidV7.NewUuid()`. PacketBuilder generates UUID automatically.

### 2. Manual Packet Construction Obsolete

**Impact:** `new PacketHeader()`, `new PlayerPos()`, etc. are obsolete

**Migration:** Replace with `PacketBuilder.Create...()` methods.

### 3. Serialization Methods Removed

**Impact:** `ToBytes()`, `FromBytes()` are obsolete

**Migration:** PacketBuilder returns complete byte array, no serialization needed.

### 4. UUID Helper Methods Obsolete

**Impact:** `SetUuid()`, `GetUuid()`, `HasUuid()` are obsolete

**Migration:** Extract UUID from packet bytes if needed for profiling.

## Checklist

Use this checklist to ensure complete migration:

### Code Changes
- [ ] Remove all `UuidV7.NewUuid()` calls
- [ ] Replace all `new PacketHeader()` constructions
- [ ] Replace all `new PlayerPos()` constructions
- [ ] Replace all `new GameState()` constructions
- [ ] Replace all `new SpriteMessage()` constructions
- [ ] Replace all `new Authenticate()` constructions
- [ ] Replace all `new KeepAlive()` constructions
- [ ] Remove all `SetUuid()` calls
- [ ] Remove all `GetUuid()` calls
- [ ] Remove all `HasUuid()` calls
- [ ] Remove all `ToBytes()` calls
- [ ] Remove all `FromBytes()` calls

### Testing
- [ ] Build project successfully (no compilation errors)
- [ ] No `[Obsolete]` warnings in Unity console
- [ ] Run all existing unit tests
- [ ] Run all integration tests
- [ ] Test player movement
- [ ] Test authentication
- [ ] Test keep-alive packets
- [ ] Test sprite messages
- [ ] Verify UUID propagation to server
- [ ] Check network profiler still works

### Verification
- [ ] Packet sizes match expectations (18-46 bytes)
- [ ] UUIDs are present in packet headers
- [ ] Network traffic unchanged (same packet structure)
- [ ] No performance regression
- [ ] All game functionality working

## Troubleshooting

### Issue: `[Obsolete]` Warnings Still Appear

**Symptom:** Unity console shows obsolete method warnings

**Solutions:**
1. Search project for obsolete method names
2. Replace with PacketBuilder API
3. Remove `using UnityNetwork;` if old namespace used
4. Rebuild project

### Issue: Packets Not Sent

**Symptom:** Network packets not reaching server

**Solutions:**
1. Verify `networkClient.Send()` still called after migration
2. Check packet is not null or empty
3. Verify network client still connected
4. Check server logs for connection errors

### Issue: UUIDs Not Propagating

**Symptom:** Server doesn't receive UUID in packet headers

**Solutions:**
1. Verify PacketBuilder is being used (not manual construction)
2. Check that packet is 18-46 bytes (not old 2-byte header)
3. Inspect bytes 2-17 of packet (should contain UUID)
4. Verify server reads UUID from correct offset

### Issue: Performance Regression

**Symptom:** Packet creation slower after migration

**Solutions:**
1. First call overhead is expected (~30-40 ms for DLL loading)
2. Add warmup calls before measuring
3. Subsequent calls should be < 1 μs
4. Profile to identify bottlenecks

### Issue: Compilation Errors

**Symptom:** Build fails with missing types/methods

**Solutions:**
1. Ensure `using Unity.Network;` is present
2. Verify `PacketBuilder.cs` is in project
3. Check that Rust dylib is built and copied to `Assets/Plugins/`
4. Verify architecture matches (ARM64 vs x86_64)

### Issue: Wrong Packet Sizes

**Symptom:** Packet sizes don't match expectations

**Solutions:**
1. Old PacketHeader: 2 bytes (without UUID)
2. New PacketHeader: 18 bytes (with UUID v7)
3. Expected packet sizes:
   - KeepAlive: 18 bytes (old: 2 bytes)
   - Authenticate: 34 bytes (old: 18 bytes)
   - PlayerPos: 44 bytes (old: 28 bytes)
   - GameState: 36 bytes (old: 32 bytes)
   - SpriteMessage: 46 bytes (old: 40 bytes)

## Rollback Plan

If issues arise after migration:

### Immediate Rollback

1. **Revert Code Changes**
   ```bash
   git revert <commit-hash>
   ```

2. **Restore Old API**
   - Remove `[Obsolete]` attributes
   - Restore `UuidV7` class
   - Restore serialization methods

3. **Verify Functionality**
   - Run all tests
   - Test game functionality
   - Verify network communication

### Plan for Retry Migration

1. **Identify Root Cause**
   - Check what failed
   - Review error logs
   - Test in isolation

2. **Fix Issues**
   - Address root cause
   - Add more tests
   - Update documentation

3. **Retry Migration**
   - Start with small subset
   - Gradually expand
   - Monitor closely

## Advanced Topics

### Custom Packet Types

To add custom packet types:

1. **Define Packet in Rust**
   ```rust
   // crates/mmorpg-protocol/src/modules/custom.rs
   #[repr(C)]
   pub struct CustomPacket {
       pub header: PacketHeader,
       pub field1: u32,
       pub field2: u32,
   }
   ```

2. **Add PacketBuilder Function**
   ```rust
   // unity-network/src/packet_builder.rs
   pub unsafe extern "C" fn packet_builder_create_custom(
       field1: u32,
       field2: u32,
       out_ptr: *mut u8,
       capacity: usize,
   ) -> i32 {
       // Implementation
   }
   ```

3. **Add C# Wrapper Method**
   ```csharp
   // unity/PacketBuilder.cs
   public static byte[] CreateCustom(uint field1, uint field2) {
       // Implementation
   }
   ```

### UUID Extraction for Profiling

If you need to extract UUID from packet for profiling:

```csharp
Guid ExtractUuidFromPacket(byte[] packet) {
    if (packet == null || packet.Length < 18) {
        return Guid.Empty;
    }
    
    byte[] uuidBytes = new byte[16];
    Array.Copy(packet, 2, uuidBytes, 0, 16);
    return new Guid(uuidBytes);
}

// Usage
byte[] packet = PacketBuilder.CreatePlayerPos(playerId, x, y);
Guid requestUuid = ExtractUuidFromPacket(packet);
profiler.TrackRequest(requestUuid, RequestType.PlayerPosUpdate);
networkClient.Send(packet);
```

## Support

For help with migration:

1. **Documentation**
   - API Reference: `PACKET_BUILDER_API.md`
   - Architecture: `ARCHITECTURE.md`
   - Unity Setup: `README.md`

2. **Handovers**
   - Handover 007: FFI Architectural Cleanup
   - Handover 008: Remaining Work Summary
   - Handover 010: .NET Testing Completion

3. **Tests**
   - Rust tests: `unity-network/src/packet_builder.rs`
   - .NET tests: `tests/PacketBuilderTests/Program.cs`

4. **Code Examples**
   - See migration patterns above
   - Check inline code comments
   - Review handover documents

## Conclusion

Migrating to PacketBuilder API simplifies your Unity code by moving all protocol logic to Rust. This establishes proper separation of concerns, reduces code duplication, and makes your application more maintainable.

The migration is straightforward:
1. Replace manual packet construction with PacketBuilder methods
2. Remove UUID generation code (handled by Rust)
3. Remove serialization code (handled by Rust)
4. Test thoroughly

If you encounter issues, use this guide's troubleshooting section and rollback plan. The migration is designed to be reversible if needed.

**Key Benefits After Migration:**
- ✅ Simpler Unity code
- ✅ No protocol logic in view layer
- ✅ Single source of truth for packet structures
- ✅ Auto-generated UUID v7
- ✅ Better performance
- ✅ Easier maintenance and extension