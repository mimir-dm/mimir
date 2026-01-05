# Mimir MCP Plugin for Claude Code

This plugin enables Claude to author D&D 5e campaigns in Mimir using the Model Context Protocol (MCP).

## Features

- Campaign and module management
- Document authoring (session notes, module overviews, etc.)
- NPC creation and assignment
- Monster, item, and trap catalog search
- Character inventory and currency management

## Installation

### Prerequisites

1. Build the mimir-mcp binary:
   ```bash
   cargo build --release -p mimir-dm-mcp
   ```

2. Locate the binary at `target/release/mimir-mcp`

3. Locate your Mimir database:
   - **macOS**: `~/Library/Application Support/com.mimir.mimir/mimir.db`
   - **Linux**: `~/.local/share/mimir/mimir.db`
   - **Windows**: `%APPDATA%\mimir\mimir.db`

### Claude Desktop

1. Open your Claude Desktop config file:
   - **macOS**: `~/Library/Application Support/Claude/claude_desktop_config.json`
   - **Windows**: `%APPDATA%\Claude\claude_desktop_config.json`

2. Add the mimir server configuration:

   ```json
   {
     "mcpServers": {
       "mimir": {
         "command": "/absolute/path/to/mimir-mcp",
         "args": [],
         "env": {
           "MIMIR_DATABASE_PATH": "/absolute/path/to/mimir.db"
         }
       }
     }
   }
   ```

3. Replace the paths with your actual paths

4. Restart Claude Desktop

**macOS Quick Install:**
```bash
./crates/mimir-dm-mcp/plugin/install-macos.sh
```

### Claude Code (CLI)

#### Option 1: Add MCP server directly

```bash
claude mcp add mimir -- mimir-mcp --env MIMIR_DATABASE_PATH=/path/to/mimir.db
```

#### Option 2: Project-level configuration

Add to your project's `.mcp.json`:

```json
{
  "mimir": {
    "command": "mimir-mcp",
    "args": [],
    "env": {
      "MIMIR_DATABASE_PATH": "/path/to/mimir.db"
    }
  }
}
```

#### Option 3: Install as a plugin

1. Copy this plugin directory to `~/.claude/plugins/mimir-campaign/`
2. Set the `MIMIR_DATABASE_PATH` environment variable
3. Restart Claude Code

## Plugin Structure

```
plugin/
├── plugin.json           # Plugin metadata
├── .mcp.json             # MCP server configuration
├── README.md             # This file
├── install-macos.sh      # macOS installer
└── skills/
    └── mimir-campaign/
        └── Skill.md      # Skill instructions for Claude
```

## Usage

Once configured, Claude can use Mimir tools. Start by asking:

> "List my Mimir campaigns"

or

> "Help me create a new D&D module called 'The Haunted Manor'"

The skill activates automatically when Claude detects relevant context.

## Available Tools

| Category | Tools |
|----------|-------|
| Campaign | `list_campaigns`, `set_active_campaign` |
| Module | `create_module`, `list_modules`, `get_module_details`, `add_monster_to_module`, `add_item_to_module` |
| Document | `list_documents`, `read_document`, `edit_document` |
| Character | `list_characters`, `get_character`, `create_npc`, `assign_npc_to_module`, `add_item_to_character`, `update_character_currency` |
| Catalog | `search_monsters`, `search_items`, `search_traps` |

## Troubleshooting

### "No active campaign" error
Call `set_active_campaign` with a valid campaign ID first. Use `list_campaigns` to see available campaigns.

### Server not connecting
1. Verify the mimir-mcp binary path is correct and executable
2. Check the database path exists
3. Run `/mcp` in Claude Code to see server status

### Database locked errors
Ensure the Mimir desktop app isn't running, or that WAL mode is enabled (default).

## Development

To run the MCP server manually for testing:

```bash
MIMIR_DATABASE_PATH=/path/to/mimir.db ./target/release/mimir-mcp
```

The server communicates over stdio using the MCP protocol.

## Sources

- [Claude Code MCP Documentation](https://code.claude.com/docs/en/mcp)
- [Creating Custom Skills](https://support.claude.com/en/articles/12512198)
- [Skills and MCP Overview](https://claude.com/blog/extending-claude-capabilities-with-skills-mcp-servers)
