//! Map Tools
//!
//! MCP tools for map and token placement management.

use mimir_core::models::campaign::LightingMode;
use mimir_core::services::{CreateMapInput, CreateTokenInput, MapService, TokenService, UpdateMapInput};
use rust_mcp_sdk::schema::{Tool, ToolInputSchema};
use serde_json::{json, Value};
use std::sync::Arc;

use super::create_properties;
use crate::context::McpContext;
use crate::response::McpResponse;
use crate::McpError;

// =============================================================================
// Tool Definitions
// =============================================================================

pub fn create_map_tool() -> Tool {
    Tool {
        name: "create_map".to_string(),
        description: Some(
            "Upload a UVTT file to create a new map. The file_path must point to a .uvtt file on disk."
                .to_string(),
        ),
        input_schema: ToolInputSchema::new(
            vec!["name".to_string(), "file_path".to_string()],
            create_properties(vec![
                ("name", "string", "Display name for the map"),
                ("file_path", "string", "Absolute path to the .uvtt file on disk"),
                ("module_id", "string", "Module ID to assign the map to (optional - omit for campaign-level map)"),
                ("description", "string", "Optional description of the map"),
                ("lighting_mode", "string", "Initial lighting: bright, dim, or dark (default: bright)"),
            ]),
            None,
        ),
        title: None,
        annotations: None,
        icons: vec![],
        execution: None,
        output_schema: None,
        meta: None,
    }
}

pub fn list_maps_tool() -> Tool {
    Tool {
        name: "list_maps".to_string(),
        description: Some(
            "List maps in the active campaign. Optionally filter by module. Omit module_id to list campaign-level maps."
                .to_string(),
        ),
        input_schema: ToolInputSchema::new(
            vec![],
            create_properties(vec![
                ("module_id", "string", "Filter by module ID (optional - omit for campaign-level maps)"),
            ]),
            None,
        ),
        title: None,
        annotations: None,
        icons: vec![],
        execution: None,
        output_schema: None,
        meta: None,
    }
}

pub fn get_map_tool() -> Tool {
    Tool {
        name: "get_map".to_string(),
        description: Some("Get detailed information about a map including token placements".to_string()),
        input_schema: ToolInputSchema::new(
            vec!["map_id".to_string()],
            create_properties(vec![("map_id", "string", "The ID of the map")]),
            None,
        ),
        title: None,
        annotations: None,
        icons: vec![],
        execution: None,
        output_schema: None,
        meta: None,
    }
}

pub fn update_map_tool() -> Tool {
    Tool {
        name: "update_map".to_string(),
        description: Some("Update map metadata (name, description, lighting, fog)".to_string()),
        input_schema: ToolInputSchema::new(
            vec!["map_id".to_string()],
            create_properties(vec![
                ("map_id", "string", "The ID of the map"),
                ("name", "string", "New display name"),
                ("description", "string", "New description"),
                ("lighting_mode", "string", "Lighting mode: bright, dim, or dark"),
                ("module_id", "string", "Move map to a module (or 'campaign' to move to campaign level)"),
            ]),
            None,
        ),
        title: None,
        annotations: None,
        icons: vec![],
        execution: None,
        output_schema: None,
        meta: None,
    }
}

pub fn delete_map_tool() -> Tool {
    Tool {
        name: "delete_map".to_string(),
        description: Some("Delete a map and its associated UVTT asset".to_string()),
        input_schema: ToolInputSchema::new(
            vec!["map_id".to_string()],
            create_properties(vec![("map_id", "string", "The ID of the map to delete")]),
            None,
        ),
        title: None,
        annotations: None,
        icons: vec![],
        execution: None,
        output_schema: None,
        meta: None,
    }
}

pub fn add_token_to_map_tool() -> Tool {
    Tool {
        name: "add_token_to_map".to_string(),
        description: Some(
            "Add a monster or NPC token to a map. Provide either module_monster_id or module_npc_id. Coordinates default to (0,0) - user can reposition in the UI."
                .to_string(),
        ),
        input_schema: ToolInputSchema::new(
            vec!["map_id".to_string()],
            create_properties(vec![
                ("map_id", "string", "The ID of the map"),
                ("module_monster_id", "string", "ID of the module monster to place (mutually exclusive with module_npc_id)"),
                ("module_npc_id", "string", "ID of the module NPC to place (mutually exclusive with module_monster_id)"),
                ("grid_x", "integer", "Grid X coordinate (default: 0)"),
                ("grid_y", "integer", "Grid Y coordinate (default: 0)"),
                ("label", "string", "Optional override label for the token"),
                ("faction_color", "string", "Faction color as hex (e.g. #FF0000)"),
                ("hidden", "boolean", "Whether token is hidden from players (default: false)"),
            ]),
            None,
        ),
        title: None,
        annotations: None,
        icons: vec![],
        execution: None,
        output_schema: None,
        meta: None,
    }
}

