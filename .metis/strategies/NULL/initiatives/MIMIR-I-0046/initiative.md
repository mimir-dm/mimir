---
id: map-token-vtt-system
level: initiative
title: "Map & Token VTT System"
short_code: "MIMIR-I-0046"
created_at: 2026-01-25T01:14:14.570932+00:00
updated_at: 2026-01-25T15:58:07.195168+00:00
parent: MIMIR-V-0001
blocked_by: []
archived: false

tags:
  - "#initiative"
  - "#phase/active"


exit_criteria_met: false
estimated_complexity: L
strategy_id: NULL
initiative_id: map-token-vtt-system
---

# Map & Token VTT System Initiative

## Context

The v1 Mimir application had a fully functional Virtual Tabletop (VTT) system for managing battle maps, tokens, light sources, and line-of-sight visibility. As part of the v0.5 architecture rewrite (MIMIR-I-0041), this system needs to be migrated to the new database-first, sqlx-based architecture.

**Current State (v2):**
- Map CRUD commands implemented (create, list, get, update, delete)
- UVTT parsing and image serving working
- Light source basic CRUD exists
- Frontend components exist but reference missing backend commands

**Migration Gaps:**
- Token CRUD commands missing (only list_tokens exists)
- Token image serving not implemented
- Fog state not persisted
- Player display IPC not migrated
- Frontend composables incomplete

Reference: Investigation documented in [[MIMIR-T-0411]]

## Goals & Non-Goals

**Goals:**
- Fully functional token system with create, update, delete, visibility toggle
- Token image upload and serving through asset system
- Light source management with presets (torch, lantern, candle)
- Fog of war with LOS (line-of-sight) blocking using UVTT wall data
- Multi-window player display with IPC synchronization
- Viewport sync between DM and player windows

**Non-Goals:**
- Real-time multiplayer networking (local multi-window only)
- 3D rendering or advanced VTT features
- Custom map drawing tools (maps imported as UVTT files)
- Initiative/turn tracking integration (separate feature)

## Architecture

### Data Model

```
MAP
├── id, campaign_id, module_id (optional)
├── name, description, lighting_mode
├── uvtt_path (filesystem asset)
├── fog_enabled (boolean)
├── 1:N → TOKEN
└── 1:N → LIGHT_SOURCE

TOKEN
├── id, map_id, name, token_type, size
├── x, y (pixel coordinates)
├── visible_to_players (boolean)
├── color, image_path (optional)
├── monster_id OR character_id (optional links)
├── vision_type (normal, darkvision, blindsight, etc.)
└── vision_range_ft (nullable)

LIGHT_SOURCE
├── id, map_id, token_id (optional - can attach to token)
├── name, light_type (torch, lantern, candle, spell, custom)
├── x, y (pixel coordinates)
├── bright_radius_ft, dim_radius_ft
├── color (hex), is_active (boolean)
```

### Component Architecture

```
┌─────────────────────────────────────────────────────────┐
│  DM Window                                              │
│  ├── DmMapViewer.vue (main map surface)                │
│  │   ├── TokenRenderer.vue (token display/interaction) │
│  │   ├── LightSourceRenderer.vue (light visualization) │
│  │   ├── FogOverlay.vue (fog of war mask)             │
│  │   └── LosDebugOverlay.vue (wall/portal debug)      │
│  └── Toolbar (zoom, tokens, lights, fog controls)      │
└─────────────────────┬───────────────────────────────────┘
                      │ IPC Events
┌─────────────────────┴───────────────────────────────────┐
│  Player Display Window                                  │
│  ├── PlayerMapViewer.vue (read-only map view)          │
│  ├── Visible tokens only                               │
│  ├── Fog/vision mask applied                           │
│  └── Synced viewport from DM                           │
└─────────────────────────────────────────────────────────┘
```

### State Architecture: Setup vs Play

The system separates **encounter setup** (persistent) from **play session** (ephemeral).

**Token Placement Panel (Database - persistent)**
- Token starting positions
- Which tokens exist on the map
- Light source placements
- Initial visibility settings
- DM prepares encounters here, saves layout

**Play Session (Runtime - ephemeral)**
- Current token positions (from dragging)
- Dead/alive status
- Light on/off toggles
- Token visible/invisible
- Doors open/closed
- **Resets to DB state on map reload**

**Explicitly NOT tracked:**
- HP (handled elsewhere or theater of mind)

This keeps the VTT focused on tactical visualization, not combat stat tracking.

### IPC Event Flow (Player Display)

Player display is **local IPC only** (Tauri window-to-window), **read-only** for players.

```
DM Action → Runtime State (Pinia/composable) → IPC Event → Player Display
                    ↑
        DB State (starting positions loaded on map open)
```

**Events:**
- `player-display:map-update` - Map image and metadata
- `player-display:tokens-update` - Visible tokens with current positions
- `player-display:fog-update` - Visibility polygons and mask state
- `player-display:light-sources-update` - Active light sources
- `player-display:viewport-sync` - Pan/zoom synchronization

## Detailed Design

### Backend Commands (Rust/Tauri)

