# Scripts

This directory contains utility scripts for the unity-ffi project. These scripts automate common development tasks and workflows.

## Available Scripts

### `build.sh`

Builds the Unity FFI library and server for macOS.

#### Usage

```bash
# From project root
./scripts/build.sh [debug|release] [arm64|x86_64]

# Examples
./scripts/build.sh release arm64     # Release build for Apple Silicon
./scripts/build.sh debug x86_64      # Debug build for Rosetta
./scripts/build.sh release            # Release build (defaults to arm64)
```

#### What It Does

- Builds Rust project with Cargo
- Compiles unity-network library (`libunity_network.dylib`)
- Compiles server binary (`unity-ffi-server`)
- Places output in `build_bin/` directory
- Supports both ARM64 (native) and x86_64 (Rosetta) architectures

#### When to Run

- Initial project setup
- After making code changes
- Before running Unity integration tests
- When switching between debug/release builds

#### Arguments

- `debug|release` (optional): Build type (default: release)
- `arm64|x86_64` (optional): Target architecture (default: arm64)

#### Output

- `build_bin/libunity_network.dylib` - Unity native library
- `build_bin/unity-ffi-server` - Server binary

---

### `run.sh`

Runs the Unity FFI WebTransport server.

#### Usage

```bash
# From project root
./scripts/run.sh [--rebuild]
```

#### What It Does

- Checks for existing server on port 4433 and stops if found
- Builds server if binary doesn't exist
- Starts the WebTransport server
- Optionally rebuilds before running

#### When to Run

- Testing server functionality
- Development work with client integration
- Debugging server issues

#### Options

- `--rebuild`: Force rebuild before running

#### Example Output

```
=====================================
Starting Unity FFI WebTransport server...
=====================================

Server listening on 0.0.0.0:4433
```

---

### `setup.sh`

Sets up a Unity project with the FFI library and scripts.

#### Usage

```bash
# From project root
./scripts/setup.sh <target-unity-project-path> [arm64|x86_64]

# Examples
./scripts/setup.sh /path/to/unity-project
./scripts/setup.sh /path/to/unity-project arm64
./scripts/setup.sh /path/to/unity-project x86_64
```

#### What It Does

- Builds the FFI library (if build script exists)
- Stops any running server
- Copies native library to Unity project's Plugins folder
- Copies C# scripts to Unity project
- Starts the server

#### When to Run

- Initial Unity project setup
- After updating native library
- When setting up new Unity projects
- After changing architectures

#### Arguments

- `target-unity-project-path` (required): Path to Unity project directory
- `arm64|x86_64` (optional): Architecture (default: arm64)

#### Prerequisites

- Unity project directory must exist
- Project should have `Assets/Plugins/macOS` folder (will be created if missing)

---

### `teardown.sh`

Stops the Unity FFI server and cleans up.

#### Usage

```bash
# From project root
./scripts/teardown.sh
```

#### What It Does

- Finds and stops server process on port 4433
- Cleans up PID file
- Verifies server is stopped

#### When to Run

- After development session
- Before rebuilding
- When server needs to be restarted
- Cleanup before system shutdown

#### Example Output

```
=====================================
Unity FFI Teardown Script
=====================================

Stopping server on port 4433...

Step 1: Finding server process...
✓ Server stopped successfully
✓ Port 4433 is now free
```

---

### `setup_profiler.sh`

Sets up profiling tools for the Unity FFI project.

#### Usage

```bash
# From project root
./scripts/setup_profiler.sh
```

#### What It Does

- Builds profiler tools
- Configures profiling environment
- Sets up profiling data collection

#### When to Run

- Initial profiling setup
- When analyzing performance issues
- Before profiling sessions

---

### `build_profiler.sh`

Builds the profiler tools and instrumentation.

#### Usage

```bash
# From project root
./scripts/build_profiler.sh [debug|release]
```

#### What It Does

- Compiles profiler utilities
- Builds instrumentation tools
- Creates profiling data collectors

#### When to Run

- After profiler code changes
- Initial profiler setup
- Before profiling sessions

---

### `quickstart_profiler.sh`

Quick setup and run for profiling with default settings.

#### Usage

```bash
# From project root
./scripts/quickstart_profiler.sh
```

#### What It Does

- Builds profiler with default settings
- Starts profiler collection
- Provides quick profiling results

#### When to Run

- Quick performance checks
- Ad-hoc profiling sessions
- Fast performance verification

---

### `rebuild_for_rosetta.sh`

