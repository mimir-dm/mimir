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
                ("location", "string", "Filter NPCs by location"),
                ("faction", "string", "Filter NPCs by faction"),
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
        description: Some("Update character fields including ability scores, currency, race, background, and roleplay traits".to_string()),
        input_schema: ToolInputSchema::new(
            vec!["character_id".to_string()],
            create_properties(vec![
                ("character_id", "string", "The ID of the character"),
                ("name", "string", "New name"),
                ("player_name", "string", "Player name (PCs only)"),
                ("race_name", "string", "Race name"),
                ("race_source", "string", "Race source book (default: PHB)"),
                ("background_name", "string", "Background name"),
                ("background_source", "string", "Background source book (default: PHB)"),
                ("strength", "integer", "Strength score"),
                ("dexterity", "integer", "Dexterity score"),
                ("constitution", "integer", "Constitution score"),
                ("intelligence", "integer", "Intelligence score"),
                ("wisdom", "integer", "Wisdom score"),
                ("charisma", "integer", "Charisma score"),
                ("cp", "integer", "Copper pieces"),
                ("sp", "integer", "Silver pieces"),
                ("ep", "integer", "Electrum pieces"),
                ("gp", "integer", "Gold pieces"),
                ("pp", "integer", "Platinum pieces"),
                ("module_id", "string", "Assign to module (NPCs only)"),
                ("npc_role", "string", "NPC's role in the module"),
                ("npc_location", "string", "NPC's location"),
                ("faction", "string", "Faction affiliation"),
                ("traits", "string", "Personality traits"),
                ("ideals", "string", "Ideals"),
                ("bonds", "string", "Bonds"),
                ("flaws", "string", "Flaws"),
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

pub fn delete_character_tool() -> Tool {
    Tool {
        name: "delete_character".to_string(),
        description: Some("Delete a character and all associated data".to_string()),
        input_schema: ToolInputSchema::new(
            vec!["character_id".to_string()],
            create_properties(vec![("character_id", "string", "The ID of the character to delete")]),
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
                ("attuned", "boolean", "Whether the item is attuned (default: false)"),
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

pub fn remove_item_from_character_tool() -> Tool {
    Tool {
        name: "remove_item_from_character".to_string(),
        description: Some("Remove an item from a character's inventory".to_string()),
        input_schema: ToolInputSchema::new(
            vec!["inventory_id".to_string()],
            create_properties(vec![
                ("inventory_id", "string", "The ID of the inventory entry to remove"),
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

pub fn update_character_inventory_tool() -> Tool {
    Tool {
        name: "update_character_inventory".to_string(),
        description: Some("Update an inventory item's quantity, equipped, or attuned state".to_string()),
        input_schema: ToolInputSchema::new(
            vec!["inventory_id".to_string()],
            create_properties(vec![
                ("inventory_id", "string", "The ID of the inventory entry"),
                ("quantity", "integer", "New quantity"),
                ("equipped", "boolean", "Whether equipped"),
                ("attuned", "boolean", "Whether attuned (max 3 attuned items per D&D 5e rules)"),
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

pub fn get_character_inventory_tool() -> Tool {
    Tool {
        name: "get_character_inventory".to_string(),
        description: Some("Get a character's inventory, optionally filtered by equipped or attuned".to_string()),
        input_schema: ToolInputSchema::new(
            vec!["character_id".to_string()],
            create_properties(vec![
                ("character_id", "string", "The ID of the character"),
                ("filter", "string", "Filter: all, equipped, or attuned (default: all)"),
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

pub fn level_up_character_tool() -> Tool {
    Tool {
        name: "level_up_character".to_string(),
        description: Some(
            "Level up a character. Handles HP, multiclass validation, ASI/feats, spells, and feature choices."
                .to_string(),
        ),
        input_schema: ToolInputSchema::new(
            vec!["character_id".to_string(), "class_name".to_string()],
            create_properties(vec![
                ("character_id", "string", "The ID of the character"),
                ("class_name", "string", "Class to level up in (e.g. Fighter, Wizard)"),
                ("class_source", "string", "Class source book (default: PHB)"),
                ("hp_method", "string", "HP gain method: average, roll, or manual"),
                ("hp_value", "integer", "Roll result or manual HP value (required for roll/manual)"),
                ("subclass_name", "string", "Subclass name if choosing this level"),
                ("subclass_source", "string", "Subclass source book (default: PHB)"),
                ("asi_type", "string", "ASI or feat: 'asi' or 'feat'"),
                ("asi_ability1", "string", "First ability to increase (for ASI)"),
                ("asi_increase1", "integer", "Amount for first ability: 1 or 2 (for ASI)"),
                ("asi_ability2", "string", "Second ability to increase (for ASI, optional)"),
                ("asi_increase2", "integer", "Amount for second ability (for ASI)"),
                ("feat_name", "string", "Feat name (if choosing feat)"),
                ("feat_source", "string", "Feat source (default: PHB)"),
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
    let location = args.get("location").and_then(|v| v.as_str());
    let faction = args.get("faction").and_then(|v| v.as_str());

    let mut db = ctx.db()?;
    let mut service = CharacterService::new(&mut db);

    let characters = if let Some(loc) = location {
        service
            .list_npcs_by_location(&campaign_id, loc)
            .map_err(|e| McpError::Internal(e.to_string()))?
    } else if let Some(fac) = faction {
        service
            .list_npcs_by_faction(&campaign_id, fac)
            .map_err(|e| McpError::Internal(e.to_string()))?
    } else {
        match character_type {
            Some("pc") => service
                .list_pcs(&campaign_id)
                .map_err(|e| McpError::Internal(e.to_string()))?,
            Some("npc") => service
                .list_npcs(&campaign_id)
                .map_err(|e| McpError::Internal(e.to_string()))?,
            _ => service
                .list_for_campaign(&campaign_id)
                .map_err(|e| McpError::Internal(e.to_string()))?,
        }
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
                "location": c.location,
                "faction": c.faction
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
            "player_name": character.player_name,
            "race_name": character.race_name,
            "race_source": character.race_source,
            "background_name": character.background_name,
            "background_source": character.background_source,
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
    if let Some(pn) = args.get("player_name").and_then(|v| v.as_str()) {
        update.player_name = Some(Some(pn.to_string()));
    }

    // Race/background
    if let Some(rn) = args.get("race_name").and_then(|v| v.as_str()) {
        update.race_name = Some(Some(rn.to_string()));
        let rs = args.get("race_source").and_then(|v| v.as_str()).unwrap_or("PHB");
        update.race_source = Some(Some(rs.to_string()));
    }
    if let Some(bn) = args.get("background_name").and_then(|v| v.as_str()) {
        update.background_name = Some(Some(bn.to_string()));
        let bs = args.get("background_source").and_then(|v| v.as_str()).unwrap_or("PHB");
        update.background_source = Some(Some(bs.to_string()));
    }

    // Ability scores — set as array if any are provided
    let str_val = args.get("strength").and_then(|v| v.as_i64());
    let dex_val = args.get("dexterity").and_then(|v| v.as_i64());
    let con_val = args.get("constitution").and_then(|v| v.as_i64());
    let int_val = args.get("intelligence").and_then(|v| v.as_i64());
    let wis_val = args.get("wisdom").and_then(|v| v.as_i64());
    let cha_val = args.get("charisma").and_then(|v| v.as_i64());

    if str_val.is_some() || dex_val.is_some() || con_val.is_some()
        || int_val.is_some() || wis_val.is_some() || cha_val.is_some()
    {
        // Need to read current values for any not provided
        let current = service
            .get(character_id)
            .map_err(|e| McpError::Internal(e.to_string()))?
            .ok_or_else(|| McpError::InvalidArguments(format!("Character '{}' not found", character_id)))?;

        update.ability_scores = Some([
            str_val.unwrap_or(current.strength as i64) as i32,
            dex_val.unwrap_or(current.dexterity as i64) as i32,
            con_val.unwrap_or(current.constitution as i64) as i32,
            int_val.unwrap_or(current.intelligence as i64) as i32,
            wis_val.unwrap_or(current.wisdom as i64) as i32,
            cha_val.unwrap_or(current.charisma as i64) as i32,
        ]);
    }

    // Currency — set as array if any are provided
    let cp = args.get("cp").and_then(|v| v.as_i64());
    let sp = args.get("sp").and_then(|v| v.as_i64());
    let ep = args.get("ep").and_then(|v| v.as_i64());
    let gp = args.get("gp").and_then(|v| v.as_i64());
    let pp = args.get("pp").and_then(|v| v.as_i64());

    if cp.is_some() || sp.is_some() || ep.is_some() || gp.is_some() || pp.is_some() {
        let current = if update.ability_scores.is_none() {
            // Only fetch if we didn't already
            Some(service
                .get(character_id)
                .map_err(|e| McpError::Internal(e.to_string()))?
                .ok_or_else(|| McpError::InvalidArguments(format!("Character '{}' not found", character_id)))?)
        } else {
            None
        };
        // If we already fetched for ability scores, we need current for currency defaults
        // Re-fetch is fine since it's just a read
        let cur = if let Some(c) = current {
            c
        } else {
            service
                .get(character_id)
                .map_err(|e| McpError::Internal(e.to_string()))?
                .ok_or_else(|| McpError::InvalidArguments(format!("Character '{}' not found", character_id)))?
        };

        update.currency = Some([
            cp.unwrap_or(cur.cp as i64) as i32,
            sp.unwrap_or(cur.sp as i64) as i32,
            ep.unwrap_or(cur.ep as i64) as i32,
            gp.unwrap_or(cur.gp as i64) as i32,
            pp.unwrap_or(cur.pp as i64) as i32,
        ]);
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
            "role": character.role,
            "location": character.location,
            "faction": character.faction
        }
    }))
}

pub async fn delete_character(ctx: &Arc<McpContext>, args: Value) -> Result<Value, McpError> {
    let character_id = args
        .get("character_id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| McpError::InvalidArguments("character_id is required".to_string()))?;

    let mut db = ctx.db()?;
    let mut service = CharacterService::new(&mut db);

    service
        .delete(character_id)
        .map_err(|e| McpError::Internal(e.to_string()))?;

    Ok(json!({ "status": "deleted", "character_id": character_id }))
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
    let attuned = args.get("attuned").and_then(|v| v.as_bool()).unwrap_or(false);

    let mut db = ctx.db()?;
    let mut service = CharacterService::new(&mut db);

    let mut input = AddInventoryInput::new(item_name, item_source);

    if let Some(qty) = quantity {
        input = input.with_quantity(qty);
    }
    if equipped {
        input = input.equipped();
    }
    if attuned {
        input = input.attuned();
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
            "equipped": inventory_item.equipped != 0,
            "attuned": inventory_item.attuned != 0
        }
    }))
}

pub async fn remove_item_from_character(ctx: &Arc<McpContext>, args: Value) -> Result<Value, McpError> {
    let inventory_id = args
        .get("inventory_id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| McpError::InvalidArguments("inventory_id is required".to_string()))?;

    let mut db = ctx.db()?;
    let mut service = CharacterService::new(&mut db);

    service
        .remove_from_inventory(inventory_id)
        .map_err(|e| McpError::Internal(e.to_string()))?;

    Ok(json!({ "status": "removed", "inventory_id": inventory_id }))
}

pub async fn update_character_inventory(ctx: &Arc<McpContext>, args: Value) -> Result<Value, McpError> {
    let inventory_id = args
        .get("inventory_id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| McpError::InvalidArguments("inventory_id is required".to_string()))?;

    let quantity = args.get("quantity").and_then(|v| v.as_i64()).map(|q| q as i32);
    let equipped = args.get("equipped").and_then(|v| v.as_bool());
    let attuned = args.get("attuned").and_then(|v| v.as_bool());

    let mut db = ctx.db()?;
    let mut service = CharacterService::new(&mut db);

    let item = service
        .update_inventory_item(inventory_id, quantity, equipped, attuned)
        .map_err(|e| McpError::Internal(e.to_string()))?;

    Ok(json!({
        "status": "updated",
        "inventory_item": {
            "id": item.id,
            "item_name": item.item_name,
            "item_source": item.item_source,
            "quantity": item.quantity,
            "equipped": item.equipped != 0,
            "attuned": item.attuned != 0
        }
    }))
}

pub async fn get_character_inventory(ctx: &Arc<McpContext>, args: Value) -> Result<Value, McpError> {
    let character_id = args
        .get("character_id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| McpError::InvalidArguments("character_id is required".to_string()))?;

    let filter = args.get("filter").and_then(|v| v.as_str()).unwrap_or("all");

    let mut db = ctx.db()?;
    let mut service = CharacterService::new(&mut db);

    let items = match filter {
        "equipped" => service
            .get_equipped_items(character_id)
            .map_err(|e| McpError::Internal(e.to_string()))?,
        "attuned" => service
            .get_attuned_items(character_id)
            .map_err(|e| McpError::Internal(e.to_string()))?,
        _ => service
            .get_inventory(character_id)
            .map_err(|e| McpError::Internal(e.to_string()))?,
    };

    let inv_data: Vec<Value> = items
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
        "filter": filter,
        "inventory": inv_data,
        "count": inv_data.len()
    }))
}

pub async fn level_up_character(ctx: &Arc<McpContext>, args: Value) -> Result<Value, McpError> {
    use mimir_core::services::{
        AsiOrFeat, HpGainMethod, LevelUpRequest, SubclassChoice,
    };

    let character_id = args
        .get("character_id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| McpError::InvalidArguments("character_id is required".to_string()))?;

    let class_name = args
        .get("class_name")
        .and_then(|v| v.as_str())
        .ok_or_else(|| McpError::InvalidArguments("class_name is required".to_string()))?;

    let class_source = args
        .get("class_source")
        .and_then(|v| v.as_str())
        .unwrap_or("PHB");

    // HP method
    let hp_method = match args.get("hp_method").and_then(|v| v.as_str()) {
        Some("roll") => {
            let val = args.get("hp_value").and_then(|v| v.as_i64())
                .ok_or_else(|| McpError::InvalidArguments("hp_value required for roll method".to_string()))? as i32;
            HpGainMethod::Roll(val)
        }
        Some("manual") => {
            let val = args.get("hp_value").and_then(|v| v.as_i64())
                .ok_or_else(|| McpError::InvalidArguments("hp_value required for manual method".to_string()))? as i32;
            HpGainMethod::Manual(val)
        }
        _ => HpGainMethod::Average,
    };

    // Subclass
    let subclass = args.get("subclass_name").and_then(|v| v.as_str()).map(|name| {
        let source = args.get("subclass_source").and_then(|v| v.as_str()).unwrap_or("PHB");
        SubclassChoice {
            name: name.to_string(),
            source: source.to_string(),
        }
    });

    // ASI or Feat
    let asi_or_feat = match args.get("asi_type").and_then(|v| v.as_str()) {
        Some("asi") => {
            let ability1 = args.get("asi_ability1").and_then(|v| v.as_str())
                .ok_or_else(|| McpError::InvalidArguments("asi_ability1 required for ASI".to_string()))?;
            let increase1 = args.get("asi_increase1").and_then(|v| v.as_i64()).unwrap_or(1) as i32;
            let ability2 = args.get("asi_ability2").and_then(|v| v.as_str()).map(|s| s.to_string());
            let increase2 = args.get("asi_increase2").and_then(|v| v.as_i64()).map(|v| v as i32);

            Some(AsiOrFeat::AbilityScoreImprovement {
                ability1: ability1.to_string(),
                increase1,
                ability2,
                increase2,
            })
        }
        Some("feat") => {
            let name = args.get("feat_name").and_then(|v| v.as_str())
                .ok_or_else(|| McpError::InvalidArguments("feat_name required for feat choice".to_string()))?;
            let source = args.get("feat_source").and_then(|v| v.as_str()).unwrap_or("PHB");
            Some(AsiOrFeat::Feat {
                name: name.to_string(),
                source: source.to_string(),
            })
        }
        _ => None,
    };

    let request = LevelUpRequest {
        class_name: class_name.to_string(),
        class_source: class_source.to_string(),
        hit_points_method: hp_method,
        subclass,
        asi_or_feat,
        spell_changes: None,
        feature_choices: None,
    };

    let mut db = ctx.db()?;
    let mut service = CharacterService::new(&mut db);

    let result = service
        .level_up(character_id, request)
        .map_err(|e| McpError::Internal(e.to_string()))?;

    Ok(json!({
        "status": "leveled_up",
        "character_id": character_id,
        "class": {
            "class_name": result.class.class_name,
            "class_source": result.class.class_source,
            "level": result.class.level,
            "subclass_name": result.class.subclass_name
        },
        "hp_gained": result.hp_gained,
        "new_total_level": result.new_total_level,
        "is_multiclass": result.is_multiclass
    }))
}
