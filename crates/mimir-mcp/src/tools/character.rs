//! Character Tools
//!
//! MCP tools for character (NPC and PC) management.

use mimir_core::dal::campaign as dal;
use mimir_core::services::{
    AddInventoryInput, CharacterService, CreateCharacterInput, UpdateCharacterInput,
};
use rust_mcp_sdk::schema::{Tool, ToolInputSchema};
use serde_json::{json, Value};
use std::sync::Arc;

use super::create_properties;
use crate::context::McpContext;
use crate::McpError;

// =============================================================================
// Tool Definitions
// =============================================================================

pub fn list_characters_tool() -> Tool {
    Tool {
        name: "list_characters".to_string(),
        description: Some("List characters in the active campaign".to_string()),
        input_schema: ToolInputSchema::new(
            vec![],
            create_properties(vec![
                ("character_type", "string", "Filter by type: pc or npc"),
                ("module_id", "string", "Filter by module assignment (NPCs only)"),
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

pub fn get_character_tool() -> Tool {
    Tool {
        name: "get_character".to_string(),
        description: Some(
            "Get detailed information about a character including classes and inventory"
                .to_string(),
        ),
        input_schema: ToolInputSchema::new(
            vec!["character_id".to_string()],
            create_properties(vec![("character_id", "string", "The ID of the character")]),
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

pub fn create_character_tool() -> Tool {
    Tool {
        name: "create_character".to_string(),
        description: Some("Create a new character (NPC or PC)".to_string()),
        input_schema: ToolInputSchema::new(
            vec!["name".to_string(), "character_type".to_string()],
            create_properties(vec![
                ("name", "string", "Name of the character"),
                ("character_type", "string", "Type: pc or npc"),
                ("race_name", "string", "Race of the character"),
                ("class_name", "string", "Starting class"),
                ("level", "integer", "Starting level (default: 1)"),
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

pub fn edit_character_tool() -> Tool {
    Tool {
        name: "edit_character".to_string(),
        description: Some("Update character fields".to_string()),
        input_schema: ToolInputSchema::new(
            vec!["character_id".to_string()],
            create_properties(vec![
                ("character_id", "string", "The ID of the character"),
                ("name", "string", "New name"),
                ("module_id", "string", "Assign to module (NPCs only)"),
                ("npc_role", "string", "NPC's role in the module"),
                ("npc_location", "string", "NPC's location"),
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

pub fn add_item_to_character_tool() -> Tool {
    Tool {
        name: "add_item_to_character".to_string(),
        description: Some("Add an item from the catalog to a character's inventory".to_string()),
        input_schema: ToolInputSchema::new(
            vec!["character_id".to_string(), "item_name".to_string()],
            create_properties(vec![
                ("character_id", "string", "The ID of the character"),
                ("item_name", "string", "Name of the item from the catalog"),
                ("quantity", "integer", "Quantity of the item (default: 1)"),
                ("equipped", "boolean", "Whether the item is equipped (default: false)"),
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

pub async fn list_characters(ctx: &Arc<McpContext>, args: Value) -> Result<Value, McpError> {
    let campaign_id = ctx
        .get_active_campaign_id()
        .ok_or(McpError::NoActiveCampaign)?;

    let character_type = args.get("character_type").and_then(|v| v.as_str());

    let mut db = ctx.db()?;
    let mut service = CharacterService::new(&mut db);

    let characters = match character_type {
        Some("pc") => service
            .list_pcs(&campaign_id)
            .map_err(|e| McpError::Internal(e.to_string()))?,
        Some("npc") => service
            .list_npcs(&campaign_id)
            .map_err(|e| McpError::Internal(e.to_string()))?,
        _ => service
            .list_for_campaign(&campaign_id)
            .map_err(|e| McpError::Internal(e.to_string()))?,
    };

    let char_data: Vec<Value> = characters
        .iter()
        .map(|c| {
            json!({
                "id": c.id,
                "name": c.name,
                "is_npc": c.is_npc(),
                "race_name": c.race_name,
                "role": c.role,
                "location": c.location
            })
        })
        .collect();

    Ok(json!({
        "characters": char_data
    }))
}

pub async fn get_character(ctx: &Arc<McpContext>, args: Value) -> Result<Value, McpError> {
    let character_id = args
        .get("character_id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| McpError::InvalidArguments("character_id is required".to_string()))?;

    let mut db = ctx.db()?;

    // Get character and inventory using service
    let (character, inventory) = {
        let mut service = CharacterService::new(&mut db);

        let character = service
            .get(character_id)
            .map_err(|e| McpError::Internal(e.to_string()))?
            .ok_or_else(|| {
                McpError::InvalidArguments(format!("Character '{}' not found", character_id))
            })?;

        let inventory = service
            .get_inventory(character_id)
            .map_err(|e| McpError::Internal(e.to_string()))?;

        (character, inventory)
    };

    // Get classes using DAL directly (service dropped so we can use db again)
    let classes = dal::list_character_classes(&mut db, character_id)
        .map_err(|e| McpError::Internal(e.to_string()))?;

    let class_data: Vec<Value> = classes
        .iter()
        .map(|c| {
            json!({
                "class_name": c.class_name,
                "class_source": c.class_source,
                "level": c.level,
                "subclass_name": c.subclass_name
            })
        })
        .collect();

    let inv_data: Vec<Value> = inventory
        .iter()
        .map(|i| {
            json!({
                "id": i.id,
                "item_name": i.item_name,
                "item_source": i.item_source,
                "quantity": i.quantity,
                "equipped": i.equipped != 0,
                "attuned": i.attuned != 0
            })
        })
        .collect();

    Ok(json!({
        "character": {
            "id": character.id,
            "name": character.name,
            "is_npc": character.is_npc(),
            "race_name": character.race_name,
            "background_name": character.background_name,
            "strength": character.strength,
            "dexterity": character.dexterity,
            "constitution": character.constitution,
            "intelligence": character.intelligence,
            "wisdom": character.wisdom,
            "charisma": character.charisma,
            "cp": character.cp,
            "sp": character.sp,
            "ep": character.ep,
            "gp": character.gp,
            "pp": character.pp,
            "traits": character.traits,
            "ideals": character.ideals,
            "bonds": character.bonds,
            "flaws": character.flaws,
            "role": character.role,
            "location": character.location,
            "faction": character.faction
        },
        "classes": class_data,
        "inventory": inv_data
    }))
}

pub async fn create_character(ctx: &Arc<McpContext>, args: Value) -> Result<Value, McpError> {
    let campaign_id = ctx
        .get_active_campaign_id()
        .ok_or(McpError::NoActiveCampaign)?;

    let name = args
        .get("name")
        .and_then(|v| v.as_str())
        .ok_or_else(|| McpError::InvalidArguments("name is required".to_string()))?;

    let character_type = args
        .get("character_type")
        .and_then(|v| v.as_str())
        .ok_or_else(|| McpError::InvalidArguments("character_type is required".to_string()))?;

    let race_name = args.get("race_name").and_then(|v| v.as_str());

    let mut db = ctx.db()?;
    let mut service = CharacterService::new(&mut db);

    // Create character based on type
    let mut input = if character_type == "npc" {
        CreateCharacterInput::new_npc(Some(&campaign_id), name)
    } else {
        let player_name = args
            .get("player_name")
            .and_then(|v| v.as_str())
            .unwrap_or("Player");
        CreateCharacterInput::new_pc(Some(&campaign_id), name, player_name)
    };

    // Set race if provided
    if let Some(race) = race_name {
        let race_source = args
            .get("race_source")
            .and_then(|v| v.as_str())
            .unwrap_or("PHB");
        input = input.with_race(race, race_source);
    }

    let character = service
        .create(input)
        .map_err(|e| McpError::Internal(e.to_string()))?;

    Ok(json!({
        "status": "created",
        "character": {
            "id": character.id,
            "name": character.name,
            "is_npc": character.is_npc(),
            "race_name": character.race_name
        }
    }))
}

pub async fn edit_character(ctx: &Arc<McpContext>, args: Value) -> Result<Value, McpError> {
    let character_id = args
        .get("character_id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| McpError::InvalidArguments("character_id is required".to_string()))?;

    let mut db = ctx.db()?;
    let mut service = CharacterService::new(&mut db);

    // Build update from provided fields
    let mut update = UpdateCharacterInput::default();

    if let Some(name) = args.get("name").and_then(|v| v.as_str()) {
        update.name = Some(name.to_string());
    }

    // NPC-specific fields
    if let Some(role) = args.get("npc_role").and_then(|v| v.as_str()) {
        update.role = Some(Some(role.to_string()));
    }
    if let Some(location) = args.get("npc_location").and_then(|v| v.as_str()) {
        update.location = Some(Some(location.to_string()));
    }
    if let Some(faction) = args.get("faction").and_then(|v| v.as_str()) {
        update.faction = Some(Some(faction.to_string()));
    }

    // Roleplay fields
    if let Some(traits) = args.get("traits").and_then(|v| v.as_str()) {
        update.traits = Some(Some(traits.to_string()));
    }
    if let Some(ideals) = args.get("ideals").and_then(|v| v.as_str()) {
        update.ideals = Some(Some(ideals.to_string()));
    }
    if let Some(bonds) = args.get("bonds").and_then(|v| v.as_str()) {
        update.bonds = Some(Some(bonds.to_string()));
    }
    if let Some(flaws) = args.get("flaws").and_then(|v| v.as_str()) {
        update.flaws = Some(Some(flaws.to_string()));
    }

    let character = service
        .update(character_id, update)
        .map_err(|e| McpError::Internal(e.to_string()))?;

    Ok(json!({
        "status": "updated",
        "character": {
            "id": character.id,
            "name": character.name,
            "is_npc": character.is_npc(),
            "role": character.role,
            "location": character.location,
            "faction": character.faction
        }
    }))
}

pub async fn add_item_to_character(ctx: &Arc<McpContext>, args: Value) -> Result<Value, McpError> {
    let character_id = args
        .get("character_id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| McpError::InvalidArguments("character_id is required".to_string()))?;

    let item_name = args
        .get("item_name")
        .and_then(|v| v.as_str())
        .ok_or_else(|| McpError::InvalidArguments("item_name is required".to_string()))?;

    let item_source = args
        .get("item_source")
        .and_then(|v| v.as_str())
        .unwrap_or("PHB");

    let quantity = args.get("quantity").and_then(|v| v.as_i64()).map(|q| q as i32);

    let equipped = args.get("equipped").and_then(|v| v.as_bool()).unwrap_or(false);

    let mut db = ctx.db()?;
    let mut service = CharacterService::new(&mut db);

    let mut input = AddInventoryInput::new(item_name, item_source);

    if let Some(qty) = quantity {
        input = input.with_quantity(qty);
    }

    if equipped {
        input = input.equipped();
    }

    let inventory_item = service
        .add_to_inventory(character_id, input)
        .map_err(|e| McpError::Internal(e.to_string()))?;

    Ok(json!({
        "status": "added",
        "inventory_item": {
            "id": inventory_item.id,
            "item_name": inventory_item.item_name,
            "item_source": inventory_item.item_source,
            "quantity": inventory_item.quantity,
            "equipped": inventory_item.equipped != 0
        }
    }))
}
