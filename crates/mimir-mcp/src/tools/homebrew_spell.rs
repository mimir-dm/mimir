//! Homebrew Spell Tools
//!
//! MCP tools for campaign homebrew spell CRUD operations.

use mimir_core::dal::campaign as dal;
use mimir_core::models::campaign::{NewCampaignHomebrewSpell, UpdateCampaignHomebrewSpell};
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

pub fn list_homebrew_spells_tool() -> Tool {
    Tool {
        name: "list_homebrew_spells".to_string(),
        description: Some("List all homebrew spells in the active campaign".to_string()),
        input_schema: ToolInputSchema::new(vec![], None, None),
        title: None,
        annotations: None,
        icons: vec![],
        execution: None,
        output_schema: None,
        meta: None,
    }
}

pub fn get_homebrew_spell_tool() -> Tool {
    Tool {
        name: "get_homebrew_spell".to_string(),
        description: Some("Get a homebrew spell by ID".to_string()),
        input_schema: ToolInputSchema::new(
            vec!["id".to_string()],
            create_properties(vec![("id", "string", "The homebrew spell ID")]),
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

pub fn create_homebrew_spell_tool() -> Tool {
    Tool {
        name: "create_homebrew_spell".to_string(),
        description: Some(
            "Create a new homebrew spell in the active campaign. Data should be a JSON string with the spell's details."
                .to_string(),
        ),
        input_schema: ToolInputSchema::new(
            vec!["name".to_string(), "data".to_string()],
            create_properties(vec![
                ("name", "string", "Name of the homebrew spell"),
                ("data", "string", "JSON string with spell data"),
                ("level", "integer", "Spell level (0 for cantrip, 1-9)"),
                ("school", "string", "School of magic (e.g. 'evocation', 'necromancy')"),
                ("cloned_from_name", "string", "Name of the catalog spell this was cloned from"),
                ("cloned_from_source", "string", "Source book of the catalog spell this was cloned from"),
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

pub fn update_homebrew_spell_tool() -> Tool {
    Tool {
        name: "update_homebrew_spell".to_string(),
        description: Some("Update a homebrew spell".to_string()),
        input_schema: ToolInputSchema::new(
            vec!["id".to_string()],
            create_properties(vec![
                ("id", "string", "The homebrew spell ID"),
                ("name", "string", "New name"),
                ("level", "integer", "New spell level"),
                ("school", "string", "New school of magic"),
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

pub fn delete_homebrew_spell_tool() -> Tool {
    Tool {
        name: "delete_homebrew_spell".to_string(),
        description: Some("Delete a homebrew spell by ID".to_string()),
        input_schema: ToolInputSchema::new(
            vec!["id".to_string()],
            create_properties(vec![("id", "string", "The homebrew spell ID to delete")]),
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

fn homebrew_spell_to_json(spell: &mimir_core::models::campaign::CampaignHomebrewSpell) -> Value {
    json!({
        "id": spell.id,
        "campaign_id": spell.campaign_id,
        "name": spell.name,
        "level": spell.level,
        "school": spell.school,
        "data": spell.data,
        "cloned_from_name": spell.cloned_from_name,
        "cloned_from_source": spell.cloned_from_source,
        "created_at": spell.created_at,
        "updated_at": spell.updated_at,
    })
}

pub async fn list_homebrew_spells(ctx: &Arc<McpContext>, _args: Value) -> Result<Value, McpError> {
    let campaign_id = ctx
        .get_active_campaign_id()
        .ok_or(McpError::NoActiveCampaign)?;

    let mut db = ctx.db()?;
    let spells = dal::list_campaign_homebrew_spells(&mut db, &campaign_id)
        .map_err(|e| McpError::Internal(e.to_string()))?;

    let spell_data: Vec<Value> = spells.iter().map(homebrew_spell_to_json).collect();

    Ok(json!({
        "spells": spell_data,
        "count": spells.len()
    }))
}

pub async fn get_homebrew_spell(ctx: &Arc<McpContext>, args: Value) -> Result<Value, McpError> {
    let id = args
        .get("id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| McpError::InvalidArguments("id is required".to_string()))?;

    let mut db = ctx.db()?;
    let spell = dal::get_campaign_homebrew_spell(&mut db, id)
        .map_err(|e| McpError::Internal(format!("Homebrew spell '{}' not found: {}", id, e)))?;

    Ok(json!({ "spell": homebrew_spell_to_json(&spell) }))
}

pub async fn create_homebrew_spell(
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
    let mut new_spell = NewCampaignHomebrewSpell::new(&id, &campaign_id, name, data);

    if let Some(level) = args.get("level").and_then(|v| v.as_i64()) {
        new_spell = new_spell.with_level(level as i32);
    }
    if let Some(school) = args.get("school").and_then(|v| v.as_str()) {
        new_spell = new_spell.with_school(school);
    }

    let cloned_from_name = args.get("cloned_from_name").and_then(|v| v.as_str());
    let cloned_from_source = args.get("cloned_from_source").and_then(|v| v.as_str());
    if let (Some(cfn), Some(cfs)) = (cloned_from_name, cloned_from_source) {
        new_spell = new_spell.cloned_from(cfn, cfs);
    }

    let mut db = ctx.db()?;
    dal::insert_campaign_homebrew_spell(&mut db, &new_spell)
        .map_err(|e| McpError::Internal(e.to_string()))?;

    let spell = dal::get_campaign_homebrew_spell(&mut db, &id)
        .map_err(|e| McpError::Internal(e.to_string()))?;

    Ok(json!({
        "status": "created",
        "spell": homebrew_spell_to_json(&spell)
    }))
}

pub async fn update_homebrew_spell(
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
    let level = args.get("level").and_then(|v| v.as_i64()).map(|l| l as i32);
    let school = args.get("school").and_then(|v| v.as_str());
    let data = args.get("data").and_then(|v| v.as_str());

    let now = chrono::Utc::now().to_rfc3339();
    let update = UpdateCampaignHomebrewSpell {
        name,
        level: if args.get("level").is_some() { Some(level) } else { None },
        school: if args.get("school").is_some() { Some(school) } else { None },
        data,
        updated_at: Some(&now),
    };

    let mut db = ctx.db()?;
    dal::update_campaign_homebrew_spell(&mut db, id, &update)
        .map_err(|e| McpError::Internal(format!("Failed to update homebrew spell: {}", e)))?;

    let spell = dal::get_campaign_homebrew_spell(&mut db, id)
        .map_err(|e| McpError::Internal(e.to_string()))?;

    Ok(json!({
        "status": "updated",
        "spell": homebrew_spell_to_json(&spell)
    }))
}

pub async fn delete_homebrew_spell(ctx: &Arc<McpContext>, args: Value) -> Result<Value, McpError> {
    let id = args
        .get("id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| McpError::InvalidArguments("id is required".to_string()))?;

    let mut db = ctx.db()?;
    dal::delete_campaign_homebrew_spell(&mut db, id)
        .map_err(|e| McpError::Internal(format!("Failed to delete homebrew spell: {}", e)))?;

    Ok(json!({
        "status": "deleted",
        "id": id
    }))
}
