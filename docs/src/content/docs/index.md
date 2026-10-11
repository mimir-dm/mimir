---
title: "Mimir - D&D Campaign Assistant"
---

> *"Remember, I know everything that can be known in the multiverse."* - Mimir, Planescape

Welcome to Mimir, your local-first D&D 5e campaign management tool. Named after the wise floating skull of Planescape lore, Mimir helps Dungeon Masters organize campaigns, run sessions, and keep everything at their fingertips.

## Quick Start

**New to Mimir?** Start with our tutorials:

1. **[Your First Campaign](tutorials/01-first-campaign/)** — Create a campaign and explore the dashboard
2. **[Your First Module](tutorials/02-first-module/)** — Build an adventure with maps and monsters
3. **[Running Your First Session](tutorials/03-first-session/)** — Use Play Mode to run an encounter
4. **[Player Display Setup](tutorials/04-player-display/)** — Set up a second screen for players
5. **[Creating Homebrew Content](tutorials/05-homebrew-content/)** — Make custom items, monsters, and spells

## Features

### Campaign Management
Create and organize campaigns with a structured workflow. Track documents, NPCs, and plot threads across multiple adventure modules.

### Interactive Maps
Upload maps (UVTT or images), configure grids, and place tokens. Manage fog of war with line-of-sight calculations and light sources.

### Play Mode
Run sessions with a dedicated play interface. Control what players see on a second display while you manage encounters behind the scenes.

### D&D 5e Reference
Search monsters, spells, items, and more from your D&D sourcebooks. Quick access to rules when you need them.

### PDF Export
Print character sheets, spell cards, and maps with token cutouts for physical play.

## Documentation

This documentation is organized using the [Diataxis](https://diataxis.fr/) framework:

| Section | Purpose |
|---------|---------|
| **[Tutorials](tutorials/)** | Step-by-step learning guides |
| **[How-To Guides](how-to/)** | Task-focused instructions |
| **[Reference](reference/)** | Detailed feature documentation |
| **[Understanding Mimir](explanation/)** | Concepts and design philosophy |

## Installation

**macOS / Linux** — one-line install:

```bash
curl -sSL https://raw.githubusercontent.com/mimir-dm/mimir/main/scripts/install.sh | sh
```

This installs the Mimir app (to `~/Applications` on macOS, `~/.local/bin` on Linux). On macOS it also installs the `mimir-mcp` CLI to `~/.local/bin`; on Linux the script installs the app only — download `mimir-mcp-<target-triple>` from [GitHub Releases](https://github.com/mimir-dm/mimir/releases) or build it with `cargo build --release -p mimir-mcp`.

To pin a specific version:

```bash
curl -sSL https://raw.githubusercontent.com/mimir-dm/mimir/main/scripts/install.sh | sh -s -- --version X.Y.Z
```

**Windows** — download and run the `.msi` installer from [GitHub Releases](https://github.com/mimir-dm/mimir/releases).

**Supported Platforms:** macOS, Windows, Linux

## Getting Help

- **Issues**: Report bugs on [GitHub Issues](https://github.com/mimir-dm/mimir/issues)
- **Discussions**: Ask questions on [GitHub Discussions](https://github.com/mimir-dm/mimir/discussions)

---

*Mimir is open source software. See the [Developer Documentation](reference/developer/) if you'd like to contribute.*
