---
id: visual-display-system
level: initiative
title: "Visual Display System"
short_code: "MIMIR-I-0015"
created_at: 2025-12-06T16:02:36.548959+00:00
updated_at: 2025-12-21T01:37:24.443428+00:00
parent: 
blocked_by: []
archived: true

tags:
  - "#initiative"
  - "#phase/active"


exit_criteria_met: false
estimated_complexity: XL
strategy_id: NULL
initiative_id: visual-display-system
---

# Visual Display System Initiative

## Context

While Mimir handles campaign data and the Physical Print initiative addresses table materials, many groups benefit from visual displays during play. A second monitor or TV showing battle maps, exploration scenes, or dungeon layouts adds immersion beyond pure "theater of the mind."

This initiative introduces a visual display system - a separate window that can be dragged to an external monitor to show players maps, scenes, and tactical grids. Unlike the print system which renders existing data, this requires **new content types** (maps, scenes) and **interactive features** (tokens, fog of war).

**Priority**: This is second priority after Physical Print (MIMIR-I-0014) due to larger scope.

## Goals & Non-Goals

**Goals:**
- Display window that can be dragged to external monitor (extended desktop)
- Battle maps with grid overlay for tactical combat
- Exploration/dungeon maps for navigation and discovery
- World/region maps for travel and context
- Interactive features: tokens, fog of war, pan/zoom
- Integration with campaign data (show encounter monsters, player character tokens)
- Optimized workflow for Dungeondraft and similar map-making tools

**Non-Goals:**
- Full VTT feature parity (Roll20, Foundry) - this is a display tool for in-person play, not a full game system
- Remote/networked play - this is for in-person table use only
- Built-in map creation tools - maps imported from external sources (Dungeondraft, Inkarnate, etc.)
- Dice rolling or mechanical automation on the display
- Video/animation support in v1

## UX Design

### Map Organization

Maps exist at two levels in the hierarchy:

```
Campaign
├── Maps (campaign-wide)
│   ├── Regional Map - Sword Coast
│   └── City Map - Waterdeep
│
└── Modules
    └── The Sunken Temple
        ├── Documents (session notes, etc.)
        └── Maps (module-specific)
            ├── Temple Entrance (with pre-placed guards)
            └── Boss Chamber (with pre-placed boss + traps)
```

- **Campaign Maps**: Regional/world maps, city overviews - broader context
- **Module Maps**: Dungeon rooms, encounter areas - localized to specific adventures

### Setup Flow

```
┌─────────────────────────────────────────────────────────────┐
│  Map Setup Screen                                           │
├─────────────────────────────────────────────────────────────┤
│  1. Upload Image                                            │
│     [Drop PNG/JPG here or click to upload]                  │
│                                                             │
│  2. Grid Configuration                                      │
│     Grid Type: ○ Square  ○ Hex  ○ None                      │
│     [Drag-to-align grid overlay on image]                   │
│     (Optimized for maps with baked-in grids like           │
│      Dungeondraft exports)                                  │
│                                                             │
│  3. Optional: Pre-place Tokens                              │
│     [+ Monster] [+ Trap] [+ Marker]                         │
│     (Set up encounters ahead of time - guards at doors,     │
│      vendors in market stalls, boss in final room)          │
│                                                             │
│  4. Optional: Associate with Module                         │
│     Module: [Select module ▾] (or save to campaign)         │
│                                                             │
│  5. [Save Map]                                              │
└─────────────────────────────────────────────────────────────┘
```

### During Play

From the session/play screen, a single button opens the display:

```
[▶ Open Display] → Opens separate window (drag to TV)
```

The DM controls everything from the main window; the display window is view-only for players.

## New Content Types Required

This initiative introduces new data models not currently in Mimir:

### Maps
- **Campaign Map**: Region/world scale maps for travel and context
- **Module Map**: Dungeon rooms, encounter areas tied to specific modules

### Map Assets
- **Tokens**: Character and monster representations (images or icons)
- **Markers**: Points of interest, notes, labels
- **Fog of War**: Hidden/revealed area tracking

## Use Cases

### Use Case 1: Combat Encounter Display
- **Actor**: DM
- **Scenario**: DM starts encounter → Opens display window on TV → Loads battle map → Places monster tokens → Players see tactical layout → DM reveals fog as combat progresses
- **Expected Outcome**: Players have clear view of battlefield on shared display

### Use Case 2: Dungeon Exploration
- **Actor**: DM
- **Scenario**: Party enters dungeon → DM loads dungeon map → Reveals rooms as party explores → Moves party token through corridors → Players track their progress visually
- **Expected Outcome**: Immersive exploration without full theater-of-mind burden

### Use Case 3: World Travel
- **Actor**: DM
- **Scenario**: Party plans travel → DM shows world map → Highlights route → Points out landmarks and dangers → Players understand geography
- **Expected Outcome**: Clear sense of world scale and travel context

## Architecture

### Overview

