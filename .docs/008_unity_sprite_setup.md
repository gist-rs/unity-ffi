# Unity Sprite Setup Guide

This guide walks you through setting up Unity to test the server-controlled sprite lifecycle system.

## Overview

The server now spawns, moves, and destroys sprites automatically using **zero-copy struct-based protocol** (not JSON). This guide shows you how to:

1. Set up a blank Unity scene
2. Add the NetworkPlayer component
3. Run the server
4. Verify sprite behavior visually and in logs

**Note:** Sprite messages use the same zero-copy struct approach as PlayerPos/GameState packets - fast, efficient, no JSON parsing!

## Prerequisites

- Unity Editor (2022.3 or later recommended)
- Rust toolchain (for running the server)
- Built Unity FFI library (`unity_network.dylib` on macOS)

## Step 1: Build the FFI Library

From the project root:

```bash
cd /Users/katopz/git/mu-ha-server/poc/unity-ffi
./scripts/build.sh debug
```

This creates `build_bin/unity_network.dylib` (on macOS).

## Step 2: Set Up Unity Project

### 2.1 Create New Project

1. Open Unity Hub
2. Create a new 2D project
3. Name it `SpriteTest`
4. Choose a location

### 2.2 Import Scripts

1. Copy `unity/` folder contents to your Unity project:
   - Copy `NetworkPlayer.cs` to `Assets/Scripts/`
   - Copy `NativeNetworkClient.cs` to `Assets/Scripts/`

### 2.3 Copy Native Library

1. Locate the built library: `build_bin/unity_network.dylib` (macOS)
2. Copy to Unity project's root folder (next to `Assets/`)
3. For standalone builds, you'll need to place it in the appropriate folder later

### 2.4 Create Blank Scene

1. In Unity, go to `File > New Scene`
2. Choose "Basic 2D" (or just start with default)
3. Save as `Assets/Scenes/SpriteTestScene`

**Scene Setup:**
- Camera: Position (0, 0, -10)
- Background color: Black (for better sprite visibility)
- Orthographic size: 70 (to see the full 128x128 sprite map)

### 2.5 Create NetworkPlayer GameObject

1. Right-click in Hierarchy
2. Select `Create Empty`
3. Name it `NetworkPlayer`
4. Add `NetworkPlayer.cs` component

**NetworkPlayer Component Settings:**

```
Server URL: https://127.0.0.1:6000
Certificate Hash: (leave empty - uses self-signed dev cert)
Player ID: 1
Update Interval: 0.05
Log Packets: ✓
Show Debug Info: ✓
```

### 2.6 Configure Camera

1. Select Main Camera
2. Set properties:
   - Position: (64, 64, -10) - Center of 128x128 map
   - Background: Black
   - Size: 80 (to fit full 128x128 area)
   - Near: 0.1
   - Far: 1000

## Step 3: Run the Server

Open a terminal and run:

```bash
cd /Users/katopz/git/mu-ha-server/poc/unity-ffi
cargo run --bin unity-ffi-server
```

You should see:

```
INFO Starting Unity FFI WebTransport server...
INFO Using self-signed certificate for development
INFO Starting circle motion broadcast (radius=5.0, speed=2.0 rad/s)
INFO Starting sprite management (spawn every 10s, update every 100ms, cleanup every 1s)
INFO Server listening on wtransport://127.0.0.1:6000
INFO Waiting for Unity connections...
```

## Step 4: Run Unity Test

1. In Unity Editor, open `SpriteTestScene`
2. Click Play button

### Expected Behavior

**Initial Connection (0-5 seconds):**
- Console: `"Connecting to server: https://127.0.0.1:6000"`
- Console: `"Connection successful!"`
- Console: `"Unity-Network FFI initialized successfully"`
- Console: `"[Sent] Hello message at tick X"`
- Network packets received as 30-byte structs (not JSON)

**First Sprite Spawn (after ~10 seconds):**
- Console: `"<color=green>[CREATE] serrif_{uuid} at (X, Y)</color>"`
- **Visual**: Green square appears in Scene view
- On-screen debug shows: `"Created: 1"`
- **Note**: Server sends 30-byte SpriteMessage struct (zero-copy, no JSON)

**Sprite Updates (every 100ms):**
- Console: `"<color=yellow>[UPDATE] serrif_{uuid} moved to (X, Y)</color>"`
- **Visual**: Sprite moves randomly 1 pixel at a time
- On-screen debug shows: `"Updated: N"` (increases continuously)
- Updates may log less frequently (Unity reduces log spam)
- **Network**: 30-byte SpriteMessage structs sent every 100ms (zero-copy, fast!)

**Multiple Sprites:**
- Every 10 seconds, new sprites appear
- Each sprite has a unique name: `serrif_{uuid}`
- On-screen debug: `"Active: 2"`, `"Active: 3"`, etc.
- Maximum of ~6 active sprites (10s lifetime × 10s spawn interval)

**Sprite Deletion (after 10s lifetime):**
- Console: `"<color=red>[DELETE] serrif_{uuid}</color>"`
- **Visual**: Sprite disappears from Scene view
- On-screen debug shows: `"Deleted: 1"`, `"Active: 5"`, etc.

**After 60 seconds:**
- Should see ~6 sprites created and deleted
- ~600 position updates
- Active sprites returns to 0-2 (depending on timing)

### On-Screen Debug Panel

You'll see a debug panel in the top-left:

```
Network Player Debug
--------------------
Status: Connected
Player ID: 1
Position: (X.XX, Y.YY)
Server Tick: X
Players: X

--- Sprite Stats ---
Created: N
Updated: N
Deleted: N
Active: N
```

