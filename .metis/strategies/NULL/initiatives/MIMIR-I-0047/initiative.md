---
id: redesign-map-vision-and-lighting
level: initiative
title: "Redesign Map Vision and Lighting System"
short_code: "MIMIR-I-0047"
created_at: 2026-01-26T02:10:53.616737+00:00
updated_at: 2026-01-26T15:05:31.554103+00:00
parent: 
blocked_by: []
archived: false

tags:
  - "#initiative"
  - "#phase/completed"


exit_criteria_met: false
estimated_complexity: M
strategy_id: NULL
initiative_id: redesign-map-vision-and-lighting
---

# Redesign Map Vision and Lighting System Initiative

## Context

The current vision/fog system has grown organically and become confusing:

**Current Problems:**
1. Multiple overlapping concepts: `fogEnabled`, `tokenOnlyLos`, `revealMap`, `useLosBlocking`
2. Vision settings split between backend (token model) and frontend (useTokenVision composable)
3. Map lights reveal areas globally, giving away map sections before PCs discover them
4. Unclear what each mode does (Fog vs Token buttons)
5. Light sources, vision types, and ambient light interact in complex ways that are hard to reason about

**What We Have Now:**
- DM window: Map with fog overlay, tokens, light sources
- Player Display: Separate window showing what players see
- UVTT data: Walls for line-of-sight blocking, embedded lights, ambient light level
- Token vision: Per-token settings for vision in different light conditions
- Multiple fog modes that don't clearly map to D&D concepts

## Goals & Non-Goals

**Goals:**
- Simple, understandable vision model that maps to D&D 5e concepts
- DM can easily configure PC vision settings via context menu
- Clear separation: what DM sees vs what players see
- Map lights only reveal when PCs discover them
- Single source of truth for vision state (frontend-only, session-based)

**Non-Goals:**
- Persisting vision settings to database (keep it session-based for now)
- Magical darkness / light spell interactions (future enhancement)
- Individual player views (all players see the same "party vision")

## D&D 5e Vision Rules Reference

### Ambient Light Levels
| Level | Effect |
|-------|--------|
| **Bright** | Normal vision works, everyone can see |
| **Dim** | Lightly obscured, disadvantage on Perception |
| **Darkness** | Heavily obscured, effectively blind without special vision |

### Vision Types
| Type | In Bright | In Dim | In Darkness |
|------|-----------|--------|-------------|
| **Normal** | See all | See all (dim) | Blind (0 ft) |
| **Darkvision 60ft** | See all | See all | See 60ft (as dim) |
| **Darkvision 120ft** | See all | See all | See 120ft (as dim) |
| **Blindsight 30ft** | See 30ft | See 30ft | See 30ft |
| **Devil's Sight 120ft** | See all | See all | See 120ft (normal) |

### Light Sources
| Source | Bright Radius | Dim Radius |
|--------|---------------|------------|
| Candle | 5 ft | 10 ft |
| Torch | 20 ft | 40 ft |
| Lantern (hooded) | 30 ft | 60 ft |
| Light cantrip | 20 ft | 40 ft |
| Daylight spell | 60 ft | 120 ft |

### Key Insight
A light source creates a zone where **ambient light is elevated**:
- Within bright radius: treat as bright light
- Within dim radius: treat as dim light
- Beyond: ambient light level applies

## Design Decisions

### Token Vision & Light Settings (Backend)

All tokens (PC, monster, NPC) store these fields in `token_placements`:

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `vision_bright_ft` | int NULL | NULL | Vision range in bright light (NULL = unlimited) |
| `vision_dim_ft` | int NULL | NULL | Vision range in dim light (NULL = unlimited) |
| `vision_dark_ft` | int | 0 | Vision range in darkness (0 = blind) |
| `light_radius_ft` | int | 0 | Light source total radius (0 = no light) |

**Light radius convention:** Store dim radius. Bright radius = light_radius_ft / 2.
- Torch: `light_radius_ft: 40` (20ft bright, 40ft dim)
- Lantern: `light_radius_ft: 60` (30ft bright, 60ft dim)

### Vision Presets (UI convenience)

| Preset | bright | dim | dark | light |
|--------|--------|-----|------|-------|
| Human | ∞ | ∞ | 0 | 0 |
| Human + Torch | ∞ | ∞ | 0 | 40 |
| Human + Lantern | ∞ | ∞ | 0 | 60 |
| Darkvision 60ft | ∞ | ∞ | 60 | 0 |
| Darkvision 120ft | ∞ | ∞ | 120 | 0 |
| Blindsight 30ft | 30 | 30 | 30 | 0 |

### Effective Light Level Calculation

```
For a point P:
  1. Start with ambient light level (map setting)
  2. For each light source (map lights + token lights):
     - If P is within light's bright radius: upgrade to BRIGHT
     - Else if P is within light's dim radius: upgrade to at least DIM
  3. Return max light level found
```

### PC Vision Calculation