pub fn list_tokens_on_map_tool() -> Tool {
    Tool {
        name: "list_tokens_on_map".to_string(),
        description: Some("List all token placements on a map".to_string()),
        input_schema: ToolInputSchema::new(
            vec!["map_id".to_string()],
            create_properties(vec![
                ("map_id", "string", "The ID of the map"),
                ("visible_only", "boolean", "Only show visible tokens (default: false)"),
            ]),
            None,
        ),
        title: None,
        annotations: None,
        icons: vec![],
        execution: None,
        output_schema: None,
        meta: None,
    }
}

pub fn remove_token_tool() -> Tool {
    Tool {
        name: "remove_token".to_string(),
        description: Some("Remove a token placement from a map".to_string()),
        input_schema: ToolInputSchema::new(
            vec!["token_id".to_string()],
            create_properties(vec![("token_id", "string", "The ID of the token placement to remove")]),
            None,
        ),
        title: None,
        annotations: None,
        icons: vec![],
        execution: None,
        output_schema: None,
        meta: None,
    }
}

// =============================================================================
// Tool Implementations
// =============================================================================

/// Get the app_data_dir from context (parent of assets_dir).
fn app_data_dir(ctx: &Arc<McpContext>) -> std::path::PathBuf {
    ctx.assets_dir
        .parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| ctx.assets_dir.clone())
}

fn parse_lighting_mode(s: &str) -> Result<LightingMode, McpError> {
    match s.to_lowercase().as_str() {
        "bright" => Ok(LightingMode::Bright),
        "dim" => Ok(LightingMode::Dim),
        "dark" => Ok(LightingMode::Dark),
        _ => Err(McpError::InvalidArguments(format!(
            "Invalid lighting_mode '{}'. Must be bright, dim, or dark.",
            s
        ))),
    }
}

pub async fn create_map(ctx: &Arc<McpContext>, args: Value) -> Result<Value, McpError> {
    let campaign_id = ctx
        .get_active_campaign_id()
        .ok_or(McpError::NoActiveCampaign)?;

    let name = args
        .get("name")
        .and_then(|v| v.as_str())
        .ok_or_else(|| McpError::InvalidArguments("name is required".to_string()))?;

    let file_path = args
        .get("file_path")
        .and_then(|v| v.as_str())
        .ok_or_else(|| McpError::InvalidArguments("file_path is required".to_string()))?;

    let module_id = args.get("module_id").and_then(|v| v.as_str());
    let description = args.get("description").and_then(|v| v.as_str());
    let lighting_mode = args
        .get("lighting_mode")
        .and_then(|v| v.as_str())
        .map(parse_lighting_mode)
        .transpose()?;

    // Read the UVTT file from disk
    let uvtt_data = std::fs::read(file_path).map_err(|e| {
        McpError::InvalidArguments(format!("Failed to read file '{}': {}", file_path, e))
    })?;

    // Extract filename from path
    let filename = std::path::Path::new(file_path)
        .file_name()
        .and_then(|f| f.to_str())
        .unwrap_or("map.uvtt")
        .to_string();

    let mut db = ctx.connect()?;
    let data_dir = app_data_dir(ctx);
    let mut service = MapService::new(&mut db, &data_dir);

    let mut input = if let Some(mid) = module_id {
        CreateMapInput::for_module(&campaign_id, mid, name, &filename, uvtt_data)
    } else {
        CreateMapInput::for_campaign(&campaign_id, name, &filename, uvtt_data)
    };

    if let Some(desc) = description {
        input = input.with_description(desc);
    }
    if let Some(mode) = lighting_mode {
        input = input.with_lighting_mode(mode);
    }

    let map = service
        .create(input)
        .map_err(|e| McpError::Internal(e.to_string()))?;

    McpResponse::created("map", json!({
        "id": map.id,
        "name": map.name,
        "description": map.description,
        "module_id": map.module_id,
        "lighting_mode": map.lighting_mode,
        "fog_enabled": map.fog_enabled != 0,
        "sort_order": map.sort_order
    }))
}

