#!/bin/bash
set -e

echo "====================================="
echo "FPS/RAM Profiler Setup Script"
echo "====================================="

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Configuration
BUILD_DIR="${SCRIPT_DIR}/build_bin"
UNITY_SCRIPTS_DIR="${SCRIPT_DIR}/unity/Profiler"
PROFILER_LIB="${BUILD_DIR}/libmmorpg_profiler.dylib"

# Function to print success message
print_success() {
    echo -e "${GREEN}✓ $1${NC}"
}

# Function to print error message
print_error() {
    echo -e "${RED}✗ $1${NC}"
}

# Function to print warning message
print_warning() {
    echo -e "${YELLOW}⚠ $1${NC}"
}

# Check if target directory is provided
if [ -z "$1" ]; then
    echo -e "${RED}Error: Target directory not specified${NC}"
    echo ""
    echo "Usage: $0 <target-unity-project-path> [arm64|x86_64]"
    echo ""
    echo "Arguments:"
    echo "  target-unity-project-path  Path to Unity project directory"
    echo "  arm64|x86_64               Architecture (default: arm64)"
    echo "                             Use x86_64 for Unity under Rosetta"
    echo ""
    echo "Example:"
    echo "  $0 ${SCRIPT_DIR}/examples/helloworld-ffi"
    echo "  $0 ${SCRIPT_DIR}/examples/helloworld-ffi arm64"
    echo "  $0 ${SCRIPT_DIR}/examples/helloworld-ffi x86_64"
    echo ""
    exit 1
fi

TARGET_DIR="$1"
BUILD_ARCH="${2:-arm64}"

# Parse architecture argument
case "$BUILD_ARCH" in
    arm64)
        echo -e "${GREEN}Building for ARM64 (native Apple Silicon)${NC}"
        ;;
    x86_64)
        echo -e "${GREEN}Building for x86_64 (Rosetta/Intel)${NC}"
        ;;
    *)
        echo -e "${RED}Invalid architecture: $BUILD_ARCH${NC}"
        echo "Usage: $0 <target-unity-project-path> [arm64|x86_64]"
        exit 1
        ;;
esac

# Validate target directory exists
if [ ! -d "$TARGET_DIR" ]; then
    echo -e "${RED}Error: Target directory does not exist: ${TARGET_DIR}${NC}"
    echo ""
    echo "Please create Unity project directory first."
    exit 1
fi

# Define paths
UNITY_PLUGINS_DIR="${TARGET_DIR}/Assets/Plugins"
UNITY_MACOS_DIR="${UNITY_PLUGINS_DIR}/macOS"
UNITY_PROFILER_DIR="${TARGET_DIR}/Assets/Scripts/Profiler"

echo ""
echo "Configuration:"
echo "  Source Build Dir:    ${BUILD_DIR}"
echo "  Source Scripts Dir:  ${UNITY_SCRIPTS_DIR}"
echo "  Target Unity Dir:    ${TARGET_DIR}"
echo "  Architecture:       ${BUILD_ARCH}"
echo ""

# Step 1: Build profiler
echo "====================================="
echo "Step 1: Building FPS/RAM Profiler"
echo "====================================="

if [ -f "${SCRIPT_DIR}/build_profiler.sh" ]; then
    echo "Running profiler build script..."
    cd "$SCRIPT_DIR"
    ./build_profiler.sh release "$BUILD_ARCH"

    if [ $? -ne 0 ]; then
        echo -e "${RED}Build failed!${NC}"
        exit 1
    fi

    echo -e "${GREEN}✓ Build successful${NC}"
else
    echo -e "${YELLOW}Warning: build_profiler.sh not found, skipping build step${NC}"
fi

echo ""

# Step 2: Copy files to Unity project
echo "====================================="
echo "Step 2: Copying Files to Unity Project"
echo "====================================="

# Check if build directory exists
if [ ! -d "$BUILD_DIR" ]; then
    echo -e "${RED}Error: Build directory not found: ${BUILD_DIR}${NC}"
    echo ""
    echo "Please run ./build_profiler.sh first to build the profiler library."
    exit 1
fi

# Check if unity/Profiler directory exists
if [ ! -d "$UNITY_SCRIPTS_DIR" ]; then
    echo -e "${RED}Error: Unity profiler scripts directory not found: ${UNITY_SCRIPTS_DIR}${NC}"
    echo -e "${RED}Expected: unity/Profiler${NC}"
    exit 1
fi

# Check if profiler library exists
if [ ! -f "$PROFILER_LIB" ]; then
    echo -e "${RED}Error: Profiler library not found: ${PROFILER_LIB}${NC}"
    echo ""
    echo "Please run ./build_profiler.sh to build the profiler library."
    exit 1
