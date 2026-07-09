//! Map Tools
//!
//! MCP tools for map and token placement management. This family is
//! registry-based: typed argument structs, handlers, and one entry each in
//! `registered_tools()`.

use mimir_core::models::campaign::LightingMode;
use mimir_core::services::{
    CreateMapInput, CreateTokenInput, MapService, TokenService, UpdateMapInput,
};
use serde_json::{json, Value};
use std::sync::Arc;

use crate::context::McpContext;
use crate::registry::RegisteredTool;
use crate::response::McpResponse;
use crate::{tool, tool_args, McpError};

// =============================================================================
// Registration
// =============================================================================

/// All map-family tools.
pub fn registered_tools() -> Vec<RegisteredTool> {
    vec![
        tool!(
            "create_map",
            "Upload a UVTT file to create a new map in the active campaign. Requires an active campaign. file_path is an absolute path to a .uvtt file on the machine running this server (not the client) — confirm the path with the user if unsure.",
            CreateMapArgs,
            create_map
        ),
        tool!(
            "list_maps",
            "List maps in the active campaign. Requires an active campaign. Optionally filter by module. Omit module_id to list campaign-level maps.",
            ListMapsArgs,
            list_maps
        ),
        tool!(
            "get_map",
            "Get detailed information about a map including token placements",
            GetMapArgs,
            get_map
        ),
        tool!(
            "update_map",
            "Update map metadata (name, description, lighting, fog)",
            UpdateMapArgs,
            update_map
        ),
        tool!(
            "delete_map",
            "Delete a map and its associated UVTT asset",
            DeleteMapArgs,
            delete_map
        ),
        tool!(
            "add_token_to_map",
            "Add a monster or NPC token to a map. Provide either module_monster_id or module_npc_id. Coordinates default to (0,0) - user can reposition in the UI.",
            AddTokenArgs,
            add_token_to_map
        ),
        tool!(
            "list_tokens_on_map",
            "List all token placements on a map",
            ListTokensArgs,
            list_tokens_on_map
        ),
        tool!(
            "remove_token",
            "Remove a token placement from a map",
            RemoveTokenArgs,
            remove_token
        ),
    ]
}

// =============================================================================
// Arguments
// =============================================================================

tool_args! {
    pub struct CreateMapArgs {
        /// Display name for the map
        pub name: String,
        /// Absolute path to the .uvtt file on disk
        pub file_path: String,
        /// Module ID to assign the map to (optional - omit for campaign-level map)
        pub module_id: Option<String>,
        /// Optional description of the map
        pub description: Option<String>,
        /// Initial lighting: bright, dim, or dark (default: bright)
        pub lighting_mode: Option<String>,
    }
}

tool_args! {
    pub struct ListMapsArgs {
        /// Filter by module ID (optional - omit for campaign-level maps)
        pub module_id: Option<String>,
    }
}

tool_args! {
    pub struct GetMapArgs {
        /// The ID of the map
        pub map_id: String,
    }
}

tool_args! {
    pub struct UpdateMapArgs {
        /// The ID of the map
        pub map_id: String,
        /// New display name
        pub name: Option<String>,
        /// New description
        pub description: Option<String>,
        /// Lighting mode: bright, dim, or dark
        pub lighting_mode: Option<String>,
        /// Move map to a module (or 'campaign' to move to campaign level)
        pub module_id: Option<String>,
    }
}

tool_args! {
    pub struct DeleteMapArgs {
        /// The ID of the map to delete
        pub map_id: String,
    }
}

tool_args! {
    pub struct AddTokenArgs {
        /// The ID of the map
        pub map_id: String,
        /// ID of the module monster to place (mutually exclusive with module_npc_id)
        pub module_monster_id: Option<String>,
        /// ID of the module NPC to place (mutually exclusive with module_monster_id)
        pub module_npc_id: Option<String>,
        /// Grid X coordinate (default: 0)
        pub grid_x: Option<i64>,
        /// Grid Y coordinate (default: 0)
        pub grid_y: Option<i64>,
        /// Optional override label for the token
        pub label: Option<String>,
        /// Faction color as hex (e.g. #FF0000)
        pub faction_color: Option<String>,
        /// Whether token is hidden from players (default: false)
        pub hidden: Option<bool>,
    }
}

tool_args! {
    pub struct ListTokensArgs {
        /// The ID of the map
        pub map_id: String,
        /// Only show visible tokens (default: false)
        pub visible_only: Option<bool>,
    }
}

tool_args! {
    pub struct RemoveTokenArgs {
        /// The ID of the token placement to remove
        pub token_id: String,
    }
}

// =============================================================================
// Handlers
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

