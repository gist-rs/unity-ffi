#!/bin/sh
# Generate Unity C# bindings from Rust FFI types
# This script runs the generation example and saves the output to Unity project
# Usage: ./scripts/generate_bindings.sh or cd scripts && ./generate_bindings.sh

# Ensure script is executable
# chmod +x scripts/generate_bindings.sh

set -e  # Exit on error

echo "=== Generating Unity C# Bindings ==="
echo ""

# Get the script directory
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"

# Navigate to project root (scripts folder is one level down)
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$PROJECT_ROOT"

echo "Working directory: $PROJECT_ROOT"
echo ""

# Run the generation example and save to Unity project
echo "Running cargo run --package unity-network --example generate_unity_cs..."
cargo run --package unity-network --example generate_unity_cs \
    --quiet \
    > unity/Generated/GameFFI.cs

# Check if file was created
if [ -f "unity/Generated/GameFFI.cs" ]; then
    FILE_SIZE=$(wc -c < "unity/Generated/GameFFI.cs")
    echo "✓ Successfully generated unity/Generated/GameFFI.cs"
    echo "  File size: $FILE_SIZE bytes"
    echo ""

    # Count number of structs generated
    STRUCT_COUNT=$(grep -c "public struct" "unity/Generated/GameFFI.cs" || echo "0")
    echo "✓ Generated $STRUCT_COUNT struct(s)"

    # Count number of enums generated
    ENUM_COUNT=$(grep -c "public enum" "unity/Generated/GameFFI.cs" || echo "0")
    echo "✓ Generated $ENUM_COUNT enum(s)"

    echo ""
    echo "Next steps:"
    echo "  1. Unity will automatically detect the file change"
    echo "  2. If not, refresh Unity project: Assets > Refresh"
    echo "  3. Verify generated bindings match Rust types"
    echo ""
    echo "⚠️  Remember: DO NOT EDIT GameFFI.cs manually!"
    echo "   Regenerate with: ./scripts/generate_bindings.sh"
else
    echo "✗ Failed to generate GameFFI.cs"
    echo "  File not found at unity/Generated/GameFFI.cs"
    exit 1
fi
