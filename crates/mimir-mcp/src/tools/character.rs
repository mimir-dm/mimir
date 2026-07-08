//! Character Tools
//!
//! MCP tools for character (NPC and PC) management. This family is
//! registry-based: typed argument structs, handlers, and one entry each in
//! `registered_tools()`.

use mimir_core::dal::campaign as dal;
use mimir_core::services::{
    AddInventoryInput, CharacterService, CreateCharacterInput, ServiceError, UpdateCharacterInput,
};
use serde_json::{json, Value};
use std::sync::Arc;

use crate::context::McpContext;
use crate::registry::RegisteredTool;
use crate::response::McpResponse;
use crate::{tool, tool_args, McpError};

// =============================================================================
// Registration
// =============================================================================

/// All character-family tools.
pub fn registered_tools() -> Vec<RegisteredTool> {
    vec![
        tool!(
            "list_characters",
            "List characters (PCs and NPCs) in the active campaign. Requires an active campaign.",
            ListCharactersArgs,
            list_characters
        ),
        tool!(
            "get_character",
            "Get detailed information about a character including classes and inventory",
            CharacterIdArgs,
            get_character
        ),
        tool!(
            "create_character",
            "Create a new character (NPC or PC) in the active campaign. Requires an active campaign. When race, class, and background are provided, proficiencies are automatically populated from catalog data.",
            CreateCharacterArgs,
            create_character
        ),
        tool!(
            "edit_character",
            "Update character fields including ability scores, currency, race, background, and roleplay traits",
            EditCharacterArgs,
            edit_character
        ),
        tool!(
            "delete_character",
            "Delete a character and all associated data",
            CharacterIdArgs,
            delete_character
        ),
        tool!(
            "add_item_to_character",
            "Add an item from the catalog to a character's inventory",
            AddItemArgs,
            add_item_to_character
        ),
        tool!(
            "remove_item_from_character",
            "Remove an item from a character's inventory",
            RemoveItemArgs,
            remove_item_from_character
        ),
        tool!(
            "update_character_inventory",
            "Update an inventory item's quantity, equipped, or attuned state",
            UpdateInventoryArgs,
            update_character_inventory
        ),
        tool!(
            "get_character_inventory",
            "Get a character's inventory, optionally filtered by equipped or attuned",
            GetInventoryArgs,
            get_character_inventory
        ),
        tool!(
            "level_up_character",
            "Level up a character. Handles HP, multiclass validation, ASI/feats, spells, and feature choices.",
            LevelUpArgs,
            level_up_character
        ),
        tool!(
            "add_character_spell",
            "Add a spell to a character's known spells. Supports both catalog and homebrew spells.",
            AddSpellArgs,
            add_character_spell
        ),
        tool!(
            "remove_character_spell",
            "Remove a spell from a character's known spells",
            RemoveSpellArgs,
            remove_character_spell
        ),
        tool!(
            "list_character_spells",
            "List all spells known by a character, optionally filtered by class",
            ListSpellsArgs,
            list_character_spells
        ),
    ]
}

// =============================================================================
// Arguments
// =============================================================================

tool_args! {
    pub struct ListCharactersArgs {
        /// Filter by type: pc or npc
        pub character_type: Option<String>,
        /// Filter by module assignment (NPCs only)
        pub module_id: Option<String>,
        /// Filter NPCs by location
        pub location: Option<String>,
        /// Filter NPCs by faction
        pub faction: Option<String>,
    }
}

tool_args! {
    pub struct CharacterIdArgs {
        /// The ID of the character
        pub character_id: String,
    }
}

tool_args! {
    pub struct CreateCharacterArgs {
        /// Name of the character
        pub name: String,
        /// Type: pc or npc
        pub character_type: String,
        /// Player name (PCs only, defaults to 'Player')
        pub player_name: Option<String>,
        /// Race name (e.g., 'Elf'). Populates race proficiencies.
        pub race_name: Option<String>,
        /// Race source book (default: PHB)
        pub race_source: Option<String>,
        /// Starting class (e.g., 'Fighter'). Populates saving throws, armor, weapon, and tool proficiencies.
        pub class_name: Option<String>,
        /// Class source book (default: PHB)
        pub class_source: Option<String>,
        /// Background name (e.g., 'Acolyte'). Populates skill, tool, and language proficiencies.
        pub background_name: Option<String>,
        /// Background source book (default: PHB)
        pub background_source: Option<String>,
        /// Skill proficiencies chosen from the class skill list (e.g., ['Perception', 'Stealth'])
        pub selected_skills: Option<Vec<String>>,
        /// Starting level (default: 1)
        pub level: Option<i64>,
    }
}

