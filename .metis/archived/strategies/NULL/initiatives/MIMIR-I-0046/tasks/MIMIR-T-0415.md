---
id: token-image-serving
level: task
title: "Token image serving"
short_code: "MIMIR-T-0415"
created_at: 2026-01-25T02:44:11.085238+00:00
updated_at: 2026-01-25T16:17:35.662337+00:00
parent: MIMIR-I-0046
blocked_by: []
archived: true

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: MIMIR-I-0046
---

# Token image serving

## Parent Initiative

[[MIMIR-I-0046]] - Map & Token VTT System

## Objective

Implement the `serve_token_image` command that serves monster token images from the catalog assets as base64 data URLs.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `serve_token_image` command implemented
- [ ] Looks up token's monster_id to get token_image_path
- [ ] Serves image from catalog assets as base64 data URL
- [ ] Returns null/error gracefully if no image exists
- [ ] Command registered in main.rs

## Implementation Notes

### Flow

1. Receive token_id
2. Query token to get monster_id
3. If no monster_id, return null (no image for PCs/markers)
4. Query monster to get token_image_path
5. If no token_image_path, return null
6. Read image from `{assets_dir}/catalog/{token_image_path}`
7. Encode as base64 data URL with appropriate MIME type
8. Return data URL string

### Command Signature

```rust
#[tauri::command]
pub async fn serve_token_image(
    state: State<'_, AppState>,
    token_id: i64,
) -> ApiResponse<Option<String>>
```

### Image Path Pattern

Images stored at: `{APP_DATA_DIR}/assets/catalog/bestiary/tokens/{source}/{name}.webp`

Example: `assets/catalog/bestiary/tokens/MM/Goblin.webp`

### Files to Create/Modify

- `crates/mimir/src/commands/map.rs` (add command)
- `crates/mimir/src/main.rs` (register command)

### Dependencies

- MIMIR-T-0413 (Token repository - to look up token's monster_id)
- Existing monster catalog with token_image_path

## Status Updates

### Completed 2026-01-25

Implemented `serve_token_image` command in `crates/mimir/src/commands/module.rs`:

**Flow:**
1. Gets token placement by ID
2. Looks up module_monster from token's module_monster_id
3. Looks up catalog monster by name/source to get token_image_path
4. Reads image file from `{app_dir}/{token_image_path}`
5. Encodes as base64 data URL with appropriate MIME type (webp/png/jpeg)

**Features:**
- Gracefully returns null for tokens without images (NPCs, markers)
- Gracefully handles missing image files with warning log
- Supports webp, png, and jpeg formats

Command registered in main.rs and build verified.