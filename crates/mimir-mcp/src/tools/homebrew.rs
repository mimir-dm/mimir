//! Homebrew Item Tools
//!
//! MCP tools for campaign homebrew item CRUD operations.

use mimir_core::services::{CreateHomebrewItemInput, HomebrewService, UpdateHomebrewItemInput};
use rust_mcp_sdk::schema::{Tool, ToolInputSchema};
use serde_json::{json, Value};
use std::sync::Arc;

use super::create_properties;
use crate::context::McpContext;
use crate::McpError;

// =============================================================================
// Tool Definitions
// =============================================================================

pub fn list_homebrew_items_tool() -> Tool {
    Tool {
        name: "list_homebrew_items".to_string(),
        description: Some("List all homebrew items in the active campaign".to_string()),
        input_schema: ToolInputSchema::new(vec![], None, None),
        title: None,
        annotations: None,
        icons: vec![],
        execution: None,
        output_schema: None,
        meta: None,
    }
}

pub fn get_homebrew_item_tool() -> Tool {
    Tool {
        name: "get_homebrew_item".to_string(),
        description: Some("Get a homebrew item by ID".to_string()),
        input_schema: ToolInputSchema::new(
            vec!["id".to_string()],
            create_properties(vec![("id", "string", "The homebrew item ID")]),
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

pub fn create_homebrew_item_tool() -> Tool {
    Tool {
        name: "create_homebrew_item".to_string(),
        description: Some(
            "Create a new homebrew item in the active campaign. Data should be a JSON string with the item's properties."
                .to_string(),
        ),
        input_schema: ToolInputSchema::new(
            vec!["name".to_string(), "data".to_string()],
            create_properties(vec![
                ("name", "string", "Name of the homebrew item"),
                ("data", "string", "JSON string with item data (description, properties, etc.)"),
                ("item_type", "string", "Item type: weapon, armor, potion, ring, rod, scroll, staff, wand, wondrous item, adventuring gear"),
                ("rarity", "string", "Rarity: common, uncommon, rare, very rare, legendary, artifact"),
                ("cloned_from_name", "string", "Name of the catalog item this was cloned from"),
                ("cloned_from_source", "string", "Source book of the catalog item this was cloned from"),
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

pub fn update_homebrew_item_tool() -> Tool {
    Tool {
        name: "update_homebrew_item".to_string(),
        description: Some("Update a homebrew item".to_string()),
        input_schema: ToolInputSchema::new(
            vec!["id".to_string()],
            create_properties(vec![
                ("id", "string", "The homebrew item ID"),
                ("name", "string", "New name"),
                ("item_type", "string", "New item type"),
                ("rarity", "string", "New rarity"),
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

pub fn delete_homebrew_item_tool() -> Tool {
    Tool {
        name: "delete_homebrew_item".to_string(),
        description: Some("Delete a homebrew item by ID".to_string()),
        input_schema: ToolInputSchema::new(
            vec!["id".to_string()],
            create_properties(vec![("id", "string", "The homebrew item ID to delete")]),
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

fn homebrew_to_json(item: &mimir_core::models::campaign::CampaignHomebrewItem) -> Value {
    json!({
        "id": item.id,
        "campaign_id": item.campaign_id,
        "name": item.name,
        "item_type": item.item_type,
        "rarity": item.rarity,
        "data": item.data,
        "cloned_from_name": item.cloned_from_name,
        "cloned_from_source": item.cloned_from_source,
        "created_at": item.created_at,
        "updated_at": item.updated_at,
    })
}

pub async fn list_homebrew_items(ctx: &Arc<McpContext>, _args: Value) -> Result<Value, McpError> {
    let campaign_id = ctx
        .get_active_campaign_id()
        .ok_or(McpError::NoActiveCampaign)?;

    let mut db = ctx.connect()?;
    let items = HomebrewService::new(&mut db).list_items(&campaign_id)?;

    let item_data: Vec<Value> = items.iter().map(homebrew_to_json).collect();

    Ok(json!({
        "items": item_data,
        "count": items.len()
    }))
}

pub async fn get_homebrew_item(ctx: &Arc<McpContext>, args: Value) -> Result<Value, McpError> {
    let id = args
        .get("id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| McpError::InvalidArguments("id is required".to_string()))?;

    let mut db = ctx.connect()?;
    let item = HomebrewService::new(&mut db).get_item(id)?;

    Ok(json!({ "item": homebrew_to_json(&item) }))
}

pub async fn create_homebrew_item(
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

    let input = CreateHomebrewItemInput {
        campaign_id,
        name: name.to_string(),
        data: data.to_string(),
        item_type: args.get("item_type").and_then(|v| v.as_str()).map(String::from),
        rarity: args.get("rarity").and_then(|v| v.as_str()).map(String::from),
        cloned_from_name: args.get("cloned_from_name").and_then(|v| v.as_str()).map(String::from),
        cloned_from_source: args.get("cloned_from_source").and_then(|v| v.as_str()).map(String::from),
    };

    let mut db = ctx.connect()?;
    let item = HomebrewService::new(&mut db).create_item(input)?;

    Ok(json!({
        "status": "created",
        "item": homebrew_to_json(&item)
    }))
}

pub async fn update_homebrew_item(
    ctx: &Arc<McpContext>,
    args: Value,
) -> Result<Value, McpError> {
    let id = args
        .get("id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| McpError::InvalidArguments("id is required".to_string()))?;

    let input = UpdateHomebrewItemInput {
        name: args.get("name").and_then(|v| v.as_str()).map(String::from),
        data: args.get("data").and_then(|v| v.as_str()).map(String::from),
        item_type: if args.get("item_type").is_some() {
            Some(args.get("item_type").and_then(|v| v.as_str()).map(String::from))
        } else {
            None
        },
        rarity: if args.get("rarity").is_some() {
            Some(args.get("rarity").and_then(|v| v.as_str()).map(String::from))
        } else {
            None
        },
    };

    let mut db = ctx.connect()?;
    let item = HomebrewService::new(&mut db).update_item(id, input)?;

    Ok(json!({
        "status": "updated",
        "item": homebrew_to_json(&item)
    }))
}

pub async fn delete_homebrew_item(ctx: &Arc<McpContext>, args: Value) -> Result<Value, McpError> {
    let id = args
        .get("id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| McpError::InvalidArguments("id is required".to_string()))?;

    let mut db = ctx.connect()?;
    HomebrewService::new(&mut db).delete_item(id)?;

    Ok(json!({
        "status": "deleted",
        "id": id
    }))
}
