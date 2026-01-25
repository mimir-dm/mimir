---
id: light-source-command-updates
level: task
title: "Light source command updates"
short_code: "MIMIR-T-0416"
created_at: 2026-01-25T02:44:11.723084+00:00
updated_at: 2026-01-25T16:19:13.559755+00:00
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

# Light source command updates

## Parent Initiative

[[MIMIR-I-0046]] - Map & Token VTT System

## Objective

Add missing light source Tauri commands. The DAL already exists at `dal/campaign/light_source.rs`.

**Existing commands** (in map.rs):
- `list_light_sources` ✅
- `create_light_source` ✅
- `toggle_light_source` ✅
- `delete_light_source` ✅

**Missing commands** (5 to add):
- `create_torch` - Convenience preset
- `create_lantern` - Convenience preset
- `update_light_source` - Edit properties
- `move_light_source` - Update position
- `delete_all_light_sources` - Clear map

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `create_torch` command - preset with 20ft/40ft bright/dim
- [ ] `create_lantern` command - preset with 30ft/60ft bright/dim
- [ ] `update_light_source` command - edit name, radii, color
- [ ] `move_light_source` command - update position only
- [ ] `delete_all_light_sources` command - clear all lights on map
- [ ] All commands registered in main.rs

## Implementation Notes

### Existing DAL Functions (in `dal/campaign/light_source.rs`)

```rust
// Already available:
pub fn insert_light_source(conn, light: &NewLightSource) -> QueryResult<String>
pub fn update_light_source(conn, id: &str, update: &UpdateLightSource) -> QueryResult<usize>
pub fn delete_light_source(conn, id: &str) -> QueryResult<usize>
pub fn delete_all_light_sources(conn, map_id: &str) -> QueryResult<usize>
```

### Existing Model Presets (in `models/campaign/light_source.rs`)

```rust
impl NewLightSource {
    pub fn torch(...) -> Self  // 20ft bright, 40ft dim, #FFAA00
    pub fn lantern(...) -> Self  // 30ft bright, 60ft dim, #FFD700
}
```

### Commands to Add

```rust
#[tauri::command]
pub fn create_torch(state: State<'_, AppState>, map_id: String, x: i32, y: i32) -> ApiResponse<LightSourceResponse>

#[tauri::command]
pub fn create_lantern(state: State<'_, AppState>, map_id: String, x: i32, y: i32) -> ApiResponse<LightSourceResponse>

#[tauri::command]
pub fn update_light_source(state: State<'_, AppState>, id: String, request: UpdateLightSourceRequest) -> ApiResponse<LightSourceResponse>

#[tauri::command]
pub fn move_light_source(state: State<'_, AppState>, id: String, x: i32, y: i32) -> ApiResponse<LightSourceResponse>

#[tauri::command]
pub fn delete_all_light_sources(state: State<'_, AppState>, map_id: String) -> ApiResponse<i32>
```

### Files to Modify

- `crates/mimir/src/commands/map.rs` - Add commands
- `crates/mimir/src/main.rs` - Register commands

### Dependencies

None - DAL and model presets already exist

## Status Updates

### Completed 2026-01-25

Added 5 light source commands to `crates/mimir/src/commands/map.rs`:

**Preset Commands:**
- `create_torch` - Creates torch with 20ft/40ft radii and orange color
- `create_lantern` - Creates lantern with 30ft/60ft radii and gold color

**CRUD Commands:**
- `update_light_source` - Updates name, radii, color, active state
- `move_light_source` - Updates position only (optimized for drag)
- `delete_all_light_sources` - Clears all lights on a map

All commands registered in main.rs and build verified.