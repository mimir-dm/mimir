---
id: player-display-ipc-event-system
level: task
title: "Player display IPC event system"
short_code: "MIMIR-T-0421"
created_at: 2026-01-25T02:44:22.585519+00:00
updated_at: 2026-01-25T16:32:46.015565+00:00
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

# Player display IPC event system

## Parent Initiative

[[MIMIR-I-0046]] - Map & Token VTT System

## Objective

Implement backend Tauri commands for player display window management. The frontend composable `usePlayerDisplay.ts` already exists and calls these commands - we need the backend implementation.

**Frontend already exists** at `composables/usePlayerDisplay.ts` - fully implemented
**Backend commands missing** - need to implement 7 commands

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `is_player_display_open` - check if window exists
- [ ] `open_player_display_window` - create the window
- [ ] `close_player_display_window` - destroy the window
- [ ] `toggle_player_display_fullscreen` - fullscreen toggle
- [ ] `send_map_to_display` - emit map data to window
- [ ] `update_display_viewport` - emit viewport to window
- [ ] `toggle_display_blackout` - emit blackout state
- [ ] All commands registered in main.rs

## Implementation Notes

### Tauri Window Management

```rust
use tauri::{AppHandle, Manager, WindowBuilder, WindowUrl};

#[tauri::command]
pub fn is_player_display_open(app: AppHandle) -> ApiResponse<bool> {
    let exists = app.get_window("player-display").is_some();
    ApiResponse::ok(exists)
}

#[tauri::command]
pub fn open_player_display_window(app: AppHandle) -> ApiResponse<()> {
    if app.get_window("player-display").is_some() {
        return ApiResponse::ok(());  // Already open
    }
    
    WindowBuilder::new(&app, "player-display", WindowUrl::App("/player-display".into()))
        .title("Player Display")
        .inner_size(1280.0, 720.0)
        .resizable(true)
        .build()
        .map_err(|e| format!("Failed to create window: {}", e))?;
    
    ApiResponse::ok(())
}

#[tauri::command]
pub fn close_player_display_window(app: AppHandle) -> ApiResponse<()> {
    if let Some(window) = app.get_window("player-display") {
        window.close().ok();
    }
    ApiResponse::ok(())
}

#[tauri::command]
pub fn toggle_player_display_fullscreen(app: AppHandle) -> ApiResponse<bool> {
    if let Some(window) = app.get_window("player-display") {
        let is_fullscreen = window.is_fullscreen().unwrap_or(false);
        window.set_fullscreen(!is_fullscreen).ok();
        ApiResponse::ok(!is_fullscreen)
    } else {
        ApiResponse::err("Player display window not open")
    }
}
```

### IPC Event Emission

```rust
#[tauri::command]
pub fn send_map_to_display(
    app: AppHandle,
    map_id: i32,
    grid_type: String,
    grid_size_px: Option<i32>,
    // ... other params
) -> ApiResponse<()> {
    if let Some(window) = app.get_window("player-display") {
        window.emit("map-update", payload)?;
        ApiResponse::ok(())
    } else {
        ApiResponse::err("Player display not open")
    }
}

#[tauri::command]
pub fn update_display_viewport(
    app: AppHandle,
    x: f64,
    y: f64,
    zoom: f64,
) -> ApiResponse<()> {
    if let Some(window) = app.get_window("player-display") {
        window.emit("viewport-update", json!({ "x": x, "y": y, "zoom": zoom }))?;
        ApiResponse::ok(())
    } else {
        ApiResponse::err("Player display not open")
    }
}

#[tauri::command]
pub fn toggle_display_blackout(app: AppHandle, is_blackout: bool) -> ApiResponse<()> {
    if let Some(window) = app.get_window("player-display") {
        window.emit("blackout-update", json!({ "isBlackout": is_blackout }))?;
        ApiResponse::ok(())
    } else {
        ApiResponse::err("Player display not open")
    }
}
```

### Files to Create/Modify

- `crates/mimir/src/commands/player_display.rs` - NEW file
- `crates/mimir/src/commands/mod.rs` - Export player_display
- `crates/mimir/src/main.rs` - Register commands

### Dependencies

None - uses Tauri window APIs

## Status Updates

### Completed 2026-01-25

Created `crates/mimir/src/commands/player_display.rs` with 7 Tauri commands:

**Window Management:**
- `is_player_display_open` - Check if window exists
- `open_player_display_window` - Create the player display window
- `close_player_display_window` - Destroy the window
- `toggle_player_display_fullscreen` - Toggle fullscreen mode

**IPC Events:**
- `send_map_to_display` - Emit map data to window via "map-update" event
- `update_display_viewport` - Emit viewport changes via "viewport-update" event
- `toggle_display_blackout` - Emit blackout state via "blackout-update" event

Uses Tauri v2 APIs:
- `WebviewWindowBuilder` for window creation
- `app.get_webview_window()` for window lookup
- `window.emit()` for IPC events

Commands registered in main.rs and exported from commands/mod.rs. Build verified.