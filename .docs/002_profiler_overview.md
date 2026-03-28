# FPS/RAM Profiler Build and Setup Scripts

This directory contains shell scripts for building and deploying the FPS/RAM profiler to Unity projects.

## Overview

The FPS/RAM profiler consists of a Rust backend (`mmorpg-profiler` crate) and Unity C# integration. These scripts automate the build and deployment process.

### Scripts

1. **build_profiler.sh** - Build the mmorpg-profiler Rust crate
2. **setup_profiler.sh** - Copy profiler files to Unity project
3. **quickstart_profiler.sh** - One-command build and setup

## Prerequisites

- **macOS** (scripts are macOS-specific)
- **Rust toolchain** (for building mmorpg-profiler)
- **Unity 2021.3+** (for C# code)
- **Xcode Command Line Tools** (for building native libraries)

### Installing Prerequisites

```bash
# Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install Xcode Command Line Tools (if not already installed)
xcode-select --install

# Verify Rust installation
rustc --version
cargo --version
```

## Quick Start

The fastest way to get started is using `quickstart_profiler.sh`:

```bash
# Build and setup for default project (examples/helloworld-ffi)
./scripts/quickstart_profiler.sh

# Build and setup for custom Unity project
./scripts/quickstart_profiler.sh release arm64 /path/to/your/unity/project

# Build debug version
./scripts/quickstart_profiler.sh debug

# Build for Rosetta (x86_64)
./scripts/quickstart_profiler.sh release x86_64
```

## Detailed Usage

### 1. build_profiler.sh

Build the mmorpg-profiler Rust crate for Unity FFI.

```bash
./scripts/build_profiler.sh [build_type] [architecture]
```

#### Arguments

- **build_type** (optional): `debug` or `release` (default: `release`)
- **architecture** (optional): `arm64` or `x86_64` (default: `arm64`)

#### Examples

```bash
# Build release for ARM64 (native Apple Silicon)
./scripts/build_profiler.sh release arm64

# Build debug for ARM64
./scripts/build_profiler.sh debug arm64

# Build release for x86_64 (Rosetta/Intel)
./scripts/build_profiler.sh release x86_64

# Build debug for x86_64
./scripts/build_profiler.sh debug x86_64
```

#### Output

The script creates:
- `build_bin/libmmorpg_profiler.dylib` - Native library for Unity
- Exports FFI symbols to console for verification

#### Architecture Guide

**How to determine your architecture:**
```bash
uname -m
# Output: arm64 or x86_64
```

**When to use each:**
- **arm64** - Native Apple Silicon (M1/M2/M3 Macs)
- **x86_64** - Rosetta translation or Intel Macs

**Unity architecture:**
- Unity Editor runs natively on ARM64 → Use `arm64`
- Unity Editor under Rosetta → Use `x86_64`
- Check Unity Editor → About Unity → check architecture

### 2. setup_profiler.sh

Copy profiler files to a Unity project.

```bash
./scripts/setup_profiler.sh <target-unity-project> [architecture]
```

#### Arguments

- **target-unity-project** (required): Path to Unity project directory
- **architecture** (optional): `arm64` or `x86_64` (default: `arm64`)

#### Examples

```bash
# Setup for default example project
./scripts/setup_profiler.sh ./examples/helloworld-ffi

# Setup for custom Unity project
./scripts/setup_profiler.sh /Users/yourname/Projects/MyGame

# Setup for Rosetta
./scripts/setup_profiler.sh ./examples/helloworld-ffi x86_64
```

#### What It Does

1. **Builds profiler** (if `build_profiler.sh` exists)
2. **Creates directory structure** in Unity project:
   - `Assets/Plugins/macOS/` - Native library
   - `Assets/Scripts/Profiler/` - C# scripts
3. **Copies files**:
   - `libmmorpg_profiler.dylib` → `Assets/Plugins/macOS/`
   - `FpsRam*.cs` → `Assets/Scripts/Profiler/`
   - `README_FPS_RAM.md` → `Assets/Scripts/Profiler/`
4. **Sets permissions** on native library
5. **Displays setup instructions**

### 3. quickstart_profiler.sh

One-command build and setup for convenience.

```bash
./scripts/quickstart_profiler.sh [build_type] [architecture] [target-unity-project]
```

#### Arguments

- **build_type** (optional): `debug` or `release` (default: `release`)
- **architecture** (optional): `arm64` or `x86_64` (default: `arm64`)
- **target-unity-project** (optional): Path to Unity project (default: `./examples/helloworld-ffi`)

#### Examples

```bash
# Default: release/arm64 for examples/helloworld-ffi
./scripts/quickstart_profiler.sh

# Debug build
./scripts/quickstart_profiler.sh debug

# Rosetta build
./scripts/quickstart_profiler.sh release x86_64

# Custom Unity project
./scripts/quickstart_profiler.sh release arm64 /path/to/your/unity/project

# All custom options
./scripts/quickstart_profiler.sh debug x86_64 /path/to/your/unity/project
```

#### What It Does

1. Executes `build_profiler.sh` to build the crate
2. Executes `setup_profiler.sh` to copy files to Unity
3. Displays final instructions and next steps

## Project Structure

After running setup, your Unity project should look like:

```
YourUnityProject/
├── Assets/
│   ├── Plugins/
│   │   └── macOS/
│   │       └── libmmorpg_profiler.dylib    ← Native library
│   └── Scripts/
│       └── Profiler/
│           ├── FpsRamProfiler.cs            ← Core wrapper
│           ├── FpsRamProfilerBehaviour.cs   ← Unity integration
│           ├── FpsRamProfilerOverlay.cs     ← UI overlay
│           ├── FpsRamProfilerTests.cs     ← Unit tests
│           ├── FpsRamProfilerTypes.cs     ← FFI types
│           └── README_FPS_RAM.md         ← Documentation
```

## Integration Steps

After running setup, follow these steps in Unity:

### 1. Import Assets

1. Open Unity project
2. Unity should auto-detect new files
3. If not, go to **Assets → Refresh** or press **Cmd+R**

### 2. Configure Project Settings

Enable unsafe code for FFI:

1. **Edit → Project Settings → Player**
2. Scroll down to **Other Settings**
3. Set **"Allow 'unsafe' Code"** to **ON**
4. Apply changes

### 3. Verify Plugin

1. Select `Assets/Plugins/macOS/libmmorpg_profiler.dylib`
2. In Inspector, verify:
   - **Any Platform** - Checked
   - **CPU**: Any CPU
   - **OS**: macOS

### 4. Add Profiler to Scene

1. Create a new GameObject (e.g., "FPS RAM Profiler")
2. Add **FpsRamProfilerBehaviour** component
3. Configure settings in Inspector:
   - Enable Frame Recording
   - Enable Memory Tracking
   - Set Memory Update Interval (default: 0.5s)
   - Configure Hotkey (default: F5)

### 5. (Optional) Add UI Overlay

1. Create a UI Canvas in your scene
2. Add a panel for the profiler UI
3. Add **FpsRamProfilerOverlay** component to a GameObject
4. Configure UI references in Inspector:
   - FPS Text
   - Frame Time Text
   - Memory Text
   - Memory Bars (Reserved, Allocated, Mono)
   - Frame Timing Graph (RawImage)
   - Tab Buttons

### 6. Test

1. Press **Play** in Unity Editor
2. Press **F5** to toggle profiler visibility
3. Verify metrics are displayed

## Configuration

### Inspector Settings (FpsRamProfilerBehaviour)

**Profiler Settings:**
- **Enable Frame Recording** - Record frame time every Update
- **Enable Memory Tracking** - Submit Unity memory metrics
- **Memory Update Interval** - How often to update memory (0.1s to 5.0s)
- **Profiler Context** - Which context to track (Unity/Rust/Total)

**Debug Settings:**
- **Enable Debug Logging** - Log profiler operations to Console
- **Show Profiler Info** - Display initialization info

**Hotkeys:**
- **Toggle Key** - Key to toggle visibility (default: F5)
- **Enable Hotkey** - Enable hotkey support

### Inspector Settings (FpsRamProfilerOverlay)

**Display Settings:**
- **Show FPS Display** - Show FPS metrics
- **Show Memory Display** - Show memory metrics
- **Show Frame Timing Graph** - Show graph visualization

**Update Rate Throttling:**
- **FPS Text Update Rate** - Updates per second (1-30, default: 3)
- **Graph Update Rate** - Updates per second (10-60, default: 30)

**Memory Display:**
- **Max Memory (MB)** - Maximum memory for bar scaling (default: 2048)

**Frame Timing Graph:**
- **Good Threshold (ms)** - Green threshold (default: 16.67)
- **Caution Threshold (ms)** - Yellow threshold (default: 33.33)
- **Graph Buffer Size** - Number of samples (default: 512)

**Hotkeys:**
- **Toggle Key** - Key to toggle visibility (default: F5)
- **Enable Hotkey** - Enable hotkey support

## Troubleshooting

### Build Issues

**Issue: `cargo: command not found`**
```bash
# Solution: Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

**Issue: `error: linker 'cc' not found`**
```bash
# Solution: Install Xcode Command Line Tools
xcode-select --install
```

**Issue: Architecture mismatch**
```bash
# Check your system architecture
uname -m

# If Unity under Rosetta, rebuild with x86_64
./scripts/build_profiler.sh release x86_64
```

**Issue: Build fails with linker errors**
```bash
# Clean and rebuild
cd ../mmorpg
cargo clean
cargo build -p mmorpg-profiler --release
```

### Unity Integration Issues

**Issue: Plugin not detected**
1. Check file exists: `ls -la Assets/Plugins/macOS/libmmorpg_profiler.dylib`
2. Check permissions: `chmod +x Assets/Plugins/macOS/libmmorpg_profiler.dylib`
3. Restart Unity Editor
4. Go to **Assets → Refresh**

**Issue: "unsafe code" errors in Unity Console**
1. **Edit → Project Settings → Player**
2. Scroll down to **Other Settings**
3. Set **"Allow 'unsafe' Code"** to **ON**
4. Apply changes

**Issue: `DllNotFoundException`**
1. Verify plugin path: `Assets/Plugins/macOS/libmmorpg_profiler.dylib`
2. Check architecture matches Unity (arm64 vs x86_64)
3. Rebuild with correct architecture

**Issue: Profiler shows 0 FPS/Memory**
1. Verify `FpsRamProfilerBehaviour` component is active
2. Check "Enable Frame Recording" and "Enable Memory Tracking"
3. Verify profiler is not disposed
4. Check Unity Console for initialization errors

### Runtime Issues

**Issue: High performance overhead**
1. Reduce FPS text update rate to 1-2
2. Reduce graph update rate to 15-20
3. Increase memory update interval to 1.0s+
4. Disable overlay when not needed

**Issue: Memory leak**
1. Verify profiler is properly disposed (automatic in OnDestroy)
2. Check for circular references in Unity components
3. Use Unity Profiler to identify leak source
4. Ensure `profiler_shutdown()` is called on exit

**Issue: F5 hotkey not working**
1. Verify "Enable Hotkey" is checked in Inspector
2. Check if F5 key is used by other components
3. Manually call `FpsRamProfilerBehaviour.Instance.ToggleVisibility()`
4. Configure different hotkey in Inspector

## Advanced Usage

### Building for Multiple Architectures

Build both ARM64 and x86_64 versions:

```bash
# Build ARM64 (native)
./scripts/build_profiler.sh release arm64

# Build x86_64 (Rosetta)
./scripts/build_profiler.sh release x86_64

# Both libraries will be in build_bin/
# - libmmorpg_profiler.dylib (arm64)
# - libmmorpg_profiler.dylib (x86_64, will overwrite)
```

To keep both versions:
```bash
# Build ARM64
./scripts/build_profiler.sh release arm64
cp build_bin/libmmorpg_profiler.dylib build_bin/libmmorpg_profiler_arm64.dylib

# Build x86_64
./scripts/build_profiler.sh release x86_64
cp build_bin/libmmorpg_profiler.dylib build_bin/libmmorpg_profiler_x86_64.dylib
```

### Debug Build

For development, use debug builds for faster compilation:

```bash
# Build debug version
./scripts/build_profiler.sh debug arm64

# Setup with debug build
./scripts/setup_profiler.sh ./examples/helloworld-ffi arm64

# Note: Debug builds are slower and larger
```

### Clean Build

Force a clean rebuild:

```bash
# Remove build directory
rm -rf build_bin/

# Clean Rust build artifacts
cd ../mmorpg
cargo clean -p mmorpg-profiler

# Rebuild
cd ../unity-ffi
./scripts/quickstart_profiler.sh
```

### Custom Build Configuration

Modify `build_profiler.sh` for custom settings:

```bash
# Edit build_profiler.sh
# Change target triple for different OS
TARGET_TRIPLE="aarch64-apple-darwin"  # macOS ARM64

# Add custom Rust features
cargo build -p mmorpg-profiler --features "custom_feature"

# Add custom linker flags
RUSTFLAGS="-C link-arg=-Wl,-install_name,@rpath/libmmorpg_profiler.dylib"
```

## Scripts Reference

### build_profiler.sh

**Purpose:** Build mmorpg-profiler Rust crate for Unity FFI

**Requirements:**
- mmorpg workspace at `../mmorpg/`
- Rust toolchain installed

**Environment Variables:**
- `RUSTFLAGS` - Custom Rust compiler flags
- `CARGO_BUILD_TARGET` - Override build target

**Exit Codes:**
- `0` - Success
- `1` - Error (invalid arguments, build failed, etc.)

### setup_profiler.sh

**Purpose:** Copy profiler files to Unity project

**Requirements:**
- Built library in `build_bin/`
- Source scripts in `unity/Profiler/`
- Valid Unity project path

**Exit Codes:**
- `0` - Success
- `1` - Error (missing files, copy failed, etc.)

### quickstart_profiler.sh

**Purpose:** One-command build and setup

**Requirements:**
- All requirements from build_profiler.sh and setup_profiler.sh

**Exit Codes:**
- `0` - Success
- `1` - Error (build or setup failed)

## File Locations

### Source Files

- **Rust backend:** `../mmorpg/crates/mmorpg-profiler/`
- **C# scripts:** `unity/Profiler/`
- **Build output:** `build_bin/`

### Destination Files (Unity)

- **Native library:** `YourUnityProject/Assets/Plugins/macOS/libmmorpg_profiler.dylib`
- **C# scripts:** `YourUnityProject/Assets/Scripts/Profiler/FpsRam*.cs`

## Performance

### Build Times

- **Debug build:** ~10-30 seconds (depends on CPU)
- **Release build:** ~30-90 seconds (depends on CPU)
- **Incremental build:** ~5-15 seconds

### Runtime Overhead

- **Frame recording:** < 1μs per frame
- **Memory submission:** < 0.5μs per call
- **FPS text update:** ~0.1ms (throttled to 3 FPS)
- **Graph update:** ~0.5ms (throttled to 30 FPS)
- **Total overhead:** < 1% of frame time

## Documentation

- **FPS/RAM Profiler README:** `unity/Profiler/README_FPS_RAM.md`
- **Implementation Plan:** `../mmorpg/plans/064_profiler_fps_ram_overlay.md`
- **Rust Backend:** `../mmorpg/crates/mmorpg-profiler/`
- **Unity FFI Scripts:** `unity/Profiler/FpsRam*.cs`

## Support

For issues:

1. Check the **Troubleshooting** section above
2. Review **README_FPS_RAM.md** for detailed API documentation
3. Check **plan 064** for implementation details
4. Review Unity Console for error messages
5. Verify Rust build with: `cargo build -p mmorpg-profiler --release -vv`

## License

The FPS/RAM profiler is adapted from [Graphy](https://github.com/Tayx94/graphy) by TalesFromScript, licensed under MIT License.