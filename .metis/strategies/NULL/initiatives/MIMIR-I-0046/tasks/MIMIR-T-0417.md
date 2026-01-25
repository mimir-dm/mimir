---
id: map-fog-state-persistence
level: task
title: "Map fog state persistence"
short_code: "MIMIR-T-0417"
created_at: 2026-01-25T02:44:12.277136+00:00
updated_at: 2026-01-25T16:28:47.892839+00:00
parent: MIMIR-I-0046
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: MIMIR-I-0046
---

# Map fog state persistence

## Parent Initiative

[[MIMIR-I-0046]] - Map & Token VTT System

## Objective

Implement the fog of war system with revealed areas. The map already has `lighting_mode` column. We need:
1. New `fog_revealed_areas` table for storing revealed regions
2. DAL for fog CRUD
3. 9 Tauri commands for fog operations

**Note**: The `useFog.ts` composable is already implemented and calls these commands.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] Migration creates `fog_revealed_areas` table
- [ ] Add `fog_enabled` column to maps table
- [ ] DAL functions for fog state CRUD
- [ ] `get_fog_state` command - returns fog_enabled + revealed_areas
- [ ] `toggle_fog` command - toggle fog_enabled
- [ ] `enable_fog` / `disable_fog` commands
- [ ] `reveal_rect` command - reveal rectangular area
- [ ] `reveal_circle` command - reveal circular area
- [ ] `reveal_all` command - reveal entire map
- [ ] `delete_revealed_area` command - remove revealed area
- [ ] `reset_fog` command - clear all revealed areas

## Implementation Notes

### Schema (Migration 021_fog)

```sql
-- Add fog_enabled to maps
ALTER TABLE maps ADD COLUMN fog_enabled INTEGER NOT NULL DEFAULT 0;

-- Revealed areas table
CREATE TABLE fog_revealed_areas (
    id TEXT PRIMARY KEY NOT NULL,
    map_id TEXT NOT NULL REFERENCES maps(id) ON DELETE CASCADE,
    x REAL NOT NULL,
    y REAL NOT NULL,
    width REAL NOT NULL,
    height REAL NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_fog_revealed_areas_map ON fog_revealed_areas(map_id);
```

### Models

```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct FogRevealedArea {
    pub id: String,
    pub map_id: String,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

#[derive(Debug, Serialize)]
pub struct FogState {
    pub map_id: String,
    pub fog_enabled: bool,
    pub revealed_areas: Vec<FogRevealedArea>,
}
```

### Commands

```rust
#[tauri::command]
pub fn get_fog_state(state: State<'_, AppState>, map_id: String) -> ApiResponse<FogState>

#[tauri::command]
pub fn toggle_fog(state: State<'_, AppState>, map_id: String) -> ApiResponse<bool>

#[tauri::command]
pub fn enable_fog(state: State<'_, AppState>, map_id: String) -> ApiResponse<()>

#[tauri::command]
pub fn disable_fog(state: State<'_, AppState>, map_id: String) -> ApiResponse<()>

#[tauri::command]
pub fn reveal_rect(state: State<'_, AppState>, request: RevealRectRequest) -> ApiResponse<FogRevealedArea>

#[tauri::command]
pub fn reveal_circle(state: State<'_, AppState>, request: RevealCircleRequest) -> ApiResponse<FogRevealedArea>

#[tauri::command]
pub fn reveal_all(state: State<'_, AppState>, request: RevealAllRequest) -> ApiResponse<FogRevealedArea>

#[tauri::command]
pub fn delete_revealed_area(state: State<'_, AppState>, id: String) -> ApiResponse<()>

#[tauri::command]
pub fn reset_fog(state: State<'_, AppState>, map_id: String) -> ApiResponse<i32>
```

### Files to Create/Modify

- `crates/mimir-core/migrations/021_fog/up.sql` - New migration
- `crates/mimir-core/migrations/021_fog/down.sql` - Rollback
- `crates/mimir-core/src/models/campaign/fog.rs` - New models
- `crates/mimir-core/src/models/campaign/mod.rs` - Export fog models
- `crates/mimir-core/src/dal/campaign/fog.rs` - New DAL
- `crates/mimir-core/src/dal/campaign/mod.rs` - Export fog DAL
- `crates/mimir-core/src/schema.rs` - Update diesel schema
- `crates/mimir/src/commands/map.rs` - Add fog commands
- `crates/mimir/src/main.rs` - Register commands

## Status Updates

### Completed 2026-01-25

**Migration (021_fog):**
- Added `fog_enabled` column to maps table
- Created `fog_revealed_areas` table with (id, map_id, x, y, width, height, created_at)
- Added index on map_id

**Models (crates/mimir-core/src/models/campaign/fog.rs):**
- `FogRevealedArea` - revealed area entity
- `NewFogRevealedArea` - insert struct with `rect()` and `circle()` constructors
- `FogState` - composite state (map_id, fog_enabled, revealed_areas)

**Map Model Updates:**
- Added `fog_enabled` field to Map struct
- Added `is_fog_enabled()` helper method
- Added `enable_fog()`, `disable_fog()`, `set_fog_enabled()` to UpdateMap

**DAL (crates/mimir-core/src/dal/campaign/fog.rs):**
- `insert_fog_revealed_area`
- `get_fog_revealed_area`
- `list_fog_revealed_areas`
- `delete_fog_revealed_area`
- `delete_all_fog_revealed_areas`
- `count_fog_revealed_areas`

**Commands (9 total):**
- `get_fog_state` - returns FogState with enabled flag and areas
- `toggle_fog` - toggle fog enabled
- `enable_fog` / `disable_fog` - explicit enable/disable
- `reveal_rect` - reveal rectangular area
- `reveal_circle` - reveal circular area (stored as bounding box)
- `reveal_all` - reveal entire map
- `delete_revealed_area` - remove one revealed area
- `reset_fog` - clear all revealed areas

All commands registered in main.rs and build verified.