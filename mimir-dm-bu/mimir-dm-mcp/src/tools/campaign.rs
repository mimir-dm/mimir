//! Campaign management tools for MCP
//!
//! Provides tools for listing campaigns, setting the active campaign,
//! and retrieving campaign details.

use crate::context::McpContext;
use crate::error::McpError;
use mimir_dm_core::services::CampaignService;
use rust_mcp_sdk::schema::{Tool, ToolInputSchema};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

/// Create a simple properties map for tool input schema
fn create_properties(
    props: Vec<(&str, &str, &str)>,
) -> Option<HashMap<String, serde_json::Map<String, serde_json::Value>>> {
    let mut map = HashMap::new();
    for (name, prop_type, description) in props {
        let mut inner = serde_json::Map::new();
        inner.insert(
            "type".to_string(),
            serde_json::Value::String(prop_type.to_string()),
        );
        inner.insert(
            "description".to_string(),
            serde_json::Value::String(description.to_string()),
        );
        map.insert(name.to_string(), inner);
    }
    if map.is_empty() {
        None
    } else {
        Some(map)
    }
}

/// Response from listing campaigns
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CampaignListItem {
    pub id: i32,
    pub name: String,
    pub status: String,
    pub directory_path: String,
    pub is_archived: bool,
    pub created_at: String,
    pub last_activity_at: String,
}

/// Input for list_campaigns tool
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ListCampaignsInput {
    /// Include archived campaigns in the list (default: false)
    #[serde(default)]
    pub include_archived: bool,
}

