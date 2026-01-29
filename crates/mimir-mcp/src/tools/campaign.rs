//! Campaign Tools
//!
//! MCP tools for campaign management.

use mimir_core::dal::campaign as dal;
use mimir_core::services::{ArchiveService, CampaignService, CharacterService, ModuleService};
use rust_mcp_sdk::schema::{Tool, ToolInputSchema};
use serde_json::{json, Value};
use std::path::Path;
use std::sync::Arc;

use super::create_properties;
use crate::context::McpContext;
use crate::McpError;

// =============================================================================
// Tool Definitions
// =============================================================================

pub fn list_campaigns_tool() -> Tool {
    Tool {
        name: "list_campaigns".to_string(),
        description: Some("List all available campaigns".to_string()),
        input_schema: ToolInputSchema::new(vec![], None, None),
        title: None,
        annotations: None,
        icons: vec![],
        execution: None,
        output_schema: None,
        meta: None,
    }
}

pub fn set_active_campaign_tool() -> Tool {
    Tool {
        name: "set_active_campaign".to_string(),
        description: Some(
            "Set the active campaign. Most other tools require an active campaign.".to_string(),
        ),
        input_schema: ToolInputSchema::new(
            vec!["campaign_id".to_string()],
            create_properties(vec![(
                "campaign_id",
                "string",
                "The ID of the campaign to set as active",
            )]),
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

pub fn get_campaign_details_tool() -> Tool {
    Tool {
        name: "get_campaign_details".to_string(),
        description: Some(
            "Get detailed information about a campaign including modules and characters"
                .to_string(),
        ),
        input_schema: ToolInputSchema::new(
            vec![],
            create_properties(vec![(
                "campaign_id",
                "string",
                "Campaign ID (optional, defaults to active campaign)",
            )]),
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

pub fn get_campaign_sources_tool() -> Tool {
    Tool {
        name: "get_campaign_sources".to_string(),
        description: Some("Get the list of enabled source books for a campaign".to_string()),
        input_schema: ToolInputSchema::new(
            vec![],
            create_properties(vec![(
                "campaign_id",
                "string",
                "Campaign ID (optional, defaults to active campaign)",
            )]),
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

pub async fn list_campaigns(ctx: &Arc<McpContext>, _args: Value) -> Result<Value, McpError> {
    let mut db = ctx.db()?;
    let mut service = CampaignService::new(&mut db);

    let campaigns = service.list(false).map_err(|e| McpError::Internal(e.to_string()))?;

    let campaign_data: Vec<Value> = campaigns
        .iter()
        .map(|c| {
            json!({
                "id": c.id,
                "name": c.name,
                "description": c.description,
                "created_at": c.created_at
            })
        })
        .collect();

    Ok(json!({
        "campaigns": campaign_data
    }))
}

pub async fn set_active_campaign(ctx: &Arc<McpContext>, args: Value) -> Result<Value, McpError> {
    let campaign_id = args
        .get("campaign_id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| McpError::InvalidArguments("campaign_id is required".to_string()))?;

    // Verify campaign exists
    let mut db = ctx.db()?;
    let mut service = CampaignService::new(&mut db);

    let campaign = service
        .get(campaign_id)
        .map_err(|e| McpError::Internal(e.to_string()))?
        .ok_or_else(|| McpError::InvalidArguments(format!("Campaign '{}' not found", campaign_id)))?;

    ctx.set_active_campaign_id(Some(campaign_id.to_string()));

    Ok(json!({
        "status": "success",
        "active_campaign_id": campaign_id,
        "campaign": {
            "id": campaign.id,
            "name": campaign.name,
            "description": campaign.description
        }
    }))
}

pub async fn get_campaign_details(ctx: &Arc<McpContext>, args: Value) -> Result<Value, McpError> {
    let campaign_id = args
        .get("campaign_id")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .or_else(|| ctx.get_active_campaign_id())
        .ok_or(McpError::NoActiveCampaign)?;

    let mut db = ctx.db()?;

    // Get campaign
    let mut campaign_service = CampaignService::new(&mut db);
    let campaign = campaign_service
        .get(&campaign_id)
        .map_err(|e| McpError::Internal(e.to_string()))?
        .ok_or_else(|| McpError::InvalidArguments(format!("Campaign '{}' not found", campaign_id)))?;

    // Get modules
    let mut module_service = ModuleService::new(&mut db);
    let modules = module_service
        .list_for_campaign(&campaign_id)
        .map_err(|e| McpError::Internal(e.to_string()))?;

    let module_data: Vec<Value> = modules
        .iter()
        .map(|m| {
            json!({
                "id": m.id,
                "name": m.name,
                "description": m.description,
                "module_number": m.module_number
            })
        })
        .collect();

    // Get characters
    let mut char_service = CharacterService::new(&mut db);
    let characters = char_service
        .list_for_campaign(&campaign_id)
        .map_err(|e| McpError::Internal(e.to_string()))?;

    let pc_count = characters.iter().filter(|c| c.is_pc()).count();
    let npc_count = characters.iter().filter(|c| c.is_npc()).count();

    let character_data: Vec<Value> = characters
        .iter()
        .map(|c| {
            json!({
                "id": c.id,
                "name": c.name,
                "is_npc": c.is_npc(),
                "race_name": c.race_name
            })
        })
        .collect();

    Ok(json!({
        "campaign": {
            "id": campaign.id,
            "name": campaign.name,
            "description": campaign.description,
            "created_at": campaign.created_at
        },
        "modules": module_data,
        "module_count": modules.len(),
        "characters": character_data,
        "pc_count": pc_count,
        "npc_count": npc_count
    }))
}

pub async fn get_campaign_sources(ctx: &Arc<McpContext>, args: Value) -> Result<Value, McpError> {
    let campaign_id = args
        .get("campaign_id")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .or_else(|| ctx.get_active_campaign_id())
        .ok_or(McpError::NoActiveCampaign)?;

    let mut db = ctx.db()?;

    let source_codes = dal::list_campaign_source_codes(&mut db, &campaign_id)
        .map_err(|e| McpError::Internal(e.to_string()))?;

    Ok(json!({
        "campaign_id": campaign_id,
        "sources": source_codes
    }))
}

// =============================================================================
// Export/Import Tool Definitions
// =============================================================================

pub fn export_campaign_tool() -> Tool {
    Tool {
        name: "export_campaign".to_string(),
        description: Some(
            "Export the active campaign as a shareable archive file (.mimir-campaign.tar.gz)".to_string(),
        ),
        input_schema: ToolInputSchema::new(
            vec!["output_path".to_string()],
            create_properties(vec![
                ("output_path", "string", "Directory path where the archive will be saved"),
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

pub fn import_campaign_tool() -> Tool {
    Tool {
        name: "import_campaign".to_string(),
        description: Some(
            "Import a campaign from an archive file (.mimir-campaign.tar.gz)".to_string(),
        ),
        input_schema: ToolInputSchema::new(
            vec!["archive_path".to_string()],
            create_properties(vec![
                ("archive_path", "string", "Path to the archive file to import"),
                ("new_name", "string", "Optional new name for the imported campaign"),
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

pub fn preview_archive_tool() -> Tool {
    Tool {
        name: "preview_archive".to_string(),
        description: Some(
            "Preview the contents of a campaign archive without importing it".to_string(),
        ),
        input_schema: ToolInputSchema::new(
            vec!["archive_path".to_string()],
            create_properties(vec![
                ("archive_path", "string", "Path to the archive file to preview"),
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
// Export/Import Tool Implementations
// =============================================================================

pub async fn export_campaign(ctx: &Arc<McpContext>, args: Value) -> Result<Value, McpError> {
    let campaign_id = ctx.get_active_campaign_id()
        .ok_or(McpError::NoActiveCampaign)?;

    let output_path = args
        .get("output_path")
        .and_then(|v| v.as_str())
        .ok_or_else(|| McpError::InvalidArguments("output_path is required".to_string()))?;

    let mut db = ctx.db()?;
    let output_dir = Path::new(output_path);
    let assets_dir = &ctx.assets_dir;

    let archive_path = ArchiveService::new(&mut db)
        .export_campaign(&campaign_id, output_dir, assets_dir)
        .map_err(|e| McpError::Internal(format!("Export failed: {}", e)))?;

    // Get file size
    let size_bytes = std::fs::metadata(&archive_path)
        .map(|m| m.len())
        .unwrap_or(0);

    Ok(json!({
        "status": "success",
        "archive_path": archive_path.display().to_string(),
        "size_bytes": size_bytes
    }))
}

pub async fn import_campaign(ctx: &Arc<McpContext>, args: Value) -> Result<Value, McpError> {
    let archive_path = args
        .get("archive_path")
        .and_then(|v| v.as_str())
        .ok_or_else(|| McpError::InvalidArguments("archive_path is required".to_string()))?;

    let new_name = args
        .get("new_name")
        .and_then(|v| v.as_str());

    let mut db = ctx.db()?;
    let archive = Path::new(archive_path);
    let assets_dir = &ctx.assets_dir;

    let result = ArchiveService::new(&mut db)
        .import_campaign(archive, assets_dir, new_name)
        .map_err(|e| McpError::Internal(format!("Import failed: {}", e)))?;

    // Set the imported campaign as active
    ctx.set_active_campaign_id(Some(result.campaign_id.clone()));

    Ok(json!({
        "status": "success",
        "campaign_id": result.campaign_id,
        "campaign_name": result.campaign_name,
        "counts": {
            "modules": result.counts.modules,
            "documents": result.counts.documents,
            "characters": result.counts.characters,
            "maps": result.counts.maps,
            "tokens": result.counts.tokens,
            "assets": result.counts.assets
        }
    }))
}

pub async fn preview_archive(_ctx: &Arc<McpContext>, args: Value) -> Result<Value, McpError> {
    let archive_path = args
        .get("archive_path")
        .and_then(|v| v.as_str())
        .ok_or_else(|| McpError::InvalidArguments("archive_path is required".to_string()))?;

    let archive = Path::new(archive_path);

    let preview = ArchiveService::preview_archive(archive)
        .map_err(|e| McpError::Internal(format!("Preview failed: {}", e)))?;

    Ok(json!({
        "campaign_name": preview.campaign_name,
        "archive_version": preview.archive_version,
        "mimir_version": preview.mimir_version,
        "created_at": preview.created_at.to_rfc3339(),
        "counts": {
            "modules": preview.counts.modules,
            "documents": preview.counts.documents,
            "characters": preview.counts.characters,
            "maps": preview.counts.maps,
            "tokens": preview.counts.tokens,
            "assets": preview.counts.assets
        },
        "catalog_references": preview.catalog_references.iter().map(|r| {
            json!({
                "type": r.ref_type,
                "name": r.name,
                "source": r.source
            })
        }).collect::<Vec<_>>()
    }))
}
