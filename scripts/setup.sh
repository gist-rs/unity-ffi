#!/bin/bash
set -e

echo "====================================="
echo "Unity FFI Setup Script"
echo "====================================="

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# Configuration
BUILD_DIR="${PROJECT_ROOT}/build_bin"
UNITY_SCRIPTS_DIR="${PROJECT_ROOT}/unity"
SERVER_BINARY="${BUILD_DIR}/unity-ffi-server"
SERVER_PORT=4433

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
    echo "  $0 ${PROJECT_ROOT}/examples/helloworld-ffi"
    echo "  $0 ${PROJECT_ROOT}/examples/helloworld-ffi arm64"
    echo "  $0 ${PROJECT_ROOT}/examples/helloworld-ffi x86_64"
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
    echo "Please create the Unity project directory first."
    exit 1
fi

# Define paths
UNITY_PLUGINS_DIR="${TARGET_DIR}/Assets/Plugins"
UNITY_MACOS_DIR="${UNITY_PLUGINS_DIR}/macOS"
UNITY_ASSETS_SCRIPTS_DIR="${TARGET_DIR}/Assets/Scripts"

echo ""
echo "Configuration:"
echo "  Source Build Dir:  ${BUILD_DIR}"
echo "  Source Scripts Dir: ${UNITY_SCRIPTS_DIR}"
echo "  Target Unity Dir:  ${TARGET_DIR}"
echo "  Server Port:       ${SERVER_PORT}"
echo "  Architecture:      ${BUILD_ARCH}"
echo ""

# Step 1: Build the project
echo "====================================="
echo "Step 1: Building Project"
echo "====================================="

if [ -f "${SCRIPT_DIR}/build.sh" ]; then
    echo "Running build script..."
    cd "$PROJECT_ROOT"
    ./scripts/build.sh debug "$BUILD_ARCH"

    if [ $? -ne 0 ]; then
        echo -e "${RED}Build failed!${NC}"
        exit 1
    fi

    echo -e "${GREEN}✓ Build successful${NC}"
else
    echo -e "${YELLOW}Warning: scripts/build.sh not found, skipping build step${NC}"
fi

echo ""

# Step 2: Kill existing server on port 4433
echo "====================================="
echo "Step 2: Checking for Running Server"
echo "====================================="

SERVER_PID=$(lsof -ti:${SERVER_PORT} 2>/dev/null || true)

if [ ! -z "$SERVER_PID" ]; then
    echo -e "${YELLOW}Found server running on port ${SERVER_PORT} (PID: ${SERVER_PID})${NC}"
    echo "Killing existing server process..."

    kill -9 $SERVER_PID 2>/dev/null || true
    sleep 1

    # Verify it's dead
    SERVER_PID=$(lsof -ti:${SERVER_PORT} 2>/dev/null || true)
    if [ -z "$SERVER_PID" ]; then
        echo -e "${GREEN}✓ Server stopped successfully${NC}"
    else
        echo -e "${RED}✗ Failed to stop server${NC}"
        exit 1
    fi
else
    echo -e "${GREEN}✓ No server running on port ${SERVER_PORT}${NC}"
fi

echo ""

# Step 3: Copy files to Unity project
echo "====================================="
echo "Step 3: Copying Files to Unity Project"
echo "====================================="

# Check if build directory exists
if [ ! -d "$BUILD_DIR" ]; then
    echo -e "${RED}Error: Build directory not found: ${BUILD_DIR}${NC}"
    echo ""
    echo "Please run ./scripts/build.sh first to build FFI library."
    exit 1
fi

# Check if unity directory exists
if [ ! -d "$UNITY_SCRIPTS_DIR" ]; then
    echo -e "${RED}Error: Unity scripts directory not found: ${UNITY_SCRIPTS_DIR}${NC}"
    echo -e "${RED}Expected: unity/${NC}"
    exit 1
fi

# Check if library exists
UNITY_LIB="${BUILD_DIR}/libunity_network.dylib"
if [ ! -f "$UNITY_LIB" ]; then
    echo -e "${RED}Error: Unity library not found: ${UNITY_LIB}${NC}"
    echo ""
    echo "Please run ./scripts/build.sh to build FFI library."
    exit 1
fi

# Check if server binary exists
if [ ! -f "$SERVER_BINARY" ]; then
    echo -e "${RED}Error: Server binary not found: ${SERVER_BINARY}${NC}"
    exit 1
fi

# Create directory structure if it doesn't exist
echo "Creating directory structure..."

if [ ! -d "$UNITY_MACOS_DIR" ]; then
    mkdir -p "$UNITY_MACOS_DIR"
    echo -e "${GREEN}✓ Created: ${UNITY_MACOS_DIR}${NC}"
fi

