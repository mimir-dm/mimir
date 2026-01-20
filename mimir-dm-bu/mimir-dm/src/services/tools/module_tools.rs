//! Module management tools for LLM interactions
//!
//! These tools allow LLMs to create and manage adventure modules

use async_trait::async_trait;
use mimir_dm_core::services::ModuleService;
use mimir_dm_core::DatabaseService;
use mimir_dm_llm::traits::{ActionDescription, ChangeDetail};
use mimir_dm_llm::ToolTrait;
use serde_json::{json, Value};
use std::error::Error;
use std::sync::Arc;
use tracing::debug;

/// Tool for creating a new adventure module
pub struct CreateModuleTool {
    db_service: Arc<DatabaseService>,
}

impl CreateModuleTool {
    pub fn new(db_service: Arc<DatabaseService>) -> Self {
        Self { db_service }
    }
}

#[async_trait]
impl ToolTrait for CreateModuleTool {
    fn name(&self) -> &str {
        "create_module"
    }

    fn description(&self) -> &str {
        "Create a new adventure module for a campaign.

Usage:
- Provide campaign_id, name, and expected_sessions
- Optionally provide module_type for specialized templates
- Creates module directory and initial documents

Module types:
- mystery: Investigation-focused adventure
- dungeon: Classic dungeon crawl
- heist: Stealth and planning focused
- horror: Survival horror theme
- political: Intrigue and diplomacy
- (default): Generic module template

When to use:
- Starting a new adventure arc
- Planning upcoming content
- After completing current module

Output:
- Created module with ID and path
- Initial documents generated from template"
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "campaign_id": {
                    "type": "integer",
                    "description": "ID of the campaign"
                },
                "name": {
                    "type": "string",
                    "description": "Module name/title"
                },
                "expected_sessions": {
                    "type": "integer",
                    "description": "Expected number of sessions (typically 3-6)"
                },
                "module_type": {
                    "type": ["string", "null"],
                    "description": "Module type for template selection (mystery, dungeon, heist, horror, political)"
                }
            },
            "required": ["campaign_id", "name", "expected_sessions"]
        })
    }

    fn requires_confirmation(&self) -> bool {
        true
    }

    fn describe_action(&self, arguments: &Value) -> Option<ActionDescription> {
        let campaign_id = arguments.get("campaign_id")?.as_i64()?;
        let name = arguments.get("name")?.as_str()?;
        let expected_sessions = arguments.get("expected_sessions")?.as_i64()?;
        let module_type = arguments
            .get("module_type")
            .and_then(|v| v.as_str())
            .unwrap_or("generic");

        Some(ActionDescription {
            title: "Create Module".to_string(),
            description: format!("Create new module '{}' for campaign {}", name, campaign_id),
            changes: ChangeDetail::Generic {
                items: vec![
                    format!("Campaign ID: {}", campaign_id),
                    format!("Name: {}", name),
                    format!("Expected sessions: {}", expected_sessions),
                    format!("Type: {}", module_type),
                ],
            },
        })
    }

    async fn execute(&self, arguments: Value) -> Result<String, Box<dyn Error + Send + Sync>> {
        let campaign_id = arguments
            .get("campaign_id")
            .and_then(|v| v.as_i64())
            .ok_or("Missing campaign_id")? as i32;

        let name = arguments
            .get("name")
            .and_then(|v| v.as_str())
            .ok_or("Missing name")?
            .to_string();

        let expected_sessions = arguments
            .get("expected_sessions")
            .and_then(|v| v.as_i64())
            .ok_or("Missing expected_sessions")? as i32;

        let module_type = arguments
            .get("module_type")
            .and_then(|v| v.as_str())
            .map(String::from);

        let mut conn = self
            .db_service
            .get_connection()
            .map_err(|e| format!("Database error: {}", e))?;

        let mut module_service = ModuleService::new(&mut conn);
        let module = module_service
            .create_module_with_documents(campaign_id, name.clone(), expected_sessions, module_type)
            .map_err(|e| format!("Failed to create module: {}", e))?;

        let result = json!({
            "success": true,
            "module_id": module.id,
            "module_number": module.module_number,
            "name": module.name,
            "status": module.status,
            "expected_sessions": module.expected_sessions,
            "message": format!("Created module '{}' (Module {})", name, module.module_number)
        });

        debug!("Created module {} for campaign {}", module.id, campaign_id);
        Ok(serde_json::to_string_pretty(&result)?)
    }
}