```
For PC at position T with vision settings:
  1. Calculate LOS polygon (raycast against walls/closed doors)
  2. For each point P in potential vision area:
     - Get effective light level at P
     - Determine if PC can see P based on:
       - BRIGHT: use vision_bright_ft
       - DIM: use vision_dim_ft  
       - DARK: use vision_dark_ft
     - Also: if PC has light, they can see within their own light radius
  3. Clip vision to appropriate radius
```

**Key rule:** PC must be INSIDE a light's radius to benefit from it. Seeing a distant torch doesn't help.

### Player Display

- Shows **union of all PC vision** (party vision)
- Assumes party is communicating
- DM handles split-party situations manually

### DM View

- Always shows full map (all tokens, all lights, all areas)
- **Always shows vision boundaries** for all tokens (outline only, not filled)
- Boundaries color-coded by token color (distinguishes PC/NPC/Monster)

### Display Mode Controls (2 Independent Toggles)

| Fog (Map) | Token LOS | Mode Name | Result |
|-----------|-----------|-----------|--------|
| Off | Off | **Reveal All** | Everything visible |
| Off | On | **Token LOS** | Map visible, tokens need LOS to be seen |
| On | On | **Full Vision** | Map + tokens both need full vision calc |
| On | Off | (disabled) | Not a valid combination |

**UI**: Two toggle buttons
- **Fog**: Controls whether map areas outside PC vision are hidden
- **Token LOS**: Controls whether tokens need LOS to be visible