tool_args! {
    pub struct EditCharacterArgs {
        /// The ID of the character
        pub character_id: String,
        /// New name
        pub name: Option<String>,
        /// Player name (PCs only)
        pub player_name: Option<String>,
        /// Race name
        pub race_name: Option<String>,
        /// Race source book (default: PHB)
        pub race_source: Option<String>,
        /// Background name
        pub background_name: Option<String>,
        /// Background source book (default: PHB)
        pub background_source: Option<String>,
        /// Strength score
        pub strength: Option<i64>,
        /// Dexterity score
        pub dexterity: Option<i64>,
        /// Constitution score
        pub constitution: Option<i64>,
        /// Intelligence score
        pub intelligence: Option<i64>,
        /// Wisdom score
        pub wisdom: Option<i64>,
        /// Charisma score
        pub charisma: Option<i64>,
        /// Copper pieces
        pub cp: Option<i64>,
        /// Silver pieces
        pub sp: Option<i64>,
        /// Electrum pieces
        pub ep: Option<i64>,
        /// Gold pieces
        pub gp: Option<i64>,
        /// Platinum pieces
        pub pp: Option<i64>,
        /// Assign to module (NPCs only)
        pub module_id: Option<String>,
        /// NPC's role in the module
        pub npc_role: Option<String>,
        /// NPC's location
        pub npc_location: Option<String>,
        /// Faction affiliation
        pub faction: Option<String>,
        /// Personality traits
        pub traits: Option<String>,
        /// Ideals
        pub ideals: Option<String>,
        /// Bonds
        pub bonds: Option<String>,
        /// Flaws
        pub flaws: Option<String>,
    }
}

tool_args! {
    pub struct AddItemArgs {
        /// The ID of the character
        pub character_id: String,
        /// Name of the item from the catalog
        pub item_name: String,
        /// Source book abbreviation (default: PHB, or HB if the name matches campaign homebrew)
        pub item_source: Option<String>,
        /// Quantity of the item (default: 1)
        pub quantity: Option<i64>,
        /// Whether the item is equipped (default: false)
        pub equipped: Option<bool>,
        /// Whether the item is attuned (default: false)
        pub attuned: Option<bool>,
    }
}

tool_args! {
    pub struct RemoveItemArgs {
        /// The ID of the inventory entry to remove
        pub inventory_id: String,
    }
}

tool_args! {
    pub struct UpdateInventoryArgs {
        /// The ID of the inventory entry
        pub inventory_id: String,
        /// New quantity
        pub quantity: Option<i64>,
        /// Whether equipped
        pub equipped: Option<bool>,
        /// Whether attuned (max 3 attuned items per D&D 5e rules)
        pub attuned: Option<bool>,
    }
}

tool_args! {
    pub struct GetInventoryArgs {
        /// The ID of the character
        pub character_id: String,
        /// Filter: all, equipped, or attuned (default: all)
        pub filter: Option<String>,
    }
}

tool_args! {
    pub struct LevelUpArgs {
        /// The ID of the character
        pub character_id: String,
        /// Class to level up in (e.g. Fighter, Wizard)
        pub class_name: String,
        /// Class source book (default: PHB)
        pub class_source: Option<String>,
        /// HP gain method: average, roll, or manual
        pub hp_method: Option<String>,
        /// Roll result or manual HP value (required for roll/manual)
        pub hp_value: Option<i64>,
        /// Subclass name if choosing this level
        pub subclass_name: Option<String>,
        /// Subclass source book (default: PHB)
        pub subclass_source: Option<String>,
        /// ASI or feat: 'asi' or 'feat'
        pub asi_type: Option<String>,
        /// First ability to increase (for ASI)
        pub asi_ability1: Option<String>,
        /// Amount for first ability: 1 or 2 (for ASI)
        pub asi_increase1: Option<i64>,
        /// Second ability to increase (for ASI, optional)
        pub asi_ability2: Option<String>,
        /// Amount for second ability (for ASI)
        pub asi_increase2: Option<i64>,
        /// Feat name (if choosing feat)
        pub feat_name: Option<String>,
        /// Feat source (default: PHB)
        pub feat_source: Option<String>,
    }
}

