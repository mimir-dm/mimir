# AI Assistant Integration

Use Mimir with Claude Code for AI-assisted campaign management via natural language.

## What It Does

The Mimir server has an MCP (Model Context Protocol) endpoint with 71 tools for Claude Code, enabling you to manage campaigns, search catalogs, generate maps, and prep sessions through conversation. Ask "create a forest map" or "find CR 5 undead monsters" and the assistant handles it.

## Prerequisites

- A running Mimir server (on your computer: `http://127.0.0.1:8740`)
- [Claude Code](https://docs.anthropic.com/en/docs/claude-code) installed
- The DM token (`MIMIR_API_TOKEN`), if the server has one

## Setup

1. Tell Claude Code where the server is:

   ```bash
   export MIMIR_URL="http://127.0.0.1:8740"   # or the server's address
   export MIMIR_API_TOKEN="…"                 # the DM token; empty in open mode
   ```

2. Install the Claude Code plugin. Inside a Claude Code session, add the
   Mimir marketplace, then install the plugin:

   ```
   /plugin marketplace add mimir-dm/mimir
   /plugin install mimir-dm@mimir
   ```

   Or add only the server:

   ```bash
   claude mcp add --transport http mimir "$MIMIR_URL/mcp" \
     --header "Authorization: Bearer $MIMIR_API_TOKEN"
   ```

Mimir's **Settings > Integrations** page shows this command with the
server's address.

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
