#!/bin/bash
# Install Mimir MCP server for Claude Desktop on macOS

set -e

# Paths
MIMIR_MCP_BIN="${1:-$HOME/Desktop/colliery/mimir/target/release/mimir-mcp}"
MIMIR_DB="${2:-$HOME/Library/Application Support/com.mimir.mimir/mimir.db}"
CLAUDE_CONFIG_DIR="$HOME/Library/Application Support/Claude"
CLAUDE_CONFIG="$CLAUDE_CONFIG_DIR/claude_desktop_config.json"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo "Mimir MCP Installer for Claude Desktop"
echo "======================================="
echo ""

# Check if mimir-mcp binary exists
if [ ! -f "$MIMIR_MCP_BIN" ]; then
    echo -e "${RED}Error: mimir-mcp binary not found at: $MIMIR_MCP_BIN${NC}"
    echo "Build it first with: cargo build --release -p mimir-dm-mcp"
    exit 1
fi

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
