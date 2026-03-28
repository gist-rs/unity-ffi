#!/bin/bash
set -e

echo "====================================="
echo "Unity FFI Build Script (macOS)"
echo "====================================="

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR/.."

# Configuration
BUILD_TYPE="${1:-release}"
BUILD_ARCH="${2:-arm64}"
BUILD_DIR="build_bin"
SERVER_BINARY="unity-ffi-server"
UNITY_LIB="libunity_network.dylib"

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

# Create build directory
echo ""
echo "Creating build directory..."
mkdir -p "$BUILD_DIR"
print_success "Build directory created: $BUILD_DIR"

# Build unity-network library (cdylib)
echo ""
echo "Building unity-network FFI library for $TARGET_TRIPLE..."
cargo build --manifest-path=crates/unity-network/Cargo.toml $CARGO_FLAGS --target "$TARGET_TRIPLE"

# Copy the built library (workspace builds put it in target dir)
cp "target/$TARGET_TRIPLE/$TARGET_DIR/$UNITY_LIB" "$BUILD_DIR/"
print_success "Unity library built: $BUILD_DIR/$UNITY_LIB (for $TARGET_TRIPLE)"

# Build server binary
echo ""
echo "Building server binary for $TARGET_TRIPLE..."
cargo build --manifest-path=crates/game-server/Cargo.toml $CARGO_FLAGS --target "$TARGET_TRIPLE" --bin unity-ffi-server

# Copy the server binary (workspace builds put it in target dir)
cp "target/$TARGET_TRIPLE/$TARGET_DIR/$SERVER_BINARY" "$BUILD_DIR/"
print_success "Server binary built: $BUILD_DIR/$SERVER_BINARY (for $TARGET_TRIPLE)"

# Run server to generate certificate hash (for development)
echo ""
echo "Generating development certificate..."
if [ "$BUILD_TYPE" = "debug" ]; then
    timeout 3 "$BUILD_DIR/$SERVER_BINARY" 2>&1 | grep "Certificate SHA-256 hash:" | sed 's/.*Certificate SHA-256 hash: //' > "$BUILD_DIR/certificate_hash.txt" || true

    if [ -f "$BUILD_DIR/certificate_hash.txt" ] && [ -s "$BUILD_DIR/certificate_hash.txt" ]; then
        CERT_HASH=$(cat "$BUILD_DIR/certificate_hash.txt")
        print_success "Certificate hash generated: $CERT_HASH"
        echo "Certificate hash saved to: $BUILD_DIR/certificate_hash.txt"
        echo ""
        echo "IMPORTANT: Use this hash in Unity's NetworkPlayer.cs:"
        echo "  certificateHash = \"$CERT_HASH\""
    else
        print_warning "Could not generate certificate hash automatically"
        echo "Run the server manually to get the certificate hash"
    fi
fi

# Display library symbols
echo ""
echo "====================================="
echo "Exported FFI Symbols:"
echo "====================================="
nm -gU "$BUILD_DIR/$UNITY_LIB" | grep -E "network_|Ffi" | awk '{print "  " $3}'

# Display build summary
echo ""
echo "====================================="
echo "Build Summary:"
echo "====================================="
echo -e "Build Type: ${GREEN}$BUILD_TYPE${NC}"
echo -e "Architecture: ${GREEN}$BUILD_ARCH${NC}"
echo -e "Target Triple: ${GREEN}$TARGET_TRIPLE${NC}"
echo -e "Unity Library: ${GREEN}$BUILD_DIR/$UNITY_LIB${NC}"
echo -e "Server Binary: ${GREEN}$BUILD_DIR/$SERVER_BINARY${NC}"
if [ -f "$BUILD_DIR/certificate_hash.txt" ]; then
    echo -e "Certificate Hash: ${GREEN}$BUILD_DIR/certificate_hash.txt${NC}"
fi
echo ""
echo "Next Steps:"
echo "1. Copy $BUILD_DIR/$UNITY_LIB to your Unity project's Plugins folder"
echo "2. Run the server: ./$BUILD_DIR/$SERVER_BINARY"
echo "3. Configure NetworkPlayer.cs in Unity with the certificate hash"
echo ""
echo "Note: If Unity is running under Rosetta, rebuild with: $0 $BUILD_TYPE x86_64"
echo "====================================="