if [ ! -d "$UNITY_ASSETS_SCRIPTS_DIR" ]; then
    mkdir -p "$UNITY_ASSETS_SCRIPTS_DIR"
    echo -e "${GREEN}✓ Created: ${UNITY_ASSETS_SCRIPTS_DIR}${NC}"
fi

echo ""
echo "Copying files..."

# Copy native library
echo -n "  Copying libunity_network.dylib (${BUILD_ARCH})... "
cp -f "$UNITY_LIB" "${UNITY_MACOS_DIR}/libunity_network.dylib"
if [ $? -eq 0 ]; then
    echo -e "${GREEN}✓ Done${NC}"
else
    echo -e "${RED}✗ Failed${NC}"
    exit 1
fi

# Set executable permissions
chmod +x "${UNITY_MACOS_DIR}/libunity_network.dylib" 2>/dev/null || true

# Copy C# scripts
SCRIPTS_COPIED=0
for script in "${UNITY_SCRIPTS_DIR}"/*.cs; do
    if [ -f "$script" ]; then
        script_name=$(basename "$script")
        echo -n "  Copying ${script_name}... "

        cp -f "$script" "${UNITY_ASSETS_SCRIPTS_DIR}/"
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

# Step 4: Start the server
echo "====================================="
echo "Step 4: Starting Server"
echo "====================================="

echo "Starting server in background..."
echo "Server binary: ${SERVER_BINARY}"
echo "Server port: ${SERVER_PORT}"
echo ""

# Clear log file before starting server
rm -f /tmp/unity-ffi-server.log

# Start server in background and capture PID
"$SERVER_BINARY" > /tmp/unity-ffi-server.log 2>&1 &
SERVER_PID=$!

# Wait a moment for server to start
sleep 2

# Check if server is still running
if ps -p $SERVER_PID > /dev/null 2>&1; then
    echo -e "${GREEN}✓ Server started successfully (PID: ${SERVER_PID})${NC}"

    # Check if it printed the listening message
    if grep -q "listening on" /tmp/unity-ffi-server.log 2>/dev/null; then
        echo "✓ Server is listening on wtransport://127.0.0.1:${SERVER_PORT}"
    fi
else
    echo -e "${RED}✗ Server failed to start${NC}"
    echo ""
    echo "Check log file: /tmp/unity-ffi-server.log"
    cat /tmp/unity-ffi-server.log
    exit 1
fi

echo ""
echo "====================================="
echo "Setup Complete!"
echo "====================================="
echo ""
echo "Summary:"
echo "  • Built project: ${GREEN}✓${NC} (${BUILD_ARCH})"
echo "  • Killed existing server: ${GREEN}✓${NC}"
echo "  • Started new server: ${GREEN}✓${NC} (PID: ${SERVER_PID})"
echo "  • Copied library: ${GREEN}✓${NC} (${UNITY_MACOS_DIR}/libunity_network.dylib, ${BUILD_ARCH})"
echo "  • Copied scripts: ${GREEN}✓${NC} (${SCRIPTS_COPIED} files to ${UNITY_ASSETS_SCRIPTS_DIR}/)"
echo ""
echo "Server Status:"
echo "  • PID: ${SERVER_PID}"
echo "  • Port: ${SERVER_PORT}"
echo "  • Log: /tmp/unity-ffi-server.log"
echo ""
echo "Next Steps:"
echo "  1. Open Unity project: ${TARGET_DIR}"
echo "  2. Unity should auto-detect plugin and scripts"
echo "  3. If not, go to Assets → Refresh"
echo "  4. Create a GameObject and add NetworkPlayer component"
echo "  5. Configure server URL (default: https://127.0.0.1:${SERVER_PORT})"
echo "  6. Press Play in Unity Editor"
echo ""
echo "Managing Server:"
echo "  • View logs: tail -f /tmp/unity-ffi-server.log"
echo "  • Stop server: kill ${SERVER_PID}"
echo "  • Restart server: ./setup.sh ${TARGET_DIR}"
echo ""
echo "Building for Different Architectures:"
echo "  • Native (ARM64): ./setup.sh ${TARGET_DIR} arm64"
echo "  • Rosetta (x86_64): ./setup.sh ${TARGET_DIR} x86_64"
echo ""
echo "Troubleshooting:"
echo "  • If plugin not detected, check Unity Console for errors"
echo "  • Ensure that plugin is at Assets/Plugins/macOS/"
echo "  • Verify file permissions: chmod +x Assets/Plugins/macOS/*.dylib"
echo "  • Restart Unity Editor after copying files"
echo "  • Check server logs for connection errors"
echo "  • IMPORTANT: Enable \"Allow 'unsafe' Code\" in Unity Project Settings:"
echo "    - Edit → Project Settings → Player"
echo "    - Scroll down to Other Settings"
echo "    - Set \"Allow 'unsafe' Code\" to ON"
echo "  • See docs/UNITY_SETUP_GUIDE.md for detailed setup instructions"
echo ""