## Step 5: Verification Checklist

### Server Logs

Watch the server terminal for:

- ✅ `"New connection from client: {uuid}"`
- ✅ `"[Sprite Create] serrif_{:?} at ({x}, {y})"` every 10 seconds
- ✅ `"[Sprite Update] {id} moved to ({x}, {y})"` (debug level)
- ✅ `"[Sprite Delete] serrif_{:?} at ({x}, {y})"` every 10 seconds
- ✅ `"[Sprite Broadcast] Sending CREATE to X clients"`

### Unity Console

Watch Unity Console for:

- ✅ Connection messages
- ✅ Green `[CREATE]` messages
- ✅ Yellow `[UPDATE]` messages
- ✅ Red `[DELETE]` messages
- ✅ No errors or warnings

### Unity Scene View

Watch the Scene view for:

- ✅ Green squares appear every 10 seconds
- ✅ Squares move randomly within bounds
- ✅ Squares disappear after 10 seconds
- ✅ Multiple sprites visible simultaneously
- ✅ Movement stays within 128x128 area

### Statistics Comparison

Compare server test vs Unity test:

| Metric | Server Test | Unity Test |
|--------|-------------|------------|
| Created | ~6 | ~6 |
| Updated | ~600 | ~600 |
| Deleted | ~6 | ~6 |
| Active (at 60s) | 0 | 0-2 |

## Troubleshooting

### Issue: "Failed to connect to server"

**Check:**
1. Server is running (`cargo run --bin unity-ffi-server`)
2. Server URL is correct: `https://127.0.0.1:6000`
3. Firewall allows port 6000
4. Server shows `"Waiting for Unity connections..."`

### Issue: "Failed to parse sprite message"

**Check:**
1. Server is sending sprite messages (look for `"[Sprite Broadcast]"` logs)
2. NetworkPlayer.cs has the latest struct-based parsing code
3. Messages are 30 bytes (SpriteMessage struct size)
4. Packet type is `PacketType.SpriteMessage` (type=3)

### Issue: Sprites not visible

**Check:**
1. Camera position is centered at (64, 64, -10)
2. Camera orthographic size is ~80
3. Background color is not white (green sprites won't show)
4. Sprites are being created (check Console for `[CREATE]` messages)

### Issue: No sprite messages

**Check:**
1. Server is running with sprite management enabled
2. Wait 10 seconds for first spawn
3. Check server logs for `"[Sprite Create]"` messages
4. Check NetworkPlayer is connected (`Status: Connected`)
5. Verify PacketType.SpriteMessage (3) is being handled

### Issue: Sprites move out of bounds

**Expected behavior:** This should NOT happen!

If you see sprites outside 0-127 range:
- Check server logs for boundary violations
- Verify `SpriteManager::random_walk()` clamping logic
- Report this as a bug

### Issue: Memory leak (sprites not deleted)

**Expected behavior:** All sprites should be deleted after 10s

If sprites accumulate:
- Check Console for `[DELETE]` messages
- Verify server cleanup task is running
- Check Unity Inspector for orphaned GameObjects
- Report this as a bug

## Technical Details

**SpriteMessage Struct (30 bytes):**
```
struct SpriteMessage {
    header: PacketHeader        // 2 bytes
    operation: u8             // 1 byte (Create=0, Update=1, Delete=2, Snapshot=3)
    padding1: u8             // 1 byte
    sprite_type: u8           // 1 byte (Serrif=0)
    padding2: [u8; 3]        // 3 bytes
    id: [u8; 16]             // 16 bytes (UUID)
    x: i16                    // 2 bytes
    y: i16                    // 2 bytes
    padding3: [u8; 2]        // 2 bytes
} // Total: 30 bytes
```

**Benefits of Struct-Based Approach:**
- ✅ Zero-copy: Direct memory mapping, no parsing overhead
- ✅ Fast: No JSON serialization/deserialization
- ✅ Small: 30 bytes vs 100+ bytes for JSON
- ✅ Type-safe: Compile-time checking in both Rust and C#
- ✅ Aligned: 4-byte aligned for efficient access

## Next Steps

Once basic sprite lifecycle is working:

1. **Add sprite visuals**: Replace green squares with actual sprite assets
2. **Adjust parameters**: Modify spawn rate, lifetime, map size in server
3. **Add interaction**: Click sprites to get info, drag to move, etc.
4. **Optimize**: Reduce logging, optimize rendering for many sprites

## Files Referenced

- `unity/NetworkPlayer.cs` - Main Unity component (struct-based sprite handling)
- `unity/NativeNetworkClient.cs` - FFI bridge (SpriteMessage struct definition)
- `unity-network/src/types.rs` - SpriteMessage struct definition (repr(C), 30 bytes)
- `unity-network/src/sprite_manager.rs` - Server sprite logic
- `server/src/main.rs` - Server with sprite tasks
- `PLAN.md` - Complete implementation plan

## Success Criteria

✅ Phase 4 is complete when:

1. Unity connects to server successfully
2. Sprites appear as green squares in Scene view
3. Sprites spawn every 10 seconds
4. Sprites move randomly within 128x128 bounds
5. Sprites disappear after 10 seconds lifetime
6. Unity Console logs match Rust test output
7. Statistics match: ~6 creates, ~6 deletes, ~600 updates in 60s
8. No memory leaks (Inspector shows stable GameObject count)
9. No errors or warnings in Unity Console or Server logs

Once these are verified, you can move to Phase 5: Full Integration Testing!