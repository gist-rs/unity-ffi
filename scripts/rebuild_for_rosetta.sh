#!/bin/bash
set -e

echo "====================================="
echo "Rebuild Unity Native Lib for Rosetta"
echo "====================================="

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$PROJECT_ROOT"

# Configuration
BUILD_TYPE="${1:-release}"
UNITY_PROJECT="$PROJECT_ROOT/examples/helloworld-ffi/Assets/Plugins/macOS"
LIB_NAME="libunity_network.dylib"

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

# Check if target is installed
if ! rustup target list --installed | grep -q "x86_64-apple-darwin"; then
    print_warning "x86_64-apple-darwin target not found, installing..."
    rustup target add x86_64-apple-darwin
fi

# Build for x86_64
echo ""
echo "Building $LIB_NAME for x86_64 (Rosetta)..."
./scripts/build.sh "$BUILD_TYPE" x86_64

# Copy to Unity project
echo ""
echo "Copying to Unity project..."
echo "Creating Plugins directory if it doesn't exist..."
mkdir -p "$UNITY_PROJECT"
print_success "Plugins directory ready: $UNITY_PROJECT"

cp "$PROJECT_ROOT/build_bin/$LIB_NAME" "$UNITY_PROJECT/"
print_success "Copied to $UNITY_PROJECT/$LIB_NAME"

# Verify architecture
ARCH=$(file "$UNITY_PROJECT/$LIB_NAME" | awk '{print $NF}')
echo ""
echo "Verification:"
echo "  Architecture: $ARCH"

if echo "$ARCH" | grep -q "x86_64"; then
    print_success "Library is correctly built for x86_64 (Rosetta)"
else
    print_error "Library is NOT x86_64, found: $ARCH"
    exit 1
fi

echo ""
echo "====================================="
echo "Summary:"
echo "====================================="
echo -e "Build Type: ${GREEN}$BUILD_TYPE${NC}"
echo -e "Architecture: ${GREEN}x86_64 (Rosetta)${NC}"
echo -e "Source: ${GREEN}$PROJECT_ROOT/build_bin/$LIB_NAME${NC}"
echo -e "Destination: ${GREEN}$UNITY_PROJECT/$LIB_NAME${NC}"
echo ""
echo "You can now run Unity (under Rosetta) and it should load the library."
echo "====================================="
