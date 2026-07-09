//! Map Generation Tools
//!
//! MCP tools for procedural Dungeondraft map generation. This family is
//! registry-based and needs no database context.

use mimir_mapgen::biomes;
use mimir_mapgen::pipeline::{generate, validate_config, MapConfig};
use serde_json::{json, Value};
use std::sync::Arc;

use crate::context::McpContext;
use crate::registry::RegisteredTool;
use crate::{tool, tool_args, McpError};

// =============================================================================
// Registration
// =============================================================================

/// All mapgen-family tools.
pub fn registered_tools() -> Vec<RegisteredTool> {
    vec![
        tool!(
            "generate_map",
            "Generate a Dungeondraft .dungeondraft_map file from a YAML config string or biome preset. The file is written to the machine running this server (not the client); output_path is an absolute path on that host. Returns the output file path. No active campaign required.",
            GenerateMapArgs,
            generate_map
        ),
        tool!(
            "list_map_presets",
            "List available biome presets for procedural map generation",
            ListMapPresetsArgs,
            list_map_presets
        ),
        tool!(
            "validate_map_config",
            "Validate a YAML map generation config without generating. Returns validation errors if any.",
            ValidateMapConfigArgs,
            validate_map_config_tool_handler
        ),
    ]
}

// =============================================================================
// Arguments
// =============================================================================

tool_args! {
    pub struct GenerateMapArgs {
        /// YAML configuration for map generation. Mutually exclusive with preset.
        pub config_yaml: Option<String>,
        /// Biome preset name. Call list_map_presets for the current set (includes forest, grassland, cave, desert, lake, arctic, swamp, and island variants). Mutually exclusive with config_yaml.
        pub preset: Option<String>,
        /// Absolute path for the output .dungeondraft_map file (required)
        pub output_path: Option<String>,
        /// Random seed override for reproducible generation
        pub seed: Option<i64>,
    }
}

tool_args! {
    pub struct ListMapPresetsArgs {}
}

tool_args! {
    pub struct ValidateMapConfigArgs {
        /// YAML configuration to validate
        pub config_yaml: String,
    }
}

// =============================================================================
// Handlers
// =============================================================================

pub async fn generate_map(
    _ctx: &Arc<McpContext>,
    args: GenerateMapArgs,
) -> Result<Value, McpError> {
    // output_path is enforced here rather than in the schema's required list
    // to preserve the published contract (schema declares no required fields).
    let output_path = args
        .output_path
        .as_deref()
        .ok_or_else(|| McpError::InvalidArguments("output_path is required".to_string()))?;

    let config = match (args.config_yaml.as_deref(), args.preset.as_deref()) {
        (Some(_), Some(_)) => {
            return Err(McpError::InvalidArguments(
                "Provide either config_yaml or preset, not both".to_string(),
            ));
        }
        (None, None) => {
            return Err(McpError::InvalidArguments(
                "Provide either config_yaml or preset".to_string(),
            ));
        }
        (Some(yaml), None) => serde_yaml::from_str::<MapConfig>(yaml)
            .map_err(|e| McpError::InvalidArguments(format!("Invalid YAML config: {e}")))?,
        (None, Some(name)) => {
            biomes::get_preset(name)
                .ok_or_else(|| {
                    let available: Vec<&str> =
                        biomes::list_presets().iter().map(|p| p.name).collect();
                    McpError::InvalidArguments(format!(
                        "Unknown preset '{}'. Available: {}",
                        name,
                        available.join(", ")
                    ))
                })?
                .config
        }
    };

    // Validate
    let errors = validate_config(&config);
    if !errors.is_empty() {
        let error_list: Vec<Value> = errors
            .iter()
            .map(|e| json!({ "field": e.field, "message": e.message }))
            .collect();
        return Ok(json!({
            "success": false,
            "errors": error_list
        }));
    }

    // Generate
    let result = generate(&config, args.seed.map(|s| s as u64));

    // Write output
    let map_json = result
        .map
        .to_json()
        .map_err(|e| McpError::Internal(format!("Failed to serialize map: {e}")))?;

    std::fs::write(output_path, &map_json)
        .map_err(|e| McpError::Internal(format!("Failed to write output file: {e}")))?;

    Ok(json!({
        "success": true,
        "output_path": output_path,
        "stats": {
            "objects_placed": result.stats.objects_placed,
            "paths_generated": result.stats.paths_generated,
            "water_polygons": result.stats.water_polygons,
            "contour_paths": result.stats.contour_paths
        }
    }))
}

pub async fn list_map_presets(
    _ctx: &Arc<McpContext>,
    _args: ListMapPresetsArgs,
) -> Result<Value, McpError> {
    let presets: Vec<Value> = biomes::list_presets()
        .iter()
        .map(|p| {
            json!({
                "name": p.name,
                "description": p.description,
                "default_size": { "width": p.default_size.0, "height": p.default_size.1 }
            })
        })
        .collect();

    Ok(json!({ "presets": presets }))
}

pub async fn validate_map_config_tool_handler(
    _ctx: &Arc<McpContext>,
    args: ValidateMapConfigArgs,
) -> Result<Value, McpError> {
    let config: MapConfig = match serde_yaml::from_str(&args.config_yaml) {
        Ok(c) => c,
        Err(e) => {
            return Ok(json!({
                "valid": false,
                "parse_error": e.to_string(),
                "errors": []
            }));
        }
    };

    let errors = validate_config(&config);
    let error_list: Vec<Value> = errors
        .iter()
        .map(|e| json!({ "field": e.field, "message": e.message }))
        .collect();

    Ok(json!({
        "valid": errors.is_empty(),
        "errors": error_list
    }))
}
