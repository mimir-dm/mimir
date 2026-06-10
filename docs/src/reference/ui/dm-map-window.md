# DM Map Window

The DM Map window is a separate window showing the battle map while the main window stays on the campaign dashboard for reference. Clicking **Play** on a module (campaign dashboard Modules tab, or the modules table on the campaign landing page) opens it.

## Window Properties

| Property | Value |
|----------|-------|
| Window label | `dm-map` |
| URL | `/dm-map?moduleId=<id>&campaignId=<id>` |
| Initial size | 1280 × 900, resizable |
| Instances | One; re-opening focuses the existing window |

## Layout

### Toolbar

| Control | Function |
|---------|----------|
| **Map selector** | Dropdown of all maps available to the module; campaign-level maps are suffixed "(Campaign)" |
| **Module name** | Currently playing module (center) |
| **Display** | Open/close the Player Display window |
| **Blackout** | Hide content from players (shown only while the display is open) |
| **Fullscreen** | Toggle fullscreen on the DM Map window |

### Map Viewer

The map area embeds the DM map viewer (documented in [Play Mode](./play-mode.md)), with these controls:

- Zoom, reset view, Add PCs, Reveal Map, Print
- Fog, LOS, ambient light, and debug overlay toggles (UVTT maps)
- Token drag, selection, context menu, vision settings
- Trap and POI markers with visibility toggles

See [Play Mode](./play-mode.md) for the full toolbar and token control reference.

### Status Bar

Shows map name, pixel dimensions, current pan offset, and Player Display connection state.

## Dashboard Link

Double-clicking a token, trap, or POI in the DM Map window emits a focus event to the main window, which selects the corresponding monster, NPC, PC, trap, or POI in the campaign dashboard.

## Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `+` / `=` | Zoom in |
| `-` | Zoom out |
| `0` | Reset view |
| `h` / `H` | Toggle visibility of selected token |
| `d` / `D` | Toggle dead state of selected token |
| `Escape` | Close context menu and deselect |

Shortcuts are ignored while typing in text inputs.

## See Also

- [Play Mode](./play-mode.md) — Session-running overview and full map viewer reference
- [Player Display](./player-display.md) — The player-facing window
- [Keyboard Shortcuts](../keyboard-shortcuts.md) — Full shortcut reference