Rebuilds the Unity native library for x86_64 (Rosetta) architecture and copies to Unity project.

#### Usage

```bash
# From project root
./scripts/rebuild_for_rosetta.sh [debug|release]
```

#### What It Does

- Ensures x86_64 target is installed
- Builds library for x86_64 architecture
- Copies to Unity project's Plugins folder
- Verifies correct architecture

#### When to Run

- When Unity runs under Rosetta
- After architecture switching
- When Rosetta-compatible builds are needed

#### Arguments

- `debug|release` (optional): Build type (default: release)

#### Prerequisites

- Unity project at `examples/helloworld-ffi/Assets/Plugins/macOS`
- x86_64 Rust target (`rustup target add x86_64-apple-darwin`)

#### Example Output

```
=====================================
Rebuild Unity Native Lib for Rosetta
=====================================

Building libunity_network.dylib for x86_64 (Rosetta)...
Copying to Unity project...
✓ Plugins directory ready: ...
✓ Copied to ...

Verification:
  Architecture: x86_64
✓ Library is correctly built for x86_64 (Rosetta)
```

---

### `generate_bindings.sh`

Generates Unity C# bindings from Rust FFI types using `#[derive(GameComponent)]` macro system.

#### Usage

```bash
# From project root
./scripts/generate_bindings.sh
```

#### What It Does

- Runs `cargo run --package unity-network --example generate_unity_cs`
- Saves output to `unity/Generated/GameFFI.cs`
- Validates file creation and provides feedback
- Counts generated structs and enums

#### When to Run

- After adding new `#[derive(GameComponent)]` structs
- After modifying existing struct fields
- After changing field types
- After adding or removing fields
- When integrating with Unity for the first time

#### Output Example

```
=== Generating Unity C# Bindings ===

Working directory: /path/to/unity-ffi

Running cargo run --package unity-network --example generate_unity_cs...
✓ Successfully generated unity/Generated/GameFFI.cs
  File size: 4521 bytes

✓ Generated 4 struct(s)
✓ Generated 4 enum(s)

Next steps:
  1. Unity will automatically detect the file change
  2. If not, refresh Unity project: Assets > Refresh
  3. Verify generated bindings match Rust types

⚠️  Remember: DO NOT EDIT GameFFI.cs manually!
   Regenerate with: ./scripts/generate_bindings.sh
```

#### Generated Content

The script generates `unity/Generated/GameFFI.cs` which contains:

- `PacketHeader` - Common packet header (2 bytes)
- `PlayerPos` - Player position update (40 bytes)
- `GameState` - Game state snapshot (20 bytes)
- `SpriteMessage` - Sprite operation message (30 bytes)

Plus enums:
- `PacketType` - Packet type discriminator
- `SpriteOp` - Sprite operation types
- `SpriteType` - Sprite type enumeration
- `FfiError` - FFI error codes

#### Architecture Notes

**Important**: The generated C# bindings provide:
- 0 lines of manual C# code
- Single source of truth (Rust)
- Zero-copy FFI with automatic binding generation
- Type safety with UUID-based protocol versioning

**Unity is VIEW-ONLY**:
- ✅ Use `PacketBuilder.CreateXxx()` to create packets
- ✅ Use `FromBytes()` to parse received packets
- ❌ Never manually create GameFFI structs in Unity
- ❌ Never manually set request_uuid fields

---

## Common Workflows

### Initial Setup

```bash
# Build everything
./scripts/build.sh release arm64

# Generate C# bindings
./scripts/generate_bindings.sh

# Setup Unity project
./scripts/setup.sh /path/to/unity-project
```

### Development Cycle

```bash
# Make code changes
vim unity-network/src/lib.rs

# Rebuild
./scripts/build.sh debug arm64

# Run server
./scripts/run.sh

# (Optional) Start Unity and test
```

### Unity Integration

```bash
# After FFI type changes
./scripts/generate_bindings.sh

# Rebuild native lib
./scripts/build.sh release arm64

# Copy to Unity project (if using custom path)
cp build_bin/libunity_network.dylib /path/to/unity/Assets/Plugins/macOS/

# Refresh Unity
```

### Rosetta Support

```bash
# Build for Rosetta
./scripts/build.sh release x86_64

# Or use helper script
./scripts/rebuild_for_rosetta.sh release

# Unity will load the x86_64 library when running under Rosetta
```

### Profiling

