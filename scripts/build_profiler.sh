#!/bin/bash
set -e

echo "====================================="
echo "FPS/RAM Profiler Build Script (macOS)"
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
BUILD_DIR="build_bin"
PROFILER_LIB="libmmorpg_profiler.dylib"
MMORPG_DIR="../mmorpg"

# Parse build type argument
case "$BUILD_TYPE" in
    debug)
        CARGO_FLAGS=""
        TARGET_DIR="debug"
        echo -e "${YELLOW}Building DEBUG configuration${NC}"
        ;;
    release)
        CARGO_FLAGS="--release"
        TARGET_DIR="release"
        echo -e "${YELLOW}Building RELEASE configuration${NC}"
        ;;
    *)
        echo -e "${RED}Invalid build type: $BUILD_TYPE${NC}"
        echo "Usage: $0 [debug|release] [arm64|x86_64]"
        exit 1
        ;;
esac

# Parse architecture argument
case "$BUILD_ARCH" in
    arm64)
        TARGET_TRIPLE="aarch64-apple-darwin"
        echo -e "${YELLOW}Building for ARM64 (native Apple Silicon)${NC}"
        ;;
    x86_64)
        TARGET_TRIPLE="x86_64-apple-darwin"
        echo -e "${YELLOW}Building for x86_64 (Rosetta/Intel)${NC}"
        ;;
    *)
        echo -e "${RED}Invalid architecture: $BUILD_ARCH${NC}"
        echo "Usage: $0 [debug|release] [arm64|x86_64]"
        exit 1
        ;;
esac

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

# Check if mmorpg directory exists
if [ ! -d "$MMORPG_DIR" ]; then
    echo -e "${RED}Error: mmorpg directory not found: ${MMORPG_DIR}${NC}"
    echo ""
    echo "This script expects to be run from: mmorpg/unity-ffi/"
    echo "And expects mmorpg workspace to be at: ${MMORPG_DIR}"
    exit 1
fi

# Create build directory
echo ""
echo "Creating build directory..."
mkdir -p "$BUILD_DIR"
print_success "Build directory created: $BUILD_DIR"

# Build mmorpg-profiler crate
echo ""
echo "Building mmorpg-profiler crate for $TARGET_TRIPLE..."
cd "$MMORPG_DIR"
cargo build -p mmorpg-profiler $CARGO_FLAGS --target "$TARGET_TRIPLE"

# Copy built library to build directory
echo ""
echo "Copying profiler library..."
cp "target/$TARGET_TRIPLE/$TARGET_DIR/$PROFILER_LIB" "$SCRIPT_DIR/$BUILD_DIR/"
print_success "Profiler library built: $BUILD_DIR/$PROFILER_LIB (for $TARGET_TRIPLE)"

# Change back to script directory
cd "$SCRIPT_DIR"

# Display library symbols
echo ""
echo "====================================="
echo "Exported FFI Symbols:"
echo "====================================="
nm -gU "$BUILD_DIR/$PROFILER_LIB" | grep -E "profiler_" | awk '{print "  " $3}' || \
    echo "  No profiler symbols found (library may not be built correctly)"

# Display build summary
echo ""
echo "====================================="
echo "Build Summary:"
echo "====================================="
echo -e "Build Type: ${GREEN}$BUILD_TYPE${NC}"
echo -e "Architecture: ${GREEN}$BUILD_ARCH${NC}"
echo -e "Target Triple: ${GREEN}$TARGET_TRIPLE${NC}"
echo -e "Profiler Library: ${GREEN}$BUILD_DIR/$PROFILER_LIB${NC}"
echo ""
echo "Next Steps:"
echo "1. Copy $BUILD_DIR/$PROFILER_LIB to your Unity project's Plugins/macOS/ folder"
echo "2. Copy unity/Profiler/FpsRam*.cs to your Unity project's Assets/Scripts/Profiler/ folder"
echo "3. Add FpsRamProfilerBehaviour component to a GameObject in your scene"
echo "4. Press F5 to toggle profiler visibility"
echo ""
echo "Note: If Unity is running under Rosetta, rebuild with: $0 $BUILD_TYPE x86_64"
echo "====================================="
