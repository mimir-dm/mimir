#!/bin/bash
# Download ALL 5etools JSON schemas from brew-fast
# Source: https://github.com/TheGiddyLimit/5etools-utils/tree/master/schema/brew-fast
#
# These schemas are pre-processed with $$merge resolved, suitable for typify.
# Run this script to refresh vendored schemas.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
OUTPUT_DIR="$SCRIPT_DIR/5etools"
TEMP_DIR=$(mktemp -d)

# Pin to master branch (or specify a commit hash for reproducibility)
COMMIT="master"
REPO_URL="https://github.com/TheGiddyLimit/5etools-utils.git"

echo "Downloading 5etools schemas from brew-fast..."
echo "Commit: $COMMIT"
echo "Output: $OUTPUT_DIR"
echo ""

# Clean up temp dir on exit
cleanup() {
    rm -rf "$TEMP_DIR"
}
trap cleanup EXIT

# Shallow clone just the branch we need
echo "Cloning repository (shallow)..."
git clone --depth 1 --branch "$COMMIT" "$REPO_URL" "$TEMP_DIR" 2>/dev/null || \
    git clone --depth 1 "$REPO_URL" "$TEMP_DIR"

# Remove old schemas and copy new ones
echo "Copying brew-fast schemas..."
rm -rf "$OUTPUT_DIR"
cp -r "$TEMP_DIR/schema/brew-fast" "$OUTPUT_DIR"

# Count what we got
TOTAL=$(find "$OUTPUT_DIR" -name "*.json" | wc -l | tr -d ' ')
echo ""
echo "Done! Downloaded $TOTAL schemas to $OUTPUT_DIR"
echo ""
echo "Directory structure:"
find "$OUTPUT_DIR" -type d | sed "s|$OUTPUT_DIR|.|" | sort