```bash
# Setup profiler
./scripts/setup_profiler.sh

# Build profiler
./scripts/build_profiler.sh release

# Quick profiling
./scripts/quickstart_profiler.sh
```

### Cleanup

```bash
# Stop server
./scripts/teardown.sh

# (Optional) Clean build artifacts
rm -rf build_bin/
```

---

## Troubleshooting

### Script Not Found

```bash
# Make sure you're in project root
cd unity-ffi

# Ensure script is executable
chmod +x scripts/build.sh

# Run with full path
./scripts/build.sh release arm64
```

### Build Failures

```bash
# Check Rust installation
rustc --version
cargo --version

# Install missing targets
rustup target add x86_64-apple-darwin

# Clean and rebuild
rm -rf target/
./scripts/build.sh release arm64
```

### Port Already in Use

```bash
# Find process on port 4433
lsof -ti:4433

# Kill it
kill -9 $(lsof -ti:4433)

# Or use teardown script
./scripts/teardown.sh
```

### Unity Can't Find Library

```bash
# Verify library exists
ls -la build_bin/libunity_network.dylib

# Check Unity project structure
ls -la /path/to/unity/Assets/Plugins/macOS/

# Verify architecture
file build_bin/libunity_network.dylib

# Copy if needed
cp build_bin/libunity_network.dylib /path/to/unity/Assets/Plugins/macOS/
```

### Generated C# Issues

```bash
# Regenerate bindings
./scripts/generate_bindings.sh

# Verify output
cat unity/Generated/GameFFI.cs

# Check memory layout
cargo run --package unity-network --example extract_layout
```

---

## Architecture Support

### ARM64 (Apple Silicon)

Native build for Apple Silicon Macs:

```bash
./scripts/build.sh release arm64
```

Default when architecture not specified.

### x86_64 (Rosetta)

Build for Intel architecture (Rosetta):

```bash
./scripts/build.sh release x86_64
```

Used when Unity runs under Rosetta.

---

## Quick Reference

| Script | Purpose | When to Run |
|--------|---------|-------------|
| `build.sh` | Build FFI library and server | After code changes |
| `run.sh` | Start server | Development/testing |
| `setup.sh` | Setup Unity project | Initial setup |
| `teardown.sh` | Stop server | Cleanup |
| `setup_profiler.sh` | Setup profiler | Profiling setup |
| `build_profiler.sh` | Build profiler tools | Before profiling |
| `quickstart_profiler.sh` | Quick profiling | Ad-hoc profiling |
| `rebuild_for_rosetta.sh` | Rebuild for Rosetta | Rosetta compatibility |
| `generate_bindings.sh` | Generate C# bindings | After FFI changes |

---

## Best Practices

1. **Always build after code changes**
   ```bash
   ./scripts/build.sh release arm64
   ```

2. **Regenerate bindings after FFI changes**
   ```bash
   ./scripts/generate_bindings.sh
   ```

3. **Use teardown to clean up**
   ```bash
   ./scripts/teardown.sh
   ```

4. **Choose correct architecture**
   - ARM64 for native Apple Silicon
   - x86_64 for Unity under Rosetta

5. **Never edit generated files**
   - `unity/Generated/GameFFI.cs` is auto-generated
   - Always regenerate from Rust

---

## Development Workflow

1. **Edit code** in `unity-network/` or `server/`
2. **Build** with `./scripts/build.sh`
3. **Run** with `./scripts/run.sh`
4. **Test** in Unity or with test clients
5. **Iterate** as needed

When modifying FFI types:
1. Edit Rust structs in `unity-network/src/types.rs`
2. Generate bindings: `./scripts/generate_bindings.sh`
3. Rebuild: `./scripts/build.sh`
4. Test in Unity

---

## Related Documentation

- [README.md](../README.md) - Main project documentation
- [GENERATING_BINDINGS.md](../GENERATING_BINDINGS.md) - Guide for generating bindings
- [unity-network/examples/](../unity-network/examples/) - Example code and tests

---

## Contributing

When adding new scripts:

1. Use `#!/bin/bash` for consistency
2. Add descriptive comments
3. Include error handling (`set -e`)
4. Use colored output for clarity
5. Provide usage examples
6. Document in this README
7. Make executable: `chmod +x scripts/your_script.sh`

---

## Support

For issues with scripts:

1. Check this README first
2. Review script's comments
3. Run with verbose output if needed
4. Check main project README for context
5. Review related documentation

---

## License

Same as main project. See [LICENSE](../LICENSE) for details.