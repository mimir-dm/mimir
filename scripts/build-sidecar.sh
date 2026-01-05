#!/bin/bash
# Build the mimir-mcp sidecar binary for Tauri bundling
# This script builds mimir-mcp and copies it to the binaries directory
# with the target-triple suffix that Tauri expects.

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
BINARIES_DIR="$PROJECT_ROOT/crates/mimir-dm/binaries"

# Detect target triple
if [ -n "$TAURI_ENV_TARGET_TRIPLE" ]; then
    TARGET="$TAURI_ENV_TARGET_TRIPLE"
elif [ -n "$1" ]; then
    TARGET="$1"
else
    # Auto-detect based on platform
    ARCH=$(uname -m)
    OS=$(uname -s)

    case "$OS" in
        Darwin)
            case "$ARCH" in
                arm64) TARGET="aarch64-apple-darwin" ;;
                x86_64) TARGET="x86_64-apple-darwin" ;;
                *) echo "Unsupported architecture: $ARCH"; exit 1 ;;
            esac
            ;;
        Linux)
            case "$ARCH" in
                x86_64) TARGET="x86_64-unknown-linux-gnu" ;;
                aarch64) TARGET="aarch64-unknown-linux-gnu" ;;
                *) echo "Unsupported architecture: $ARCH"; exit 1 ;;
            esac
            ;;
        *)
            echo "Unsupported OS: $OS"
            exit 1
            ;;
    esac
fi

echo "Building mimir-mcp for target: $TARGET"

# Build mimir-mcp in release mode
cd "$PROJECT_ROOT"
cargo build --release -p mimir-dm-mcp --target "$TARGET"

# Ensure binaries directory exists
mkdir -p "$BINARIES_DIR"

# Copy binary with target suffix
SOURCE="$PROJECT_ROOT/target/$TARGET/release/mimir-mcp"
DEST="$BINARIES_DIR/mimir-mcp-$TARGET"

if [ -f "$SOURCE" ]; then
    cp "$SOURCE" "$DEST"
    chmod +x "$DEST"
    echo "Copied: $DEST"
else
    echo "Error: Binary not found at $SOURCE"
    exit 1
fi

echo "Sidecar build complete!"