pub async fn create_map(ctx: &Arc<McpContext>, args: CreateMapArgs) -> Result<Value, McpError> {
    let campaign_id = ctx
        .get_active_campaign_id()
        .ok_or(McpError::NoActiveCampaign)?;

    let lighting_mode = args
        .lighting_mode
        .as_deref()
        .map(parse_lighting_mode)
        .transpose()?;

    // Read the UVTT file from disk
    let uvtt_data = std::fs::read(&args.file_path).map_err(|e| {
        McpError::InvalidArguments(format!("Failed to read file '{}': {}", args.file_path, e))
    })?;

    // Extract filename from path
    let filename = std::path::Path::new(&args.file_path)
        .file_name()
        .and_then(|f| f.to_str())
        .unwrap_or("map.uvtt")
        .to_string();

    let mut db = ctx.connect()?;
    let data_dir = app_data_dir(ctx);
    let mut service = MapService::new(&mut db, &data_dir);

    let mut input = if let Some(mid) = args.module_id.as_deref() {
        CreateMapInput::for_module(&campaign_id, mid, &args.name, &filename, uvtt_data)
    } else {
        CreateMapInput::for_campaign(&campaign_id, &args.name, &filename, uvtt_data)
    };

    if let Some(desc) = args.description {
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

pub async fn list_maps(ctx: &Arc<McpContext>, args: ListMapsArgs) -> Result<Value, McpError> {
    let campaign_id = ctx
        .get_active_campaign_id()
        .ok_or(McpError::NoActiveCampaign)?;

    let mut db = ctx.connect()?;
    let data_dir = app_data_dir(ctx);
    let mut service = MapService::new(&mut db, &data_dir);

    let maps = if let Some(mid) = args.module_id.as_deref() {
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

pub async fn get_map(ctx: &Arc<McpContext>, args: GetMapArgs) -> Result<Value, McpError> {
    let map_id = &args.map_id;

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

pub async fn update_map(ctx: &Arc<McpContext>, args: UpdateMapArgs) -> Result<Value, McpError> {
    let mut update = UpdateMapInput::default();

    if let Some(name) = args.name {
        update.name = Some(name);
    }
    if let Some(desc) = args.description {
        update.description = Some(Some(desc));
    }
    if let Some(mode) = args.lighting_mode.as_deref() {
        update.lighting_mode = Some(parse_lighting_mode(mode)?);
    }
    if let Some(mid) = args.module_id {
        if mid == "campaign" {
            update.module_id = Some(None); // Move to campaign level
        } else {
            update.module_id = Some(Some(mid));
        }
    }

    let mut db = ctx.connect()?;
    let data_dir = app_data_dir(ctx);
    let mut service = MapService::new(&mut db, &data_dir);

    let map = service
        .update(&args.map_id, update)
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

pub async fn delete_map(ctx: &Arc<McpContext>, args: DeleteMapArgs) -> Result<Value, McpError> {
    let mut db = ctx.connect()?;
    let data_dir = app_data_dir(ctx);
    let mut service = MapService::new(&mut db, &data_dir);

    service
        .delete(&args.map_id)
        .map_err(|e| McpError::Internal(e.to_string()))?;

    McpResponse::deleted(&args.map_id)
}

pub async fn add_token_to_map(
    ctx: &Arc<McpContext>,
    args: AddTokenArgs,
) -> Result<Value, McpError> {
    // Validation: need either monster, npc, or label (for PC tokens)
    if args.module_monster_id.is_none() && args.module_npc_id.is_none() && args.label.is_none() {
        return Err(McpError::InvalidArguments(
            "Either module_monster_id, module_npc_id, or label is required".to_string(),
        ));
    }
    if args.module_monster_id.is_some() && args.module_npc_id.is_some() {
        return Err(McpError::InvalidArguments(
            "Provide either module_monster_id or module_npc_id, not both".to_string(),
        ));
    }

    let input = CreateTokenInput {
        map_id: args.map_id,
        module_monster_id: args.module_monster_id,
        module_npc_id: args.module_npc_id,
        grid_x: args.grid_x.unwrap_or(0) as i32,
        grid_y: args.grid_y.unwrap_or(0) as i32,
        label: args.label,
        faction_color: args.faction_color,
        hidden: args.hidden.unwrap_or(false),
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

pub async fn list_tokens_on_map(
    ctx: &Arc<McpContext>,
    args: ListTokensArgs,
) -> Result<Value, McpError> {
    let map_id = &args.map_id;
    let visible_only = args.visible_only.unwrap_or(false);

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

pub async fn remove_token(
    ctx: &Arc<McpContext>,
    args: RemoveTokenArgs,
) -> Result<Value, McpError> {
    let mut db = ctx.connect()?;
    let data_dir = app_data_dir(ctx);

    TokenService::new(&mut db, &data_dir)
        .delete(&args.token_id)
        .map_err(|e| McpError::Internal(e.to_string()))?;

    McpResponse::removed(&args.token_id)
}