impl ListCampaignsInput {
    /// Get the tool definition
    pub fn tool() -> Tool {
        Tool {
            name: "list_campaigns".to_string(),
            description: Some(
                "List all campaigns. Use include_archived=true to also show archived campaigns."
                    .to_string(),
            ),
            input_schema: ToolInputSchema::new(
                vec![],
                create_properties(vec![(
                    "include_archived",
                    "boolean",
                    "Include archived campaigns in the list (default: false)",
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

    /// Execute the list_campaigns tool
    pub async fn execute(
        &self,
        context: Arc<McpContext>,
    ) -> Result<Vec<CampaignListItem>, McpError> {
        let mut conn = context.get_connection()?;
        let mut service = CampaignService::new(&mut conn);

        let campaigns = if self.include_archived {
            service
                .list_campaigns()
                .map_err(|e| McpError::Service(e.to_string()))?
        } else {
            service
                .list_active_campaigns()
                .map_err(|e| McpError::Service(e.to_string()))?
        };

        let items: Vec<CampaignListItem> = campaigns
            .into_iter()
            .map(|c| CampaignListItem {
                id: c.id,
                name: c.name,
                status: c.status,
                directory_path: c.directory_path,
                is_archived: c.archived_at.is_some(),
                created_at: c.created_at,
                last_activity_at: c.last_activity_at,
            })
            .collect();

        Ok(items)
    }
}

/// Input for set_active_campaign tool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetActiveCampaignInput {
    /// The campaign ID to set as active
    pub campaign_id: i32,
}

impl SetActiveCampaignInput {
    /// Get the tool definition
    pub fn tool() -> Tool {
        Tool {
            name: "set_active_campaign".to_string(),
            description: Some(
                "Set the active campaign for subsequent operations. Most other tools require an active campaign to be set first."
                    .to_string(),
            ),
            input_schema: ToolInputSchema::new(
                vec!["campaign_id".to_string()],
                create_properties(vec![(
                    "campaign_id",
                    "integer",
                    "The campaign ID to set as active",
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

    /// Execute the set_active_campaign tool
    pub async fn execute(
        &self,
        context: Arc<McpContext>,
    ) -> Result<SetActiveCampaignResponse, McpError> {
        let mut conn = context.get_connection()?;
        let mut service = CampaignService::new(&mut conn);

        let campaign = service
            .get_campaign(self.campaign_id)
            .map_err(|e| McpError::Service(e.to_string()))?
            .ok_or_else(|| McpError::CampaignNotFound(self.campaign_id.to_string()))?;

        let response = SetActiveCampaignResponse {
            success: true,
            campaign_id: campaign.id,
            campaign_name: campaign.name.clone(),
            status: campaign.status.clone(),
            directory_path: campaign.directory_path.clone(),
        };

        context.set_active_campaign(campaign).await;

        Ok(response)
    }
}

/// Response from set_active_campaign
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetActiveCampaignResponse {
    pub success: bool,
    pub campaign_id: i32,
    pub campaign_name: String,
    pub status: String,
    pub directory_path: String,
}

/// Input for get_campaign_details tool
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GetCampaignDetailsInput {}

impl GetCampaignDetailsInput {
    /// Get the tool definition
    pub fn tool() -> Tool {
        Tool {
            name: "get_campaign_details".to_string(),
            description: Some(
                "Get detailed information about the active campaign, including its current stage, modules, and completion status."
                    .to_string(),
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

    /// Execute the get_campaign_details tool
    pub async fn execute(
        &self,
        context: Arc<McpContext>,
    ) -> Result<CampaignDetailsResponse, McpError> {
        let campaign = context.require_active_campaign().await?;

        let mut conn = context.get_connection()?;

        // Get modules for this campaign
        let modules = {
            use mimir_dm_core::services::ModuleService;
            let mut module_service = ModuleService::new(&mut conn);
            module_service
                .list_campaign_modules(campaign.id)
                .map_err(|e| McpError::Service(e.to_string()))?
        };

        let module_summaries: Vec<ModuleSummary> = modules
            .into_iter()
            .map(|m| ModuleSummary {
                id: m.id,
                name: m.name,
                status: m.status,
                module_number: m.module_number,
            })
            .collect();

        // Get stage completion status
        let stage_completion = {
            let mut campaign_service = CampaignService::new(&mut conn);
            match campaign_service.check_stage_completion(campaign.id) {
                Ok(status) => Some(StageCompletionInfo {
                    current_stage: status.current_stage,
                    total_required_documents: status.total_required_documents,
                    completed_required_documents: status.completed_required_documents,
                    missing_required_documents: status.missing_required_documents,
                    can_progress: status.can_progress,
                    next_stage: status.next_stage,
                }),
                Err(_) => None,
            }
        };

        Ok(CampaignDetailsResponse {
            id: campaign.id,
            name: campaign.name,
            status: campaign.status,
            directory_path: campaign.directory_path,
            created_at: campaign.created_at,
            session_zero_date: campaign.session_zero_date,
            first_session_date: campaign.first_session_date,
            last_activity_at: campaign.last_activity_at,
            modules: module_summaries,
            stage_completion,
        })
    }
}

/// Module summary for campaign details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleSummary {
    pub id: i32,
    pub name: String,
    pub status: String,
    pub module_number: i32,
}

/// Campaign details response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CampaignDetailsResponse {
    pub id: i32,
    pub name: String,
    pub status: String,
    pub directory_path: String,
    pub created_at: String,
    pub session_zero_date: Option<String>,
    pub first_session_date: Option<String>,
    pub last_activity_at: String,
    pub modules: Vec<ModuleSummary>,
    pub stage_completion: Option<StageCompletionInfo>,
}

/// Stage completion information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StageCompletionInfo {
    pub current_stage: String,
    pub total_required_documents: usize,
    pub completed_required_documents: usize,
    pub missing_required_documents: Vec<String>,
    pub can_progress: bool,
    pub next_stage: Option<String>,
}

/// Input for create_module tool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateModuleInput {
    /// Module name/title
    pub name: String,

    /// Module type for template selection: "mystery", "dungeon", "heist", "horror", "political", or none for generic
    #[serde(default)]
    pub module_type: Option<String>,
}

impl CreateModuleInput {
    /// Get the tool definition
    pub fn tool() -> Tool {
        Tool {
            name: "create_module".to_string(),
            description: Some(
                "Create a new module (adventure/arc) in the active campaign. Types: mystery, dungeon, heist, horror, political (or omit for generic)."
                    .to_string(),
            ),
            input_schema: ToolInputSchema::new(
                vec!["name".to_string()],
                create_properties(vec![
                    ("name", "string", "Module name/title"),
                    (
                        "module_type",
                        "string",
                        "Type for template: mystery, dungeon, heist, horror, political (optional)",
                    ),
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

    /// Execute the create_module tool
    pub async fn execute(
        &self,
        context: Arc<McpContext>,
    ) -> Result<CreateModuleResponse, McpError> {
        let campaign = context.require_active_campaign().await?;
        let mut conn = context.get_connection()?;

        use mimir_dm_core::services::ModuleService;
        let mut service = ModuleService::new(&mut conn);

        let module = service
            .create_module_with_documents(
                campaign.id,
                self.name.clone(),
                3, // Default expected sessions
                self.module_type.clone(),
            )
            .map_err(|e| McpError::Service(e.to_string()))?;

        Ok(CreateModuleResponse {
            success: true,
            module_id: module.id,
            name: module.name,
            module_number: module.module_number,
            status: module.status,
        })
    }
}

/// Response from create_module
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateModuleResponse {
    pub success: bool,
    pub module_id: i32,
    pub name: String,
    pub module_number: i32,
    pub status: String,
}

/// Input for list_modules tool
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ListModulesInput {
    /// Filter by status: "planning", "active", "completed", or "all" (default: "all")
    #[serde(default)]
    pub status: Option<String>,
}

impl ListModulesInput {
    /// Get the tool definition
    pub fn tool() -> Tool {
        Tool {
            name: "list_modules".to_string(),
            description: Some(
                "List modules in the active campaign. Filter by status: planning, active, completed, or all (default)."
                    .to_string(),
            ),
            input_schema: ToolInputSchema::new(
                vec![],
                create_properties(vec![(
                    "status",
                    "string",
                    "Filter by status: planning, active, completed, or all (default: all)",
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

    /// Execute the list_modules tool
    pub async fn execute(
        &self,
        context: Arc<McpContext>,
    ) -> Result<Vec<ModuleListItem>, McpError> {
        let campaign = context.require_active_campaign().await?;
        let mut conn = context.get_connection()?;

        use mimir_dm_core::services::ModuleService;
        let mut service = ModuleService::new(&mut conn);

        let modules = match self.status.as_deref() {
            Some("all") | None => service
                .list_campaign_modules(campaign.id)
                .map_err(|e| McpError::Service(e.to_string()))?,
            Some(status) => service
                .list_modules_by_status(campaign.id, status)
                .map_err(|e| McpError::Service(e.to_string()))?,
        };

        let items: Vec<ModuleListItem> = modules
            .into_iter()
            .map(|m| ModuleListItem {
                id: m.id,
                name: m.name,
                module_number: m.module_number,
                status: m.status,
                expected_sessions: m.expected_sessions,
                actual_sessions: m.actual_sessions,
                started_at: m.started_at,
                completed_at: m.completed_at,
            })
            .collect();

        Ok(items)
    }
}

/// Module list item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleListItem {
    pub id: i32,
    pub name: String,
    pub module_number: i32,
    pub status: String,
    pub expected_sessions: i32,
    pub actual_sessions: i32,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
}

/// Input for get_module_details tool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetModuleDetailsInput {
    /// The module ID to retrieve
    pub module_id: i32,
}

impl GetModuleDetailsInput {
    /// Get the tool definition
    pub fn tool() -> Tool {
        Tool {
            name: "get_module_details".to_string(),
            description: Some(
                "Get detailed information about a module, including its documents and NPCs."
                    .to_string(),
            ),
            input_schema: ToolInputSchema::new(
                vec!["module_id".to_string()],
                create_properties(vec![(
                    "module_id",
                    "integer",
                    "The module ID to retrieve",
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

    /// Execute the get_module_details tool
    pub async fn execute(
        &self,
        context: Arc<McpContext>,
    ) -> Result<ModuleDetailsResponse, McpError> {
        let _campaign = context.require_active_campaign().await?;
        let mut conn = context.get_connection()?;

        use mimir_dm_core::services::{ModuleNpcService, ModuleService};

        let mut service = ModuleService::new(&mut conn);

        let module = service
            .get_module(self.module_id)
            .map_err(|e| McpError::Service(e.to_string()))?
            .ok_or_else(|| McpError::Service(format!("Module {} not found", self.module_id)))?;

        // Get documents for this module
        let documents = service
            .get_module_documents(self.module_id)
            .map_err(|e| McpError::Service(e.to_string()))?;

        let document_summaries: Vec<ModuleDocumentSummary> = documents
            .into_iter()
            .map(|d| ModuleDocumentSummary {
                id: d.id,
                title: d.title,
                document_type: d.document_type,
            })
            .collect();

        // Get NPCs for this module
        let mut npc_service = ModuleNpcService::new(&mut conn);
        let npcs = npc_service
            .get_npcs_with_character_data(self.module_id)
            .map_err(|e| McpError::Service(e.to_string()))?;

        let npc_summaries: Vec<ModuleNpcSummary> = npcs
            .into_iter()
            .map(|n| ModuleNpcSummary {
                character_id: n.character_id,
                name: n.character_name,
                role: n.role,
                encounter_tag: n.encounter_tag,
            })
            .collect();

        Ok(ModuleDetailsResponse {
            id: module.id,
            name: module.name,
            module_number: module.module_number,
            status: module.status,
            created_at: module.created_at,
            documents: document_summaries,
            npcs: npc_summaries,
        })
    }
}

/// Module details response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleDetailsResponse {
    pub id: i32,
    pub name: String,
    pub module_number: i32,
    pub status: String,
    pub created_at: String,
    pub documents: Vec<ModuleDocumentSummary>,
    pub npcs: Vec<ModuleNpcSummary>,
}

/// Document summary for module details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleDocumentSummary {
    pub id: i32,
    pub title: String,
    pub document_type: String,
}

/// NPC summary for module details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleNpcSummary {
    pub character_id: i32,
    pub name: String,
    pub role: Option<String>,
    pub encounter_tag: Option<String>,
}

/// Module completion info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleCompletionInfo {
    pub total_required: usize,
    pub completed_required: usize,
    pub missing_documents: Vec<String>,
    pub can_progress: bool,
}

/// Input for add_monster_to_module tool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddMonsterToModuleInput {
    /// The module ID to add the monster to
    pub module_id: i32,

    /// Monster name (must exist in catalog)
    pub monster_name: String,

    /// Monster source book (e.g., "MM", "PHB")
    pub monster_source: String,

    /// Number of this monster (default: 1)
    #[serde(default = "default_quantity")]
    pub quantity: i32,

    /// Encounter tag for grouping (e.g., "goblin_ambush", "boss_fight")
    #[serde(default)]
    pub encounter_tag: Option<String>,

    /// Custom display name for this monster (e.g., "Frost Wight" when using goblin stats)
    #[serde(default)]
    pub display_name: Option<String>,

    /// DM notes about customizations or thematic changes
    #[serde(default)]
    pub notes: Option<String>,
}

fn default_quantity() -> i32 {
    1
}

impl AddMonsterToModuleInput {
    /// Get the tool definition
    pub fn tool() -> Tool {
        Tool {
            name: "add_monster_to_module".to_string(),
            description: Some(
                "Add a monster from the catalog to a module. Use encounter_tag to group monsters for specific encounters. Use display_name to give it a custom name (e.g., 'Frost Wight' when using goblin stats)."
                    .to_string(),
            ),
            input_schema: ToolInputSchema::new(
                vec![
                    "module_id".to_string(),
                    "monster_name".to_string(),
                    "monster_source".to_string(),
                ],
                create_properties(vec![
                    ("module_id", "integer", "The module ID to add the monster to"),
                    ("monster_name", "string", "Monster name (must exist in catalog)"),
                    ("monster_source", "string", "Monster source book (e.g., MM, PHB)"),
                    ("quantity", "integer", "Number of this monster (default: 1)"),
                    (
                        "encounter_tag",
                        "string",
                        "Encounter tag for grouping (e.g., goblin_ambush, boss_fight)",
                    ),
                    (
                        "display_name",
                        "string",
                        "Custom display name (e.g., 'Frost Wight' when using goblin stats)",
                    ),
                    (
                        "notes",
                        "string",
                        "DM notes about customizations or thematic changes",
                    ),
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

    /// Execute the add_monster_to_module tool
    pub async fn execute(
        &self,
        context: Arc<McpContext>,
    ) -> Result<AddMonsterResponse, McpError> {
        let _campaign = context.require_active_campaign().await?;
        let mut conn = context.get_connection()?;

        use mimir_dm_core::services::ModuleMonsterService;
        let mut service = ModuleMonsterService::new(&mut conn);

        let monster = service
            .add_monster(
                self.module_id,
                self.monster_name.clone(),
                self.monster_source.clone(),
                self.quantity,
                self.encounter_tag.clone(),
                self.display_name.clone(),
                self.notes.clone(),
            )
            .map_err(|e| McpError::Service(e.to_string()))?;

        Ok(AddMonsterResponse {
            success: true,
            monster_id: monster.id,
            module_id: monster.module_id,
            monster_name: monster.monster_name,
            monster_source: monster.monster_source,
            quantity: monster.quantity,
            encounter_tag: monster.encounter_tag,
            display_name: monster.display_name,
            notes: monster.notes,
        })
    }
}

/// Response from add_monster_to_module
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddMonsterResponse {
    pub success: bool,
    pub monster_id: i32,
    pub module_id: i32,
    pub monster_name: String,
    pub monster_source: String,
    pub quantity: i32,
    pub encounter_tag: Option<String>,
    pub display_name: Option<String>,
    pub notes: Option<String>,
}

/// Input for add_item_to_module tool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddItemToModuleInput {
    /// The module ID to add the item to
    pub module_id: i32,

    /// Item name (must exist in catalog)
    pub item_name: String,

    /// Item source book (e.g., "PHB", "DMG")
    pub item_source: String,

    /// Quantity of this item (default: 1)
    #[serde(default = "default_quantity")]
    pub quantity: i32,

    /// Location where item can be found (e.g., "treasure chest", "boss loot")
    #[serde(default)]
    pub location: Option<String>,

    /// Notes about the item
    #[serde(default)]
    pub notes: Option<String>,
}

impl AddItemToModuleInput {
    /// Get the tool definition
    pub fn tool() -> Tool {
        Tool {
            name: "add_item_to_module".to_string(),
            description: Some(
                "Add an item from the catalog to a module as treasure or loot. Use location to specify where it can be found."
                    .to_string(),
            ),
            input_schema: ToolInputSchema::new(
                vec![
                    "module_id".to_string(),
                    "item_name".to_string(),
                    "item_source".to_string(),
                ],
                create_properties(vec![
                    ("module_id", "integer", "The module ID to add the item to"),
                    ("item_name", "string", "Item name (must exist in catalog)"),
                    ("item_source", "string", "Item source book (e.g., PHB, DMG)"),
                    ("quantity", "integer", "Quantity of this item (default: 1)"),
                    (
                        "location",
                        "string",
                        "Location where item can be found (e.g., treasure chest, boss loot)",
                    ),
                    ("notes", "string", "Notes about the item"),
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

    /// Execute the add_item_to_module tool
    pub async fn execute(
        &self,
        context: Arc<McpContext>,
    ) -> Result<AddModuleItemResponse, McpError> {
        let _campaign = context.require_active_campaign().await?;
        let mut conn = context.get_connection()?;

        use mimir_dm_core::services::ModuleItemService;
        let mut service = ModuleItemService::new(&mut conn);

        let item = service
            .add_item(
                self.module_id,
                self.item_name.clone(),
                self.item_source.clone(),
                self.quantity,
                self.location.clone(),
                self.notes.clone(),
            )
            .map_err(|e| McpError::Service(e.to_string()))?;

        Ok(AddModuleItemResponse {
            success: true,
            item_id: item.id,
            module_id: item.module_id,
            item_name: item.name,
            item_source: item.source,
            quantity: item.quantity,
            location: item.location,
        })
    }
}

/// Response from add_item_to_module
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddModuleItemResponse {
    pub success: bool,
    pub item_id: i32,
    pub module_id: i32,
    pub item_name: String,
    pub item_source: String,
    pub quantity: i32,
    pub location: Option<String>,
}

/// Input for update_module_monster tool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateModuleMonsterInput {
    /// The module monster ID to update
    pub monster_id: i32,

    /// New quantity (optional)
    #[serde(default)]
    pub quantity: Option<i32>,

    /// New encounter tag (optional, use null to clear)
    #[serde(default)]
    pub encounter_tag: Option<Option<String>>,

    /// New custom display name (optional, use null to clear)
    #[serde(default)]
    pub display_name: Option<Option<String>>,

    /// New DM notes (optional, use null to clear)
    #[serde(default)]
    pub notes: Option<Option<String>>,
}

impl UpdateModuleMonsterInput {
    /// Get the tool definition
    pub fn tool() -> Tool {
        Tool {
            name: "update_module_monster".to_string(),
            description: Some(
                "Update a monster entry in a module. Can change quantity, encounter tag, display name, or notes. Use null to clear optional fields."
                    .to_string(),
            ),
            input_schema: ToolInputSchema::new(
                vec!["monster_id".to_string()],
                create_properties(vec![
                    ("monster_id", "integer", "The module monster ID to update"),
                    ("quantity", "integer", "New quantity (optional)"),
                    (
                        "encounter_tag",
                        "string",
                        "New encounter tag (optional, use null to clear)",
                    ),
                    (
                        "display_name",
                        "string",
                        "New custom display name (optional, use null to clear)",
                    ),
                    (
                        "notes",
                        "string",
                        "New DM notes about customizations (optional, use null to clear)",
                    ),
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

    /// Execute the update_module_monster tool
    pub async fn execute(
        &self,
        context: Arc<McpContext>,
    ) -> Result<UpdateMonsterResponse, McpError> {
        let _campaign = context.require_active_campaign().await?;
        let mut conn = context.get_connection()?;

        use mimir_dm_core::services::ModuleMonsterService;
        let mut service = ModuleMonsterService::new(&mut conn);

        let monster = service
            .update_monster(
                self.monster_id,
                self.quantity,
                self.encounter_tag.clone(),
                self.display_name.clone(),
                self.notes.clone(),
            )
            .map_err(|e| McpError::Service(e.to_string()))?;

        Ok(UpdateMonsterResponse {
            success: true,
            monster_id: monster.id,
            module_id: monster.module_id,
            monster_name: monster.monster_name,
            monster_source: monster.monster_source,
            quantity: monster.quantity,
            encounter_tag: monster.encounter_tag,
            display_name: monster.display_name,
            notes: monster.notes,
        })
    }
}

/// Response from update_module_monster
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateMonsterResponse {
    pub success: bool,
    pub monster_id: i32,
    pub module_id: i32,
    pub monster_name: String,
    pub monster_source: String,
    pub quantity: i32,
    pub encounter_tag: Option<String>,
    pub display_name: Option<String>,
    pub notes: Option<String>,
}
