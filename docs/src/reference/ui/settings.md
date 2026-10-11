# Settings

The settings page (`/settings`) has four tabs. The tab is in the address (`?section=campaigns`, `books`, `integrations`, `about`).

## Campaigns

### Import a campaign

| Control | Function |
|---------|----------|
| **Archive** | Drop or choose a campaign archive (`.mimir-campaign.tar.gz`) made by **Export**. A preview shows what it holds; nothing is made yet. |
| **Name of the new campaign** | The name of the new campaign. It starts as the archive's. |
| **Import** | Import. The message lists the modules, documents, characters and maps that came in. |

An import always makes a new campaign. **Import campaign** on the campaign list opens the same import. Every upload on this page takes a dropped file too.

### Campaigns table

| Column | Contents |
|--------|----------|
| **Campaign** | The name |
| **State** | Active or Archived |
| **Export** | Download the campaign archive |
| **Archive** / **Restore** | Archive or restore the campaign |
| **Delete…** | Delete an archived campaign, after you type its name |

## Books

### Import 5etools data

The 5etools data archive (`.tar.gz`): the core and supplement books, their entries and their text. Choose the file and click **Import**. The upload shows its progress, then the import runs on the server. A full import takes a few minutes.

### Import 5etools images

The 5etools image archive (`5etools-img-….tar.gz`): pictures for the books, monsters and items. Choose the file and click **Import**.

### Sources table

| Column | Contents |
|--------|----------|
| **Code** | The source code, for example PHB |
| **Book** | The name of the book |
| **Readable** | Yes when the book text can be read in the reference |
| **×** | Remove the source, after a confirmation. Its catalog entries and book text go. Characters and campaigns keep what they hold. |

## Integrations

How to connect Claude to Mimir over MCP.

| Element | Contents |
|---------|----------|
| **Endpoint** | The MCP address of this server: `<server>/mcp` |
| Claude Code | A `claude mcp add --transport http mimir …/mcp` command, with an `Authorization: Bearer <your DM token>` header |
| Plugin | How to install the Claude Code plugin; set `MIMIR_URL` to the server address and `MIMIR_API_TOKEN` to the DM token first |

Only the DM token works at `/mcp`. A player link is refused. See [MCP Server](../mcp-server.md).

## About

| Field | Contents |
|-------|----------|
| **Mimir** | The version |
| **Sign-in** | Token or Open |

## See Also

- [Manage Source Books](../../how-to/campaigns/manage-sources.md)
- [Export a Campaign](../../how-to/campaigns/export-campaign.md)
- [MCP Server](../mcp-server.md)
- [Environment Variables](../environment-variables.md)
