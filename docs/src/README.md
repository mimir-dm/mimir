# Mimir - D&D Campaign Assistant

> *"Remember, I know everything that can be known in the multiverse."* - Mimir, Planescape

Mimir is a D&D 5e campaign manager for Dungeon Masters. It runs as a web app on your own server. You prepare campaigns in the browser, run sessions from the DM map, and show the map to your players on a TV or on their own devices.

## Quick Start

**New to Mimir?** Start with the tutorials:

1. **[Your First Campaign](./tutorials/01-first-campaign.md)**: sign in, make a campaign, import the catalog
2. **[Your First Module](./tutorials/02-first-module.md)**: add a module, a map and monsters
3. **[Running Your First Session](./tutorials/03-first-session.md)**: place tokens, use fog of war and the initiative tracker
4. **[The Player Display](./tutorials/04-player-display.md)**: show the map on a TV and give players their links
5. **[Creating Homebrew Content](./tutorials/05-homebrew-content.md)**: make your own items, monsters and spells

## Features

### Campaign Management
Organize a campaign in modules. Each campaign and module has its own documents, NPCs and maps. A rich editor writes markdown and saves as you type.

### Interactive Maps
Upload UVTT files or images. Place tokens, lights, traps and points of interest. Fog of war follows what the player characters can see.

### Running Sessions
Run combat with the initiative tracker beside the map. Show the map to players on the player display, with fog of war and hidden tokens kept from them.

### Players
Give each player a link to their character. A player sees the map, edits their own sheet, writes notes and shares their sheet with the party.

### D&D 5e Reference
Search monsters, spells, items and more from the source books you import. Read the books with their cross-references.

### PDF Export
Print character sheets, spell cards, monster cards, documents and maps with token cutouts.

## Documentation

This documentation uses the [Diataxis](https://diataxis.fr/) framework:

| Section | Purpose |
|---------|---------|
| **[Tutorials](./tutorials/README.md)** | Step-by-step lessons |
| **[How-To Guides](./how-to/README.md)** | Instructions for one task |
| **[Reference](./reference/README.md)** | Screens, data and settings |
| **[Understanding Mimir](./explanation/README.md)** | Concepts and design |

## Running Mimir

Mimir is one server program, `mimir-server`. It serves the web app, its API and an MCP endpoint for Claude. All data stays in one data directory on the server. See [Run the Server](./how-to/run-server.md).

## Getting Help

- **Issues**: Report bugs on [GitHub Issues](https://github.com/mimir-dm/mimir/issues)
- **Discussions**: Ask questions on [GitHub Discussions](https://github.com/mimir-dm/mimir/discussions)

---

*Mimir is open source software. See the [Developer Documentation](./developer/README.md) if you want to contribute.*