tool_args! {
    pub struct AddSpellArgs {
        /// The ID of the character
        pub character_id: String,
        /// Name of the spell (e.g. Fireball)
        pub spell_name: String,
        /// Source book abbreviation (e.g. PHB, XGE) or HB for homebrew
        pub spell_source: String,
        /// Class that grants this spell (e.g. Wizard, Cleric)
        pub source_class: String,
        /// Whether the spell starts prepared (default: false)
        pub prepared: Option<bool>,
    }
}

tool_args! {
    pub struct RemoveSpellArgs {
        /// The ID of the character
        pub character_id: String,
        /// Name of the spell to remove
        pub spell_name: String,
        /// Class that granted the spell (optional — if omitted, removes all instances)
        pub source_class: Option<String>,
    }
}

tool_args! {
    pub struct ListSpellsArgs {
        /// The ID of the character
        pub character_id: String,
        /// Filter by granting class (e.g. Wizard)
        pub source_class: Option<String>,
        /// Only return prepared spells
        pub prepared_only: Option<bool>,
    }
}

// =============================================================================
// Handlers
// =============================================================================

pub async fn list_characters(
    ctx: &Arc<McpContext>,
    args: ListCharactersArgs,
) -> Result<Value, McpError> {
    let campaign_id = ctx
        .get_active_campaign_id()
        .ok_or(McpError::NoActiveCampaign)?;

    let mut db = ctx.connect()?;
    let mut service = CharacterService::new(&mut db);

    let characters = if let Some(loc) = args.location.as_deref() {
        service
            .list_npcs_by_location(&campaign_id, loc)
            .map_err(|e| McpError::Internal(e.to_string()))?
    } else if let Some(fac) = args.faction.as_deref() {
        service
            .list_npcs_by_faction(&campaign_id, fac)
            .map_err(|e| McpError::Internal(e.to_string()))?
    } else {
        match args.character_type.as_deref() {
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

    McpResponse::list("characters", char_data)
}

pub async fn get_character(
    ctx: &Arc<McpContext>,
    args: CharacterIdArgs,
) -> Result<Value, McpError> {
    let character_id = &args.character_id;
    let mut db = ctx.connect()?;

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

    McpResponse::ok(json!({
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

pub async fn create_character(
    ctx: &Arc<McpContext>,
    args: CreateCharacterArgs,
) -> Result<Value, McpError> {
    let campaign_id = ctx
        .get_active_campaign_id()
        .ok_or(McpError::NoActiveCampaign)?;

    let mut db = ctx.connect()?;
    let mut service = CharacterService::new(&mut db);

    // Create character based on type
    let mut input = if args.character_type == "npc" {
        CreateCharacterInput::new_npc(Some(&campaign_id), &args.name)
    } else {
        let player_name = args.player_name.as_deref().unwrap_or("Player");
        CreateCharacterInput::new_pc(Some(&campaign_id), &args.name, player_name)
    };

    // Set race if provided
    if let Some(race) = args.race_name.as_deref() {
        let race_source = args.race_source.as_deref().unwrap_or("PHB");
        input = input.with_race(race, race_source);
    }

    // Set class if provided (enables proficiency auto-population)
    if let Some(class_name) = args.class_name.as_deref() {
        let class_source = args.class_source.as_deref().unwrap_or("PHB");
        input = input.with_class(class_name, class_source);
    }

    // Set background if provided (enables proficiency auto-population)
    if let Some(bg_name) = args.background_name.as_deref() {
        let bg_source = args.background_source.as_deref().unwrap_or("PHB");
        input = input.with_background(bg_name, bg_source);
    }

    // Set selected skills if provided
    if let Some(skills) = args.selected_skills {
        if !skills.is_empty() {
            input = input.with_skills(skills);
        }
    }

    let character = service
        .create(input)
        .map_err(|e| McpError::Internal(e.to_string()))?;

    McpResponse::created("character", json!({
        "id": character.id,
        "name": character.name,
        "is_npc": character.is_npc(),
        "race_name": character.race_name
    }))
}

pub async fn edit_character(
    ctx: &Arc<McpContext>,
    args: EditCharacterArgs,
) -> Result<Value, McpError> {
    let character_id = &args.character_id;

    let mut db = ctx.connect()?;
    let mut service = CharacterService::new(&mut db);

    // Build update from provided fields
    let mut update = UpdateCharacterInput::default();

    if let Some(name) = args.name {
        update.name = Some(name);
    }
    if let Some(pn) = args.player_name {
        update.player_name = Some(Some(pn));
    }

    // Race/background
    if let Some(rn) = args.race_name {
        update.race_name = Some(Some(rn));
        let rs = args.race_source.as_deref().unwrap_or("PHB");
        update.race_source = Some(Some(rs.to_string()));
    }
    if let Some(bn) = args.background_name {
        update.background_name = Some(Some(bn));
        let bs = args.background_source.as_deref().unwrap_or("PHB");
        update.background_source = Some(Some(bs.to_string()));
    }

    // Ability scores — set as array if any are provided; unspecified ones keep
    // the character's current value (requires a read)
    if args.strength.is_some()
        || args.dexterity.is_some()
        || args.constitution.is_some()
        || args.intelligence.is_some()
        || args.wisdom.is_some()
        || args.charisma.is_some()
    {
        let current = service
            .get(character_id)
            .map_err(|e| McpError::Internal(e.to_string()))?
            .ok_or_else(|| {
                McpError::InvalidArguments(format!("Character '{}' not found", character_id))
            })?;

        update.ability_scores = Some([
            args.strength.unwrap_or(current.strength as i64) as i32,
            args.dexterity.unwrap_or(current.dexterity as i64) as i32,
            args.constitution.unwrap_or(current.constitution as i64) as i32,
            args.intelligence.unwrap_or(current.intelligence as i64) as i32,
            args.wisdom.unwrap_or(current.wisdom as i64) as i32,
            args.charisma.unwrap_or(current.charisma as i64) as i32,
        ]);
    }

    // Currency — same pattern
    if args.cp.is_some()
        || args.sp.is_some()
        || args.ep.is_some()
        || args.gp.is_some()
        || args.pp.is_some()
    {
        let cur = service
            .get(character_id)
            .map_err(|e| McpError::Internal(e.to_string()))?
            .ok_or_else(|| {
                McpError::InvalidArguments(format!("Character '{}' not found", character_id))
            })?;

        update.currency = Some([
            args.cp.unwrap_or(cur.cp as i64) as i32,
            args.sp.unwrap_or(cur.sp as i64) as i32,
            args.ep.unwrap_or(cur.ep as i64) as i32,
            args.gp.unwrap_or(cur.gp as i64) as i32,
            args.pp.unwrap_or(cur.pp as i64) as i32,
        ]);
    }

    // NPC-specific fields
    if let Some(role) = args.npc_role {
        update.role = Some(Some(role));
    }
    if let Some(location) = args.npc_location {
        update.location = Some(Some(location));
    }
    if let Some(faction) = args.faction {
        update.faction = Some(Some(faction));
    }

    // Roleplay fields
    if let Some(traits) = args.traits {
        update.traits = Some(Some(traits));
    }
    if let Some(ideals) = args.ideals {
        update.ideals = Some(Some(ideals));
    }
    if let Some(bonds) = args.bonds {
        update.bonds = Some(Some(bonds));
    }
    if let Some(flaws) = args.flaws {
        update.flaws = Some(Some(flaws));
    }

    let character = service
        .update(character_id, update)
        .map_err(|e| McpError::Internal(e.to_string()))?;

    McpResponse::updated("character", json!({
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
    }))
}

pub async fn delete_character(
    ctx: &Arc<McpContext>,
    args: CharacterIdArgs,
) -> Result<Value, McpError> {
    let mut db = ctx.connect()?;
    let mut service = CharacterService::new(&mut db);

    service
        .delete(&args.character_id)
        .map_err(|e| McpError::Internal(e.to_string()))?;

    McpResponse::deleted(&args.character_id)
}

pub async fn add_item_to_character(
    ctx: &Arc<McpContext>,
    args: AddItemArgs,
) -> Result<Value, McpError> {
    // If no explicit source was provided, check if this item exists as homebrew
    // in the active campaign — if so, use "HB" source instead of "PHB".
    // (MCP-only convenience; the service seam takes an explicit source.)
    let item_source = match args.item_source.as_deref() {
        Some(source) => source.to_string(),
        None => {
            let mut detected = "PHB".to_string();
            if let Some(campaign_id) = ctx.get_active_campaign_id() {
                let mut db = ctx.connect()?;
                if let Ok(Some(_)) = mimir_core::dal::campaign::get_campaign_homebrew_item_by_name(
                    &mut db,
                    &campaign_id,
                    &args.item_name,
                ) {
                    detected = "HB".to_string();
                }
            }
            detected
        }
    };

    let mut db = ctx.connect()?;
    let mut service = CharacterService::new(&mut db);

    let mut input = AddInventoryInput::new(&args.item_name, &item_source);
    if let Some(qty) = args.quantity {
        input = input.with_quantity(qty as i32);
    }
    if args.equipped.unwrap_or(false) {
        input = input.equipped();
    }
    if args.attuned.unwrap_or(false) {
        input = input.attuned();
    }

    let inventory_item = service
        .add_to_inventory(&args.character_id, input)
        .map_err(|e| McpError::Internal(e.to_string()))?;

    McpResponse::added("inventory_item", json!({
        "id": inventory_item.id,
        "item_name": inventory_item.item_name,
        "item_source": inventory_item.item_source,
        "quantity": inventory_item.quantity,
        "equipped": inventory_item.equipped != 0,
        "attuned": inventory_item.attuned != 0
    }))
}

pub async fn remove_item_from_character(
    ctx: &Arc<McpContext>,
    args: RemoveItemArgs,
) -> Result<Value, McpError> {
    let mut db = ctx.connect()?;
    let mut service = CharacterService::new(&mut db);

    service
        .remove_from_inventory(&args.inventory_id)
        .map_err(|e| McpError::Internal(e.to_string()))?;

    McpResponse::removed(&args.inventory_id)
}

pub async fn update_character_inventory(
    ctx: &Arc<McpContext>,
    args: UpdateInventoryArgs,
) -> Result<Value, McpError> {
    let mut db = ctx.connect()?;
    let mut service = CharacterService::new(&mut db);

    let item = service
        .update_inventory_item(
            &args.inventory_id,
            args.quantity.map(|q| q as i32),
            args.equipped,
            args.attuned,
        )
        .map_err(|e| McpError::Internal(e.to_string()))?;

    McpResponse::updated("inventory_item", json!({
        "id": item.id,
        "item_name": item.item_name,
        "item_source": item.item_source,
        "quantity": item.quantity,
        "equipped": item.equipped != 0,
        "attuned": item.attuned != 0
    }))
}

pub async fn get_character_inventory(
    ctx: &Arc<McpContext>,
    args: GetInventoryArgs,
) -> Result<Value, McpError> {
    let character_id = &args.character_id;
    let filter = args.filter.as_deref().unwrap_or("all");

    let mut db = ctx.connect()?;
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

    McpResponse::ok(json!({
        "filter": filter,
        "inventory": inv_data,
        "count": inv_data.len()
    }))
}

pub async fn level_up_character(
    ctx: &Arc<McpContext>,
    args: LevelUpArgs,
) -> Result<Value, McpError> {
    use mimir_core::services::{AsiOrFeat, HpGainMethod, LevelUpRequest, SubclassChoice};

    let character_id = &args.character_id;
    let class_source = args.class_source.as_deref().unwrap_or("PHB");

    // HP method
    let hp_method = match args.hp_method.as_deref() {
        Some("roll") => {
            let val = args.hp_value.ok_or_else(|| {
                McpError::InvalidArguments("hp_value required for roll method".to_string())
            })? as i32;
            HpGainMethod::Roll(val)
        }
        Some("manual") => {
            let val = args.hp_value.ok_or_else(|| {
                McpError::InvalidArguments("hp_value required for manual method".to_string())
            })? as i32;
            HpGainMethod::Manual(val)
        }
        _ => HpGainMethod::Average,
    };

    // Subclass
    let subclass = args.subclass_name.map(|name| SubclassChoice {
        name,
        source: args
            .subclass_source
            .unwrap_or_else(|| "PHB".to_string()),
    });

    // ASI or Feat
    let asi_or_feat = match args.asi_type.as_deref() {
        Some("asi") => {
            let ability1 = args.asi_ability1.ok_or_else(|| {
                McpError::InvalidArguments("asi_ability1 required for ASI".to_string())
            })?;
            Some(AsiOrFeat::AbilityScoreImprovement {
                ability1,
                increase1: args.asi_increase1.unwrap_or(1) as i32,
                ability2: args.asi_ability2,
                increase2: args.asi_increase2.map(|v| v as i32),
            })
        }
        Some("feat") => {
            let name = args.feat_name.ok_or_else(|| {
                McpError::InvalidArguments("feat_name required for feat choice".to_string())
            })?;
            Some(AsiOrFeat::Feat {
                name,
                source: args.feat_source.unwrap_or_else(|| "PHB".to_string()),
            })
        }
        _ => None,
    };

    let request = LevelUpRequest {
        class_name: args.class_name.clone(),
        class_source: class_source.to_string(),
        hit_points_method: hp_method,
        subclass,
        asi_or_feat,
        spell_changes: None,
        feature_choices: None,
    };

    let mut db = ctx.connect()?;
    let mut service = CharacterService::new(&mut db);

    let result = service
        .level_up(character_id, request)
        .map_err(|e| McpError::Internal(e.to_string()))?;

    McpResponse::success(json!({
        "action": "leveled_up",
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

pub async fn add_character_spell(
    ctx: &Arc<McpContext>,
    args: AddSpellArgs,
) -> Result<Value, McpError> {
    let mut db = ctx.connect()?;

    let spell = CharacterService::new(&mut db)
        .add_spell(
            &args.character_id,
            &args.spell_name,
            &args.spell_source,
            &args.source_class,
            args.prepared.unwrap_or(false),
        )
        .map_err(|e| match e {
            ServiceError::Validation(msg) => McpError::InvalidArguments(msg),
            other => McpError::Internal(other.to_string()),
        })?;

    McpResponse::success(json!({
        "action": "spell_added",
        "character_id": args.character_id,
        "spell": {
            "id": spell.id,
            "spell_name": spell.spell_name,
            "spell_source": spell.spell_source,
            "source_class": spell.source_class,
            "prepared": spell.is_prepared()
        }
    }))
}

pub async fn remove_character_spell(
    ctx: &Arc<McpContext>,
    args: RemoveSpellArgs,
) -> Result<Value, McpError> {
    let mut db = ctx.connect()?;

    CharacterService::new(&mut db)
        .remove_spell(
            &args.character_id,
            &args.spell_name,
            args.source_class.as_deref(),
        )
        .map_err(|e| match e {
            ServiceError::Validation(msg) => McpError::InvalidArguments(msg),
            other => McpError::Internal(other.to_string()),
        })?;

    McpResponse::success(json!({
        "action": "spell_removed",
        "character_id": args.character_id,
        "spell_name": args.spell_name
    }))
}

pub async fn list_character_spells(
    ctx: &Arc<McpContext>,
    args: ListSpellsArgs,
) -> Result<Value, McpError> {
    let character_id = &args.character_id;
    let source_class = args.source_class.as_deref();
    let prepared_only = args.prepared_only.unwrap_or(false);

    let mut db = ctx.connect()?;

    let spells = if let Some(class) = source_class {
        dal::list_spells_by_class(&mut db, character_id, class)
    } else if prepared_only {
        dal::list_prepared_spells(&mut db, character_id)
    } else {
        dal::list_character_spells(&mut db, character_id)
    }
    .map_err(|e| McpError::Internal(e.to_string()))?;

    // Apply prepared filter if class filter was used
    let spells: Vec<_> = if prepared_only && source_class.is_some() {
        spells.into_iter().filter(|s| s.is_prepared()).collect()
    } else {
        spells
    };

    let spell_data: Vec<Value> = spells
        .iter()
        .map(|s| {
            json!({
                "id": s.id,
                "spell_name": s.spell_name,
                "spell_source": s.spell_source,
                "source_class": s.source_class,
                "prepared": s.is_prepared()
            })
        })
        .collect();

    McpResponse::success(json!({
        "character_id": character_id,
        "spell_count": spell_data.len(),
        "spells": spell_data
    }))
}
