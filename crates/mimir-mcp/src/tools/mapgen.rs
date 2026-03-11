//! Map Generation Tools
//!
//! MCP tools for procedural Dungeondraft map generation.

use mimir_mapgen::biomes;
use mimir_mapgen::pipeline::{generate, validate_config, MapConfig};
use rust_mcp_sdk::schema::{Tool, ToolInputSchema};
use serde_json::{json, Value};

use super::create_properties;
use crate::McpError;

// =============================================================================
// Tool Definitions
// =============================================================================

pub fn generate_map_tool() -> Tool {
    Tool {
        name: "generate_map".to_string(),
        description: Some(
            "Generate a Dungeondraft .dungeondraft_map file from a YAML config string or biome preset. Returns the output file path."
                .to_string(),
        ),
        input_schema: ToolInputSchema::new(
            vec![],
            create_properties(vec![
                ("config_yaml", "string", "YAML configuration for map generation. Mutually exclusive with preset."),
                ("preset", "string", "Biome preset name (forest, grassland, cave). Mutually exclusive with config_yaml."),
                ("output_path", "string", "Absolute path for the output .dungeondraft_map file (required)"),
                ("seed", "integer", "Random seed override for reproducible generation"),
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

pub fn list_map_presets_tool() -> Tool {
    Tool {
        name: "list_map_presets".to_string(),
        description: Some(
            "List available biome presets for procedural map generation".to_string(),
        ),
        input_schema: ToolInputSchema::new(vec![], None, None),
        title: None,
        annotations: None,
        icons: vec![],
        execution: None,
        output_schema: None,
        meta: None,
    }
}

pub fn validate_map_config_tool() -> Tool {
    Tool {
        name: "validate_map_config".to_string(),
        description: Some(
            "Validate a YAML map generation config without generating. Returns validation errors if any."
                .to_string(),
        ),
        input_schema: ToolInputSchema::new(
            vec!["config_yaml".to_string()],
            create_properties(vec![
                ("config_yaml", "string", "YAML configuration to validate"),
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

// =============================================================================
// Tool Implementations
// =============================================================================

pub async fn generate_map(args: Value) -> Result<Value, McpError> {
    let config_yaml = args.get("config_yaml").and_then(|v| v.as_str());
    let preset_name = args.get("preset").and_then(|v| v.as_str());
    let output_path = args
        .get("output_path")
        .and_then(|v| v.as_str())
        .ok_or_else(|| McpError::InvalidArguments("output_path is required".to_string()))?;
    let seed = args.get("seed").and_then(|v| v.as_u64());

    let config = match (config_yaml, preset_name) {
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
        (Some(yaml), None) => {
            serde_yaml::from_str::<MapConfig>(yaml).map_err(|e| {
                McpError::InvalidArguments(format!("Invalid YAML config: {e}"))
            })?
        }
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
    let result = generate(&config, seed);

    // Write output
    let map_json = result.map.to_json().map_err(|e| {
        McpError::Internal(format!("Failed to serialize map: {e}"))
    })?;

    std::fs::write(output_path, &map_json).map_err(|e| {
        McpError::Internal(format!("Failed to write output file: {e}"))
    })?;

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

pub async fn list_map_presets(_args: Value) -> Result<Value, McpError> {
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

pub async fn validate_map_config(args: Value) -> Result<Value, McpError> {
    let config_yaml = args
        .get("config_yaml")
        .and_then(|v| v.as_str())
        .ok_or_else(|| McpError::InvalidArguments("config_yaml is required".to_string()))?;

    let config: MapConfig = match serde_yaml::from_str(config_yaml) {
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
