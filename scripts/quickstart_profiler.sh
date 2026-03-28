#!/bin/bash
set -e

echo "====================================="
echo "FPS/RAM Profiler Quickstart"
echo "====================================="

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

# Configuration
BUILD_TYPE="${1:-release}"
BUILD_ARCH="${2:-arm64}"
TARGET_DIR="${3:-./examples/helloworld-ffi}"

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

# Display usage if help requested
if [ "$1" = "-h" ] || [ "$1" = "--help" ]; then
    echo ""
    echo "Quickstart script for FPS/RAM Profiler - Build and Setup in one command"
    echo ""
    echo "Usage:"
    echo "  $0 [build_type] [architecture] [target-unity-project]"
    echo ""
    echo "Arguments:"
    echo "  build_type              debug or release (default: release)"
    echo "  architecture            arm64 or x86_64 (default: arm64)"
    echo "  target-unity-project   Path to Unity project (default: ./examples/helloworld-ffi)"
    echo ""
    echo "Examples:"
    echo "  $0                              # Build release/arm64 for examples/helloworld-ffi"
    echo "  $0 debug                        # Build debug/arm64 for examples/helloworld-ffi"
    echo "  $0 release x86_64                # Build release/x86_64 for Rosetta"
    echo "  $0 release arm64 /path/to/unity   # Build for custom Unity project"
    echo ""
    echo "Architecture Guide:"
    echo "  • arm64   - Native Apple Silicon (M1/M2/M3 Macs)"
    echo "  • x86_64  - Rosetta or Intel Macs"
    echo ""
    echo "How to determine architecture:"
    echo "  • Run: uname -m"
    echo "  • arm64 = Use arm64"
    echo "  • x86_64 = Use x86_64"
    echo ""
    echo "What this script does:"
    echo "  1. Builds mmorpg-profiler crate (from mmorpg workspace)"
    echo "  2. Copies libmmorpg_profiler.dylib to Unity Plugins/macOS/"
    echo "  3. Copies FpsRam*.cs scripts to Unity Assets/Scripts/Profiler/"
    echo "  4. Displays setup instructions"
    echo ""
    exit 0
fi

# Validate target directory
if [ ! -d "$TARGET_DIR" ]; then
    echo -e "${RED}Error: Target directory does not exist: ${TARGET_DIR}${NC}"
    echo ""
    echo "Please create Unity project directory first or provide a valid path."
    echo ""
    echo "Examples:"
    echo "  $0 $BUILD_TYPE $BUILD_ARCH ./examples/helloworld-ffi"
    echo ""
    exit 1
fi

echo ""
echo "Configuration:"
echo "  Build Type:  ${BUILD_TYPE}"
echo "  Architecture: ${BUILD_ARCH}"
echo "  Target Dir:  ${TARGET_DIR}"
echo ""

# Step 1: Build profiler
echo "====================================="
echo "Step 1: Building FPS/RAM Profiler"
echo "====================================="
echo ""

if [ ! -f "./build_profiler.sh" ]; then
    echo -e "${RED}Error: build_profiler.sh not found${NC}"
    echo "This script must be run from the unity-ffi directory."
    exit 1
fi

# Execute build script
./build_profiler.sh "$BUILD_TYPE" "$BUILD_ARCH"

if [ $? -ne 0 ]; then
    echo -e "${RED}Build failed!${NC}"
    exit 1
fi

echo ""

# Step 2: Setup profiler in Unity project
echo "====================================="
echo "Step 2: Setting Up Profiler in Unity Project"
echo "====================================="
echo ""

if [ ! -f "./setup_profiler.sh" ]; then
    echo -e "${RED}Error: setup_profiler.sh not found${NC}"
    exit 1
fi

# Execute setup script
./setup_profiler.sh "$TARGET_DIR" "$BUILD_ARCH"

if [ $? -ne 0 ]; then
    echo -e "${RED}Setup failed!${NC}"
    exit 1
fi

# Step 3: Display final instructions
echo "====================================="
echo "Quickstart Complete!"
echo "====================================="
echo ""
echo "Summary:"
echo -e "  Build Type:    ${GREEN}$BUILD_TYPE${NC}"
echo -e "  Architecture:   ${GREEN}$BUILD_ARCH${NC}"
echo -e "  Target Project: ${GREEN}$TARGET_DIR${NC}"
echo ""
echo "What was done:"
echo -e "  ${GREEN}✓${NC} Built mmorpg-profiler crate"
echo -e "  ${GREEN}✓${NC} Copied libmmorpg_profiler.dylib to Unity Plugins/macOS/"
echo -e "  ${GREEN}✓${NC} Copied FpsRam*.cs scripts to Unity Assets/Scripts/Profiler/"
echo ""
echo "Next Steps:"
echo "  1. Open Unity project: ${TARGET_DIR}"
echo "  2. Unity should auto-detect the plugin and scripts"
echo "  3. If not, go to Assets → Refresh"
echo "  4. Create a GameObject in your scene"
echo "  5. Add FpsRamProfilerBehaviour component"
echo "  6. (Optional) Add FpsRamProfilerOverlay component for UI"
echo "  7. Press Play in Unity Editor"
echo "  8. Press F5 to toggle profiler visibility"
echo ""
echo "Configuration (via Inspector):"
echo "  • Enable/Disable frame recording"
echo "  • Enable/Disable memory tracking"
echo "  • Set memory update interval (default: 0.5s)"
echo "  • Configure hotkey (default: F5)"
echo "  • Enable/disable debug logging"
echo ""
echo "Profiler Features:"
echo "  • Real-time FPS monitoring (current, avg, 1% low, 0.1% low)"
echo "  • Memory usage tracking (allocated, reserved, mono heap)"
echo "  • Frame timing graph with color-coded thresholds"
echo "  • Multi-context support (Unity, Rust, Total)"
echo "  • Tab switching between contexts"
echo "  • Memory bar visualization (Graphy-style: pink/green/blue)"
echo ""
echo "Hotkeys:"
echo "  • F5 - Toggle profiler visibility"
echo ""
echo "Common Issues:"
echo "  • Plugin not detected: Restart Unity Editor"
echo "  • FFI errors: Enable 'unsafe' code in Project Settings"
echo "  • Rosetta issues: Rebuild with x86_64: $0 $BUILD_TYPE x86_64"
echo ""
echo "Documentation:"
echo "  • Quick reference: See unity/Profiler/README_FPS_RAM.md"
echo "  • Implementation plan: See mmorpg/plans/064_profiler_fps_ram_overlay.md"
echo "  • Rust backend: See mmorpg/crates/mmorpg-profiler/"
echo ""
echo "Troubleshooting Commands:"
echo "  • Check architecture:    uname -m"
echo "  • Check plugin exists:   ls -la ${TARGET_DIR}/Assets/Plugins/macOS/"
echo "  • Check scripts exist:   ls -la ${TARGET_DIR}/Assets/Scripts/Profiler/FpsRam*.cs"
echo "  • View Unity logs:      Unity Editor → Console"
echo "  • Rebuild clean:        rm -rf build_bin/ && $0 $BUILD_TYPE $BUILD_ARCH"
echo ""
echo "====================================="
