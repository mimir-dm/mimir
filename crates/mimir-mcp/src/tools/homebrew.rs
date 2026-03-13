//! Homebrew Tools
//!
//! MCP tools for campaign homebrew CRUD operations (items, monsters, spells).
//! Uses a `content_type` parameter to dispatch to the appropriate service.

use mimir_core::services::{
    CreateHomebrewItemInput, CreateHomebrewMonsterInput, CreateHomebrewSpellInput, HomebrewService,
    UpdateHomebrewItemInput, UpdateHomebrewMonsterInput, UpdateHomebrewSpellInput,
};
use rust_mcp_sdk::schema::{Tool, ToolInputSchema};
use serde_json::{json, Value};
use std::sync::Arc;

use super::create_properties;
use crate::context::McpContext;
use crate::response::McpResponse;
use crate::McpError;

// =============================================================================
// Content type validation
// =============================================================================

const VALID_CONTENT_TYPES: &[&str] = &["item", "monster", "spell"];

fn parse_content_type(args: &Value) -> Result<&str, McpError> {
    let ct = args
        .get("content_type")
        .and_then(|v| v.as_str())
        .ok_or_else(|| {
            McpError::InvalidArguments(
                "content_type is required (item, monster, or spell)".to_string(),
            )
        })?;

    if !VALID_CONTENT_TYPES.contains(&ct) {
        return Err(McpError::InvalidArguments(format!(
            "Invalid content_type '{}'. Must be one of: item, monster, spell",
            ct
        )));
    }

    Ok(ct)
}

// =============================================================================
// Tool Definitions
// =============================================================================

