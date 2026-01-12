#!/bin/bash
# Install Mimir MCP server for Claude Desktop (standalone app) on Linux
# NOTE: For Claude Code (CLI), use the plugin's .mcp.json instead - no script needed.

set -e

# Default paths - can be overridden via arguments or environment variables
DEFAULT_BIN_LOCATIONS=(
    "./target/release/mimir-mcp"
    "../../../target/release/mimir-mcp"
    "$HOME/.cargo/bin/mimir-mcp"
    "/usr/local/bin/mimir-mcp"
)
DEFAULT_DB_PATH="$HOME/.local/share/mimir/mimir.db"

# Use arguments, environment variables, or defaults
MIMIR_MCP_BIN="${1:-${MIMIR_MCP_BIN:-}}"
MIMIR_DB="${2:-${MIMIR_DATABASE_PATH:-$DEFAULT_DB_PATH}}"
CLAUDE_CONFIG_DIR="$HOME/.config/Claude"
CLAUDE_CONFIG="$CLAUDE_CONFIG_DIR/claude_desktop_config.json"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo "Mimir MCP Installer for Claude Desktop"
echo "======================================="
echo ""

# Find mimir-mcp binary if not specified
if [ -z "$MIMIR_MCP_BIN" ]; then
    for location in "${DEFAULT_BIN_LOCATIONS[@]}"; do
        if [ -f "$location" ]; then
            MIMIR_MCP_BIN="$location"
            break
        fi
    done
fi

# Check if mimir-mcp binary exists
if [ -z "$MIMIR_MCP_BIN" ] || [ ! -f "$MIMIR_MCP_BIN" ]; then
    echo -e "${RED}Error: mimir-mcp binary not found${NC}"
    echo ""
    echo "Searched locations:"
    for location in "${DEFAULT_BIN_LOCATIONS[@]}"; do
        echo "  - $location"
    done
    echo ""
    echo "Build it first with: cargo build --release -p mimir-dm-mcp"
    echo "Or specify the path: $0 /path/to/mimir-mcp"
    exit 1
fi

# Convert to absolute path
MIMIR_MCP_BIN="$(cd "$(dirname "$MIMIR_MCP_BIN")" && pwd)/$(basename "$MIMIR_MCP_BIN")"

# Check if database exists
if [ ! -f "$MIMIR_DB" ]; then
    echo -e "${YELLOW}Warning: Mimir database not found at: $MIMIR_DB${NC}"
    echo "The MCP server will fail until a database exists."
    echo ""
fi

# Create Claude config directory if needed
mkdir -p "$CLAUDE_CONFIG_DIR"

# Generate the config snippet
CONFIG_SNIPPET=$(cat <<EOF
{
  "mcpServers": {
    "mimir": {
      "command": "$MIMIR_MCP_BIN",
      "args": [],
      "env": {
        "MIMIR_DATABASE_PATH": "$MIMIR_DB"
      }
    }
  }
}
EOF
)

# Check if config already exists
if [ -f "$CLAUDE_CONFIG" ]; then
    echo -e "${YELLOW}Existing Claude config found at: $CLAUDE_CONFIG${NC}"
    echo ""
    echo "Add this to your mcpServers section:"
    echo ""
    echo "    \"mimir\": {"
    echo "      \"command\": \"$MIMIR_MCP_BIN\","
    echo "      \"args\": [],"
    echo "      \"env\": {"
    echo "        \"MIMIR_DATABASE_PATH\": \"$MIMIR_DB\""
    echo "      }"
    echo "    }"
    echo ""
    echo -e "${YELLOW}Or backup and replace with:${NC}"
    echo "  cp \"$CLAUDE_CONFIG\" \"$CLAUDE_CONFIG.backup\""
    echo ""
else
    # Write new config
    echo "$CONFIG_SNIPPET" > "$CLAUDE_CONFIG"
    echo -e "${GREEN}Created Claude config at: $CLAUDE_CONFIG${NC}"
fi

echo ""
echo "Configuration:"
echo "  Binary: $MIMIR_MCP_BIN"
echo "  Database: $MIMIR_DB"
echo ""
echo -e "${GREEN}Done! Restart Claude Desktop to load the MCP server.${NC}"