fi

# Create directory structure if it doesn't exist
echo "Creating directory structure..."

if [ ! -d "$UNITY_MACOS_DIR" ]; then
    mkdir -p "$UNITY_MACOS_DIR"
    echo -e "${GREEN}✓ Created: ${UNITY_MACOS_DIR}${NC}"
fi

if [ ! -d "$UNITY_PROFILER_DIR" ]; then
    mkdir -p "$UNITY_PROFILER_DIR"
    echo -e "${GREEN}✓ Created: ${UNITY_PROFILER_DIR}${NC}"
fi

echo ""
echo "Copying files..."

# Copy native library
echo -n "  Copying libmmorpg_profiler.dylib (${BUILD_ARCH})... "
cp -f "$PROFILER_LIB" "${UNITY_MACOS_DIR}/libmmorpg_profiler.dylib"
if [ $? -eq 0 ]; then
    echo -e "${GREEN}✓ Done${NC}"
else
    echo -e "${RED}✗ Failed${NC}"
    exit 1
fi

# Set executable permissions
chmod +x "${UNITY_MACOS_DIR}/libmmorpg_profiler.dylib" 2>/dev/null || true

# Copy C# profiler scripts
SCRIPTS_COPIED=0
for script in "${UNITY_SCRIPTS_DIR}"/FpsRam*.cs "${UNITY_SCRIPTS_DIR}"/README_FPS_RAM.md; do
    if [ -f "$script" ]; then
        script_name=$(basename "$script")
        echo -n "  Copying ${script_name}... "

        cp -f "$script" "${UNITY_PROFILER_DIR}/"
        if [ $? -eq 0 ]; then
            echo -e "${GREEN}✓ Done${NC}"
            ((SCRIPTS_COPIED++))
        else
            echo -e "${RED}✗ Failed${NC}"
            exit 1
        fi
    fi
done

echo ""

# Step 3: Display summary
echo "====================================="
echo "Setup Complete!"
echo "====================================="
echo ""
echo "Summary:"
echo "  • Built profiler: ${GREEN}✓${NC} (${BUILD_ARCH})"
echo "  • Copied library: ${GREEN}✓${NC} (${UNITY_MACOS_DIR}/libmmorpg_profiler.dylib, ${BUILD_ARCH})"
echo "  • Copied scripts: ${GREEN}✓${NC} (${SCRIPTS_COPIED} files to ${UNITY_PROFILER_DIR}/)"
echo ""
echo "Next Steps:"
echo "  1. Open Unity project: ${TARGET_DIR}"
echo "  2. Unity should auto-detect plugin and scripts"
echo "  3. If not, go to Assets → Refresh"
echo "  4. Create a GameObject in your scene"
echo "  5. Add FpsRamProfilerBehaviour component to the GameObject"
echo "  6. (Optional) Add FpsRamProfilerOverlay component for UI visualization"
echo "  7. Press Play in Unity Editor"
echo "  8. Press F5 to toggle profiler visibility"
echo ""
echo "Usage:"
echo "  • Profiler automatically records frame time and memory metrics"
echo "  • Switch contexts: Unity (engine), Rust (backend), or Total (combined)"
echo "  • View FPS metrics: Current, Average, 1% Low, 0.1% Low"
echo "  • View memory metrics: Reserved, Allocated, Mono heap"
echo "  • View frame timing graph with color-coded thresholds"
echo "  • Configure settings via Inspector (update rates, thresholds, etc.)"
echo ""
echo "Building for Different Architectures:"
echo "  • Native (ARM64): ./setup_profiler.sh ${TARGET_DIR} arm64"
echo "  • Rosetta (x86_64): ./setup_profiler.sh ${TARGET_DIR} x86_64"
echo ""
echo "Troubleshooting:"
echo "  • If plugin not detected, check Unity Console for errors"
echo "  • Ensure that plugin is at Assets/Plugins/macOS/"
echo "  • Verify file permissions: chmod +x Assets/Plugins/macOS/*.dylib"
echo "  • Restart Unity Editor after copying files"
echo "  • IMPORTANT: Enable \"Allow 'unsafe' Code\" in Unity Project Settings:"
echo "    - Edit → Project Settings → Player"
echo "    - Scroll down to Other Settings"
echo "    - Set \"Allow 'unsafe' Code\" to ON"
echo "  • See unity/Profiler/README_FPS_RAM.md for detailed documentation"
echo "  • See plan 064: mmorpg/plans/064_profiler_fps_ram_overlay.md"
echo ""
