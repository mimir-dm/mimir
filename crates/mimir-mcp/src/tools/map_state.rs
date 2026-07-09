//! Map Authoring Tools
//!
//! MCP tools for placing and editing traps, POIs, and light sources on maps.
//!
//! AUTHORING ONLY: agents build and refine the world; people run the game.
//! Live-play operations — trap trigger/reset, player-visibility toggles,
//! flipping lights on/off, all fog control — are deliberately NOT exposed
//! here and remain DM-only in the desktop UI.

use mimir_core::models::campaign::{LightSource, MapPoi, MapTrap};
use mimir_core::services::{
    CreateLightInput, CreatePoiInput, CreateTrapInput, MapStateService, UpdateLightInput,
    UpdatePoiInput, UpdateTrapInput,
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

/// All map-authoring tools (traps, POIs, lights — placement only).
pub fn registered_tools() -> Vec<RegisteredTool> {
    vec![
        // Traps
        tool!(
            "add_trap_to_map",
            "Place a trap on a map (get map ids from list_maps/get_map). Authoring only: traps are placed hidden and inactive concerns like triggering or revealing to players are DM-only in the desktop UI.",
            AddTrapArgs,
            add_trap_to_map
        ),
        tool!(
            "list_map_traps",
            "List all traps placed on a map, including their authored state (position, DC, descriptions).",
            MapIdArgs,
            list_map_traps
        ),
        tool!(
            "update_map_trap",
            "Update a trap's authored fields (name, descriptions, DC) and/or move it (grid_x + grid_y). Does not trigger, reset, or reveal traps — that is DM-only during play.",
            UpdateTrapArgs,
            update_map_trap
        ),
        tool!(
            "remove_map_trap",
            "Remove a trap from a map",
            TrapIdArgs,
            remove_map_trap
        ),
        // POIs
        tool!(
            "add_poi_to_map",
            "Place a point of interest on a map (get map ids from list_maps/get_map). Authoring only: revealing POIs to players is DM-only in the desktop UI.",
            AddPoiArgs,
            add_poi_to_map
        ),
        tool!(
            "list_map_pois",
            "List all points of interest placed on a map.",
            MapIdArgs,
            list_map_pois
        ),
        tool!(
            "update_map_poi",
            "Update a POI's authored fields (name, description, icon, color) and/or move it (grid_x + grid_y). Does not change player visibility — that is DM-only during play.",
            UpdatePoiArgs,
            update_map_poi
        ),
        tool!(
            "remove_map_poi",
            "Remove a point of interest from a map",
            PoiIdArgs,
            remove_map_poi
        ),
        // Lights
        tool!(
            "add_light_to_map",
            "Place a light source on a map. Use preset 'torch' (20ft bright/40ft dim) or 'lantern' (30ft bright/60ft dim), or omit preset and give name + bright_radius_ft + dim_radius_ft. Coordinates are grid units. Turning lights on/off during play is DM-only in the desktop UI.",
            AddLightArgs,
            add_light_to_map
        ),
        tool!(
            "list_map_lights",
            "List all light sources placed on a map (grid coordinates).",
            MapIdArgs,
            list_map_lights
        ),
        tool!(
            "update_map_light",
            "Update a light source's authored fields (name, radii, color) and/or move it (grid_x + grid_y). Does not toggle lights on/off — that is DM-only during play.",
            UpdateLightArgs,
            update_map_light
        ),
        tool!(
            "remove_map_light",
            "Remove a light source from a map",
            LightIdArgs,
            remove_map_light
        ),
    ]
}

// =============================================================================
// Arguments
// =============================================================================

tool_args! {
    pub struct MapIdArgs {
        /// The ID of the map
        pub map_id: String,
    }
}

tool_args! {
    pub struct AddTrapArgs {
        /// The ID of the map
        pub map_id: String,
        /// Trap name
        pub name: String,
        /// Grid X coordinate
        pub grid_x: i64,
        /// Grid Y coordinate
        pub grid_y: i64,
        /// What the trap is
        pub description: Option<String>,
        /// What sets it off
        pub trigger_description: Option<String>,
        /// What happens when it fires
        pub effect_description: Option<String>,
        /// Save/detection DC
        pub dc: Option<i64>,
    }
}

tool_args! {
    pub struct UpdateTrapArgs {
        /// The ID of the trap
        pub trap_id: String,
        /// New name
        pub name: Option<String>,
        /// New description
        pub description: Option<String>,
        /// New trigger description
        pub trigger_description: Option<String>,
        /// New effect description
        pub effect_description: Option<String>,
        /// New DC
        pub dc: Option<i64>,
        /// New grid X (move — provide with grid_y)
        pub grid_x: Option<i64>,
        /// New grid Y (move — provide with grid_x)
        pub grid_y: Option<i64>,
    }
}

tool_args! {
    pub struct TrapIdArgs {
        /// The ID of the trap to remove
        pub trap_id: String,
    }
}

tool_args! {
    pub struct AddPoiArgs {
        /// The ID of the map
        pub map_id: String,
        /// POI name
        pub name: String,
        /// Grid X coordinate
        pub grid_x: i64,
        /// Grid Y coordinate
        pub grid_y: i64,
        /// Description
        pub description: Option<String>,
        /// Icon identifier
        pub icon: Option<String>,
        /// Color as hex (e.g. #88CCFF)
        pub color: Option<String>,
    }
}

tool_args! {
    pub struct UpdatePoiArgs {
        /// The ID of the POI
        pub poi_id: String,
        /// New name
        pub name: Option<String>,
        /// New description
        pub description: Option<String>,
        /// New icon
        pub icon: Option<String>,
        /// New color
        pub color: Option<String>,
        /// New grid X (move — provide with grid_y)
        pub grid_x: Option<i64>,
        /// New grid Y (move — provide with grid_x)
        pub grid_y: Option<i64>,
    }
}

tool_args! {
    pub struct PoiIdArgs {
        /// The ID of the POI to remove
        pub poi_id: String,
    }
}

tool_args! {
    pub struct AddLightArgs {
        /// The ID of the map
        pub map_id: String,
        /// Grid X coordinate
        pub grid_x: i64,
        /// Grid Y coordinate
        pub grid_y: i64,
        /// Preset: torch (20/40ft) or lantern (30/60ft). Omit to specify radii manually.
        pub preset: Option<String>,
        /// Display name (required when no preset)
        pub name: Option<String>,
        /// Bright light radius in feet (required when no preset)
        pub bright_radius_ft: Option<i64>,
        /// Dim light radius in feet (required when no preset)
        pub dim_radius_ft: Option<i64>,
        /// Color as hex (e.g. #FFAA00)
        pub color: Option<String>,
    }
}

tool_args! {
    pub struct UpdateLightArgs {
        /// The ID of the light source
        pub light_id: String,
        /// New name
        pub name: Option<String>,
        /// New bright radius in feet
        pub bright_radius_ft: Option<i64>,
        /// New dim radius in feet
        pub dim_radius_ft: Option<i64>,
        /// New color
        pub color: Option<String>,
        /// New grid X (move — provide with grid_y)
        pub grid_x: Option<i64>,
        /// New grid Y (move — provide with grid_x)
        pub grid_y: Option<i64>,
    }
}

tool_args! {
    pub struct LightIdArgs {
        /// The ID of the light source to remove
        pub light_id: String,
    }
}

// =============================================================================
// Serialization
// =============================================================================

fn trap_to_json(t: &MapTrap) -> Value {
    json!({
        "id": t.id,
        "map_id": t.map_id,
        "name": t.name,
        "grid_x": t.grid_x,
        "grid_y": t.grid_y,
        "description": t.description,
        "trigger_description": t.trigger_description,
        "effect_description": t.effect_description,
        "dc": t.dc,
    })
}

fn poi_to_json(p: &MapPoi) -> Value {
    json!({
        "id": p.id,
        "map_id": p.map_id,
        "name": p.name,
        "grid_x": p.grid_x,
        "grid_y": p.grid_y,
        "description": p.description,
        "icon": p.icon,
        "color": p.color,
    })
}

fn light_to_json(l: &LightSource) -> Value {
    json!({
        "id": l.id,
        "map_id": l.map_id,
        "name": l.name,
        "grid_x": l.grid_x,
        "grid_y": l.grid_y,
        "bright_radius_ft": l.bright_radius,
        "dim_radius_ft": l.dim_radius,
        "color": l.color,
    })
}

/// Both-or-neither validation for move coordinates.
fn move_coords(
    grid_x: Option<i64>,
    grid_y: Option<i64>,
) -> Result<Option<(i32, i32)>, McpError> {
    match (grid_x, grid_y) {
        (Some(x), Some(y)) => Ok(Some((x as i32, y as i32))),
        (None, None) => Ok(None),
        _ => Err(McpError::InvalidArguments(
            "Provide both grid_x and grid_y to move, or neither".to_string(),
        )),
    }
}

// =============================================================================
// Handlers — traps
// =============================================================================

pub async fn add_trap_to_map(
    ctx: &Arc<McpContext>,
    args: AddTrapArgs,
) -> Result<Value, McpError> {
    let mut db = ctx.connect()?;

    let trap = MapStateService::new(&mut db)
        .create_trap(CreateTrapInput {
            map_id: args.map_id,
            name: args.name,
            grid_x: args.grid_x as i32,
            grid_y: args.grid_y as i32,
            description: args.description,
            trigger_description: args.trigger_description,
            effect_description: args.effect_description,
            dc: args.dc.map(|d| d as i32),
            // Authored traps start hidden; revealing is a live-play (DM) action
            visible: false,
        })
        .map_err(McpError::caller_fault)?;

    McpResponse::added("trap", trap_to_json(&trap))
}

pub async fn list_map_traps(
    ctx: &Arc<McpContext>,
    args: MapIdArgs,
) -> Result<Value, McpError> {
    let mut db = ctx.connect()?;
    let traps = MapStateService::new(&mut db)
        .list_traps(&args.map_id)
        .map_err(McpError::caller_fault)?;
    McpResponse::list("traps", traps.iter().map(trap_to_json).collect())
}

pub async fn update_map_trap(
    ctx: &Arc<McpContext>,
    args: UpdateTrapArgs,
) -> Result<Value, McpError> {
    let coords = move_coords(args.grid_x, args.grid_y)?;
    let has_field_updates = args.name.is_some()
        || args.description.is_some()
        || args.trigger_description.is_some()
        || args.effect_description.is_some()
        || args.dc.is_some();

    if coords.is_none() && !has_field_updates {
        return Err(McpError::InvalidArguments(
            "Provide at least one field to update, or grid_x + grid_y to move".to_string(),
        ));
    }

    let mut db = ctx.connect()?;
    let mut svc = MapStateService::new(&mut db);

    if has_field_updates {
        svc.update_trap(
            &args.trap_id,
            UpdateTrapInput {
                name: args.name,
                description: args.description,
                trigger_description: args.trigger_description,
                effect_description: args.effect_description,
                dc: args.dc.map(|d| d as i32),
            },
        )
        .map_err(McpError::caller_fault)?;
    }

    let trap = if let Some((x, y)) = coords {
        svc.move_trap(&args.trap_id, x, y)
            .map_err(McpError::caller_fault)?
    } else {
        svc.get_trap(&args.trap_id).map_err(McpError::caller_fault)?
    };

    McpResponse::updated("trap", trap_to_json(&trap))
}

pub async fn remove_map_trap(
    ctx: &Arc<McpContext>,
    args: TrapIdArgs,
) -> Result<Value, McpError> {
    let mut db = ctx.connect()?;
    let mut svc = MapStateService::new(&mut db);

    // Verify it exists so removals of unknown ids error clearly
    svc.get_trap(&args.trap_id).map_err(McpError::caller_fault)?;
    svc.delete_trap(&args.trap_id).map_err(McpError::caller_fault)?;

    McpResponse::removed(&args.trap_id)
}

// =============================================================================
// Handlers — POIs
// =============================================================================

pub async fn add_poi_to_map(
    ctx: &Arc<McpContext>,
    args: AddPoiArgs,
) -> Result<Value, McpError> {
    let mut db = ctx.connect()?;

    let poi = MapStateService::new(&mut db)
        .create_poi(CreatePoiInput {
            map_id: args.map_id,
            name: args.name,
            grid_x: args.grid_x as i32,
            grid_y: args.grid_y as i32,
            description: args.description,
            icon: args.icon,
            color: args.color,
            // Authored POIs start hidden; revealing is a live-play (DM) action
            visible: false,
        })
        .map_err(McpError::caller_fault)?;

    McpResponse::added("poi", poi_to_json(&poi))
}

pub async fn list_map_pois(
    ctx: &Arc<McpContext>,
    args: MapIdArgs,
) -> Result<Value, McpError> {
    let mut db = ctx.connect()?;
    let pois = MapStateService::new(&mut db)
        .list_pois(&args.map_id)
        .map_err(McpError::caller_fault)?;
    McpResponse::list("pois", pois.iter().map(poi_to_json).collect())
}

pub async fn update_map_poi(
    ctx: &Arc<McpContext>,
    args: UpdatePoiArgs,
) -> Result<Value, McpError> {
    let coords = move_coords(args.grid_x, args.grid_y)?;
    let has_field_updates = args.name.is_some()
        || args.description.is_some()
        || args.icon.is_some()
        || args.color.is_some();

    if coords.is_none() && !has_field_updates {
        return Err(McpError::InvalidArguments(
            "Provide at least one field to update, or grid_x + grid_y to move".to_string(),
        ));
    }

    let mut db = ctx.connect()?;
    let mut svc = MapStateService::new(&mut db);

    if has_field_updates {
        svc.update_poi(
            &args.poi_id,
            UpdatePoiInput {
                name: args.name,
                description: args.description,
                icon: args.icon,
                color: args.color,
            },
        )
        .map_err(McpError::caller_fault)?;
    }

    let poi = if let Some((x, y)) = coords {
        svc.move_poi(&args.poi_id, x, y)
            .map_err(McpError::caller_fault)?
    } else {
        svc.get_poi(&args.poi_id).map_err(McpError::caller_fault)?
    };

    McpResponse::updated("poi", poi_to_json(&poi))
}

pub async fn remove_map_poi(
    ctx: &Arc<McpContext>,
    args: PoiIdArgs,
) -> Result<Value, McpError> {
    let mut db = ctx.connect()?;
    let mut svc = MapStateService::new(&mut db);

    svc.get_poi(&args.poi_id).map_err(McpError::caller_fault)?;
    svc.delete_poi(&args.poi_id).map_err(McpError::caller_fault)?;

    McpResponse::removed(&args.poi_id)
}

// =============================================================================
// Handlers — lights
// =============================================================================

pub async fn add_light_to_map(
    ctx: &Arc<McpContext>,
    args: AddLightArgs,
) -> Result<Value, McpError> {
    let mut db = ctx.connect()?;
    let mut svc = MapStateService::new(&mut db);

    let x = args.grid_x as i32;
    let y = args.grid_y as i32;

    let light = match args.preset.as_deref() {
        Some("torch") => svc.create_torch(&args.map_id, x, y),
        Some("lantern") => svc.create_lantern(&args.map_id, x, y),
        Some(other) => {
            return Err(McpError::InvalidArguments(format!(
                "Unknown preset '{}'. Use torch or lantern, or omit preset and provide name + bright_radius_ft + dim_radius_ft.",
                other
            )));
        }
        None => {
            let name = args.name.ok_or_else(|| {
                McpError::InvalidArguments(
                    "name is required when no preset is given".to_string(),
                )
            })?;
            let bright = args.bright_radius_ft.ok_or_else(|| {
                McpError::InvalidArguments(
                    "bright_radius_ft is required when no preset is given".to_string(),
                )
            })?;
            let dim = args.dim_radius_ft.ok_or_else(|| {
                McpError::InvalidArguments(
                    "dim_radius_ft is required when no preset is given".to_string(),
                )
            })?;
            svc.create_light(CreateLightInput {
                map_id: args.map_id,
                grid_x: x,
                grid_y: y,
                bright_radius_ft: bright as i32,
                dim_radius_ft: dim as i32,
                name,
                color: args.color,
                is_active: true,
            })
        }
    }
    .map_err(McpError::caller_fault)?;

    McpResponse::added("light", light_to_json(&light))
}

pub async fn list_map_lights(
    ctx: &Arc<McpContext>,
    args: MapIdArgs,
) -> Result<Value, McpError> {
    let mut db = ctx.connect()?;
    let lights = MapStateService::new(&mut db)
        .list_lights(&args.map_id)
        .map_err(McpError::caller_fault)?;
    McpResponse::list("lights", lights.iter().map(light_to_json).collect())
}

pub async fn update_map_light(
    ctx: &Arc<McpContext>,
    args: UpdateLightArgs,
) -> Result<Value, McpError> {
    let coords = move_coords(args.grid_x, args.grid_y)?;
    let has_field_updates = args.name.is_some()
        || args.bright_radius_ft.is_some()
        || args.dim_radius_ft.is_some()
        || args.color.is_some();

    if coords.is_none() && !has_field_updates {
        return Err(McpError::InvalidArguments(
            "Provide at least one field to update, or grid_x + grid_y to move".to_string(),
        ));
    }

    let mut db = ctx.connect()?;
    let mut svc = MapStateService::new(&mut db);

    if has_field_updates {
        svc.update_light(
            &args.light_id,
            UpdateLightInput {
                name: args.name.map(Some),
                bright_radius_ft: args.bright_radius_ft.map(|v| v as i32),
                dim_radius_ft: args.dim_radius_ft.map(|v| v as i32),
                color: args.color.map(Some),
                // Deliberately no is_active here — toggling lights during
                // play is DM-only (desktop UI)
                is_active: None,
            },
        )
        .map_err(McpError::caller_fault)?;
    }

    let light = if let Some((x, y)) = coords {
        svc.move_light(&args.light_id, x, y)
            .map_err(McpError::caller_fault)?
    } else {
        svc.get_light(&args.light_id)
            .map_err(McpError::caller_fault)?
    };

    McpResponse::updated("light", light_to_json(&light))
}

pub async fn remove_map_light(
    ctx: &Arc<McpContext>,
    args: LightIdArgs,
) -> Result<Value, McpError> {
    let mut db = ctx.connect()?;
    let mut svc = MapStateService::new(&mut db);

    // Verify it exists so removals of unknown ids error clearly
    svc.get_light(&args.light_id)
        .map_err(McpError::caller_fault)?;
    svc.delete_light(&args.light_id)
        .map_err(McpError::caller_fault)?;

    McpResponse::removed(&args.light_id)
}