pub async fn list_maps(ctx: &Arc<McpContext>, args: Value) -> Result<Value, McpError> {
    let campaign_id = ctx
        .get_active_campaign_id()
        .ok_or(McpError::NoActiveCampaign)?;

    let module_id = args.get("module_id").and_then(|v| v.as_str());

    let mut db = ctx.connect()?;
    let data_dir = app_data_dir(ctx);
    let mut service = MapService::new(&mut db, &data_dir);

    let maps = if let Some(mid) = module_id {
        service
            .list_for_module(mid)
            .map_err(|e| McpError::Internal(e.to_string()))?
    } else {
        service
            .list_campaign_level(&campaign_id)
            .map_err(|e| McpError::Internal(e.to_string()))?
    };

    let map_data: Vec<Value> = maps
        .iter()
        .map(|m| {
            json!({
                "id": m.id,
                "name": m.name,
                "description": m.description,
                "module_id": m.module_id,
                "lighting_mode": m.lighting_mode,
                "sort_order": m.sort_order
            })
        })
        .collect();

    McpResponse::list("maps", map_data)
}

pub async fn get_map(ctx: &Arc<McpContext>, args: Value) -> Result<Value, McpError> {
    let map_id = args
        .get("map_id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| McpError::InvalidArguments("map_id is required".to_string()))?;

    let mut db = ctx.connect()?;
    let data_dir = app_data_dir(ctx);

    let map = {
        let mut service = MapService::new(&mut db, &data_dir);
        service
            .get(map_id)
            .map_err(|e| McpError::Internal(e.to_string()))?
            .ok_or_else(|| McpError::InvalidArguments(format!("Map '{}' not found", map_id)))?
    };

    // Get enriched token placements
    let tokens = TokenService::new(&mut db, &data_dir)
        .list(map_id)
        .map_err(|e| McpError::Internal(e.to_string()))?;

    let token_data: Vec<Value> = tokens
        .iter()
        .map(|t| {
            json!({
                "id": t.id,
                "name": t.name,
                "token_type": t.token_type,
                "size": t.size,
                "grid_x": t.grid_x,
                "grid_y": t.grid_y,
                "visible_to_players": t.visible_to_players,
                "color": t.color,
                "monster_id": t.monster_id,
                "character_id": t.character_id
            })
        })
        .collect();

    McpResponse::ok(json!({
        "map": {
            "id": map.id,
            "name": map.name,
            "description": map.description,
            "module_id": map.module_id,
            "lighting_mode": map.lighting_mode,
            "fog_enabled": map.fog_enabled != 0,
            "sort_order": map.sort_order
        },
        "tokens": token_data
    }))
}

pub async fn update_map(ctx: &Arc<McpContext>, args: Value) -> Result<Value, McpError> {
    let map_id = args
        .get("map_id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| McpError::InvalidArguments("map_id is required".to_string()))?;

    let mut update = UpdateMapInput::default();

    if let Some(name) = args.get("name").and_then(|v| v.as_str()) {
        update.name = Some(name.to_string());
    }
    if let Some(desc) = args.get("description").and_then(|v| v.as_str()) {
        update.description = Some(Some(desc.to_string()));
    }
    if let Some(mode) = args.get("lighting_mode").and_then(|v| v.as_str()) {
        update.lighting_mode = Some(parse_lighting_mode(mode)?);
    }
    if let Some(mid) = args.get("module_id").and_then(|v| v.as_str()) {
        if mid == "campaign" {
            update.module_id = Some(None); // Move to campaign level
        } else {
            update.module_id = Some(Some(mid.to_string()));
        }
    }

    let mut db = ctx.connect()?;
    let data_dir = app_data_dir(ctx);
    let mut service = MapService::new(&mut db, &data_dir);

    let map = service
        .update(map_id, update)
        .map_err(|e| McpError::Internal(e.to_string()))?;

    McpResponse::updated("map", json!({
        "id": map.id,
        "name": map.name,
        "description": map.description,
        "module_id": map.module_id,
        "lighting_mode": map.lighting_mode,
        "fog_enabled": map.fog_enabled != 0,
        "sort_order": map.sort_order
    }))
}

pub async fn delete_map(ctx: &Arc<McpContext>, args: Value) -> Result<Value, McpError> {
    let map_id = args
        .get("map_id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| McpError::InvalidArguments("map_id is required".to_string()))?;

    let mut db = ctx.connect()?;
    let data_dir = app_data_dir(ctx);
    let mut service = MapService::new(&mut db, &data_dir);

    service
        .delete(map_id)
        .map_err(|e| McpError::Internal(e.to_string()))?;

    McpResponse::deleted(map_id)
}

pub async fn add_token_to_map(ctx: &Arc<McpContext>, args: Value) -> Result<Value, McpError> {
    let map_id = args
        .get("map_id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| McpError::InvalidArguments("map_id is required".to_string()))?;

    let module_monster_id = args.get("module_monster_id").and_then(|v| v.as_str());
    let module_npc_id = args.get("module_npc_id").and_then(|v| v.as_str());
    let label = args.get("label").and_then(|v| v.as_str());

    // Validation: need either monster, npc, or label (for PC tokens)
    if module_monster_id.is_none() && module_npc_id.is_none() && label.is_none() {
        return Err(McpError::InvalidArguments(
            "Either module_monster_id, module_npc_id, or label is required".to_string(),
        ));
    }
    if module_monster_id.is_some() && module_npc_id.is_some() {
        return Err(McpError::InvalidArguments(
            "Provide either module_monster_id or module_npc_id, not both".to_string(),
        ));
    }

    let grid_x = args.get("grid_x").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
    let grid_y = args.get("grid_y").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
    let faction_color = args.get("faction_color").and_then(|v| v.as_str());
    let hidden = args.get("hidden").and_then(|v| v.as_bool()).unwrap_or(false);

    let input = CreateTokenInput {
        map_id: map_id.to_string(),
        module_monster_id: module_monster_id.map(|s| s.to_string()),
        module_npc_id: module_npc_id.map(|s| s.to_string()),
        grid_x,
        grid_y,
        label: label.map(|s| s.to_string()),
        faction_color: faction_color.map(|s| s.to_string()),
        hidden,
    };

    let mut db = ctx.connect()?;
    let data_dir = app_data_dir(ctx);

    let token = TokenService::new(&mut db, &data_dir)
        .create(input)
        .map_err(|e| McpError::Internal(e.to_string()))?;

    McpResponse::added("token", json!({
        "id": token.id,
        "map_id": token.map_id,
        "name": token.name,
        "token_type": token.token_type,
        "size": token.size,
        "grid_x": token.grid_x,
        "grid_y": token.grid_y,
        "visible_to_players": token.visible_to_players,
        "color": token.color
    }))
}

pub async fn list_tokens_on_map(ctx: &Arc<McpContext>, args: Value) -> Result<Value, McpError> {
    let map_id = args
        .get("map_id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| McpError::InvalidArguments("map_id is required".to_string()))?;

    let visible_only = args
        .get("visible_only")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    let mut db = ctx.connect()?;
    let data_dir = app_data_dir(ctx);
    let mut service = TokenService::new(&mut db, &data_dir);

    let tokens = if visible_only {
        service.list_visible(map_id)
    } else {
        service.list(map_id)
    }
    .map_err(|e| McpError::Internal(e.to_string()))?;

    let token_data: Vec<Value> = tokens
        .iter()
        .map(|t| {
            json!({
                "id": t.id,
                "name": t.name,
                "token_type": t.token_type,
                "size": t.size,
                "grid_x": t.grid_x,
                "grid_y": t.grid_y,
                "visible_to_players": t.visible_to_players,
                "color": t.color,
                "monster_id": t.monster_id,
                "character_id": t.character_id
            })
        })
        .collect();

    McpResponse::list("tokens", token_data)
}

pub async fn remove_token(ctx: &Arc<McpContext>, args: Value) -> Result<Value, McpError> {
    let token_id = args
        .get("token_id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| McpError::InvalidArguments("token_id is required".to_string()))?;

    let mut db = ctx.connect()?;
    let data_dir = app_data_dir(ctx);

    TokenService::new(&mut db, &data_dir)
        .delete(token_id)
        .map_err(|e| McpError::Internal(e.to_string()))?;

    McpResponse::removed(token_id)
}