```
┌─────────────────────────────────────────────────────────────┐
│  Main Mimir Window (DM Screen)                              │
│  ┌─────────────────────┐  ┌─────────────────────┐          │
│  │  Campaign/Session   │  │  Display Control    │          │
│  │  Management         │  │  Panel              │          │
│  │                     │  │  - Map selection    │          │
│  │                     │  │  - Token controls   │          │
│  │                     │  │  - Fog of war       │          │
│  │                     │  │  - Reveal/hide      │          │
│  └─────────────────────┘  └─────────────────────┘          │
└─────────────────────────────────────────────────────────────┘
                              │
                              │ IPC (state sync)
                              ▼
┌─────────────────────────────────────────────────────────────┐
│  Player Display Window (TV/External Monitor)                │
│  ┌─────────────────────────────────────────────────────┐   │
│  │                                                      │   │
│  │              Battle Map / Scene View                 │   │
│  │                                                      │   │
│  │    [Token] [Token]                                   │   │
│  │              ████████ (fog)                          │   │
│  │    [Token]   ████████                                │   │
│  │                                                      │   │
│  └─────────────────────────────────────────────────────┘   │
│  (No controls - display only)                              │
└─────────────────────────────────────────────────────────────┘
```

### Tech Stack

- **Canvas Rendering**: HTML5 Canvas or WebGL for map/token rendering
- **Multi-Window**: Tauri's multi-window support for display window
- **State Sync**: IPC between main window and display window
- **Image Handling**: Support for common map image formats (PNG, JPG, WebP)
- **Grid System**: Configurable grid overlay (square, hex)

## Detailed Design

### Data Models

```rust
// Map - can belong to campaign (regional) or module (encounter/dungeon)
pub struct Map {
    pub id: i32,
    pub campaign_id: i32,
    pub module_id: Option<i32>,  // None = campaign-level map
    pub name: String,
    pub image_path: String,
    pub width_px: u32,
    pub height_px: u32,
    // Grid alignment (drag-to-fit result)
    pub grid_type: GridType,     // Square, Hex, None
    pub grid_size_px: Option<u32>,   // pixels per grid cell
    pub grid_offset_x: Option<i32>,  // alignment offset
    pub grid_offset_y: Option<i32>,
}

// Token - pre-placed during setup or added during play
pub struct Token {
    pub id: i32,
    pub map_id: i32,
    pub name: String,
    pub token_type: TokenType,   // Monster, PC, NPC, Trap, Marker
    pub image_path: Option<String>,
    pub icon: Option<String>,    // Fallback icon if no image
    pub size: TokenSize,         // Tiny, Small, Medium, Large, Huge, Gargantuan
    pub x: f32,                  // Grid position
    pub y: f32,
    pub visible_to_players: bool,
    // Link to catalog/character data
    pub monster_id: Option<i32>,
    pub character_id: Option<i32>,
}

pub struct FogOfWar {
    pub map_id: i32,
    pub revealed_areas: Vec<RevealedArea>,  // Polygons or grid cells
}
```

### Display Window

The player display window is a separate Tauri window:
- No menu bar or controls (clean display)
- Receives state updates via IPC from main window
- Renders current map, visible tokens, revealed fog
- Supports pan/zoom (DM controlled)

### DM Control Panel

In the main window, a control panel allows the DM to manage the player display:

**Essential Controls (v1):**
- Select active map (from campaign or module maps)
- Pan/zoom control for display
- Place/move tokens (drag & drop)
- Toggle token visibility (show/hide individual tokens for ambushes, invisibility)
- Quick token add (spawn monster mid-combat without leaving play screen)
- Reveal/hide fog areas (paint or polygon)
- Switch maps (quick scene transitions)
- Blackout mode (blank screen between scenes - show logo or solid color)

**Nice to Have (defer if needed):**
- Ping/highlight location (pulse effect to draw player attention)
- Token labels on/off toggle
- Grid visibility toggle (some maps look better without overlay)

## Implementation Plan

### Phase 1: Foundation & Multi-Window
- Database models for maps (campaign-level and module-level)
- Map image upload (PNG/JPG) and storage
- Basic display window (Tauri multi-window, shows static image)
- IPC setup between main window and display window
- "Open Display" button in play screen

### Phase 2: Grid & Alignment
- Grid overlay rendering (square, hex)
- Drag-to-align grid tool (for Dungeondraft-style maps with baked grids)
- Pan/zoom controls (DM-controlled)
- Map scaling to fit display
- Blackout mode (blank screen between scenes)

### Phase 3: Tokens
- Token data model (with monster/character links)
- Token placement during setup (pre-place encounters)
- Token movement during play (drag & drop)
- Token visibility toggle (ambush reveals)
- Quick token add (spawn mid-combat)
- Token size handling (Tiny → Gargantuan)

### Phase 4: Fog of War
- Fog overlay rendering
- Reveal tools (rectangle, polygon, brush)
- Fog state persistence per map
- Hide/reveal transitions

### Phase 5: Polish & Integration
- Map quick-switch in DM control panel
- Module map list integration
- Grid visibility toggle
- Performance optimization for large maps

### Physical Print Export

For users who prefer physical battlemats, the system should support exporting:
- **Map image** - Full resolution for printing at scale
- **Token sheet** - Separate printable sheet with tokens for cutting out

This bridges the digital display system with the existing Physical Print initiative - same maps work both ways.

### Future Considerations (Not in v1)
- Map creation/editing tools
- Animated effects (spell templates)
- Sound/ambiance triggers
- Dynamic lighting
- Ping/highlight effects
- Token labels