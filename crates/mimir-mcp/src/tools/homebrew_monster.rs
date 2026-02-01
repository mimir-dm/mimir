//! Homebrew Monster Tools
//!
//! MCP tools for campaign homebrew monster CRUD operations.

use mimir_core::dal::campaign as dal;
use mimir_core::models::campaign::{NewCampaignHomebrewMonster, UpdateCampaignHomebrewMonster};
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

pub fn list_homebrew_monsters_tool() -> Tool {
    Tool {
        name: "list_homebrew_monsters".to_string(),
        description: Some("List all homebrew monsters in the active campaign".to_string()),
        input_schema: ToolInputSchema::new(vec![], None, None),
        title: None,
        annotations: None,
        icons: vec![],
        execution: None,
        output_schema: None,
        meta: None,
    }
}

pub fn get_homebrew_monster_tool() -> Tool {
    Tool {
        name: "get_homebrew_monster".to_string(),
        description: Some("Get a homebrew monster by ID".to_string()),
        input_schema: ToolInputSchema::new(
            vec!["id".to_string()],
            create_properties(vec![("id", "string", "The homebrew monster ID")]),
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

pub fn create_homebrew_monster_tool() -> Tool {
    Tool {
        name: "create_homebrew_monster".to_string(),
        description: Some(
            "Create a new homebrew monster in the active campaign. Data should be a JSON string with the monster's stat block."
                .to_string(),
        ),
        input_schema: ToolInputSchema::new(
            vec!["name".to_string(), "data".to_string()],
            create_properties(vec![
                ("name", "string", "Name of the homebrew monster"),
                ("data", "string", "JSON string with monster stat block data"),
                ("cr", "string", "Challenge rating (e.g. '1/4', '1', '5', '20')"),
                ("creature_type", "string", "Creature type (e.g. 'humanoid', 'dragon', 'undead')"),
                ("size", "string", "Size: T, S, M, L, H, G"),
                ("cloned_from_name", "string", "Name of the catalog monster this was cloned from"),
                ("cloned_from_source", "string", "Source book of the catalog monster this was cloned from"),
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

pub fn update_homebrew_monster_tool() -> Tool {
    Tool {
        name: "update_homebrew_monster".to_string(),
        description: Some("Update a homebrew monster".to_string()),
        input_schema: ToolInputSchema::new(
            vec!["id".to_string()],
            create_properties(vec![
                ("id", "string", "The homebrew monster ID"),
                ("name", "string", "New name"),
                ("cr", "string", "New challenge rating"),
                ("creature_type", "string", "New creature type"),
                ("size", "string", "New size"),
                ("data", "string", "New JSON data string"),
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

pub fn delete_homebrew_monster_tool() -> Tool {
    Tool {
        name: "delete_homebrew_monster".to_string(),
        description: Some("Delete a homebrew monster by ID".to_string()),
        input_schema: ToolInputSchema::new(
            vec!["id".to_string()],
            create_properties(vec![("id", "string", "The homebrew monster ID to delete")]),
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

fn homebrew_monster_to_json(monster: &mimir_core::models::campaign::CampaignHomebrewMonster) -> Value {
    json!({
        "id": monster.id,
        "campaign_id": monster.campaign_id,
        "name": monster.name,
        "cr": monster.cr,
        "creature_type": monster.creature_type,
        "size": monster.size,
        "data": monster.data,
        "cloned_from_name": monster.cloned_from_name,
        "cloned_from_source": monster.cloned_from_source,
        "created_at": monster.created_at,
        "updated_at": monster.updated_at,
    })
}

pub async fn list_homebrew_monsters(ctx: &Arc<McpContext>, _args: Value) -> Result<Value, McpError> {
    let campaign_id = ctx
        .get_active_campaign_id()
        .ok_or(McpError::NoActiveCampaign)?;

    let mut db = ctx.db()?;
    let monsters = dal::list_campaign_homebrew_monsters(&mut db, &campaign_id)
        .map_err(|e| McpError::Internal(e.to_string()))?;

    let monster_data: Vec<Value> = monsters.iter().map(homebrew_monster_to_json).collect();

    Ok(json!({
        "monsters": monster_data,
        "count": monsters.len()
    }))
}

pub async fn get_homebrew_monster(ctx: &Arc<McpContext>, args: Value) -> Result<Value, McpError> {
    let id = args
        .get("id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| McpError::InvalidArguments("id is required".to_string()))?;

    let mut db = ctx.db()?;
    let monster = dal::get_campaign_homebrew_monster(&mut db, id)
        .map_err(|e| McpError::Internal(format!("Homebrew monster '{}' not found: {}", id, e)))?;

    Ok(json!({ "monster": homebrew_monster_to_json(&monster) }))
}

pub async fn create_homebrew_monster(
    ctx: &Arc<McpContext>,
    args: Value,
) -> Result<Value, McpError> {
    let campaign_id = ctx
        .get_active_campaign_id()
        .ok_or(McpError::NoActiveCampaign)?;

    let name = args
        .get("name")
        .and_then(|v| v.as_str())
        .ok_or_else(|| McpError::InvalidArguments("name is required".to_string()))?;

    let data = args
        .get("data")
        .and_then(|v| v.as_str())
        .ok_or_else(|| McpError::InvalidArguments("data is required".to_string()))?;

    // Validate JSON
    serde_json::from_str::<Value>(data)
        .map_err(|e| McpError::InvalidArguments(format!("data must be valid JSON: {}", e)))?;

    let id = Uuid::new_v4().to_string();
    let mut new_monster = NewCampaignHomebrewMonster::new(&id, &campaign_id, name, data);

    if let Some(cr) = args.get("cr").and_then(|v| v.as_str()) {
        new_monster = new_monster.with_cr(cr);
    }
    if let Some(creature_type) = args.get("creature_type").and_then(|v| v.as_str()) {
        new_monster = new_monster.with_creature_type(creature_type);
    }
    if let Some(size) = args.get("size").and_then(|v| v.as_str()) {
        new_monster = new_monster.with_size(size);
    }

    let cloned_from_name = args.get("cloned_from_name").and_then(|v| v.as_str());
    let cloned_from_source = args.get("cloned_from_source").and_then(|v| v.as_str());
    if let (Some(cfn), Some(cfs)) = (cloned_from_name, cloned_from_source) {
        new_monster = new_monster.cloned_from(cfn, cfs);
    }

    let mut db = ctx.db()?;
    dal::insert_campaign_homebrew_monster(&mut db, &new_monster)
        .map_err(|e| McpError::Internal(e.to_string()))?;

    let monster = dal::get_campaign_homebrew_monster(&mut db, &id)
        .map_err(|e| McpError::Internal(e.to_string()))?;

    Ok(json!({
        "status": "created",
        "monster": homebrew_monster_to_json(&monster)
    }))
}

pub async fn update_homebrew_monster(
    ctx: &Arc<McpContext>,
    args: Value,
) -> Result<Value, McpError> {
    let id = args
        .get("id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| McpError::InvalidArguments("id is required".to_string()))?;

    // Validate data JSON if provided
    if let Some(data) = args.get("data").and_then(|v| v.as_str()) {
        serde_json::from_str::<Value>(data)
            .map_err(|e| McpError::InvalidArguments(format!("data must be valid JSON: {}", e)))?;
    }

    let name = args.get("name").and_then(|v| v.as_str());
    let cr = args.get("cr").and_then(|v| v.as_str());
    let creature_type = args.get("creature_type").and_then(|v| v.as_str());
    let size = args.get("size").and_then(|v| v.as_str());
    let data = args.get("data").and_then(|v| v.as_str());

    let now = chrono::Utc::now().to_rfc3339();
    let update = UpdateCampaignHomebrewMonster {
        name,
        cr: if args.get("cr").is_some() { Some(cr) } else { None },
        creature_type: if args.get("creature_type").is_some() { Some(creature_type) } else { None },
        size: if args.get("size").is_some() { Some(size) } else { None },
        data,
        updated_at: Some(&now),
    };

    let mut db = ctx.db()?;
    dal::update_campaign_homebrew_monster(&mut db, id, &update)
        .map_err(|e| McpError::Internal(format!("Failed to update homebrew monster: {}", e)))?;

    let monster = dal::get_campaign_homebrew_monster(&mut db, id)
        .map_err(|e| McpError::Internal(e.to_string()))?;

    Ok(json!({
        "status": "updated",
        "monster": homebrew_monster_to_json(&monster)
    }))
}

pub async fn delete_homebrew_monster(ctx: &Arc<McpContext>, args: Value) -> Result<Value, McpError> {
    let id = args
        .get("id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| McpError::InvalidArguments("id is required".to_string()))?;

    let mut db = ctx.db()?;
    dal::delete_campaign_homebrew_monster(&mut db, id)
        .map_err(|e| McpError::Internal(format!("Failed to delete homebrew monster: {}", e)))?;

    Ok(json!({
        "status": "deleted",
        "id": id
    }))
}
