# Settings

The Settings screen provides application-wide configuration. Access it via the gear icon in the header bar.

The sidebar is divided into three groups: **Admin Tools**, **Application**, and (in development builds only) **Developer**.

## Admin Tools

### Manage Campaigns

Opens the Campaign Management modal for archiving, restoring, and deleting campaigns.

### Import Books

Opens the **Manage Catalog Sources** modal for the D&D 5e source data that powers catalog searches:

| Action | Description |
|--------|-------------|
| **Import 5etools Data** | Import a source data archive to populate the catalog |
| **Import Images** | Import an image archive to add token art, cover art, and book illustrations |
| **Enable/Disable** | Toggle individual source books on or off globally |
| **Remove** | Delete a source book and all its catalog entries from the database |

See [Manage Campaign Sources](../../how-to/campaigns/manage-sources.md) for the import workflow.

## Application

### Theme

Choose between three visual themes:

| Theme | Description |
|-------|-------------|
| **Light** | Light background, dark text |
| **Dark** | Dark background, light text |
| **Hyper** | High-contrast dark theme with vivid accent colors |

Theme changes apply to all open windows (main, DM map, player display, sources).

### Integrations

Configuration for connecting Claude Code or Claude Desktop to Mimir via the MCP server. Displays:

| Element | Description |
|---------|-------------|
| **Database Path** | Location of your Mimir database, with a copy button. Used as the `MIMIR_DATABASE_PATH` value in MCP configuration |
| **Claude Code (CLI)** | A ready-to-copy `claude mcp add mimir` command that registers the `mimir-mcp` server with your database path |
| **Claude Desktop** | A ready-to-copy JSON `mcpServers` snippet for `claude_desktop_config.json` |

Claude starts the MCP server automatically; no manual server management is required.

### About

| Field | Description |
|-------|-------------|
| **Version** | Current Mimir version |

## Developer

Only visible in development builds.

### Dev Tools

| Element | Description |
|---------|-------------|
| **Status** | Whether test data is currently present (`Test Data Present` / `Not Seeded`) |
| **Seed Test Data** | Creates the "Lost Mine of Phandelver" test campaign with modules, characters, monsters, and NPCs |
| **Reseed (Clear & Recreate)** | Deletes the test campaign and seeds it fresh |
| **Clear Test Data** | Deletes the test campaign without reseeding |

All dev commands are rejected by the backend outside debug builds.

## See Also

- [Home Screen](./home-screen.md) — Navigation overview
- [MCP Server](../mcp-server.md) — The integration the Integrations section configures
- [Environment Variables](../environment-variables.md) — Advanced configuration
