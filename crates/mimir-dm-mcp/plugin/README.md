# Mimir DM Plugin for Claude Code

A distributable Claude Code plugin for D&D 5e campaign authoring with Mimir.

## Features

- Campaign and module management
- Document authoring (session notes, module overviews, etc.)
- Full character creation (both PCs and NPCs) and editing
- Monster, item, and trap catalog search
- Character inventory and currency management

## Installation

### From Marketplace (Recommended)

```bash
claude plugins add mimir-dm
```

### From GitHub

```bash
claude plugins add github:mimir-dm/mimir/crates/mimir-dm-mcp/plugin
```

### Manual Installation

1. Clone or copy this plugin directory to `~/.claude/plugins/mimir-dm/`
2. Set up the MCP server (see below)

## MCP Server Setup

The plugin requires the `mimir-mcp` binary and a Mimir database.

### Prerequisites

1. Build the mimir-mcp binary:
   ```bash
   cargo build --release -p mimir-dm-mcp
   ```

2. Set the `MIMIR_DATABASE_PATH` environment variable:
   - **macOS**: `~/Library/Application Support/com.mimir.mimir/mimir.db`
   - **Linux**: `~/.local/share/mimir/mimir.db`
   - **Windows**: `%APPDATA%\com.mimir.mimir\mimir.db`

### Quick Install Scripts (Claude Desktop)

The scripts in `scripts/` configure **Claude Desktop** (the standalone app), not Claude Code (the CLI).

```bash
# macOS
./scripts/install-macos.sh

# Linux
./scripts/install-linux.sh

# Windows (PowerShell)
.\scripts\install-windows.ps1
```

**For Claude Code**: The `.mcp.json` file in this plugin directory is automatically used when the plugin is installed. No additional configuration needed.

## Plugin Structure

```
mimir-dm/
├── .claude-plugin/
│   └── plugin.json           # Plugin manifest
├── .mcp.json                  # MCP server configuration
├── commands/                  # Slash commands
│   ├── mimir-campaigns.md    # /mimir-campaigns
│   ├── create-module.md      # /create-module
│   └── search-monsters.md    # /search-monsters
├── skills/
│   └── mimir-dm/
│       ├── SKILL.md          # Main skill definition
│       ├── references/       # Detailed tool documentation
│       └── examples/         # Workflow examples
├── scripts/                  # Installation scripts
└── README.md
```

## Usage

### Slash Commands

| Command | Description |
|---------|-------------|
| `/mimir-campaigns` | List all campaigns |
| `/create-module <name> [type]` | Create a new module |
| `/search-monsters [query]` | Search the monster catalog |

### Natural Language

The skill activates automatically. Try:

> "List my Mimir campaigns"

> "Create a dungeon module called 'The Sunken Crypt'"

> "Search for undead monsters with CR 5 or less"

> "Add a goblin to the current module"

## Available MCP Tools (21 total)

| Category | Tools |
|----------|-------|
| Campaign | `list_campaigns`, `set_active_campaign` |
| Module | `create_module`, `list_modules`, `get_module_details`, `add_monster_to_module`, `add_item_to_module` |
| Document | `list_documents`, `read_document`, `edit_document`, `create_user_document` |
| Character | `list_characters`, `get_character`, `create_character`, `edit_character`, `assign_npc_to_module`, `add_item_to_character`, `update_character_currency` |
| Catalog | `search_monsters`, `search_items`, `search_traps` |

## Troubleshooting

### "No active campaign" error
Call `set_active_campaign` with a valid campaign ID first. Use `list_campaigns` to see available campaigns.

### Server not connecting
1. Verify the mimir-mcp binary is in your PATH or specify full path
2. Check that `MIMIR_DATABASE_PATH` is set correctly
3. Run `/mcp` in Claude Code to see server status

### Database locked errors
Ensure the Mimir desktop app isn't running, or that WAL mode is enabled (default).

## Development

To run the MCP server manually for testing:

```bash
MIMIR_DATABASE_PATH=/path/to/mimir.db mimir-mcp
```

The server communicates over stdio using the MCP protocol.

## License

MIT

## Links

- [Mimir Repository](https://github.com/mimir-dm/mimir)
- [Claude Code Plugins Documentation](https://docs.anthropic.com/en/docs/claude-code/plugins)
