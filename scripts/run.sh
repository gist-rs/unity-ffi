#!/bin/bash
set -e

echo "====================================="
echo "Unity FFI Server Run Script"
echo "====================================="

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Configuration
SERVER_PORT=4433
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
BUILD_DIR="$PROJECT_ROOT/build_bin"
SERVER_BINARY="$BUILD_DIR/unity-ffi-server"
cd "$PROJECT_ROOT"

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

# Step 1: Kill any existing server on port 4433
echo ""
echo "Step 1: Checking for existing server on port ${SERVER_PORT}..."
SERVER_PID=$(lsof -ti:${SERVER_PORT} 2>/dev/null || true)

if [ -n "$SERVER_PID" ]; then
    echo -e "${YELLOW}Found server process (PID: ${SERVER_PID}) on port ${SERVER_PORT}${NC}"
    echo "Stopping server..."
    kill -9 $SERVER_PID 2>/dev/null || true
    sleep 1

    # Verify it's stopped
    SERVER_PID=$(lsof -ti:${SERVER_PORT} 2>/dev/null || true)
    if [ -z "$SERVER_PID" ]; then
        print_success "Server stopped successfully"
    else
        print_error "Failed to stop server (PID: ${SERVER_PID})"
        exit 1
    fi
else
    print_success "Port ${SERVER_PORT} is free"
fi

# Step 2: Build the server if it doesn't exist
echo ""
echo "Step 2: Checking if server binary exists..."

if [ ! -f "$SERVER_BINARY" ]; then
    echo -e "${YELLOW}Server binary not found. Building...${NC}"
    if [ -f "./scripts/build.sh" ]; then
        ./scripts/build.sh release arm64
    else
        print_error "scripts/build.sh not found"
        exit 1
    fi

    if [ ! -f "$SERVER_BINARY" ]; then
        print_error "Failed to build server binary"
        exit 1
    fi
    print_success "Server binary built successfully"
else
    print_success "Server binary exists: $SERVER_BINARY"

    # Check if user wants to rebuild
    if [ "$1" == "--rebuild" ]; then
        echo -e "${YELLOW}Rebuilding server...${NC}"
        ./scripts/build.sh release arm64
        print_success "Server rebuilt successfully"
    fi
fi

# Step 3: Run the server
echo ""
echo "====================================="
echo "Starting Unity FFI WebTransport server..."
echo "====================================="
echo ""

# Make sure the binary is executable
chmod +x "$SERVER_BINARY"

# Run the server
exec "$SERVER_BINARY"