pub fn list_homebrew_tool() -> Tool {
    Tool {
        name: "list_homebrew".to_string(),
        description: Some(
            "List all homebrew content of a given type in the active campaign".to_string(),
        ),
        input_schema: ToolInputSchema::new(
            vec!["content_type".to_string()],
            create_properties(vec![(
                "content_type",
                "string",
                "Type of homebrew content: item, monster, or spell",
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

pub fn get_homebrew_tool() -> Tool {
    Tool {
        name: "get_homebrew".to_string(),
        description: Some("Get a homebrew item, monster, or spell by ID".to_string()),
        input_schema: ToolInputSchema::new(
            vec!["content_type".to_string(), "id".to_string()],
            create_properties(vec![
                (
                    "content_type",
                    "string",
                    "Type of homebrew content: item, monster, or spell",
                ),
                ("id", "string", "The homebrew content ID"),
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

pub fn create_homebrew_tool() -> Tool {
    Tool {
        name: "create_homebrew".to_string(),
        description: Some(
            "Create a new homebrew item, monster, or spell in the active campaign. To clone from a catalog entry, provide cloned_from_name and cloned_from_source — the catalog entry's full data will be used as the base, and any fields in data will override specific properties. When cloning, data is optional. IMPORTANT: Use search_catalog first to find the exact name and source before cloning."
                .to_string(),
        ),
        input_schema: ToolInputSchema::new(
            vec!["content_type".to_string(), "name".to_string()],
            create_properties(vec![
                ("content_type", "string", "Type of homebrew content: item, monster, or spell"),
                ("name", "string", "Name of the homebrew content"),
                ("data", "string", "JSON string with content data. Required when not cloning. When cloning, fields here override the catalog data."),
                // Item-specific
                ("item_type", "string", "Item type (items only): weapon, armor, potion, ring, rod, scroll, staff, wand, wondrous item, adventuring gear"),
                ("rarity", "string", "Rarity (items only): common, uncommon, rare, very rare, legendary, artifact"),
                // Monster-specific
                ("cr", "string", "Challenge rating (monsters only, e.g. '1/4', '1', '5', '20')"),
                ("creature_type", "string", "Creature type (monsters only, e.g. 'humanoid', 'dragon', 'undead')"),
                ("size", "string", "Size (monsters only): T, S, M, L, H, G"),
                // Spell-specific
                ("level", "integer", "Spell level (spells only, 0 for cantrip, 1-9)"),
                ("school", "string", "School of magic (spells only, e.g. 'evocation', 'necromancy')"),
                // Cloning
                ("cloned_from_name", "string", "Name of the catalog entry to clone from. Must be used with cloned_from_source."),
                ("cloned_from_source", "string", "Source book of the catalog entry to clone from (e.g. PHB, DMG, MM). Must be used with cloned_from_name."),
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

pub fn update_homebrew_tool() -> Tool {
    Tool {
        name: "update_homebrew".to_string(),
        description: Some("Update a homebrew item, monster, or spell".to_string()),
        input_schema: ToolInputSchema::new(
            vec!["content_type".to_string(), "id".to_string()],
            create_properties(vec![
                (
                    "content_type",
                    "string",
                    "Type of homebrew content: item, monster, or spell",
                ),
                ("id", "string", "The homebrew content ID"),
                ("name", "string", "New name"),
                ("data", "string", "New JSON data string"),
                // Item-specific
                ("item_type", "string", "New item type (items only)"),
                ("rarity", "string", "New rarity (items only)"),
                // Monster-specific
                ("cr", "string", "New challenge rating (monsters only)"),
                (
                    "creature_type",
                    "string",
                    "New creature type (monsters only)",
                ),
                ("size", "string", "New size (monsters only)"),
                // Spell-specific
                ("level", "integer", "New spell level (spells only)"),
                ("school", "string", "New school of magic (spells only)"),
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

pub fn delete_homebrew_tool() -> Tool {
    Tool {
        name: "delete_homebrew".to_string(),
        description: Some("Delete a homebrew item, monster, or spell by ID".to_string()),
        input_schema: ToolInputSchema::new(
            vec!["content_type".to_string(), "id".to_string()],
            create_properties(vec![
                (
                    "content_type",
                    "string",
                    "Type of homebrew content: item, monster, or spell",
                ),
                ("id", "string", "The homebrew content ID to delete"),
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
// JSON serialization helpers
// =============================================================================

fn item_to_json(item: &mimir_core::models::campaign::CampaignHomebrewItem) -> Value {
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

fn monster_to_json(monster: &mimir_core::models::campaign::CampaignHomebrewMonster) -> Value {
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

fn spell_to_json(spell: &mimir_core::models::campaign::CampaignHomebrewSpell) -> Value {
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

// =============================================================================
// Tool Implementations
// =============================================================================

pub async fn list_homebrew(ctx: &Arc<McpContext>, args: Value) -> Result<Value, McpError> {
    let content_type = parse_content_type(&args)?;
    let campaign_id = ctx
        .get_active_campaign_id()
        .ok_or(McpError::NoActiveCampaign)?;

    let mut db = ctx.connect()?;
    let mut svc = HomebrewService::new(&mut db);

    match content_type {
        "item" => {
            let items = svc.list_items(&campaign_id)?;
            McpResponse::list("items", items.iter().map(item_to_json).collect())
        }
        "monster" => {
            let monsters = svc.list_monsters(&campaign_id)?;
            McpResponse::list("monsters", monsters.iter().map(monster_to_json).collect())
        }
        "spell" => {
            let spells = svc.list_spells(&campaign_id)?;
            McpResponse::list("spells", spells.iter().map(spell_to_json).collect())
        }
        _ => unreachable!(),
    }
}

pub async fn get_homebrew(ctx: &Arc<McpContext>, args: Value) -> Result<Value, McpError> {
    let content_type = parse_content_type(&args)?;
    let id = args
        .get("id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| McpError::InvalidArguments("id is required".to_string()))?;

    let mut db = ctx.connect()?;
    let mut svc = HomebrewService::new(&mut db);

    match content_type {
        "item" => {
            let item = svc.get_item(id)?;
            McpResponse::get("item", item_to_json(&item))
        }
        "monster" => {
            let monster = svc.get_monster(id)?;
            McpResponse::get("monster", monster_to_json(&monster))
        }
        "spell" => {
            let spell = svc.get_spell(id)?;
            McpResponse::get("spell", spell_to_json(&spell))
        }
        _ => unreachable!(),
    }
}

pub async fn create_homebrew(ctx: &Arc<McpContext>, args: Value) -> Result<Value, McpError> {
    let content_type = parse_content_type(&args)?;
    let campaign_id = ctx
        .get_active_campaign_id()
        .ok_or(McpError::NoActiveCampaign)?;

    let name = args
        .get("name")
        .and_then(|v| v.as_str())
        .ok_or_else(|| McpError::InvalidArguments("name is required".to_string()))?;

    let data = args.get("data").and_then(|v| v.as_str()).map(String::from);
    let cloned_from_name = args
        .get("cloned_from_name")
        .and_then(|v| v.as_str())
        .map(String::from);
    let cloned_from_source = args
        .get("cloned_from_source")
        .and_then(|v| v.as_str())
        .map(String::from);

    // data is required when not cloning
    if data.is_none() && (cloned_from_name.is_none() || cloned_from_source.is_none()) {
        return Err(McpError::InvalidArguments(
            "data is required when not cloning from catalog (provide cloned_from_name and cloned_from_source to clone)".to_string(),
        ));
    }

    let mut db = ctx.connect()?;
    let mut svc = HomebrewService::new(&mut db);

    match content_type {
        "item" => {
            let input = CreateHomebrewItemInput {
                campaign_id,
                name: name.to_string(),
                data,
                item_type: args
                    .get("item_type")
                    .and_then(|v| v.as_str())
                    .map(String::from),
                rarity: args
                    .get("rarity")
                    .and_then(|v| v.as_str())
                    .map(String::from),
                cloned_from_name,
                cloned_from_source,
            };
            let item = svc.create_item(input)?;
            McpResponse::created("item", item_to_json(&item))
        }
        "monster" => {
            let input = CreateHomebrewMonsterInput {
                campaign_id,
                name: name.to_string(),
                data,
                cr: args.get("cr").and_then(|v| v.as_str()).map(String::from),
                creature_type: args
                    .get("creature_type")
                    .and_then(|v| v.as_str())
                    .map(String::from),
                size: args.get("size").and_then(|v| v.as_str()).map(String::from),
                cloned_from_name,
                cloned_from_source,
            };
            let monster = svc.create_monster(input)?;
            McpResponse::created("monster", monster_to_json(&monster))
        }
        "spell" => {
            let input = CreateHomebrewSpellInput {
                campaign_id,
                name: name.to_string(),
                data,
                level: args.get("level").and_then(|v| v.as_i64()).map(|l| l as i32),
                school: args
                    .get("school")
                    .and_then(|v| v.as_str())
                    .map(String::from),
                cloned_from_name,
                cloned_from_source,
            };
            let spell = svc.create_spell(input)?;
            McpResponse::created("spell", spell_to_json(&spell))
        }
        _ => unreachable!(),
    }
}

pub async fn update_homebrew(ctx: &Arc<McpContext>, args: Value) -> Result<Value, McpError> {
    let content_type = parse_content_type(&args)?;
    let id = args
        .get("id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| McpError::InvalidArguments("id is required".to_string()))?;

    let mut db = ctx.connect()?;
    let mut svc = HomebrewService::new(&mut db);

    match content_type {
        "item" => {
            let input = UpdateHomebrewItemInput {
                name: args.get("name").and_then(|v| v.as_str()).map(String::from),
                data: args.get("data").and_then(|v| v.as_str()).map(String::from),
                item_type: if args.get("item_type").is_some() {
                    Some(
                        args.get("item_type")
                            .and_then(|v| v.as_str())
                            .map(String::from),
                    )
                } else {
                    None
                },
                rarity: if args.get("rarity").is_some() {
                    Some(
                        args.get("rarity")
                            .and_then(|v| v.as_str())
                            .map(String::from),
                    )
                } else {
                    None
                },
            };
            let item = svc.update_item(id, input)?;
            McpResponse::updated("item", item_to_json(&item))
        }
        "monster" => {
            let input = UpdateHomebrewMonsterInput {
                name: args.get("name").and_then(|v| v.as_str()).map(String::from),
                data: args.get("data").and_then(|v| v.as_str()).map(String::from),
                cr: if args.get("cr").is_some() {
                    Some(args.get("cr").and_then(|v| v.as_str()).map(String::from))
                } else {
                    None
                },
                creature_type: if args.get("creature_type").is_some() {
                    Some(
                        args.get("creature_type")
                            .and_then(|v| v.as_str())
                            .map(String::from),
                    )
                } else {
                    None
                },
                size: if args.get("size").is_some() {
                    Some(args.get("size").and_then(|v| v.as_str()).map(String::from))
                } else {
                    None
                },
            };
            let monster = svc.update_monster(id, input)?;
            McpResponse::updated("monster", monster_to_json(&monster))
        }
        "spell" => {
            let input = UpdateHomebrewSpellInput {
                name: args.get("name").and_then(|v| v.as_str()).map(String::from),
                data: args.get("data").and_then(|v| v.as_str()).map(String::from),
                level: if args.get("level").is_some() {
                    Some(args.get("level").and_then(|v| v.as_i64()).map(|l| l as i32))
                } else {
                    None
                },
                school: if args.get("school").is_some() {
                    Some(
                        args.get("school")
                            .and_then(|v| v.as_str())
                            .map(String::from),
                    )
                } else {
                    None
                },
            };
            let spell = svc.update_spell(id, input)?;
            McpResponse::updated("spell", spell_to_json(&spell))
        }
        _ => unreachable!(),
    }
}

pub async fn delete_homebrew(ctx: &Arc<McpContext>, args: Value) -> Result<Value, McpError> {
    let content_type = parse_content_type(&args)?;
    let id = args
        .get("id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| McpError::InvalidArguments("id is required".to_string()))?;

    let mut db = ctx.connect()?;
    let mut svc = HomebrewService::new(&mut db);

    match content_type {
        "item" => svc.delete_item(id)?,
        "monster" => svc.delete_monster(id)?,
        "spell" => svc.delete_spell(id)?,
        _ => unreachable!(),
    }

    McpResponse::deleted(id)
}
