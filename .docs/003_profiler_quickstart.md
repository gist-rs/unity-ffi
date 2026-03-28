# FPS/RAM Profiler Scripts - Quick Summary

Automated build and deployment scripts for FPS/RAM profiler integration with Unity.

## Scripts Overview

| Script | Purpose | Usage |
|---------|----------|--------|
| `build_profiler.sh` | Build mmorpg-profiler Rust crate | `./scripts/build_profiler.sh [release|debug] [arm64|x86_64]` |
| `setup_profiler.sh` | Copy profiler files to Unity project | `./scripts/setup_profiler.sh <unity-project> [arm64|x86_64]` |
| `quickstart_profiler.sh` | One-command build and setup | `./scripts/quickstart_profiler.sh [release|debug] [arm64|x86_64] [unity-project]` |

## Quick Start

**Fastest way to get started:**

```bash
# Default: Build release/arm64 for examples/helloworld-ffi
./scripts/quickstart_profiler.sh

# Debug build
./scripts/quickstart_profiler.sh debug

# Rosetta build (x86_64)
./scripts/quickstart_profiler.sh release x86_64

# Custom Unity project
./scripts/quickstart_profiler.sh release arm64 /path/to/your/unity/project
```

## Architecture Guide

**Determine your architecture:**
```bash
uname -m
# Output: arm64 or x86_64
```

**When to use each:**
- **arm64** - Native Apple Silicon (M1/M2/M3 Macs)
- **x86_64** - Rosetta translation or Intel Macs

**Check Unity architecture:**
- Unity Editor → About Unity → Check for "Apple Silicon" or "Intel"
- Or run: `file /Applications/Unity/Hub/Editor/*/Unity.app/Contents/MacOS/Unity`

## Script Details

### 1. build_profiler.sh

**Builds** mmorpg-profiler Rust crate for Unity FFI.

**Output:** `build_bin/libmmorpg_profiler.dylib`

**Examples:**
```bash
./scripts/build_profiler.sh release arm64      # Release build for ARM64
./scripts/build_profiler.sh debug arm64        # Debug build for ARM64
./scripts/build_profiler.sh release x86_64    # Release build for Rosetta
```

**Requirements:**
- mmorpg workspace at `../mmorpg/`
- Rust toolchain installed

---

### 2. setup_profiler.sh

**Copies** profiler files to Unity project and builds if needed.

**What it does:**
1. Builds mmorpg-profiler (if `build_profiler.sh` exists)
2. Creates `Assets/Plugins/macOS/` directory
3. Creates `Assets/Scripts/Profiler/` directory
4. Copies `libmmorpg_profiler.dylib` to Plugins
5. Copies `FpsRam*.cs` files to Scripts
6. Sets executable permissions

**Examples:**
```bash
./scripts/setup_profiler.sh ./examples/helloworld-ffi
./scripts/setup_profiler.sh ./examples/helloworld-ffi x86_64
./scripts/setup_profiler.sh /Users/name/Projects/MyGame arm64
```

**Requirements:**
- Built library in `build_bin/`
- Source scripts in `unity/Profiler/`
- Valid Unity project path

---

### 3. quickstart_profiler.sh

**Combines** build and setup in one command.

**What it does:**
1. Executes `build_profiler.sh`
2. Executes `setup_profiler.sh`
3. Displays final instructions

**Examples:**
```bash
./scripts/quickstart_profiler.sh                    # Default (release/arm64/examples/helloworld-ffi)
./scripts/quickstart_profiler.sh debug               # Debug build
./scripts/quickstart_profiler.sh release x86_64       # Rosetta build
./scripts/quickstart_profiler.sh release arm64 /path/to/unity   # Custom project
```

**Best for:** First-time setup, quick iterations, CI/CD pipelines

## After Setup

### Unity Configuration

1. **Open Unity project**
2. **Enable unsafe code:**
   - Edit → Project Settings → Player
   - Other Settings → "Allow 'unsafe' Code" → ON
3. **Refresh Assets:** Assets → Refresh (or Cmd+R)

### Add to Scene

1. Create GameObject (e.g., "FPS RAM Profiler")
2. Add `FpsRamProfilerBehaviour` component
3. (Optional) Add `FpsRamProfilerOverlay` component for UI
4. Press Play

### Test

- Press **F5** to toggle profiler
- Check Unity Console for initialization messages
- Verify FPS and memory metrics are displayed

## File Structure After Setup

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

## Troubleshooting

### Build Issues

**cargo: command not found**
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

**Architecture mismatch**
```bash
# Check system architecture
uname -m

# If Unity under Rosetta, rebuild with x86_64
./scripts/quickstart_profiler.sh release x86_64
```

### Unity Issues

**Plugin not detected**
1. Check file exists: `ls Assets/Plugins/macOS/libmmorpg_profiler.dylib`
2. Restart Unity Editor
3. Go to Assets → Refresh

**unsafe code errors**
1. Edit → Project Settings → Player
2. Other Settings → "Allow 'unsafe' Code" → ON

**DllNotFoundException**
1. Verify architecture matches Unity (arm64 vs x86_64)
2. Rebuild with correct architecture
3. Check file permissions: `chmod +x Assets/Plugins/macOS/*.dylib`

### Performance Issues

**High overhead**
1. Reduce FPS text update rate (Inspector: 1-2)
2. Reduce graph update rate (Inspector: 15-20)
3. Increase memory update interval (Inspector: 1.0s+)

## Common Commands

```bash
# Help
./scripts/quickstart_profiler.sh --help

# Clean build
rm -rf build_bin/
./scripts/quickstart_profiler.sh

# Check architecture
uname -m

# Verify plugin
ls -la Assets/Plugins/macOS/libmmorpg_profiler.dylib

# Check scripts
ls -la Assets/Scripts/Profiler/FpsRam*.cs

# View FFI symbols
nm -gU build_bin/libmmorpg_profiler.dylib | grep profiler_

# Rebuild clean
cd ../mmorpg && cargo clean -p mmorpg-profiler && cd ../unity-ffi
./quickstart_profiler.sh
```

## Documentation

- **Detailed scripts guide:** `README_PROFILER_SCRIPTS.md`
- **Profiler API:** `unity/Profiler/README_FPS_RAM.md`
- **Implementation plan:** `../mmorpg/plans/064_profiler_fps_ram_overlay.md`
- **Rust backend:** `../mmorpg/crates/mmorpg-profiler/`
- **Graphy original:** https://github.com/Tayx94/graphy

## License

Adapted from [Graphy](https://github.com/Tayx94/graphy) by TalesFromScript (MIT License).