# Changelog

## 1.0.0

Mimir is a web app: one server, `mimir-server`, serves the app to the browser, its API, a live socket and an MCP endpoint. The desktop app (Tauri and Vue) is gone.

- **Web app.** Campaigns, modules, documents with a rich editor, characters (creation, sheet, inventory, level-up), homebrew, the catalog and the book reader, settings, imports and PDF export, in the browser or on a tablet.
- **At the table.** The DM map (fog, line of sight, lights, doors, traps, points of interest, tokens), the initiative tracker, and a player display that follows the DM live.
- **Players.** A link for each character: the player display, their own sheet, inventory, level-up and notes, and the sheets the party shares.
- **Claude.** MCP over HTTP at `/mcp`.
- **Adventures.** Read official and homebrew 5etools adventures; turn one into a campaign; export a campaign as a 5etools homebrew adventure, as a Mimir adventure pack, or as one printed book. Documents show read-aloud boxes, sidebars and stat blocks.
- **Deploy.** A Docker image (`ghcr.io/mimir-dm/mimir`, amd64 and arm64) and a Compose stack with a Tailscale sidecar and scheduled backups.

Fixed on the way: current 5etools data archives brought no book text; a document with an image could not export to PDF; a campaign archive left out module map files.
