---
id: create-tokenservice-with-crud
level: task
title: "Create TokenService with CRUD operations"
short_code: "MIMIR-T-0206"
created_at: 2025-12-21T22:15:21.002760+00:00
updated_at: 2025-12-22T01:46:40.352053+00:00
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

# Create TokenService with CRUD operations

## Parent Initiative
[[MIMIR-I-0015]] - Visual Display System

## Objective
Create a TokenService in mimir-dm-core that provides CRUD operations for tokens, plus Tauri commands to expose this functionality to the frontend.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] TokenService with create, read, update, delete operations
- [ ] List tokens by map_id
- [ ] Bulk update for moving multiple tokens
- [ ] Tauri commands: create_token, get_token, list_tokens, update_token, delete_token
- [ ] Update token position command for drag operations
- [ ] Toggle token visibility command

## Implementation Notes

### TokenService Methods

```rust
impl TokenService {
    pub fn create_token(&mut self, new_token: NewToken) -> Result<Token>;
    pub fn get_token(&mut self, id: i32) -> Result<Token>;
    pub fn list_tokens_for_map(&mut self, map_id: i32) -> Result<Vec<Token>>;
    pub fn list_visible_tokens_for_map(&mut self, map_id: i32) -> Result<Vec<Token>>;
    pub fn update_token(&mut self, id: i32, update: UpdateToken) -> Result<Token>;
    pub fn update_token_position(&mut self, id: i32, x: f32, y: f32) -> Result<Token>;
    pub fn toggle_token_visibility(&mut self, id: i32) -> Result<Token>;
    pub fn delete_token(&mut self, id: i32) -> Result<()>;
    pub fn delete_tokens_for_map(&mut self, map_id: i32) -> Result<usize>;
}
```

### Tauri Commands

```rust
#[tauri::command]
pub async fn create_token(request: CreateTokenRequest, state: State<'_, AppState>) -> Result<ApiResponse<Token>, ApiError>;

#[tauri::command]
pub async fn list_tokens(map_id: i32, state: State<'_, AppState>) -> Result<ApiResponse<Vec<Token>>, ApiError>;

#[tauri::command]
pub async fn update_token_position(id: i32, x: f32, y: f32, state: State<'_, AppState>) -> Result<ApiResponse<Token>, ApiError>;

#[tauri::command]
pub async fn toggle_token_visibility(id: i32, state: State<'_, AppState>) -> Result<ApiResponse<Token>, ApiError>;

#[tauri::command]
pub async fn delete_token(id: i32, state: State<'_, AppState>) -> Result<ApiResponse<()>, ApiError>;
```

### Files to Create/Modify
- `crates/mimir-dm-core/src/services/token_service.rs`
- `crates/mimir-dm-core/src/services/mod.rs` (add export)
- `crates/mimir-dm/src/commands/campaign/tokens.rs`
- `crates/mimir-dm/src/commands/campaign/mod.rs` (add export)
- `crates/mimir-dm/src/main.rs` (register commands)

### Dependencies
- T-0205 (tokens schema and model) must be complete