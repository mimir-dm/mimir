# Changelog

## 1.0.1

- **Module NPCs.** A module's NPCs tab can add, edit and remove NPCs: their part in that module (role, description, appearance, personality, motivation, secrets), by hand or starting from a campaign NPC. Over MCP: `add_npc_to_module`, `update_module_npc`, `remove_npc_from_module`; `get_module_details` lists the NPCs (MIMIR-T-0750).
- The MCP tools `edit_character` and `list_characters` no longer take a `module_id` that they ignored.

## 1.0.0

Mimir is a web app: one server, `mimir-server`, serves the app to the browser, its API, a live socket and an MCP endpoint. The desktop app (Tauri and Vue) is gone.

- **Web app.** Campaigns, modules, documents with a rich editor, characters (creation, sheet, inventory, level-up), homebrew, the catalog and the book reader, settings, imports and PDF export, in the browser or on a tablet.
- **At the table.** The DM map (fog, line of sight, lights, doors, traps, points of interest, tokens), the initiative tracker, and a player display that follows the DM live.
- **Players.** A link for each character: the player display, their own sheet, inventory, level-up and notes, and the sheets the party shares.
- **Claude.** MCP over HTTP at `/mcp`, and a Claude Code plugin that connects to it. Tools take no file paths: a file goes in through an upload id and comes out as a one-time download link (MIMIR-A-0012).
- **Adventures.** Read official and homebrew 5etools adventures; turn one into a campaign; export a campaign as a 5etools homebrew adventure, as a Mimir adventure pack, or as one printed book. Documents show read-aloud boxes, sidebars and stat blocks.
- **Deploy.** A Docker image (`ghcr.io/mimir-dm/mimir`, amd64 and arm64) and a Compose stack with a Tailscale sidecar and scheduled backups.
- **Docs.** The documentation site is Astro Starlight.
- **Build.** Rust 1.100.0, pinned in `rust-toolchain.toml` for local builds, CI, the release and the image.

Fixed on the way: current 5etools data archives brought no book text; a document with an image could not export to PDF; a campaign archive left out module map files.
