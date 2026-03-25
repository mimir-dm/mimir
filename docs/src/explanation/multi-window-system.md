# The Multi-Window System

Why Mimir uses four separate windows and how they communicate.

## The Problem

Running a D&D session with a digital tool presents a tension: the DM needs full control of maps, monsters, and notes — but players should only see what's been revealed. A single window can't serve both needs at once.

## Four Windows, Four Roles

Mimir solves this with four distinct application windows, each built as a separate Vite entrypoint with its own HTML file and Vue app instance:

### Main Window

The primary interface. Campaign management, character sheets, module prep, settings — everything you do outside of active play happens here. This is a full Vue Router application with navigation between campaigns, modules, characters, and reference materials.

### DM Map Window

A dedicated map view for the DM during play sessions. Shows the full map with all tokens (including hidden ones), fog of war controls, and light source management. Opens as a separate window so the DM can position it on their screen while the player display runs on a projector or TV.

### Player Display Window

What your players see. Shows only the revealed portions of the map — fog of war hides unexplored areas, hidden tokens are invisible, and the viewport is controlled by the DM. Designed for a second monitor, projector, or TV facing the players.

The player display has no interactive controls. Players can't pan, zoom, or click on anything. The DM controls exactly what appears.

### Sources Window

A standalone D&D 5e reference browser. Monster stat blocks, spell descriptions, item properties — all searchable and browsable. Opens as a separate window so you can keep reference material visible while working in the main window or running a session.

## Why Separate Windows?

### Independent Positioning

Each window can live on a different monitor. A typical two-monitor setup:

- **Monitor 1 (DM)**: Main window + DM Map Window side by side
- **Monitor 2 (Players)**: Player Display full-screen

Or with three monitors:

- **Monitor 1**: Main window (prep, notes)
- **Monitor 2**: DM Map Window (tactical view)
- **Monitor 3**: Player Display (player-facing)

### Isolation

Each window is a separate Vue application instance. A crash or heavy rendering in one window doesn't affect the others. The player display stays smooth even while the DM is loading a complex map in another window.

### Focused UI

The player display strips away everything players shouldn't see — no toolbars, no sidebars, no token lists. The DM map view shows everything. Same data, different presentations.

## Communication Between Windows

The windows share data through two mechanisms:

### Shared Database

All windows read from the same SQLite database. When the DM places a token in the main window, it's written to the database. When the DM map window loads, it reads the current state.

### Tauri Events

Real-time synchronization uses Tauri's event system. When the DM:

- Reveals fog of war → the player display updates immediately
- Moves a token → both map windows reflect the change
- Changes the viewport → the player display pans to match
- Toggles blackout → the player display goes dark instantly

Events are typed and scoped — the player display only receives events it needs to render, never internal DM state.

### Theme Synchronization

When you change the application theme (light, dark, or hyper), a Tauri event broadcasts to all windows so they switch together. The DM map and player display default to dark theme regardless, since they're typically used in dimly lit game rooms.

## The Blackout Feature

A small but critical feature: the DM can instantly black out the player display with a single toggle. This is essential for:

- Switching maps between encounters without spoiling the next area
- Pausing to set up tokens before revealing a new room
- "Theater of the mind" moments where you want players focused on narration, not the screen

## See Also

- [Tutorial: Player Display Setup](../tutorials/04-player-display.md) — Hands-on guide to configuring the display
- [Use Player Display](../how-to/play-mode/use-player-display.md) — How-to for common display tasks
- [Architecture Overview](./architecture-overview.md) — The technical stack behind the windows