/// Tool for listing modules in a campaign
pub struct ListModulesTool {
    db_service: Arc<DatabaseService>,
}

impl ListModulesTool {
    pub fn new(db_service: Arc<DatabaseService>) -> Self {
        Self { db_service }
    }
}

#[async_trait]
impl ToolTrait for ListModulesTool {
    fn name(&self) -> &str {
        "list_modules"
    }

    fn description(&self) -> &str {
        "List all modules for a campaign.

Usage:
- Provide campaign_id
- Optionally filter by status

When to use:
- Reviewing campaign progress
- Finding active or completed modules
- Checking module backlog

Output:
- List of modules with status and session counts"
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "campaign_id": {
                    "type": "integer",
                    "description": "ID of the campaign"
                },
                "status": {
                    "type": ["string", "null"],
                    "description": "Filter by status (planning, preparation, running, completed)"
                }
            },
            "required": ["campaign_id"]
        })
    }

    fn requires_confirmation(&self) -> bool {
        false
    }

    async fn execute(&self, arguments: Value) -> Result<String, Box<dyn Error + Send + Sync>> {
        let campaign_id = arguments
            .get("campaign_id")
            .and_then(|v| v.as_i64())
            .ok_or("Missing campaign_id")? as i32;

        let status_filter = arguments
            .get("status")
            .and_then(|v| v.as_str());

        let mut conn = self
            .db_service
            .get_connection()
            .map_err(|e| format!("Database error: {}", e))?;

        let mut module_service = ModuleService::new(&mut conn);
        let modules = if let Some(status) = status_filter {
            module_service
                .list_modules_by_status(campaign_id, status)
                .map_err(|e| format!("Failed to list modules: {}", e))?
        } else {
            module_service
                .list_campaign_modules(campaign_id)
                .map_err(|e| format!("Failed to list modules: {}", e))?
        };

        let result = json!({
            "count": modules.len(),
            "modules": modules.iter().map(|m| json!({
                "id": m.id,
                "module_number": m.module_number,
                "name": m.name,
                "status": m.status,
                "expected_sessions": m.expected_sessions,
                "actual_sessions": m.actual_sessions
            })).collect::<Vec<_>>()
        });

        debug!("Listed {} modules for campaign {}", modules.len(), campaign_id);
        Ok(serde_json::to_string_pretty(&result)?)
    }
}

/// Tool for getting module details
pub struct GetModuleTool {
    db_service: Arc<DatabaseService>,
}

impl GetModuleTool {
    pub fn new(db_service: Arc<DatabaseService>) -> Self {
        Self { db_service }
    }
}

#[async_trait]
impl ToolTrait for GetModuleTool {
    fn name(&self) -> &str {
        "get_module"
    }

