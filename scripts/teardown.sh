#!/bin/bash
set -e

echo "====================================="
echo "Unity FFI Teardown Script"
echo "====================================="

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Configuration
SERVER_PORT=4433
SERVER_PID_FILE="/tmp/unity-ffi-server.pid"

echo "Stopping server on port ${SERVER_PORT}..."
echo ""

# Step 1: Find server process by port
echo "Step 1: Finding server process..."
SERVER_PID=$(lsof -ti:${SERVER_PORT} 2>/dev/null || true)

if [ -z "$SERVER_PID" ]; then
    echo -e "${GREEN}✓ No server found running on port ${SERVER_PORT}${NC}"
else
    echo -e "${YELLOW}Found server process (PID: ${SERVER_PID})${NC}"

    # Step 2: Kill the server
    echo "Stopping server..."
    kill -9 $SERVER_PID 2>/dev/null || true

    # Wait a moment for process to terminate
    sleep 1

    # Verify it's stopped
    SERVER_PID=$(lsof -ti:${SERVER_PORT} 2>/dev/null || true)
    if [ -z "$SERVER_PID" ]; then
        echo -e "${GREEN}✓ Server stopped successfully${NC}"
    else
        echo -e "${RED}✗ Failed to stop server (PID: ${SERVER_PID})${NC}"
        exit 1
    fi
fi

echo ""

# Step 3: Clean up PID file if it exists
if [ -f "$SERVER_PID_FILE" ]; then
    echo "Step 2: Cleaning up PID file..."
    rm -f "$SERVER_PID_FILE"
    echo -e "${GREEN}✓ Removed PID file${NC}"
    echo ""
fi

# Step 4: Clean up log file (optional, uncomment if desired)
# if [ -f "/tmp/unity-ffi-server.log" ]; then
#     echo "Step 3: Cleaning up log file..."
#     rm -f /tmp/unity-ffi-server.log
#     echo -e "${GREEN}✓ Removed log file${NC}"
#     echo ""
# fi

echo "====================================="
echo "Teardown Complete!"
echo "====================================="
echo ""
echo "Summary:"
echo "  • Server stopped: ${GREEN}✓${NC}"
echo "  • Port ${SERVER_PORT} is now free"
echo ""
echo "Note: Log file is preserved at /tmp/unity-ffi-server.log"
echo "      You can view it with: cat /tmp/unity-ffi-server.log"
echo ""