When Fog is ON, Token LOS is forced ON (can't have fog without token LOS).

### Ambient Light Control

- Manual DM dropdown: Bright / Dim / Dark
- Applies to entire map
- UVTT ambient light ignored (DM has full control)

### Map Light Sources

- Can be placed in Map Editor / Token Placer (persisted)
- Can be placed at runtime during Play (persisted)
- Stored in `light_sources` table with `bright_radius` and `dim_radius`

## Architecture

### Database Changes

**Migration: Add vision/light fields to `token_placements`**

```sql
ALTER TABLE token_placements ADD COLUMN vision_bright_ft INTEGER;     -- NULL = unlimited
ALTER TABLE token_placements ADD COLUMN vision_dim_ft INTEGER;        -- NULL = unlimited
ALTER TABLE token_placements ADD COLUMN vision_dark_ft INTEGER NOT NULL DEFAULT 0;
ALTER TABLE token_placements ADD COLUMN light_radius_ft INTEGER NOT NULL DEFAULT 0;
```

**Existing tables (no changes needed):**
- `light_sources` - map-placed lights (already has bright_radius, dim_radius)
- `maps` - has `lighting_mode` for ambient (repurpose or add explicit field)

### Backend Changes (Rust)

**1. Update `TokenPlacement` model** (`mimir-core/src/models/campaign/token_placement.rs`):
- Add `vision_bright_ft: Option<i32>`
- Add `vision_dim_ft: Option<i32>`
- Add `vision_dark_ft: i32`
- Add `light_radius_ft: i32`

**2. Update `NewTokenPlacement` and `UpdateTokenPlacement`**:
- Add corresponding fields for insert/update

**3. Update token commands** (`mimir/src/commands/token.rs` or similar):
- `update_token_vision` command to set vision settings
- Include vision fields in token responses

**4. No changes needed to `light_sources`** - already has what we need

### Frontend Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                        DmMapViewer.vue                          │
│  ┌──────────────┐  ┌──────────────┐  ┌───────────────────────┐ │
│  │ Mode Controls│  │Ambient Light │  │ Token Context Menu    │ │
│  │ [Fog][TokLOS]│  │ [Bright/Dim] │  │ → Vision Settings     │ │
│  └──────────────┘  └──────────────┘  └───────────────────────┘ │
│                                                                 │
│  ┌─────────────────────────────────────────────────────────────┐│
│  │                    Map Viewport                             ││
│  │  ┌─────────────┐                                            ││
│  │  │ Map Image   │ (always visible to DM)                     ││
│  │  ├─────────────┤                                            ││
│  │  │ Light Layer │ (visual effect only on DM view)            ││
│  │  ├─────────────┤                                            ││
│  │  │ Token Layer │ (all tokens visible to DM)                 ││
│  │  ├─────────────┤                                            ││
│  │  │ Vision      │ (outlines showing each token's vision)     ││
│  │  │ Boundaries  │                                            ││
│  │  └─────────────┘                                            ││
│  └─────────────────────────────────────────────────────────────┘│
└─────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────┐
│                    PlayerDisplayWindow.vue                       │
│  ┌─────────────────────────────────────────────────────────────┐│
│  │                    Map Viewport                             ││
│  │  ┌─────────────┐                                            ││
│  │  │ Map Image   │ (masked by party vision if Fog ON)         ││
│  │  ├─────────────┤                                            ││
│  │  │ Light Layer │ (visual effect in visible areas)           ││
│  │  ├─────────────┤                                            ││
│  │  │ Token Layer │ (filtered by LOS if Token LOS ON)          ││
│  │  ├─────────────┤                                            ││
│  │  │ Fog Overlay │ (hides areas outside party vision)         ││
│  │  └─────────────┘                                            ││
│  └─────────────────────────────────────────────────────────────┘│
└─────────────────────────────────────────────────────────────────┘
```

### Key Composables

**1. `useTokenVision.ts`** (already created, needs backend integration)
- Manage per-token vision settings
- Presets for common configurations
- Sync with backend on save

**2. `useVisionCalculation.ts`** (rewrite)
- Input: tokens, walls, lights, ambient level
- Output: per-token visibility polygons
- Handles the "PC must be inside light radius to benefit" rule

**3. `usePartyVision.ts`** (new)
- Combines individual PC vision into union
- Determines which tokens are visible to players
- Used by PlayerDisplayWindow

### Data Flow

```
1. DM sets ambient light level → stored in component state (or map)
2. DM places tokens with vision settings → saved to backend
3. DM moves token during play → triggers vision recalculation
4. Vision calculation runs:
   a. Get all light sources (map + token lights)
   b. For each PC token:
      - Calculate effective light at PC position
      - Determine vision radius based on light level
      - Raycast against walls to get visibility polygon
      - Clip to vision radius
   c. Union all PC polygons → party vision
5. Send to Player Display:
   - If Fog ON: mask map with party vision
   - If Token LOS ON: filter tokens by LOS check
```

## UI/UX Design

### DM Controls (Top Bar)

```
┌─────────────────────────────────────────────────────────────────┐
│ [Fog: OFF/ON] [Token LOS: OFF/ON]  │  Ambient: [Bright ▼]      │
└─────────────────────────────────────────────────────────────────┘
```

- **Fog toggle**: When ON, Token LOS is forced ON and disabled
- **Token LOS toggle**: Independent when Fog is OFF
- **Ambient dropdown**: Bright / Dim / Dark

### Token Context Menu (Right-click)

```
┌──────────────────────────┐
│ Hide from Players    [H] │
│ Vision...            [V] │  ← Opens vision popup
│ ─────────────────────────│
│ Extinguish Light     [L] │  ← If token has light
│ ─────────────────────────│
│ Mark Dead            [D] │
└──────────────────────────┘
```

### Vision Settings Popup

```
┌─────────────────────────────────┐
│ Token Name              [X]    │
├─────────────────────────────────┤
│ Presets: [Choose...        ▼]  │
├─────────────────────────────────┤
│ Vision Range (ft)              │
│   Bright: [____] (∞ = empty)   │
│   Dim:    [____]               │
│   Dark:   [__0_]               │
├─────────────────────────────────┤
│ Light Source (ft)              │
│   Radius: [__0_]               │
│   [None] [Torch] [Lantern]     │
├─────────────────────────────────┤
│              [Reset to Default]│
└─────────────────────────────────┘
```

### DM Map View - Vision Boundaries

- Each token with vision shows an **outline** of their vision area
- Outline color = token's color (from token type or custom)
- Outlines drawn on top layer, always visible
- No fill, just the boundary line

## Code Cleanup

**Remove/Replace:**
- `fogEnabled` ref → replaced by `fogMode` toggle
- `tokenOnlyLos` ref → replaced by `tokenLosMode` toggle  
- `revealMap` ref → remove (redundant with both toggles OFF)
- `useLosBlocking` ref → always true when UVTT data available
- Old `getTokenVisionRadiusPx` function → use new composable
- `fogLightZones` computed → remove (lights handled differently)
- Frontend-only `useTokenVision` → integrate with backend

**Keep:**
- `useVisibilityPolygon.ts` → core raycast algorithm still useful
- `useUvttMap.ts` → still need walls/doors from UVTT
- `LightOverlay.vue` → visual light rendering (z-index below fog)
- `light_sources` table → map-placed lights

## Implementation Plan

### Phase 1: Database & Backend
1. Create migration for `token_placements` vision fields
2. Update `TokenPlacement` model with new fields
3. Update `NewTokenPlacement` / `UpdateTokenPlacement`
4. Add `update_token_vision` command
5. Include vision fields in token list/get responses

### Phase 2: Frontend - Core Vision System
1. Rewrite `useVisionCalculation.ts` with new logic
2. Create `usePartyVision.ts` for union calculation
3. Update `useTokenVision.ts` to sync with backend
4. Implement "PC must be inside light" rule

### Phase 3: Frontend - DM View
1. Replace Fog/Token buttons with new toggle UI
2. Add ambient light dropdown
3. Render vision boundary outlines for all tokens
4. Update context menu with Vision option
5. Wire up vision popup to backend save

### Phase 4: Frontend - Player Display
1. Update fog mask to use new party vision
2. Implement token LOS filtering
3. Ensure lights only render in visible areas
4. Test all mode combinations

### Phase 5: Cleanup
1. Remove deprecated refs and computed properties
2. Remove old vision calculation code
3. Update any affected tests
4. Manual testing of all scenarios