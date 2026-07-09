//! Campaign Tools
//!
//! MCP tools for campaign management, including archive export/import. This
//! family is registry-based: typed argument structs, handlers, and one entry
//! each in `registered_tools()`.

use mimir_core::dal::campaign as dal;
use mimir_core::services::{
    ArchiveService, CampaignService, CharacterService, CreateCampaignInput, ModuleService,
    UpdateCampaignInput,
};
use serde_json::{json, Value};
use std::path::Path;
use std::sync::Arc;

use crate::context::McpContext;
use crate::registry::RegisteredTool;
use crate::response::McpResponse;
use crate::{tool, tool_args, McpError};

// =============================================================================
// Registration
// =============================================================================

/// All campaign-family tools.
pub fn registered_tools() -> Vec<RegisteredTool> {
    vec![
        tool!(
            "list_campaigns",
            "List every campaign in this Mimir database. Start here to discover campaigns, then pass a returned id to set_active_campaign. Returns each campaign's id, name, description, and created_at.",
            ListCampaignsArgs,
            list_campaigns
        ),
        tool!(
            "get_active_campaign",
            "Report which campaign is currently active — the one most tools operate on. Never errors: when nothing is selected it returns active=false with guidance. Call this first to orient yourself before using campaign-scoped tools.",
            GetActiveCampaignArgs,
            get_active_campaign
        ),
        tool!(
            "set_active_campaign",
            "Select the campaign that subsequent tools operate on. The selection is held server-side for the rest of the session. Most module, character, document, map, and homebrew tools require an active campaign.",
            SetActiveCampaignArgs,
            set_active_campaign
        ),
        tool!(
            "get_campaign_details",
            "Get detailed information about a campaign including modules and characters",
            CampaignIdOptionalArgs,
            get_campaign_details
        ),
        tool!(
            "get_campaign_sources",
            "Get the list of enabled source books for a campaign",
            CampaignIdOptionalArgs,
            get_campaign_sources
        ),
        tool!(
            "create_campaign",
            "Create a new campaign. The new campaign is automatically set active, so you can immediately create modules and characters in it without a separate set_active_campaign call.",
            CreateCampaignArgs,
            create_campaign
        ),
        tool!(
            "update_campaign",
            "Update campaign name or description",
            UpdateCampaignArgs,
            update_campaign
        ),
        tool!(
            "delete_campaign",
            "Delete a campaign and all its data",
            DeleteCampaignArgs,
            delete_campaign
        ),
        tool!(
            "export_campaign",
            "Export the active campaign as a shareable archive file (.mimir-campaign.tar.gz). Requires an active campaign. The archive is written to the machine running this server (not the client).",
            ExportCampaignArgs,
            export_campaign
        ),
        tool!(
            "import_campaign",
            "Import a campaign from an archive file (.mimir-campaign.tar.gz). The imported campaign is automatically set active. The archive is read from the machine running this server (not the client).",
            ImportCampaignArgs,
            import_campaign
        ),
        tool!(
            "preview_archive",
            "Preview the contents of a campaign archive without importing it",
            PreviewArchiveArgs,
            preview_archive
        ),
    ]
}

// =============================================================================
// Arguments
// =============================================================================

tool_args! {
    pub struct ListCampaignsArgs {}
}

tool_args! {
    pub struct GetActiveCampaignArgs {}
}

tool_args! {
    pub struct SetActiveCampaignArgs {
        /// The ID of the campaign to set as active
        pub campaign_id: String,
    }
}

tool_args! {
    pub struct CampaignIdOptionalArgs {
        /// Campaign ID (optional, defaults to active campaign)
        pub campaign_id: Option<String>,
    }
}

tool_args! {
    pub struct CreateCampaignArgs {
        /// Name of the campaign
        pub name: String,
        /// Description of the campaign
        pub description: Option<String>,
    }
}

tool_args! {
    pub struct UpdateCampaignArgs {
        /// Campaign ID (optional, defaults to active campaign)
        pub campaign_id: Option<String>,
        /// New campaign name
        pub name: Option<String>,
        /// New campaign description
        pub description: Option<String>,
    }
}