**Token Commands:**
- `create_token(map_id, name, token_type, size, x, y, ...)` → Token
- `update_token(id, ...)` → Token
- `update_token_position(id, x, y)` → Token (optimized for drag)
- `delete_token(id)` → ()
- `toggle_token_visibility(id)` → Token
- `serve_token_image(token_id)` → Base64 data URL

### Token Image System

Token images come from the **5etools catalog import** - no separate upload needed.

**Storage:** `{APP_DATA_DIR}/assets/catalog/bestiary/tokens/{source}/{name}.webp`

**Flow:**
1. DM adds monster to map → token created with `monster_id` link
2. Frontend requests image → `serve_token_image` looks up monster's `token_image_path`
3. Image served from catalog assets as base64 data URL

**Fallbacks:**
- Monster with no catalog image → colored circle (red) + monster name
- PC token → colored circle (green) + character name
- NPC token → colored circle (blue) + name

**Note:** Images are global catalog assets shared across all campaigns. No per-campaign token image storage.

**Fog Commands:**
- `get_fog_state(map_id)` → FogState
- `set_fog_enabled(map_id, enabled)` → ()

**Light Commands (additions):**
- `update_light_source(id, ...)` → LightSource
- `move_light_source(id, x, y)` → LightSource

### Frontend Composables

**useTokens.ts:**
- Token CRUD operations via Tauri invoke
- Token image caching (Map<id, dataUrl>)
- Computed: visibleTokens, tokensByType
- Methods: createToken, updateToken, deleteToken, toggleVisibility

**useLightSources.ts:**
- Light CRUD with presets
- Distance conversion (feet ↔ pixels via gridSize)
- Methods: createLight, updateLight, toggleLight, deleteLight

**useVisibilityPolygon.ts:**
- Ray-casting algorithm for LOS calculation
- Input: token position, wall segments, portals
- Output: visibility polygon path for SVG clipping

**usePlayerDisplay.ts:**
- IPC event emission to player window
- Viewport sync (auto-sync toggle, manual push)
- Connection status tracking

### Vision & Player View System

The system separates **simulation** (D&D lighting rules) from **presentation** (what the VTT shows players).

**Lighting** (D&D simulation - dropdown)
- **Bright Light**: Unlimited vision range for all tokens
- **Dim Light**: Vision limited by range (default 60ft) and walls
- **Darkness**: Only darkvision, blindsight, or active light sources provide vision

**Player View** (VTT presentation - two independent toggles)
- **Mask Unexplored**: Hide map areas outside PC vision polygons
- **Mask Hidden Tokens**: Hide enemy tokens until PCs have line of sight

**Vision Calculation:**
```
if lighting == bright: vision_range = unlimited
if lighting == dim: vision_range = token.vision_range_ft (default 60)
if lighting == darkness:
  if has_darkvision: vision_range = darkvision_range
  elif has_light_source: vision_range = bright_radius + dim_radius
  else: vision_range = 0
```

**Example Scenarios:**
| Scenario | Lighting | Mask Unexplored | Mask Hidden Tokens |
|----------|----------|-----------------|-------------------|
| Outdoor daylight combat | Bright | Off | Off |
| Dungeon exploration | Darkness | On | On |
| Tactical combat, known room | Dim | Off | On |
| Theater of mind, full trust | Any | Off | Off |

**LOS Blocking:**
- Walls from UVTT file block vision
- Portals (doors) can be toggled open/closed
- Closed portals act as blocking walls
- Ray-casting from token center to determine visible area

## Alternatives Considered

**WebSocket for player sync:** Rejected - IPC between Tauri windows is simpler and more reliable for local-only use case. WebSockets add complexity without benefit since we're not doing network multiplayer.

**Canvas rendering instead of SVG:** Rejected - SVG with GPU-accelerated transforms provides smooth pan/zoom. Canvas would require manual redraw logic. Current approach works well.

**Separate token table per map:** Rejected - Single tokens table with map_id foreign key is simpler and allows for cross-map token queries if needed later.

## Implementation Plan

### Phase 1: Backend Token System
1. Token table schema & migration
2. Token repository layer (CRUD with sqlx)
3. Token Tauri commands
4. Token image serving

### Phase 2: Backend Light & Fog
5. Light source command updates
6. Fog state persistence
7. Fog commands

### Phase 3: Frontend Integration
8. useTokens composable backend integration
9. useLightSources composable verification
10. DmMapViewer token/light wiring

### Phase 4: Player Display
11. Player display IPC event system
12. Player display window/view
13. Viewport synchronization

### Phase 5: Vision & Fog Rendering
14. useVisibilityPolygon raycasting port
15. Fog overlay SVG mask rendering
16. LOS debug visualization

### Phase 6: Polish
17. Token context menus
18. End-to-end testing & bug fixes

### Token Context Menu (right-click)

- **Toggle Light** - Add torch / extinguish (for tokens that can carry light)
- **Toggle Dead** - Mark alive/dead (visual skull overlay)
- **Toggle Visible** - Show/hide from player display

**Visibility use cases:**
- Hidden enemies until spotted
- Traps revealed when triggered
- Points of interest on discovery
- Secret doors / hidden features

No keyboard shortcuts - context menu only.