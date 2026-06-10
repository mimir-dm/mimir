# AI Assistant Integration

Use Mimir with Claude Code for AI-assisted campaign management via natural language.

## What It Does

Mimir includes an MCP (Model Context Protocol) server that exposes 54 tools to Claude Code, enabling you to manage campaigns, search catalogs, generate maps, and prep sessions through conversation. Ask "create a forest map" or "find CR 5 undead monsters" and the assistant handles it.

## Prerequisites

- Mimir desktop app installed
- [Claude Code](https://docs.anthropic.com/en/docs/claude-code) installed
- The `mimir-mcp` binary (installed alongside the app, or built from source)

## Setup

### For App Users

Inside the desktop app, the MCP server runs as a sidecar — a companion process bundled with the app that launches automatically. For Claude Code, you use the standalone `mimir-mcp` binary instead. On macOS, the install script places it at `~/.local/bin/mimir-mcp`, so it's already on your PATH (if not, add `~/.local/bin` to your PATH). On Linux and Windows, download the binary for your platform from [GitHub Releases](https://github.com/mimir-dm/mimir/releases) (e.g. `mimir-mcp-x86_64-unknown-linux-gnu` or `mimir-mcp-x86_64-pc-windows-msvc.exe`), or build it from source as described below.

1. Register the server with Claude Code:

   ```bash
   claude mcp add mimir -- mimir-mcp
   ```

2. Install the Claude Code plugin. Inside a Claude Code session, add the Mimir marketplace, then install the plugin:

   ```
   /plugin marketplace add mimir-dm/mimir
   /plugin install mimir-dm@mimir
   ```

### From Source

If you're working from the Mimir repository, build the binary yourself:

```bash
cargo build --release -p mimir-mcp
```

Then register the built binary (`target/release/mimir-mcp`) with `claude mcp add` as above.

### Database Path

The server auto-detects the Mimir database on macOS (`~/Library/Application Support/com.mimir.app/data/mimir.db`) and Linux (`$XDG_DATA_HOME/com.mimir.app/data/mimir.db`, falling back to `~/.local/share/...`). On Windows there is no default — you must set the environment variable:

```bash
# macOS / Linux
export MIMIR_DATABASE_PATH=/path/to/mimir.db
```

```powershell
# Windows (PowerShell)
$env:MIMIR_DATABASE_PATH = "C:\Users\you\AppData\Roaming\com.mimir.app\data\mimir.db"
```

You can find your database path in the app under **Settings > Integrations**, which displays it with a copy button.

See [Environment Variables](../../reference/environment-variables.md) for details.

## Available Tools, Commands, and Skills

The full listing of MCP tools, plugin slash commands, and skills lives in the [MCP Server Reference](../../reference/mcp-server.md). In brief, tools cover campaign, module, document, character, and map management, map generation, homebrew content, and catalog search.

## Example Workflows

**Campaign setup:**
> "Create a new campaign called 'Curse of Strahd' and add a module for Death House"

**Monster search:**
> "Find all CR 3-5 undead monsters from the Monster Manual"

**Map generation:**
> "Generate a dark swamp map with standing water and dim lighting"

**Session prep:**
> "Run a session prep check on the Goblin Hideout module"

**Character management:**
> "Add a +1 longsword to Aldric's inventory"

## Important Notes

- MCP tools modify the database directly — there is no undo
- [Export your campaign](../campaigns/export-campaign.md) regularly as a backup
- The assistant is the DM's tool — it prompts for choices rather than making creative decisions autonomously
- Set the active campaign with `set_active_campaign` before using campaign-specific tools