    fn description(&self) -> &str {
        "Get details about a specific module including documents and completion status.

Usage:
- Provide module_id

When to use:
- Checking module progress
- Reviewing module documents
- Preparing for session planning

Output:
- Module details with documents and completion metrics"
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "module_id": {
                    "type": "integer",
                    "description": "ID of the module"
                }
            },
            "required": ["module_id"]
        })
    }

    fn requires_confirmation(&self) -> bool {
        false
    }

    async fn execute(&self, arguments: Value) -> Result<String, Box<dyn Error + Send + Sync>> {
        let module_id = arguments
            .get("module_id")
            .and_then(|v| v.as_i64())
            .ok_or("Missing module_id")? as i32;

        let mut conn = self
            .db_service
            .get_connection()
            .map_err(|e| format!("Database error: {}", e))?;

        let mut module_service = ModuleService::new(&mut conn);

        let module = module_service
            .get_module(module_id)
            .map_err(|e| format!("Failed to get module: {}", e))?
            .ok_or("Module not found")?;

        let documents = module_service
            .get_module_documents(module_id)
            .map_err(|e| format!("Failed to get documents: {}", e))?;

        let completion = module_service
            .check_module_completion(module_id)
            .map_err(|e| format!("Failed to check completion: {}", e))?;

        let result = json!({
            "module": {
                "id": module.id,
                "module_number": module.module_number,
                "name": module.name,
                "status": module.status,
                "expected_sessions": module.expected_sessions,
                "actual_sessions": module.actual_sessions
            },
            "documents": documents.iter().map(|d| json!({
                "id": d.id,
                "title": d.title,
                "document_type": d.document_type,
                "completed": d.completed_at.is_some()
            })).collect::<Vec<_>>(),
            "completion": {
                "current_stage": completion.current_stage,
                "required_complete": format!("{}/{}", completion.completed_required_documents, completion.total_required_documents),
                "optional_complete": format!("{}/{}", completion.completed_optional_documents, completion.total_optional_documents),
                "missing_required": completion.missing_required_documents,
                "can_progress": completion.can_progress,
                "next_stage": completion.next_stage
            }
        });

        debug!("Retrieved module {} details", module_id);
        Ok(serde_json::to_string_pretty(&result)?)
    }
}

/// Tool for updating module status
pub struct UpdateModuleStatusTool {
    db_service: Arc<DatabaseService>,
}

impl UpdateModuleStatusTool {
    pub fn new(db_service: Arc<DatabaseService>) -> Self {
        Self { db_service }
    }
}

#[async_trait]
impl ToolTrait for UpdateModuleStatusTool {
    fn name(&self) -> &str {
        "update_module_status"
    }

    fn description(&self) -> &str {
        "Update a module's status or transition to a new stage.

Usage:
- Provide module_id and new_status
- Status must be valid for current stage

Valid statuses:
- planning: Initial planning stage
- preparation: Preparing content and documents
- running: Currently active module
- completed: Module finished

When to use:
- Starting to run a planned module
- Completing a module
- Moving through workflow stages

Output:
- Updated module status confirmed"
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "module_id": {
                    "type": "integer",
                    "description": "ID of the module"
                },
                "new_status": {
                    "type": "string",
                    "description": "New status (planning, preparation, running, completed)"
                }
            },
            "required": ["module_id", "new_status"]
        })
    }

    fn requires_confirmation(&self) -> bool {
        true
    }

    fn describe_action(&self, arguments: &Value) -> Option<ActionDescription> {
        let module_id = arguments.get("module_id")?.as_i64()?;
        let new_status = arguments.get("new_status")?.as_str()?;

        Some(ActionDescription {
            title: "Update Module Status".to_string(),
            description: format!("Transition module {} to '{}'", module_id, new_status),
            changes: ChangeDetail::Generic {
                items: vec![
                    format!("Module ID: {}", module_id),
                    format!("New status: {}", new_status),
                ],
            },
        })
    }

    async fn execute(&self, arguments: Value) -> Result<String, Box<dyn Error + Send + Sync>> {
        let module_id = arguments
            .get("module_id")
            .and_then(|v| v.as_i64())
            .ok_or("Missing module_id")? as i32;

        let new_status = arguments
            .get("new_status")
            .and_then(|v| v.as_str())
            .ok_or("Missing new_status")?;

        let mut conn = self
            .db_service
            .get_connection()
            .map_err(|e| format!("Database error: {}", e))?;

        let mut module_service = ModuleService::new(&mut conn);

        let module = module_service
            .transition_module_stage(module_id, new_status)
            .map_err(|e| format!("Failed to transition module: {}", e))?;

        let result = json!({
            "success": true,
            "module_id": module.id,
            "name": module.name,
            "previous_status": "transitioned",
            "new_status": module.status,
            "message": format!("Module '{}' transitioned to '{}'", module.name, module.status)
        });

        debug!("Transitioned module {} to {}", module_id, new_status);
        Ok(serde_json::to_string_pretty(&result)?)
    }
}