tool_args! {
    pub struct DeleteCampaignArgs {
        /// The ID of the campaign to delete
        pub campaign_id: String,
    }
}

tool_args! {
    pub struct ExportCampaignArgs {
        /// Absolute directory path (on the server host) where the archive will be saved
        pub output_path: String,
    }
}

tool_args! {
    pub struct ImportCampaignArgs {
        /// Absolute path (on the server host) to the archive file to import
        pub archive_path: String,
        /// Optional new name for the imported campaign
        pub new_name: Option<String>,
    }
}

tool_args! {
    pub struct PreviewArchiveArgs {
        /// Path to the archive file to preview
        pub archive_path: String,
    }
}

// =============================================================================
// Handlers
// =============================================================================

pub async fn list_campaigns(
    ctx: &Arc<McpContext>,
    _args: ListCampaignsArgs,
) -> Result<Value, McpError> {
    let mut db = ctx.connect()?;
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

    McpResponse::list("campaigns", campaign_data)
}

pub async fn get_active_campaign(
    ctx: &Arc<McpContext>,
    _args: GetActiveCampaignArgs,
) -> Result<Value, McpError> {
    let Some(campaign_id) = ctx.get_active_campaign_id() else {
        return McpResponse::ok(json!({
            "active": false,
            "campaign": null,
            "hint": "No campaign is active. Call list_campaigns to see options and \
                     set_active_campaign to choose one, or create_campaign to start a new one."
        }));
    };

    let mut db = ctx.connect()?;
    let mut service = CampaignService::new(&mut db);

    match service
        .get(&campaign_id)
        .map_err(|e| McpError::Internal(e.to_string()))?
    {
        Some(campaign) => McpResponse::ok(json!({
            "active": true,
            "campaign": {
                "id": campaign.id,
                "name": campaign.name,
                "description": campaign.description
            }
        })),
        None => {
            // The selection points at a campaign that no longer exists — self-heal
            // so later calls don't keep failing against a phantom id.
            ctx.set_active_campaign_id(None);
            McpResponse::ok(json!({
                "active": false,
                "campaign": null,
                "hint": "The previously active campaign no longer exists and has been cleared. \
                         Call list_campaigns and set_active_campaign, or create_campaign."
            }))
        }
    }
}

pub async fn set_active_campaign(
    ctx: &Arc<McpContext>,
    args: SetActiveCampaignArgs,
) -> Result<Value, McpError> {
    let campaign_id = &args.campaign_id;

    // Verify campaign exists
    let mut db = ctx.connect()?;
    let mut service = CampaignService::new(&mut db);

    let campaign = service
        .get(campaign_id)
        .map_err(|e| McpError::Internal(e.to_string()))?
        .ok_or_else(|| McpError::InvalidArguments(format!("Campaign '{}' not found", campaign_id)))?;

    ctx.set_active_campaign_id(Some(campaign_id.clone()));

    McpResponse::success(json!({
        "active_campaign_id": campaign_id,
        "campaign": {
            "id": campaign.id,
            "name": campaign.name,
            "description": campaign.description
        }
    }))
}

