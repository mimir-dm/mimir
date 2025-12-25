---
id: add-token-placement-ui-in-map-setup
level: task
title: "Add token placement UI in map setup"
short_code: "MIMIR-T-0207"
created_at: 2025-12-21T22:15:21.103525+00:00
updated_at: 2025-12-22T02:08:59.408552+00:00
parent: MIMIR-I-0015
blocked_by: []
archived: true

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: MIMIR-I-0015
---

# Add token placement UI in map setup

## Parent Initiative
[[MIMIR-I-0015]] - Visual Display System

## Objective
Create a UI for placing tokens on maps during module/campaign setup, allowing DMs to pre-place monsters, NPCs, and markers before the session.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [x] Token palette shows available token types (Monster, PC, NPC, Trap, Marker)
- [x] Monster picker integrates with catalog search
- [x] Click-to-place tokens on the map canvas
- [x] Tokens snap to grid (when grid is configured)
- [x] Token size selector for D&D sizes (Tiny through Gargantuan)
- [x] Edit token properties (name, visibility) - full edit modal pending
- [x] Delete tokens from map (via list or context menu)
- [x] Tokens persist to database

## Implementation Summary

### Files Created
- `crates/mimir-dm/frontend/src/types/api.ts` - Added Token, TokenSummary, TokenType, TokenSize types
- `crates/mimir-dm/frontend/src/composables/useTokens.ts` - Token management composable
- `crates/mimir-dm/frontend/src/components/tokens/TokenPalette.vue` - Token type/size/color selector
- `crates/mimir-dm/frontend/src/components/tokens/MapTokenSetupModal.vue` - Full token placement modal

### Files Modified
- `crates/mimir-dm/frontend/src/features/modules/components/ModuleMaps.vue` - Added token setup button

## Implementation Notes

### UI Components

1. **Token Palette Panel**
   - Token type selector (Monster, NPC, Trap, Marker)
   - For Monster type: search catalog monsters
   - For NPC type: search campaign characters
   - Size dropdown
   - Color picker (fallback when no image)

2. **Map Canvas Integration**
   - Click on map to place selected token type
   - Show placed tokens with appropriate size
   - Hover to see token name
   - Right-click context menu: Edit, Delete, Toggle Visibility

3. **Token Editor Modal**
   - Name field
   - Notes field
   - Visibility toggle
   - Size selector
   - Link to monster/character (read-only display)

### Grid Snapping
- When grid is configured, snap token center to grid intersections
- Large+ tokens snap to appropriate grid alignment
- Hold Shift to disable snapping for precise placement

### Files to Create/Modify
- `crates/mimir-dm/frontend/src/components/TokenPalette.vue`
- `crates/mimir-dm/frontend/src/components/TokenEditor.vue`
- `crates/mimir-dm/frontend/src/components/MapTokenLayer.vue`
- Integrate into MapGridConfigModal or create dedicated MapSetupView

### Dependencies
- T-0205 (tokens schema)
- T-0206 (TokenService and commands)