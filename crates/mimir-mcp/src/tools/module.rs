//! Module Tools
//!
//! MCP tools for module management.

use mimir_core::dal::campaign as dal;
use mimir_core::models::campaign::NewModuleMonster;
use mimir_core::services::{CreateModuleInput, ModuleService, ModuleType, UpdateModuleInput};
use rust_mcp_sdk::schema::{Tool, ToolInputSchema};
use serde_json::{json, Value};
use std::sync::Arc;
use uuid::Uuid;

use super::create_properties;
use crate::context::McpContext;
use crate::McpError;

// =============================================================================
// Tool Definitions
// =============================================================================

pub fn create_module_tool() -> Tool {
    Tool {
        name: "create_module".to_string(),
        description: Some("Create a new module in the active campaign".to_string()),
        input_schema: ToolInputSchema::new(
            vec!["name".to_string()],
            create_properties(vec![
                ("name", "string", "Name of the module"),
                ("description", "string", "Description of the module"),
                (
                    "module_type",
                    "string",
                    "Type of module: adventure, location, organization (default: adventure)",
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

pub fn list_modules_tool() -> Tool {
    Tool {
        name: "list_modules".to_string(),
        description: Some("List all modules in the active campaign".to_string()),
        input_schema: ToolInputSchema::new(vec![], None, None),
        title: None,
        annotations: None,
        icons: vec![],
        execution: None,
        output_schema: None,
        meta: None,
    }
}

pub fn get_module_details_tool() -> Tool {
    Tool {
        name: "get_module_details".to_string(),
        description: Some(
            "Get detailed information about a module including documents, monsters, and items"
                .to_string(),
        ),
        input_schema: ToolInputSchema::new(
            vec!["module_id".to_string()],
            create_properties(vec![("module_id", "string", "The ID of the module")]),
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

pub fn add_monster_to_module_tool() -> Tool {
    Tool {
        name: "add_monster_to_module".to_string(),
        description: Some("Add a monster from the catalog to a module".to_string()),
        input_schema: ToolInputSchema::new(
            vec!["module_id".to_string(), "monster_name".to_string()],
            create_properties(vec![
                ("module_id", "string", "The ID of the module"),
                ("monster_name", "string", "Name of the monster from the catalog"),
                ("count", "integer", "Number of this monster (default: 1)"),
                ("notes", "string", "Optional notes about this monster"),
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

pub fn add_item_to_module_tool() -> Tool {
    Tool {
        name: "add_item_to_module".to_string(),
        description: Some("Add an item from the catalog to a module as loot".to_string()),
        input_schema: ToolInputSchema::new(
            vec!["module_id".to_string(), "item_name".to_string()],
            create_properties(vec![
                ("module_id", "string", "The ID of the module"),
                ("item_name", "string", "Name of the item from the catalog"),
                ("quantity", "integer", "Quantity of this item (default: 1)"),
                ("notes", "string", "Optional notes about this item"),
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

pub fn update_module_tool() -> Tool {
    Tool {
        name: "update_module".to_string(),
        description: Some("Update a module's name or description".to_string()),
        input_schema: ToolInputSchema::new(
            vec!["module_id".to_string()],
            create_properties(vec![
                ("module_id", "string", "The ID of the module"),
                ("name", "string", "New module name"),
                ("description", "string", "New module description"),
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

pub fn delete_module_tool() -> Tool {
    Tool {
        name: "delete_module".to_string(),
        description: Some("Delete a module and all its contents".to_string()),
        input_schema: ToolInputSchema::new(
            vec!["module_id".to_string()],
            create_properties(vec![
                ("module_id", "string", "The ID of the module to delete"),
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

pub async fn create_module(ctx: &Arc<McpContext>, args: Value) -> Result<Value, McpError> {
    let campaign_id = ctx
        .get_active_campaign_id()
        .ok_or(McpError::NoActiveCampaign)?;

    let name = args
        .get("name")
        .and_then(|v| v.as_str())
        .ok_or_else(|| McpError::InvalidArguments("name is required".to_string()))?;

    let description = args.get("description").and_then(|v| v.as_str());

    // Parse module type
    let module_type = match args.get("module_type").and_then(|v| v.as_str()) {
        Some("mystery") => ModuleType::Mystery,
        Some("dungeon") => ModuleType::Dungeon,
        Some("heist") => ModuleType::Heist,
        Some("horror") => ModuleType::Horror,
        Some("political") => ModuleType::Political,
        _ => ModuleType::General,
    };

    let mut db = ctx.db()?;
    let mut service = ModuleService::new(&mut db);

    let mut input = CreateModuleInput::new(&campaign_id, name).with_type(module_type);
    if let Some(desc) = description {
        input = input.with_description(desc);
    }

    let module = service
        .create(input)
        .map_err(|e| McpError::Internal(e.to_string()))?;

    Ok(json!({
        "status": "created",
        "module": {
            "id": module.id,
            "name": module.name,
            "description": module.description,
            "module_number": module.module_number
        }
    }))
}

pub async fn list_modules(ctx: &Arc<McpContext>, _args: Value) -> Result<Value, McpError> {
    let campaign_id = ctx
        .get_active_campaign_id()
        .ok_or(McpError::NoActiveCampaign)?;

    let mut db = ctx.db()?;
    let mut service = ModuleService::new(&mut db);

    let modules = service
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

    Ok(json!({
        "modules": module_data
    }))
}

pub async fn get_module_details(ctx: &Arc<McpContext>, args: Value) -> Result<Value, McpError> {
    let module_id = args
        .get("module_id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| McpError::InvalidArguments("module_id is required".to_string()))?;

    let mut db = ctx.db()?;

    // Get module
    let mut service = ModuleService::new(&mut db);
    let module = service
        .get(module_id)
        .map_err(|e| McpError::Internal(e.to_string()))?
        .ok_or_else(|| McpError::InvalidArguments(format!("Module '{}' not found", module_id)))?;

    // Get documents for this module
    let documents = dal::list_module_documents(&mut db, module_id)
        .map_err(|e| McpError::Internal(e.to_string()))?;

    let doc_data: Vec<Value> = documents
        .iter()
        .map(|d| {
            json!({
                "id": d.id,
                "title": d.title,
                "doc_type": d.doc_type
            })
        })
        .collect();

    // Get monsters for this module
    let monsters = dal::list_module_monsters(&mut db, module_id)
        .map_err(|e| McpError::Internal(e.to_string()))?;

    let monster_data: Vec<Value> = monsters
        .iter()
        .map(|m| {
            json!({
                "id": m.id,
                "monster_name": m.monster_name,
                "monster_source": m.monster_source,
                "display_name": m.display_name,
                "quantity": m.quantity,
                "notes": m.notes
            })
        })
        .collect();

    Ok(json!({
        "module": {
            "id": module.id,
            "name": module.name,
            "description": module.description,
            "module_number": module.module_number
        },
        "documents": doc_data,
        "monsters": monster_data
    }))
}

pub async fn add_monster_to_module(ctx: &Arc<McpContext>, args: Value) -> Result<Value, McpError> {
    let module_id = args
        .get("module_id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| McpError::InvalidArguments("module_id is required".to_string()))?;

    let monster_name = args
        .get("monster_name")
        .and_then(|v| v.as_str())
        .ok_or_else(|| McpError::InvalidArguments("monster_name is required".to_string()))?;

    let count = args
        .get("count")
        .and_then(|v| v.as_i64())
        .unwrap_or(1) as i32;

    let notes = args.get("notes").and_then(|v| v.as_str());

    // Default to MM (Monster Manual) as source if not specified
    let monster_source = args
        .get("monster_source")
        .and_then(|v| v.as_str())
        .unwrap_or("MM");

    let mut db = ctx.db()?;

    // Verify module exists
    let mut service = ModuleService::new(&mut db);
    if service.get(module_id).map_err(|e| McpError::Internal(e.to_string()))?.is_none() {
        return Err(McpError::InvalidArguments(format!(
            "Module '{}' not found",
            module_id
        )));
    }

    // Create module monster
    let id = Uuid::new_v4().to_string();
    let mut new_monster = NewModuleMonster::new(&id, module_id, monster_name, monster_source)
        .with_quantity(count);

    if let Some(n) = notes {
        new_monster = new_monster.with_notes(n);
    }

    dal::insert_module_monster(&mut db, &new_monster)
        .map_err(|e| McpError::Internal(e.to_string()))?;

    Ok(json!({
        "status": "added",
        "module_monster": {
            "id": id,
            "monster_name": monster_name,
            "monster_source": monster_source,
            "quantity": count,
            "notes": notes
        }
    }))
}

pub async fn add_item_to_module(_ctx: &Arc<McpContext>, args: Value) -> Result<Value, McpError> {
    let _module_id = args
        .get("module_id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| McpError::InvalidArguments("module_id is required".to_string()))?;

    let _item_name = args
        .get("item_name")
        .and_then(|v| v.as_str())
        .ok_or_else(|| McpError::InvalidArguments("item_name is required".to_string()))?;

    // Module items table doesn't exist yet - this feature is not implemented
    Err(McpError::Internal(
        "Module item tracking is not yet implemented. Items can be added to character inventories instead.".to_string()
    ))
}

pub async fn update_module(ctx: &Arc<McpContext>, args: Value) -> Result<Value, McpError> {
    let module_id = args
        .get("module_id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| McpError::InvalidArguments("module_id is required".to_string()))?;

    let mut input = UpdateModuleInput::default();

    if let Some(name) = args.get("name").and_then(|v| v.as_str()) {
        input.name = Some(name.to_string());
    }
    if let Some(desc) = args.get("description").and_then(|v| v.as_str()) {
        input.description = Some(Some(desc.to_string()));
    }

    let mut db = ctx.db()?;
    let mut service = ModuleService::new(&mut db);

    let module = service
        .update(module_id, input)
        .map_err(|e| McpError::Internal(e.to_string()))?;

    Ok(json!({
        "status": "updated",
        "module": {
            "id": module.id,
            "name": module.name,
            "description": module.description,
            "module_number": module.module_number
        }
    }))
}

pub async fn delete_module(ctx: &Arc<McpContext>, args: Value) -> Result<Value, McpError> {
    let module_id = args
        .get("module_id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| McpError::InvalidArguments("module_id is required".to_string()))?;

    let mut db = ctx.db()?;
    let mut service = ModuleService::new(&mut db);

    service
        .delete(module_id)
        .map_err(|e| McpError::Internal(e.to_string()))?;

    Ok(json!({
        "status": "deleted",
        "module_id": module_id
    }))
}