pub async fn get_campaign_details(
    ctx: &Arc<McpContext>,
    args: CampaignIdOptionalArgs,
) -> Result<Value, McpError> {
    let campaign_id = args
        .campaign_id
        .or_else(|| ctx.get_active_campaign_id())
        .ok_or(McpError::NoActiveCampaign)?;

    let mut db = ctx.connect()?;

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

    McpResponse::ok(json!({
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

pub async fn get_campaign_sources(
    ctx: &Arc<McpContext>,
    args: CampaignIdOptionalArgs,
) -> Result<Value, McpError> {
    let campaign_id = args
        .campaign_id
        .or_else(|| ctx.get_active_campaign_id())
        .ok_or(McpError::NoActiveCampaign)?;

    let mut db = ctx.connect()?;

    let source_codes = dal::list_campaign_source_codes(&mut db, &campaign_id)
        .map_err(|e| McpError::Internal(e.to_string()))?;

    McpResponse::ok(json!({
        "campaign_id": campaign_id,
        "sources": source_codes
    }))
}

pub async fn create_campaign(
    ctx: &Arc<McpContext>,
    args: CreateCampaignArgs,
) -> Result<Value, McpError> {
    let mut db = ctx.connect()?;
    let mut service = CampaignService::new(&mut db);

    let mut input = CreateCampaignInput::new(&args.name);
    if let Some(desc) = args.description {
        input = input.with_description(desc);
    }

    let campaign = service
        .create(input)
        .map_err(|e| McpError::Internal(e.to_string()))?;

    // Auto-set as active
    ctx.set_active_campaign_id(Some(campaign.id.clone()));

    McpResponse::created("campaign", json!({
        "id": campaign.id,
        "name": campaign.name,
        "description": campaign.description
    }))
}

pub async fn update_campaign(
    ctx: &Arc<McpContext>,
    args: UpdateCampaignArgs,
) -> Result<Value, McpError> {
    let campaign_id = args
        .campaign_id
        .or_else(|| ctx.get_active_campaign_id())
        .ok_or(McpError::NoActiveCampaign)?;

    let mut input = UpdateCampaignInput::default();
    if let Some(name) = args.name {
        input.name = Some(name);
    }
    if let Some(desc) = args.description {
        input.description = Some(Some(desc));
    }

    let mut db = ctx.connect()?;
    let mut service = CampaignService::new(&mut db);

    let campaign = service
        .update(&campaign_id, input)
        .map_err(|e| McpError::Internal(e.to_string()))?;

    McpResponse::updated("campaign", json!({
        "id": campaign.id,
        "name": campaign.name,
        "description": campaign.description
    }))
}

pub async fn delete_campaign(
    ctx: &Arc<McpContext>,
    args: DeleteCampaignArgs,
) -> Result<Value, McpError> {
    let campaign_id = &args.campaign_id;

    let mut db = ctx.connect()?;
    let mut service = CampaignService::new(&mut db);

    service
        .delete(campaign_id)
        .map_err(|e| McpError::Internal(e.to_string()))?;

    // Clear active campaign if it was the deleted one
    if ctx.get_active_campaign_id().as_deref() == Some(campaign_id.as_str()) {
        ctx.set_active_campaign_id(None);
    }

    McpResponse::deleted(campaign_id)
}

pub async fn export_campaign(
    ctx: &Arc<McpContext>,
    args: ExportCampaignArgs,
) -> Result<Value, McpError> {
    let campaign_id = ctx.get_active_campaign_id()
        .ok_or(McpError::NoActiveCampaign)?;

    let mut db = ctx.connect()?;
    let output_dir = Path::new(&args.output_path);
    let assets_dir = &ctx.assets_dir;

    let archive_path = ArchiveService::new(&mut db)
        .export_campaign(&campaign_id, output_dir, assets_dir)
        .map_err(|e| McpError::Internal(format!("Export failed: {}", e)))?;

    // Get file size
    let size_bytes = std::fs::metadata(&archive_path)
        .map(|m| m.len())
        .unwrap_or(0);

    McpResponse::success(json!({
        "archive_path": archive_path.display().to_string(),
        "size_bytes": size_bytes
    }))
}

pub async fn import_campaign(
    ctx: &Arc<McpContext>,
    args: ImportCampaignArgs,
) -> Result<Value, McpError> {
    let mut db = ctx.connect()?;
    let archive = Path::new(&args.archive_path);
    let assets_dir = &ctx.assets_dir;

    let result = ArchiveService::new(&mut db)
        .import_campaign(archive, assets_dir, args.new_name.as_deref())
        .map_err(|e| McpError::Internal(format!("Import failed: {}", e)))?;

    // Set the imported campaign as active
    ctx.set_active_campaign_id(Some(result.campaign_id.clone()));

    McpResponse::success(json!({
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

pub async fn preview_archive(
    _ctx: &Arc<McpContext>,
    args: PreviewArchiveArgs,
) -> Result<Value, McpError> {
    let archive = Path::new(&args.archive_path);

    let preview = ArchiveService::preview_archive(archive)
        .map_err(|e| McpError::Internal(format!("Preview failed: {}", e)))?;

    McpResponse::ok(json!({
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
