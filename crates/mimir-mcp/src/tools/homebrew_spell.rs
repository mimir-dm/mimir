//! Homebrew Spell Tools
//!
//! MCP tools for campaign homebrew spell CRUD operations.

use mimir_core::services::{CreateHomebrewSpellInput, HomebrewService, UpdateHomebrewSpellInput};
use rust_mcp_sdk::schema::{Tool, ToolInputSchema};
use serde_json::{json, Value};
use std::sync::Arc;

use super::create_properties;
use crate::context::McpContext;
use crate::response::McpResponse;
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
            "Create a new homebrew spell in the active campaign. To clone from a catalog spell, provide cloned_from_name and cloned_from_source — the catalog spell's full data will be used as the base, and any fields in data will override specific properties. When cloning, data is optional. IMPORTANT: Use search_spells first to find the exact name and source of the catalog spell before cloning."
                .to_string(),
        ),
        input_schema: ToolInputSchema::new(
            vec!["name".to_string()],
            create_properties(vec![
                ("name", "string", "Name of the homebrew spell"),
                ("data", "string", "JSON string with spell data. Required when not cloning. When cloning, fields here override the catalog data."),
                ("level", "integer", "Spell level (0 for cantrip, 1-9)"),
                ("school", "string", "School of magic (e.g. 'evocation', 'necromancy')"),
                ("cloned_from_name", "string", "Name of the catalog spell to clone from. Must be used with cloned_from_source."),
                ("cloned_from_source", "string", "Source book of the catalog spell to clone from (e.g. PHB, XGE). Must be used with cloned_from_name."),
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

    let mut db = ctx.connect()?;
    let spells = HomebrewService::new(&mut db).list_spells(&campaign_id)?;

    let spell_data: Vec<Value> = spells.iter().map(homebrew_spell_to_json).collect();

    McpResponse::list("spells", spell_data)
}

pub async fn get_homebrew_spell(ctx: &Arc<McpContext>, args: Value) -> Result<Value, McpError> {
    let id = args
        .get("id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| McpError::InvalidArguments("id is required".to_string()))?;

    let mut db = ctx.connect()?;
    let spell = HomebrewService::new(&mut db).get_spell(id)?;

    McpResponse::get("spell", homebrew_spell_to_json(&spell))
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

    let data = args.get("data").and_then(|v| v.as_str()).map(String::from);
    let cloned_from_name = args.get("cloned_from_name").and_then(|v| v.as_str()).map(String::from);
    let cloned_from_source = args.get("cloned_from_source").and_then(|v| v.as_str()).map(String::from);

    // data is required when not cloning
    if data.is_none() && (cloned_from_name.is_none() || cloned_from_source.is_none()) {
        return Err(McpError::InvalidArguments(
            "data is required when not cloning from catalog (provide cloned_from_name and cloned_from_source to clone)".to_string(),
        ));
    }

    let input = CreateHomebrewSpellInput {
        campaign_id,
        name: name.to_string(),
        data,
        level: args.get("level").and_then(|v| v.as_i64()).map(|l| l as i32),
        school: args.get("school").and_then(|v| v.as_str()).map(String::from),
        cloned_from_name,
        cloned_from_source,
    };

    let mut db = ctx.connect()?;
    let spell = HomebrewService::new(&mut db).create_spell(input)?;

    McpResponse::created("spell", homebrew_spell_to_json(&spell))
}

pub async fn update_homebrew_spell(
    ctx: &Arc<McpContext>,
    args: Value,
) -> Result<Value, McpError> {
    let id = args
        .get("id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| McpError::InvalidArguments("id is required".to_string()))?;

    let input = UpdateHomebrewSpellInput {
        name: args.get("name").and_then(|v| v.as_str()).map(String::from),
        data: args.get("data").and_then(|v| v.as_str()).map(String::from),
        level: if args.get("level").is_some() {
            Some(args.get("level").and_then(|v| v.as_i64()).map(|l| l as i32))
        } else {
            None
        },
        school: if args.get("school").is_some() {
            Some(args.get("school").and_then(|v| v.as_str()).map(String::from))
        } else {
            None
        },
    };

    let mut db = ctx.connect()?;
    let spell = HomebrewService::new(&mut db).update_spell(id, input)?;

    McpResponse::updated("spell", homebrew_spell_to_json(&spell))
}

pub async fn delete_homebrew_spell(ctx: &Arc<McpContext>, args: Value) -> Result<Value, McpError> {
    let id = args
        .get("id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| McpError::InvalidArguments("id is required".to_string()))?;

    let mut db = ctx.connect()?;
    HomebrewService::new(&mut db).delete_spell(id)?;

    McpResponse::deleted(id)
}
