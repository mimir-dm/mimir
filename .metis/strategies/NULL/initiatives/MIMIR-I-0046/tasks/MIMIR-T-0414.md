---
id: token-tauri-commands
level: task
title: "Token Tauri commands"
short_code: "MIMIR-T-0414"
created_at: 2026-01-25T02:44:10.452553+00:00
updated_at: 2026-01-25T16:15:57.999664+00:00
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

# Token Tauri commands

## Parent Initiative

[[MIMIR-I-0046]] - Map & Token VTT System

## Objective

Add remaining Tauri commands for token CRUD operations. The DAL already exists at `dal/campaign/token_placement.rs` - we just need Tauri command wrappers.

**Note**: `list_tokens` already exists in `module.rs`. We need 6 additional commands.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `create_token` command - wraps `dal::insert_token_placement`
- [ ] `update_token` command - wraps `dal::update_token_placement`
- [ ] `update_token_position` command - optimized position-only update
- [ ] `delete_token` command - wraps `dal::delete_token_placement`
- [ ] `toggle_token_visibility` command - toggle hidden field
- [ ] `list_token_summaries` command - like list_tokens but with more data
- [ ] All commands registered in main.rs

## Implementation Notes

### Existing DAL Functions (in `dal/campaign/token_placement.rs`)

```rust
// Already available:
pub fn insert_token_placement(conn, placement: &NewTokenPlacement) -> QueryResult<String>
pub fn get_token_placement(conn, id: &str) -> QueryResult<TokenPlacement>
pub fn list_token_placements(conn, map_id: &str) -> QueryResult<Vec<TokenPlacement>>
pub fn list_visible_token_placements(conn, map_id: &str) -> QueryResult<Vec<TokenPlacement>>
pub fn update_token_placement(conn, id: &str, update: &UpdateTokenPlacement) -> QueryResult<usize>
pub fn delete_token_placement(conn, id: &str) -> QueryResult<usize>
```

### Commands to Add (in `map.rs`)

```rust
#[derive(Debug, Deserialize)]
pub struct CreateTokenRequest {
    pub map_id: String,
    pub module_monster_id: Option<String>,
    pub module_npc_id: Option<String>,
    pub grid_x: i32,
    pub grid_y: i32,
    pub label: Option<String>,
    pub faction_color: Option<String>,
    pub hidden: bool,
}

#[tauri::command]
pub fn create_token(state: State<'_, AppState>, request: CreateTokenRequest) -> ApiResponse<TokenWithData>

#[tauri::command]
pub fn update_token(state: State<'_, AppState>, id: String, request: UpdateTokenRequest) -> ApiResponse<TokenWithData>

#[tauri::command]
pub fn update_token_position(state: State<'_, AppState>, id: String, grid_x: i32, grid_y: i32) -> ApiResponse<TokenWithData>

#[tauri::command]
pub fn delete_token(state: State<'_, AppState>, id: String) -> ApiResponse<()>

#[tauri::command]
pub fn toggle_token_visibility(state: State<'_, AppState>, id: String) -> ApiResponse<TokenWithData>
```

### Files to Modify

- `crates/mimir/src/commands/map.rs` - Add token commands
- `crates/mimir/src/main.rs` - Register new commands

### Dependencies

None - DAL already exists and is complete

## Status Updates

### Completed 2026-01-25

All token Tauri commands implemented in `crates/mimir/src/commands/module.rs`:

**Commands added:**
- `create_token` - Creates token placement with monster/NPC reference
- `update_token` - Updates token properties (position, label, color, hidden)
- `update_token_position` - Optimized position-only update for drag operations
- `toggle_token_visibility` - Toggles hidden field
- `delete_token` - Removes token placement
- `list_token_summaries` - Alias for list_tokens for frontend compatibility

**Updated `list_tokens`** to return `TokenWithData` which includes:
- Flattened TokenPlacement fields
- token_type: "monster" or "npc"
- name: Resolved display name from module_monster or module_npc
- monster_source: Source book for monster tokens

**Helper function:** `resolve_token_names()` - Looks up monster/NPC names for a token

All commands registered in `main.rs` and build verified.