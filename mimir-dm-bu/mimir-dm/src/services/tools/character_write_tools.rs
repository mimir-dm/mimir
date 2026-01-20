//! Write-capable character tools for LLM interactions
//!
//! These tools allow LLMs to modify character data with user confirmation

use async_trait::async_trait;
use mimir_dm_core::dal::campaign::campaigns::CampaignRepository;
use mimir_dm_core::models::character::data::{InventoryItem, Personality};
use mimir_dm_core::services::character::creation::{AbilityScoreMethod, CharacterBuilder};
use mimir_dm_core::services::character::spell_management::RestType;
use mimir_dm_core::{services::CharacterService, DatabaseService};
use mimir_dm_llm::traits::{ActionDescription, ChangeDetail};
use mimir_dm_llm::ToolTrait;
use serde_json::{json, Value};
use std::error::Error;
use std::sync::Arc;
use tracing::debug;

/// Tool for updating character HP (damage/healing)
pub struct UpdateCharacterHpTool {
    db_service: Arc<DatabaseService>,
}

impl UpdateCharacterHpTool {
    pub fn new(db_service: Arc<DatabaseService>) -> Self {
        Self { db_service }
    }
}

#[async_trait]
impl ToolTrait for UpdateCharacterHpTool {
    fn name(&self) -> &str {
        "update_character_hp"
    }

    fn description(&self) -> &str {
        "Update a character's current HP (apply damage or healing).

Usage:
- Provide character_id and new_hp value
- Optionally provide reason for HP change
- Creates new character version snapshot
- Respects max HP limits

When to use:
- After combat encounters
- When healing spells or potions are used
- Recording damage during session
- Tracking character health status

Output:
- Updated character with new HP value
- Character version created for history"
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "character_id": {
                    "type": "integer",
                    "description": "ID of the character to update"
                },
                "new_hp": {
                    "type": "integer",
                    "description": "New current HP value (0 to max_hp)"
                },
                "reason": {
                    "type": ["string", "null"],
                    "description": "Reason for HP change (e.g., 'Took 10 damage from goblin', 'Healed 8 HP from potion')"
                }
            },
            "required": ["character_id", "new_hp"]
        })
    }

    fn requires_confirmation(&self) -> bool {
        true
    }

    fn describe_action(&self, arguments: &Value) -> Option<ActionDescription> {
        let character_id = arguments.get("character_id")?.as_i64()?;
        let new_hp = arguments.get("new_hp")?.as_i64()?;
        let reason = arguments
            .get("reason")
            .and_then(|v| v.as_str())
            .unwrap_or("HP updated");

        // Try to get current character data for comparison
        let current_hp_info = if let Ok(mut conn) = self.db_service.get_connection() {
            let mut char_service = CharacterService::new(&mut conn);
            if let Ok((_, char_data)) = char_service.get_character(character_id as i32) {
                Some((
                    char_data.character_name.clone(),
                    char_data.current_hp,
                    char_data.max_hp,
                ))
            } else {
                None
            }
        } else {
            None
        };

        let description = if let Some((name, current, max)) = current_hp_info {
            let change = new_hp as i32 - current;
            let change_desc = if change > 0 {
                format!("heal {} HP", change)
            } else if change < 0 {
                format!("take {} damage", -change)
            } else {
                "no change".to_string()
            };

            format!(
                "Update {}'s HP from {}/{} to {}/{} ({})\nReason: {}",
                name, current, max, new_hp, max, change_desc, reason
            )
        } else {
            format!(
                "Update character {} HP to {}\nReason: {}",
                character_id, new_hp, reason
            )
        };

        Some(ActionDescription {
            title: "Update Character HP".to_string(),
            description,
            changes: ChangeDetail::Generic {
                items: vec![
                    format!("Character ID: {}", character_id),
                    format!("New HP: {}", new_hp),
                    format!("Reason: {}", reason),
                ],
            },
        })
    }

    async fn execute(&self, arguments: Value) -> Result<String, Box<dyn Error + Send + Sync>> {
        let character_id = arguments
            .get("character_id")
            .and_then(|v| v.as_i64())
            .ok_or("Missing 'character_id' parameter")? as i32;

        let new_hp = arguments
            .get("new_hp")
            .and_then(|v| v.as_i64())
            .ok_or("Missing 'new_hp' parameter")? as i32;

        let reason = arguments
            .get("reason")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let mut conn = self
            .db_service
            .get_connection()
            .map_err(|e| format!("Failed to get database connection: {}", e))?;

        let mut char_service = CharacterService::new(&mut conn);
        let (_, mut char_data) = char_service
            .get_character(character_id)
            .map_err(|e| format!("Failed to retrieve character: {}", e))?;

        let old_hp = char_data.current_hp;
        char_data.current_hp = new_hp.max(0).min(char_data.max_hp);

        let snapshot_reason = reason
            .unwrap_or_else(|| format!("HP updated from {} to {}", old_hp, char_data.current_hp));

        char_service
            .update_character(character_id, char_data.clone(), Some(snapshot_reason))
            .map_err(|e| format!("Failed to update character: {}", e))?;

        let result = json!({
            "success": true,
            "character_id": character_id,
            "character_name": char_data.character_name,
            "old_hp": old_hp,
            "new_hp": char_data.current_hp,
            "max_hp": char_data.max_hp,
            "message": format!("Updated {} HP from {} to {}", char_data.character_name, old_hp, char_data.current_hp)
        });

        debug!(
            "Updated character {} HP: {} -> {}",
            character_id, old_hp, char_data.current_hp
        );
        Ok(serde_json::to_string_pretty(&result)?)
    }
}

/// Tool for adding items to character inventory
pub struct AddInventoryItemTool {
    db_service: Arc<DatabaseService>,
}

impl AddInventoryItemTool {
    pub fn new(db_service: Arc<DatabaseService>) -> Self {
        Self { db_service }
    }
}

#[async_trait]
impl ToolTrait for AddInventoryItemTool {
    fn name(&self) -> &str {
        "add_inventory_item"
    }

    fn description(&self) -> &str {
        "Add an item to a character's inventory.

Usage:
- Provide character_id, item name, and quantity
- Optionally provide weight, value, and notes
- Creates new character version snapshot

When to use:
- After looting enemies or treasure
- When characters purchase items
- Recording quest rewards
- Adding starting equipment

Output:
- Updated character with new inventory item
- Character version created for history"
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "character_id": {
                    "type": "integer",
                    "description": "ID of the character"
                },
                "item_name": {
                    "type": "string",
                    "description": "Name of the item to add"
                },
                "item_source": {
                    "type": ["string", "null"],
                    "description": "Source book of the item (e.g., PHB, DMG)"
                },
                "quantity": {
                    "type": ["integer", "null"],
                    "description": "Quantity to add (default: 1)"
                },
                "weight": {
                    "type": ["number", "null"],
                    "description": "Weight per item in pounds (optional)"
                },
                "value": {
                    "type": ["number", "null"],
                    "description": "Value per item in gold pieces (optional)"
                },
                "notes": {
                    "type": ["string", "null"],
                    "description": "Additional notes about the item (optional)"
                }
            },
            "required": ["character_id", "item_name"]
        })
    }

    fn requires_confirmation(&self) -> bool {
        true
    }

    fn describe_action(&self, arguments: &Value) -> Option<ActionDescription> {
        let character_id = arguments.get("character_id")?.as_i64()?;
        let item_name = arguments.get("item_name")?.as_str()?;
        let quantity = arguments
            .get("quantity")
            .and_then(|v| v.as_i64())
            .unwrap_or(1);
        let weight = arguments.get("weight").and_then(|v| v.as_f64());
        let value = arguments.get("value").and_then(|v| v.as_f64());

        let mut details = vec![
            format!("Character ID: {}", character_id),
            format!("Item: {}", item_name),
            format!("Quantity: {}", quantity),
        ];

        if let Some(w) = weight {
            details.push(format!("Weight: {} lb", w));
        }
        if let Some(v) = value {
            details.push(format!("Value: {} gp", v));
        }

        Some(ActionDescription {
            title: "Add Inventory Item".to_string(),
            description: format!("Add {} × {} to character inventory", quantity, item_name),
            changes: ChangeDetail::Generic { items: details },
        })
    }

    async fn execute(&self, arguments: Value) -> Result<String, Box<dyn Error + Send + Sync>> {
        let character_id = arguments
            .get("character_id")
            .and_then(|v| v.as_i64())
            .ok_or("Missing 'character_id' parameter")? as i32;

        let item_name = arguments
            .get("item_name")
            .and_then(|v| v.as_str())
            .ok_or("Missing 'item_name' parameter")?;

        let item_source = arguments
            .get("item_source")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let quantity = arguments
            .get("quantity")
            .and_then(|v| v.as_i64())
            .unwrap_or(1) as i32;

        let weight = arguments
            .get("weight")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);

        let value = arguments
            .get("value")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);

        let notes = arguments
            .get("notes")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let mut conn = self
            .db_service
            .get_connection()
            .map_err(|e| format!("Failed to get database connection: {}", e))?;

        let mut char_service = CharacterService::new(&mut conn);
        let (_, mut char_data) = char_service
            .get_character(character_id)
            .map_err(|e| format!("Failed to retrieve character: {}", e))?;

        // Check if item already exists in inventory
        if let Some(existing) = char_data.inventory.iter_mut().find(|i| i.name == item_name) {
            existing.quantity += quantity;
        } else {
            char_data.inventory.push(InventoryItem {
                name: item_name.to_string(),
                source: item_source,
                quantity,
                weight,
                value,
                notes,
            });
        }

        let snapshot_reason = format!("Added {} × {} to inventory", quantity, item_name);
        char_service
            .update_character(character_id, char_data.clone(), Some(snapshot_reason))
            .map_err(|e| format!("Failed to update character: {}", e))?;

        let result = json!({
            "success": true,
            "character_id": character_id,
            "character_name": char_data.character_name,
            "item_added": item_name,
            "quantity": quantity,
            "message": format!("Added {} × {} to {}'s inventory", quantity, item_name, char_data.character_name)
        });

        debug!(
            "Added item to character {}: {} × {}",
            character_id, quantity, item_name
        );
        Ok(serde_json::to_string_pretty(&result)?)
    }
}

/// Tool for casting spells (consumes spell slots)
pub struct CastSpellTool {
    db_service: Arc<DatabaseService>,
}

impl CastSpellTool {
    pub fn new(db_service: Arc<DatabaseService>) -> Self {
        Self { db_service }
    }
}

#[async_trait]
impl ToolTrait for CastSpellTool {
    fn name(&self) -> &str {
        "cast_spell"
    }

    fn description(&self) -> &str {
        "Cast a spell and consume the appropriate spell slot.

Usage:
- Provide character_id, spell_name, and spell_level
- Automatically reduces available spell slots
- Creates new character version snapshot
- Validates character has available slots

When to use:
- During combat when spells are cast
- Recording spell usage in sessions
- Tracking spell slot consumption
- Before rest/long rest recovery

Output:
- Updated character with reduced spell slots
- Character version created for history"
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "character_id": {
                    "type": "integer",
                    "description": "ID of the character casting the spell"
                },
                "spell_name": {
                    "type": "string",
                    "description": "Name of the spell being cast"
                },
                "spell_level": {
                    "type": "integer",
                    "description": "Spell level (1-9, use 0 for cantrips)"
                }
            },
            "required": ["character_id", "spell_name", "spell_level"]
        })
    }

    fn requires_confirmation(&self) -> bool {
        true
    }

    fn describe_action(&self, arguments: &Value) -> Option<ActionDescription> {
        let character_id = arguments.get("character_id")?.as_i64()?;
        let spell_name = arguments.get("spell_name")?.as_str()?;
        let spell_level = arguments.get("spell_level")?.as_i64()?;

        // Try to get current character data for slot availability
        let slot_info = if let Ok(mut conn) = self.db_service.get_connection() {
            let mut char_service = CharacterService::new(&mut conn);
            if let Ok((_, char_data)) = char_service.get_character(character_id as i32) {
                char_data
                    .spells
                    .spell_slots
                    .get(&(spell_level as i32))
                    .map(|slots| (char_data.character_name.clone(), slots.current, slots.max))
            } else {
                None
            }
        } else {
            None
        };

        let description = if spell_level == 0 {
            format!("Cast cantrip: {} (no slot required)", spell_name)
        } else if let Some((name, current, max)) = slot_info {
            format!(
                "{} casts {} (level {})\nCurrent level {} slots: {}/{}\nAfter cast: {}/{}",
                name,
                spell_name,
                spell_level,
                spell_level,
                current,
                max,
                current - 1,
                max
            )
        } else {
            format!(
                "Cast {} (level {}) - consumes 1 spell slot",
                spell_name, spell_level
            )
        };

        Some(ActionDescription {
            title: "Cast Spell".to_string(),
            description,
            changes: ChangeDetail::Generic {
                items: vec![
                    format!("Character ID: {}", character_id),
                    format!("Spell: {}", spell_name),
                    format!("Level: {}", spell_level),
                    if spell_level > 0 {
                        "Consumes 1 spell slot".to_string()
                    } else {
                        "Cantrip (no slot consumed)".to_string()
                    },
                ],
            },
        })
    }

    async fn execute(&self, arguments: Value) -> Result<String, Box<dyn Error + Send + Sync>> {
        let character_id = arguments
            .get("character_id")
            .and_then(|v| v.as_i64())
            .ok_or("Missing 'character_id' parameter")? as i32;

        let spell_name = arguments
            .get("spell_name")
            .and_then(|v| v.as_str())
            .ok_or("Missing 'spell_name' parameter")?;

        let spell_level = arguments
            .get("spell_level")
            .and_then(|v| v.as_i64())
            .ok_or("Missing 'spell_level' parameter")? as i32;

        if spell_level == 0 {
            return Ok(json!({
                "success": true,
                "message": format!("Cast cantrip {} (no slot consumed)", spell_name)
            })
            .to_string());
        }

        let mut conn = self
            .db_service
            .get_connection()
            .map_err(|e| format!("Failed to get database connection: {}", e))?;

        let mut char_service = CharacterService::new(&mut conn);
        let (_, mut char_data) = char_service
            .get_character(character_id)
            .map_err(|e| format!("Failed to retrieve character: {}", e))?;

        // Check if character has spell slots for this level
        let slots = char_data
            .spells
            .spell_slots
            .get_mut(&spell_level)
            .ok_or(format!(
                "Character has no level {} spell slots",
                spell_level
            ))?;

        if slots.current <= 0 {
            return Err(format!(
                "No level {} spell slots remaining (0/{})",
                spell_level, slots.max
            )
            .into());
        }

        // Consume spell slot
        slots.current -= 1;

        // Capture values before the borrow ends
        let slots_remaining = slots.current;
        let slots_max = slots.max;

        let snapshot_reason = format!(
            "Cast {} (level {}) - {} slots remaining",
            spell_name, spell_level, slots_remaining
        );

        char_service
            .update_character(character_id, char_data.clone(), Some(snapshot_reason))
            .map_err(|e| format!("Failed to update character: {}", e))?;

        let result = json!({
            "success": true,
            "character_id": character_id,
            "character_name": char_data.character_name,
            "spell_cast": spell_name,
            "spell_level": spell_level,
            "slots_remaining": slots_remaining,
            "slots_max": slots_max,
            "message": format!(
                "{} cast {} (level {}). {} slots remaining.",
                char_data.character_name, spell_name, spell_level, slots_remaining
            )
        });

        debug!(
            "Character {} cast spell: {} (level {})",
            character_id, spell_name, spell_level
        );
        Ok(serde_json::to_string_pretty(&result)?)
    }
}

/// Tool for creating a new character
pub struct CreateCharacterTool {
    db_service: Arc<DatabaseService>,
}

impl CreateCharacterTool {
    pub fn new(db_service: Arc<DatabaseService>) -> Self {
        Self { db_service }
    }
}

#[async_trait]
impl ToolTrait for CreateCharacterTool {
    fn name(&self) -> &str {
        "create_character"
    }

    fn description(&self) -> &str {
        "Create a new D&D 5e character at LEVEL 1 with full rule support.

IMPORTANT: Characters are always created at level 1. For higher levels, use level_up tool after creation.

Usage:
- First use list_players to find the correct player_id
- Provide player_id, character_name, race, race_source, class, class_source, background, background_source
- Provide ability_scores as object with strength, dexterity, constitution, intelligence, wisdom, charisma
- Optionally provide campaign_id, subrace, subclass, alignment, personality traits
- Source is typically 'PHB' for Player's Handbook content
- If campaign_id is not provided, the character is created in the general character pool

For higher level characters:
1. Create the character with create_character (creates at level 1)
2. Use level_up tool with target_level and optional max_hp to reach desired level

Character creation includes:
- Racial traits and ability bonuses applied automatically
- Class features and proficiencies from class/background
- Starting HP calculated from class hit dice + CON modifier (level 1)
- Spell slots calculated for spellcasting classes
- Speed and other racial attributes

Output:
- Created character with database ID
- Character name, level 1, race, and class confirmed
- Use level_up to advance to higher levels"
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "player_id": {
                    "type": "integer",
                    "description": "ID of the player who owns this character"
                },
                "campaign_id": {
                    "type": ["integer", "null"],
                    "description": "Optional campaign ID to associate with this character (omit or null for general character pool)"
                },
                "character_name": {
                    "type": "string",
                    "description": "Name of the character"
                },
                "race": {
                    "type": "string",
                    "description": "Character race (e.g., Human, Elf, Dwarf)"
                },
                "race_source": {
                    "type": "string",
                    "description": "Source book for race (e.g., PHB)"
                },
                "subrace": {
                    "type": ["string", "null"],
                    "description": "Character subrace if applicable"
                },
                "class": {
                    "type": "string",
                    "description": "Character class (e.g., Fighter, Wizard)"
                },
                "class_source": {
                    "type": "string",
                    "description": "Source book for class (e.g., PHB)"
                },
                "subclass": {
                    "type": ["string", "null"],
                    "description": "Character subclass if applicable"
                },
                "background": {
                    "type": "string",
                    "description": "Character background (e.g., Soldier, Sage)"
                },
                "background_source": {
                    "type": "string",
                    "description": "Source book for background (e.g., PHB)"
                },
                "ability_scores": {
                    "type": "object",
                    "description": "Ability scores",
                    "properties": {
                        "strength": { "type": "integer" },
                        "dexterity": { "type": "integer" },
                        "constitution": { "type": "integer" },
                        "intelligence": { "type": "integer" },
                        "wisdom": { "type": "integer" },
                        "charisma": { "type": "integer" }
                    },
                    "required": ["strength", "dexterity", "constitution", "intelligence", "wisdom", "charisma"]
                },
                "alignment": {
                    "type": ["string", "null"],
                    "description": "Character alignment (e.g., Lawful Good)"
                },
                "personality": {
                    "type": ["object", "null"],
                    "description": "Personality traits",
                    "properties": {
                        "traits": { "type": ["string", "null"] },
                        "ideals": { "type": ["string", "null"] },
                        "bonds": { "type": ["string", "null"] },
                        "flaws": { "type": ["string", "null"] }
                    }
                }
            },
            "required": ["player_id", "character_name", "race", "race_source", "class", "class_source", "background", "background_source", "ability_scores"]
        })
    }

    fn requires_confirmation(&self) -> bool {
        true
    }

    fn describe_action(&self, arguments: &Value) -> Option<ActionDescription> {
        let character_name = arguments.get("character_name")?.as_str()?;
        let race = arguments.get("race")?.as_str()?;
        let class = arguments.get("class")?.as_str()?;

        Some(ActionDescription {
            title: "Create Character".to_string(),
            description: format!(
                "Create new character: {} the {} {}",
                character_name, race, class
            ),
            changes: ChangeDetail::Generic {
                items: vec![
                    format!("Name: {}", character_name),
                    format!("Race: {}", race),
                    format!("Class: {}", class),
                ],
            },
        })
    }

    async fn execute(&self, arguments: Value) -> Result<String, Box<dyn Error + Send + Sync>> {
        let player_id = arguments
            .get("player_id")
            .and_then(|v| v.as_i64())
            .ok_or("Missing player_id")? as i32;

        let campaign_id = arguments
            .get("campaign_id")
            .and_then(|v| v.as_i64())
            .map(|v| v as i32);

        let character_name = arguments
            .get("character_name")
            .and_then(|v| v.as_str())
            .ok_or("Missing character_name")?
            .to_string();

        let race = arguments
            .get("race")
            .and_then(|v| v.as_str())
            .ok_or("Missing race")?
            .to_string();

        let race_source = arguments
            .get("race_source")
            .and_then(|v| v.as_str())
            .ok_or("Missing race_source")?
            .to_string();

        let subrace = arguments
            .get("subrace")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let class = arguments
            .get("class")
            .and_then(|v| v.as_str())
            .ok_or("Missing class")?
            .to_string();

        let class_source = arguments
            .get("class_source")
            .and_then(|v| v.as_str())
            .ok_or("Missing class_source")?
            .to_string();

        let subclass = arguments
            .get("subclass")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let background = arguments
            .get("background")
            .and_then(|v| v.as_str())
            .ok_or("Missing background")?
            .to_string();

        let background_source = arguments
            .get("background_source")
            .and_then(|v| v.as_str())
            .ok_or("Missing background_source")?
            .to_string();

        let ability_scores = arguments
            .get("ability_scores")
            .ok_or("Missing ability_scores")?;

        let alignment = arguments
            .get("alignment")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let personality = arguments.get("personality").map(|p| Personality {
            traits: p
                .get("traits")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            ideals: p
                .get("ideals")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            bonds: p
                .get("bonds")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            flaws: p
                .get("flaws")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
        });

        let mut conn = self
            .db_service
            .get_connection()
            .map_err(|e| format!("Database error: {}", e))?;

        // Look up campaign directory if campaign_id is provided
        let base_directory = if let Some(cid) = campaign_id {
            let mut campaign_repo = CampaignRepository::new(&mut conn);
            let campaign = campaign_repo
                .find_by_id(cid)
                .map_err(|e| format!("Failed to find campaign: {}", e))?
                .ok_or_else(|| format!("Campaign with id {} not found", cid))?;
            campaign.directory_path
        } else {
            String::new()
        };

        // Build ability scores
        let scores = AbilityScoreMethod::Manual {
            strength: ability_scores
                .get("strength")
                .and_then(|v| v.as_i64())
                .unwrap_or(10) as i32,
            dexterity: ability_scores
                .get("dexterity")
                .and_then(|v| v.as_i64())
                .unwrap_or(10) as i32,
            constitution: ability_scores
                .get("constitution")
                .and_then(|v| v.as_i64())
                .unwrap_or(10) as i32,
            intelligence: ability_scores
                .get("intelligence")
                .and_then(|v| v.as_i64())
                .unwrap_or(10) as i32,
            wisdom: ability_scores
                .get("wisdom")
                .and_then(|v| v.as_i64())
                .unwrap_or(10) as i32,
            charisma: ability_scores
                .get("charisma")
                .and_then(|v| v.as_i64())
                .unwrap_or(10) as i32,
        };

        // Create character using builder
        let mut builder = CharacterBuilder::new(&mut conn)
            .set_identity(character_name.clone(), Some(player_id))
            .set_race(&race, &race_source, subrace)
            .map_err(|e| format!("Failed to set race: {}", e))?
            .set_class(&class, &class_source, subclass)
            .map_err(|e| format!("Failed to set class: {}", e))?
            .set_ability_scores(scores)
            .map_err(|e| format!("Failed to set ability scores: {}", e))?
            .set_background(&background, &background_source)
            .map_err(|e| format!("Failed to set background: {}", e))?;

        if let Some(align) = alignment {
            builder = builder.set_alignment(align);
        }

        if let Some(pers) = personality {
            builder = builder.set_personality(pers);
        }

        let char_data = builder
            .build()
            .map_err(|e| format!("Failed to create character: {}", e))?;

        // Store the character in the database
        let mut char_service = CharacterService::new(&mut conn);
        let character = char_service
            .create_character(campaign_id, Some(player_id), false, &base_directory, char_data.clone())
            .map_err(|e| format!("Failed to store character: {}", e))?;

        let result = json!({
            "success": true,
            "character_id": character.id,
            "character_name": char_data.character_name,
            "level": char_data.level,
            "race": char_data.race,
            "class": char_data.classes[0].class_name,
            "message": format!("Created {} - Level {} {} {}",
                char_data.character_name,
                char_data.level,
                char_data.race,
                char_data.classes[0].class_name
            )
        });

        debug!(
            "Created character: {} (ID: {})",
            char_data.character_name, character.id
        );
        Ok(serde_json::to_string_pretty(&result)?)
    }
}

/// Tool for creating NPCs (Non-Player Characters)
pub struct CreateNpcTool {
    db_service: Arc<DatabaseService>,
}

impl CreateNpcTool {
    pub fn new(db_service: Arc<DatabaseService>) -> Self {
        Self { db_service }
    }
}

#[async_trait]
impl ToolTrait for CreateNpcTool {
    fn name(&self) -> &str {
        "create_npc"
    }

    fn description(&self) -> &str {
        "Create a new NPC (Non-Player Character) for the campaign.

NPCs are characters controlled by the DM, not player characters. Use this for:
- Important NPCs the party will interact with (shopkeepers, quest givers, allies)
- Named antagonists and villains
- Recurring characters in the story

Usage:
- Provide campaign_id, name, and race (required)
- Optionally provide role (e.g., 'Innkeeper', 'Guard Captain'), location, faction
- For NPCs with combat stats, also provide class and ability_scores
- Use npc_notes for any additional details about the NPC

Output:
- Created NPC with database ID
- NPC name and details confirmed"
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "campaign_id": {
                    "type": "integer",
                    "description": "Campaign ID this NPC belongs to"
                },
                "name": {
                    "type": "string",
                    "description": "Name of the NPC"
                },
                "race": {
                    "type": "string",
                    "description": "NPC race (e.g., Human, Elf, Dwarf)"
                },
                "race_source": {
                    "type": "string",
                    "description": "Source book for race (e.g., PHB). Defaults to PHB."
                },
                "class": {
                    "type": ["string", "null"],
                    "description": "Optional class for NPCs with combat stats"
                },
                "class_source": {
                    "type": ["string", "null"],
                    "description": "Source book for class (e.g., PHB)"
                },
                "role": {
                    "type": ["string", "null"],
                    "description": "NPC role (e.g., 'Innkeeper', 'Guard Captain', 'Merchant')"
                },
                "location": {
                    "type": ["string", "null"],
                    "description": "Where this NPC is typically found"
                },
                "faction": {
                    "type": ["string", "null"],
                    "description": "Faction or organization the NPC belongs to"
                },
                "npc_notes": {
                    "type": ["string", "null"],
                    "description": "Additional notes about the NPC"
                },
                "ability_scores": {
                    "type": ["object", "null"],
                    "description": "Optional ability scores for NPCs with combat stats",
                    "properties": {
                        "strength": { "type": "integer" },
                        "dexterity": { "type": "integer" },
                        "constitution": { "type": "integer" },
                        "intelligence": { "type": "integer" },
                        "wisdom": { "type": "integer" },
                        "charisma": { "type": "integer" }
                    }
                }
            },
            "required": ["campaign_id", "name", "race"]
        })
    }

    fn requires_confirmation(&self) -> bool {
        true
    }

    fn describe_action(&self, arguments: &Value) -> Option<ActionDescription> {
        let name = arguments.get("name")?.as_str()?;
        let race = arguments.get("race")?.as_str()?;
        let role = arguments
            .get("role")
            .and_then(|v| v.as_str())
            .unwrap_or("NPC");

        Some(ActionDescription {
            title: "Create NPC".to_string(),
            description: format!("Create new NPC: {} ({} {})", name, race, role),
            changes: ChangeDetail::Generic {
                items: vec![
                    format!("Name: {}", name),
                    format!("Race: {}", race),
                    format!("Role: {}", role),
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

        let race = arguments
            .get("race")
            .and_then(|v| v.as_str())
            .ok_or("Missing race")?
            .to_string();

        let race_source = arguments
            .get("race_source")
            .and_then(|v| v.as_str())
            .unwrap_or("PHB")
            .to_string();

        let class = arguments
            .get("class")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let class_source = arguments
            .get("class_source")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let role = arguments
            .get("role")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let location = arguments
            .get("location")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let faction = arguments
            .get("faction")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let npc_notes = arguments
            .get("npc_notes")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let ability_scores = arguments.get("ability_scores");

        let mut conn = self
            .db_service
            .get_connection()
            .map_err(|e| format!("Database error: {}", e))?;

        // Get campaign directory
        let mut campaign_repo = CampaignRepository::new(&mut conn);
        let campaign = campaign_repo
            .find_by_id(campaign_id)
            .map_err(|e| format!("Failed to find campaign: {}", e))?
            .ok_or_else(|| format!("Campaign with id {} not found", campaign_id))?;
        let base_directory = campaign.directory_path;

        // Build ability scores (use defaults for non-combat NPCs)
        let scores = if let Some(abs) = ability_scores {
            AbilityScoreMethod::Manual {
                strength: abs.get("strength").and_then(|v| v.as_i64()).unwrap_or(10) as i32,
                dexterity: abs.get("dexterity").and_then(|v| v.as_i64()).unwrap_or(10) as i32,
                constitution: abs.get("constitution").and_then(|v| v.as_i64()).unwrap_or(10) as i32,
                intelligence: abs.get("intelligence").and_then(|v| v.as_i64()).unwrap_or(10) as i32,
                wisdom: abs.get("wisdom").and_then(|v| v.as_i64()).unwrap_or(10) as i32,
                charisma: abs.get("charisma").and_then(|v| v.as_i64()).unwrap_or(10) as i32,
            }
        } else {
            AbilityScoreMethod::Manual {
                strength: 10,
                dexterity: 10,
                constitution: 10,
                intelligence: 10,
                wisdom: 10,
                charisma: 10,
            }
        };

        // Build NPC using builder
        // Use set_race_name_only for NPCs since they may use monster/creature races
        // that aren't in the standard race catalog (e.g., "Goblin", "Yeti")
        let mut builder = CharacterBuilder::new(&mut conn)
            .set_identity(name.clone(), None) // No player_id for NPCs
            .set_race_name_only(&race, &race_source)
            .set_ability_scores(scores)
            .map_err(|e| format!("Failed to set ability scores: {}", e))?;

        // Set class if provided, otherwise use Commoner
        if let Some(ref cls) = class {
            let cls_source = class_source.as_deref().unwrap_or("PHB");
            builder = builder
                .set_class(cls, cls_source, None)
                .map_err(|e| format!("Failed to set class: {}", e))?;
        } else {
            // For non-combat NPCs, we'll set a minimal Commoner-like class
            builder = builder
                .set_class("Fighter", "PHB", None)
                .map_err(|e| format!("Failed to set default class: {}", e))?;
        }

        // Set background (defaults to Acolyte for NPCs - a common NPC background)
        builder = builder
            .set_background("Acolyte", "PHB")
            .map_err(|e| format!("Failed to set background: {}", e))?;

        let mut char_data = builder
            .build()
            .map_err(|e| format!("Failed to create NPC: {}", e))?;

        // Set NPC-specific fields directly on char_data
        char_data.npc_role = role.clone();
        char_data.npc_location = location.clone();
        char_data.npc_faction = faction.clone();
        char_data.npc_notes = npc_notes.clone();

        // Store the NPC in the database (is_npc = true)
        let mut char_service = CharacterService::new(&mut conn);
        let character = char_service
            .create_character(Some(campaign_id), None, true, &base_directory, char_data.clone())
            .map_err(|e| format!("Failed to store NPC: {}", e))?;

        let result = json!({
            "success": true,
            "npc_id": character.id,
            "name": char_data.character_name,
            "race": char_data.race,
            "role": role,
            "location": location,
            "faction": faction,
            "message": format!("Created NPC: {} ({} {})",
                char_data.character_name,
                char_data.race,
                role.as_deref().unwrap_or("NPC")
            )
        });

        debug!(
            "Created NPC: {} (ID: {})",
            char_data.character_name, character.id
        );
        Ok(serde_json::to_string_pretty(&result)?)
    }
}

/// Tool for updating character details
pub struct UpdateCharacterTool {
    db_service: Arc<DatabaseService>,
}

impl UpdateCharacterTool {
    pub fn new(db_service: Arc<DatabaseService>) -> Self {
        Self { db_service }
    }
}

#[async_trait]
impl ToolTrait for UpdateCharacterTool {
    fn name(&self) -> &str {
        "update_character"
    }

    fn description(&self) -> &str {
        "Update character details like name, alignment, or personality traits.

Usage:
- Provide character_id (required)
- Provide any fields to update: character_name, alignment, personality
- Only provided fields are updated; others remain unchanged
- Personality is an object with: traits, ideals, bonds, flaws
- Creates version snapshot for history tracking

When to use:
- Character development or story changes (alignment shift)
- Correcting character information
- Adding or updating personality traits during roleplay
- Player requests to rename character
- Recording character growth from campaign events

CANNOT change (use other tools instead):
- Level/XP → use level_up tool
- HP → use update_character_hp tool
- Inventory → use add_inventory_item / remove_inventory_item
- Spell slots → use cast_spell / take_rest
- Equipment → use update_equipped

Output:
- Confirmation of updated character
- New version snapshot created
- Changes immediately reflected in character sheet"
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "character_id": {
                    "type": "integer",
                    "description": "ID of the character to update"
                },
                "character_name": {
                    "type": ["string", "null"],
                    "description": "New character name"
                },
                "alignment": {
                    "type": ["string", "null"],
                    "description": "New alignment"
                },
                "personality": {
                    "type": ["object", "null"],
                    "description": "Personality traits to update",
                    "properties": {
                        "traits": { "type": ["string", "null"] },
                        "ideals": { "type": ["string", "null"] },
                        "bonds": { "type": ["string", "null"] },
                        "flaws": { "type": ["string", "null"] }
                    }
                }
            },
            "required": ["character_id"]
        })
    }

    fn requires_confirmation(&self) -> bool {
        true
    }

    fn describe_action(&self, arguments: &Value) -> Option<ActionDescription> {
        let character_id = arguments.get("character_id")?.as_i64()?;

        let mut items = vec![format!("Character ID: {}", character_id)];

        if let Some(name) = arguments.get("character_name").and_then(|v| v.as_str()) {
            items.push(format!("New name: {}", name));
        }

        if let Some(alignment) = arguments.get("alignment").and_then(|v| v.as_str()) {
            items.push(format!("New alignment: {}", alignment));
        }

        if arguments.get("personality").is_some() {
            items.push("Personality traits updated".to_string());
        }

        Some(ActionDescription {
            title: "Update Character".to_string(),
            description: format!("Update character {} details", character_id),
            changes: ChangeDetail::Generic { items },
        })
    }

    async fn execute(&self, arguments: Value) -> Result<String, Box<dyn Error + Send + Sync>> {
        let character_id = arguments
            .get("character_id")
            .and_then(|v| v.as_i64())
            .ok_or("Missing character_id")? as i32;

        let mut conn = self
            .db_service
            .get_connection()
            .map_err(|e| format!("Database error: {}", e))?;

        let mut char_service = CharacterService::new(&mut conn);
        let (_character, mut char_data) = char_service
            .get_character(character_id)
            .map_err(|e| format!("Character not found: {}", e))?;

        // Update fields if provided
        if let Some(name) = arguments.get("character_name").and_then(|v| v.as_str()) {
            char_data.character_name = name.to_string();
        }

        if let Some(alignment) = arguments.get("alignment").and_then(|v| v.as_str()) {
            char_data.alignment = Some(alignment.to_string());
        }

        if let Some(personality) = arguments.get("personality") {
            if let Some(traits) = personality.get("traits").and_then(|v| v.as_str()) {
                char_data.personality.traits = Some(traits.to_string());
            }
            if let Some(ideals) = personality.get("ideals").and_then(|v| v.as_str()) {
                char_data.personality.ideals = Some(ideals.to_string());
            }
            if let Some(bonds) = personality.get("bonds").and_then(|v| v.as_str()) {
                char_data.personality.bonds = Some(bonds.to_string());
            }
            if let Some(flaws) = personality.get("flaws").and_then(|v| v.as_str()) {
                char_data.personality.flaws = Some(flaws.to_string());
            }
        }

        char_service
            .update_character(
                character_id,
                char_data.clone(),
                Some("AI-assisted update".to_string()),
            )
            .map_err(|e| format!("Failed to update character: {}", e))?;

        let result = json!({
            "success": true,
            "character_id": character_id,
            "character_name": char_data.character_name,
            "message": format!("Updated character: {}", char_data.character_name)
        });

        debug!(
            "Updated character: {} (ID: {})",
            char_data.character_name, character_id
        );
        Ok(serde_json::to_string_pretty(&result)?)
    }
}

/// Tool for leveling up a character
pub struct LevelUpTool {
    db_service: Arc<DatabaseService>,
}

impl LevelUpTool {
    pub fn new(db_service: Arc<DatabaseService>) -> Self {
        Self { db_service }
    }

    /// Get hit die value for a class
    fn get_hit_die(class_name: &str) -> i32 {
        match class_name.to_lowercase().as_str() {
            "barbarian" => 12,
            "fighter" | "paladin" | "ranger" => 10,
            "bard" | "cleric" | "druid" | "monk" | "rogue" | "warlock" => 8,
            "sorcerer" | "wizard" => 6,
            _ => 8, // Default to d8
        }
    }
}

#[async_trait]
impl ToolTrait for LevelUpTool {
    fn name(&self) -> &str {
        "level_up"
    }

    fn description(&self) -> &str {
        "Level up a character to a target level.

Usage:
- Provide character_id and target_level
- Optionally provide hp_increase_method: 'average' (default) or 'max'
- Optionally provide custom max_hp to override calculated HP
- Creates version snapshot for history tracking

Level up effects:
- Increases character level
- Calculates HP gain based on class hit dice + CON modifier
- Updates hit dice pool
- For multiclass characters, uses primary (first) class hit die

HP calculation per level:
- Average: (hit_die / 2 + 1) + CON modifier per level
- Max: hit_die + CON modifier per level

When to use:
- Character gains enough XP to level up
- Starting a character at higher level
- Advancing NPCs between sessions
- Campaign milestone leveling

Output:
- New level confirmed
- HP before and after
- Hit dice updated"
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "character_id": {
                    "type": "integer",
                    "description": "ID of the character to level up"
                },
                "target_level": {
                    "type": "integer",
                    "description": "Target level (must be higher than current level, max 20)"
                },
                "hp_increase_method": {
                    "type": ["string", "null"],
                    "enum": ["average", "max"],
                    "description": "How to calculate HP gain: 'average' (default) or 'max'"
                },
                "max_hp": {
                    "type": ["integer", "null"],
                    "description": "Override calculated max HP with this value (optional)"
                }
            },
            "required": ["character_id", "target_level"]
        })
    }

    fn requires_confirmation(&self) -> bool {
        true
    }

    fn describe_action(&self, arguments: &Value) -> Option<ActionDescription> {
        let character_id = arguments.get("character_id")?.as_i64()?;
        let target_level = arguments.get("target_level")?.as_i64()?;
        let hp_method = arguments
            .get("hp_increase_method")
            .and_then(|v| v.as_str())
            .unwrap_or("average");

        Some(ActionDescription {
            title: "Level Up Character".to_string(),
            description: format!(
                "Level up character {} to level {} (HP method: {})",
                character_id, target_level, hp_method
            ),
            changes: ChangeDetail::Generic {
                items: vec![
                    format!("Character ID: {}", character_id),
                    format!("Target level: {}", target_level),
                    format!("HP method: {}", hp_method),
                ],
            },
        })
    }

    async fn execute(&self, arguments: Value) -> Result<String, Box<dyn Error + Send + Sync>> {
        let character_id = arguments
            .get("character_id")
            .and_then(|v| v.as_i64())
            .ok_or("Missing character_id")? as i32;

        let target_level = arguments
            .get("target_level")
            .and_then(|v| v.as_i64())
            .ok_or("Missing target_level")? as i32;

        let hp_method = arguments
            .get("hp_increase_method")
            .and_then(|v| v.as_str())
            .unwrap_or("average");

        let custom_max_hp = arguments
            .get("max_hp")
            .and_then(|v| v.as_i64())
            .map(|v| v as i32);

        if target_level < 1 || target_level > 20 {
            return Err("Target level must be between 1 and 20".into());
        }

        let mut conn = self
            .db_service
            .get_connection()
            .map_err(|e| format!("Database error: {}", e))?;

        let mut char_service = CharacterService::new(&mut conn);
        let (_character, mut char_data) = char_service
            .get_character(character_id)
            .map_err(|e| format!("Character not found: {}", e))?;

        let current_level = char_data.level;

        if target_level <= current_level {
            return Err(format!(
                "Target level {} must be higher than current level {}",
                target_level, current_level
            )
            .into());
        }

        let levels_gained = target_level - current_level;
        let old_hp = char_data.max_hp;

        // Get primary class hit die
        let hit_die = char_data
            .classes
            .first()
            .map(|c| Self::get_hit_die(&c.class_name))
            .unwrap_or(8);

        // Calculate CON modifier
        let con_mod = (char_data.abilities.constitution - 10) / 2;

        // Calculate HP gain
        let hp_per_level = match hp_method {
            "max" => hit_die + con_mod,
            _ => (hit_die / 2 + 1) + con_mod, // average
        };

        let hp_gain = hp_per_level * levels_gained;

        // Update character
        char_data.level = target_level;

        // Use custom max_hp if provided, otherwise calculate
        if let Some(custom_hp) = custom_max_hp {
            char_data.max_hp = custom_hp;
        } else {
            char_data.max_hp = old_hp + hp_gain;
        }

        char_data.current_hp = char_data.max_hp; // Full HP on level up

        // Update class level (primary class)
        if let Some(class) = char_data.classes.first_mut() {
            class.level = target_level;
            class.hit_dice_remaining = target_level;
        }

        let snapshot_reason = format!(
            "Leveled up from {} to {} (HP: {} -> {})",
            current_level, target_level, old_hp, char_data.max_hp
        );
        char_service
            .update_character(character_id, char_data.clone(), Some(snapshot_reason))
            .map_err(|e| format!("Failed to update character: {}", e))?;

        let result = json!({
            "success": true,
            "character_id": character_id,
            "character_name": char_data.character_name,
            "old_level": current_level,
            "new_level": target_level,
            "levels_gained": levels_gained,
            "old_hp": old_hp,
            "new_hp": char_data.max_hp,
            "hp_gain": char_data.max_hp - old_hp,
            "message": format!(
                "{} leveled up from {} to {}! HP: {} -> {}",
                char_data.character_name, current_level, target_level, old_hp, char_data.max_hp
            )
        });

        debug!(
            "Leveled up character {}: {} -> {}",
            character_id, current_level, target_level
        );
        Ok(serde_json::to_string_pretty(&result)?)
    }
}

/// Tool for taking a rest (short or long)
pub struct TakeRestTool {
    db_service: Arc<DatabaseService>,
}

impl TakeRestTool {
    pub fn new(db_service: Arc<DatabaseService>) -> Self {
        Self { db_service }
    }
}

#[async_trait]
impl ToolTrait for TakeRestTool {
    fn name(&self) -> &str {
        "take_rest"
    }

    fn description(&self) -> &str {
        "Have a character take a short or long rest to restore resources.

Usage:
- Provide character_id and rest_type ('short' or 'long')
- Creates version snapshot for history tracking

Short rest effects:
- Currently records the rest (hit dice spending can be done manually)
- Certain class features may restore on short rest

Long rest effects:
- Restores all HP to maximum
- Restores all hit dice
- Restores all spell slots to maximum
- Resets daily abilities

When to use:
- After combat encounters when party rests
- End of adventuring day (long rest)
- Mid-dungeon recovery (short rest)
- Before major encounters to ensure full resources
- Tracking passage of time in the campaign

Output:
- HP before and after rest
- Confirmation of rest completion
- All restored resources noted
- Version snapshot created for session history"
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "character_id": {
                    "type": "integer",
                    "description": "ID of the character taking a rest"
                },
                "rest_type": {
                    "type": "string",
                    "enum": ["short", "long"],
                    "description": "Type of rest (short or long)"
                }
            },
            "required": ["character_id", "rest_type"]
        })
    }

    fn requires_confirmation(&self) -> bool {
        true
    }

    fn describe_action(&self, arguments: &Value) -> Option<ActionDescription> {
        let character_id = arguments.get("character_id")?.as_i64()?;
        let rest_type = arguments.get("rest_type")?.as_str()?;

        Some(ActionDescription {
            title: format!(
                "{} Rest",
                if rest_type == "long" { "Long" } else { "Short" }
            ),
            description: format!("Character {} takes a {} rest", character_id, rest_type),
            changes: ChangeDetail::Generic {
                items: vec![
                    format!("Character ID: {}", character_id),
                    format!("Rest type: {}", rest_type),
                ],
            },
        })
    }

    async fn execute(&self, arguments: Value) -> Result<String, Box<dyn Error + Send + Sync>> {
        let character_id = arguments
            .get("character_id")
            .and_then(|v| v.as_i64())
            .ok_or("Missing character_id")? as i32;

        let rest_type_str = arguments
            .get("rest_type")
            .and_then(|v| v.as_str())
            .ok_or("Missing rest_type")?;

        let rest_type = match rest_type_str {
            "short" => RestType::Short,
            "long" => RestType::Long,
            _ => return Err("Invalid rest_type, must be 'short' or 'long'".into()),
        };

        let mut conn = self
            .db_service
            .get_connection()
            .map_err(|e| format!("Database error: {}", e))?;

        let mut char_service = CharacterService::new(&mut conn);
        let (_character, mut char_data) = char_service
            .get_character(character_id)
            .map_err(|e| format!("Character not found: {}", e))?;

        let old_hp = char_data.current_hp;

        match rest_type {
            RestType::Short => {
                // Short rest: can spend hit dice (simplified - just note the rest)
            }
            RestType::Long => {
                // Long rest: restore all HP
                char_data.current_hp = char_data.max_hp;

                // Restore all hit dice
                for class in &mut char_data.classes {
                    class.hit_dice_remaining = class.level;
                }

                // Restore all spell slots
                for slots in char_data.spells.spell_slots.values_mut() {
                    slots.current = slots.max;
                }
            }
        }

        let snapshot_reason = format!(
            "{} rest",
            if rest_type == RestType::Long {
                "Long"
            } else {
                "Short"
            }
        );
        char_service
            .update_character(character_id, char_data.clone(), Some(snapshot_reason))
            .map_err(|e| format!("Failed to update character: {}", e))?;

        let result = json!({
            "success": true,
            "character_id": character_id,
            "character_name": char_data.character_name,
            "rest_type": rest_type_str,
            "hp_before": old_hp,
            "hp_after": char_data.current_hp,
            "message": format!("{} completed {} rest. HP: {}/{}",
                char_data.character_name,
                rest_type_str,
                char_data.current_hp,
                char_data.max_hp
            )
        });

        debug!("Character {} took {} rest", character_id, rest_type_str);
        Ok(serde_json::to_string_pretty(&result)?)
    }
}

/// Tool for updating equipped items (armor, shield, weapons)
pub struct UpdateEquippedTool {
    db_service: Arc<DatabaseService>,
}

impl UpdateEquippedTool {
    pub fn new(db_service: Arc<DatabaseService>) -> Self {
        Self { db_service }
    }
}

#[async_trait]
impl ToolTrait for UpdateEquippedTool {
    fn name(&self) -> &str {
        "update_equipped"
    }

    fn description(&self) -> &str {
        "Update a character's equipped items (armor, shield, weapons).

Usage:
- Provide character_id (required)
- Provide any slots to update: armor, shield, main_hand, off_hand
- Set to null/empty string to unequip a slot
- Creates version snapshot for history tracking

When to use:
- Character equips new armor or weapons
- Swapping weapons during combat
- Unequipping items before rest
- Setting up character loadout

Output:
- Updated equipped items confirmed
- Character version snapshot created"
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "character_id": {
                    "type": "integer",
                    "description": "ID of the character"
                },
                "armor": {
                    "type": ["string", "null"],
                    "description": "Armor to equip (null to unequip)"
                },
                "shield": {
                    "type": ["string", "null"],
                    "description": "Shield to equip (null to unequip)"
                },
                "main_hand": {
                    "type": ["string", "null"],
                    "description": "Main hand weapon (null to unequip)"
                },
                "off_hand": {
                    "type": ["string", "null"],
                    "description": "Off hand item/weapon (null to unequip)"
                }
            },
            "required": ["character_id"]
        })
    }

    fn requires_confirmation(&self) -> bool {
        true
    }

    fn describe_action(&self, arguments: &Value) -> Option<ActionDescription> {
        let character_id = arguments.get("character_id")?.as_i64()?;

        let mut items = vec![format!("Character ID: {}", character_id)];

        if let Some(armor) = arguments.get("armor") {
            let armor_str = armor.as_str().unwrap_or("(unequip)");
            items.push(format!("Armor: {}", if armor_str.is_empty() { "(unequip)" } else { armor_str }));
        }
        if let Some(shield) = arguments.get("shield") {
            let shield_str = shield.as_str().unwrap_or("(unequip)");
            items.push(format!("Shield: {}", if shield_str.is_empty() { "(unequip)" } else { shield_str }));
        }
        if let Some(main_hand) = arguments.get("main_hand") {
            let mh_str = main_hand.as_str().unwrap_or("(unequip)");
            items.push(format!("Main hand: {}", if mh_str.is_empty() { "(unequip)" } else { mh_str }));
        }
        if let Some(off_hand) = arguments.get("off_hand") {
            let oh_str = off_hand.as_str().unwrap_or("(unequip)");
            items.push(format!("Off hand: {}", if oh_str.is_empty() { "(unequip)" } else { oh_str }));
        }

        Some(ActionDescription {
            title: "Update Equipped Items".to_string(),
            description: format!("Update equipped items for character {}", character_id),
            changes: ChangeDetail::Generic { items },
        })
    }

    async fn execute(&self, arguments: Value) -> Result<String, Box<dyn Error + Send + Sync>> {
        let character_id = arguments
            .get("character_id")
            .and_then(|v| v.as_i64())
            .ok_or("Missing character_id")? as i32;

        let mut conn = self
            .db_service
            .get_connection()
            .map_err(|e| format!("Database error: {}", e))?;

        let mut char_service = CharacterService::new(&mut conn);
        let (_character, mut char_data) = char_service
            .get_character(character_id)
            .map_err(|e| format!("Character not found: {}", e))?;

        let mut changes = Vec::new();

        // Update armor if provided
        if arguments.get("armor").is_some() {
            let armor = arguments
                .get("armor")
                .and_then(|v| v.as_str())
                .filter(|s| !s.is_empty())
                .map(|s| s.to_string());
            if char_data.equipped.armor != armor {
                changes.push(format!("armor: {:?} -> {:?}", char_data.equipped.armor, armor));
                char_data.equipped.armor = armor;
            }
        }

        // Update shield if provided
        if arguments.get("shield").is_some() {
            let shield = arguments
                .get("shield")
                .and_then(|v| v.as_str())
                .filter(|s| !s.is_empty())
                .map(|s| s.to_string());
            if char_data.equipped.shield != shield {
                changes.push(format!("shield: {:?} -> {:?}", char_data.equipped.shield, shield));
                char_data.equipped.shield = shield;
            }
        }

        // Update main_hand if provided
        if arguments.get("main_hand").is_some() {
            let main_hand = arguments
                .get("main_hand")
                .and_then(|v| v.as_str())
                .filter(|s| !s.is_empty())
                .map(|s| s.to_string());
            if char_data.equipped.main_hand != main_hand {
                changes.push(format!("main_hand: {:?} -> {:?}", char_data.equipped.main_hand, main_hand));
                char_data.equipped.main_hand = main_hand;
            }
        }

        // Update off_hand if provided
        if arguments.get("off_hand").is_some() {
            let off_hand = arguments
                .get("off_hand")
                .and_then(|v| v.as_str())
                .filter(|s| !s.is_empty())
                .map(|s| s.to_string());
            if char_data.equipped.off_hand != off_hand {
                changes.push(format!("off_hand: {:?} -> {:?}", char_data.equipped.off_hand, off_hand));
                char_data.equipped.off_hand = off_hand;
            }
        }

        if changes.is_empty() {
            return Ok(json!({
                "success": true,
                "message": "No changes made"
            }).to_string());
        }

        let snapshot_reason = format!("Equipment updated: {}", changes.join(", "));
        char_service
            .update_character(character_id, char_data.clone(), Some(snapshot_reason))
            .map_err(|e| format!("Failed to update character: {}", e))?;

        let result = json!({
            "success": true,
            "character_id": character_id,
            "character_name": char_data.character_name,
            "equipped": {
                "armor": char_data.equipped.armor,
                "shield": char_data.equipped.shield,
                "main_hand": char_data.equipped.main_hand,
                "off_hand": char_data.equipped.off_hand
            },
            "message": format!("Updated equipment for {}", char_data.character_name)
        });

        debug!("Updated equipment for character {}", character_id);
        Ok(serde_json::to_string_pretty(&result)?)
    }
}

/// Tool for removing items from inventory
pub struct RemoveInventoryItemTool {
    db_service: Arc<DatabaseService>,
}

impl RemoveInventoryItemTool {
    pub fn new(db_service: Arc<DatabaseService>) -> Self {
        Self { db_service }
    }
}

#[async_trait]
impl ToolTrait for RemoveInventoryItemTool {
    fn name(&self) -> &str {
        "remove_inventory_item"
    }

    fn description(&self) -> &str {
        "Remove an item from a character's inventory.

Usage:
- Provide character_id and item_name
- Optionally provide quantity (default: removes all)
- Creates version snapshot for history tracking

When to use:
- Character sells or trades items
- Items are consumed or destroyed
- Correcting inventory mistakes
- Dropping items during gameplay

Output:
- Confirmation of removed items
- Updated inventory state"
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "character_id": {
                    "type": "integer",
                    "description": "ID of the character"
                },
                "item_name": {
                    "type": "string",
                    "description": "Name of the item to remove"
                },
                "quantity": {
                    "type": ["integer", "null"],
                    "description": "Quantity to remove (default: all)"
                }
            },
            "required": ["character_id", "item_name"]
        })
    }

    fn requires_confirmation(&self) -> bool {
        true
    }

    fn describe_action(&self, arguments: &Value) -> Option<ActionDescription> {
        let character_id = arguments.get("character_id")?.as_i64()?;
        let item_name = arguments.get("item_name")?.as_str()?;
        let quantity = arguments.get("quantity").and_then(|v| v.as_i64());

        let qty_str = quantity
            .map(|q| q.to_string())
            .unwrap_or_else(|| "all".to_string());

        Some(ActionDescription {
            title: "Remove Inventory Item".to_string(),
            description: format!("Remove {} × {} from character {}", qty_str, item_name, character_id),
            changes: ChangeDetail::Generic {
                items: vec![
                    format!("Character ID: {}", character_id),
                    format!("Item: {}", item_name),
                    format!("Quantity: {}", qty_str),
                ],
            },
        })
    }

    async fn execute(&self, arguments: Value) -> Result<String, Box<dyn Error + Send + Sync>> {
        let character_id = arguments
            .get("character_id")
            .and_then(|v| v.as_i64())
            .ok_or("Missing character_id")? as i32;

        let item_name = arguments
            .get("item_name")
            .and_then(|v| v.as_str())
            .ok_or("Missing item_name")?;

        let quantity = arguments
            .get("quantity")
            .and_then(|v| v.as_i64())
            .map(|q| q as i32);

        let mut conn = self
            .db_service
            .get_connection()
            .map_err(|e| format!("Database error: {}", e))?;

        let mut char_service = CharacterService::new(&mut conn);
        let (_character, mut char_data) = char_service
            .get_character(character_id)
            .map_err(|e| format!("Character not found: {}", e))?;

        // Find and remove the item
        let item_index = char_data
            .inventory
            .iter()
            .position(|i| i.name.eq_ignore_ascii_case(item_name));

        let removed_qty = match item_index {
            Some(idx) => {
                let item = &mut char_data.inventory[idx];
                let to_remove = quantity.unwrap_or(item.quantity);

                if to_remove >= item.quantity {
                    let removed = item.quantity;
                    char_data.inventory.remove(idx);
                    removed
                } else {
                    item.quantity -= to_remove;
                    to_remove
                }
            }
            None => {
                return Err(format!("Item '{}' not found in inventory", item_name).into());
            }
        };

        let snapshot_reason = format!("Removed {} × {} from inventory", removed_qty, item_name);
        char_service
            .update_character(character_id, char_data.clone(), Some(snapshot_reason))
            .map_err(|e| format!("Failed to update character: {}", e))?;

        let result = json!({
            "success": true,
            "character_id": character_id,
            "character_name": char_data.character_name,
            "item_removed": item_name,
            "quantity_removed": removed_qty,
            "message": format!("Removed {} × {} from {}'s inventory", removed_qty, item_name, char_data.character_name)
        });

        debug!("Removed {} × {} from character {}", removed_qty, item_name, character_id);
        Ok(serde_json::to_string_pretty(&result)?)
    }
}

/// Tool for updating character currency
pub struct UpdateCurrencyTool {
    db_service: Arc<DatabaseService>,
}

impl UpdateCurrencyTool {
    pub fn new(db_service: Arc<DatabaseService>) -> Self {
        Self { db_service }
    }
}

#[async_trait]
impl ToolTrait for UpdateCurrencyTool {
    fn name(&self) -> &str {
        "update_currency"
    }

    fn description(&self) -> &str {
        "Update a character's currency (gold, silver, copper, electrum, platinum).

Usage:
- Provide character_id (required)
- Provide currency changes as positive (add) or negative (remove) values
- Only provided currencies are modified; others remain unchanged
- Creates version snapshot for history tracking

When to use:
- After selling items or receiving payment
- Purchasing equipment or services
- Splitting treasure among party
- Tracking expenses during downtime

Output:
- Updated currency amounts
- Character version snapshot created"
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "character_id": {
                    "type": "integer",
                    "description": "ID of the character"
                },
                "copper": {
                    "type": ["integer", "null"],
                    "description": "Copper to add (positive) or remove (negative)"
                },
                "silver": {
                    "type": ["integer", "null"],
                    "description": "Silver to add (positive) or remove (negative)"
                },
                "electrum": {
                    "type": ["integer", "null"],
                    "description": "Electrum to add (positive) or remove (negative)"
                },
                "gold": {
                    "type": ["integer", "null"],
                    "description": "Gold to add (positive) or remove (negative)"
                },
                "platinum": {
                    "type": ["integer", "null"],
                    "description": "Platinum to add (positive) or remove (negative)"
                },
                "reason": {
                    "type": ["string", "null"],
                    "description": "Reason for currency change"
                }
            },
            "required": ["character_id"]
        })
    }

    fn requires_confirmation(&self) -> bool {
        true
    }

    fn describe_action(&self, arguments: &Value) -> Option<ActionDescription> {
        let character_id = arguments.get("character_id")?.as_i64()?;

        let mut items = vec![format!("Character ID: {}", character_id)];

        if let Some(cp) = arguments.get("copper").and_then(|v| v.as_i64()) {
            items.push(format!("Copper: {:+}", cp));
        }
        if let Some(sp) = arguments.get("silver").and_then(|v| v.as_i64()) {
            items.push(format!("Silver: {:+}", sp));
        }
        if let Some(ep) = arguments.get("electrum").and_then(|v| v.as_i64()) {
            items.push(format!("Electrum: {:+}", ep));
        }
        if let Some(gp) = arguments.get("gold").and_then(|v| v.as_i64()) {
            items.push(format!("Gold: {:+}", gp));
        }
        if let Some(pp) = arguments.get("platinum").and_then(|v| v.as_i64()) {
            items.push(format!("Platinum: {:+}", pp));
        }

        Some(ActionDescription {
            title: "Update Currency".to_string(),
            description: format!("Update currency for character {}", character_id),
            changes: ChangeDetail::Generic { items },
        })
    }

    async fn execute(&self, arguments: Value) -> Result<String, Box<dyn Error + Send + Sync>> {
        let character_id = arguments
            .get("character_id")
            .and_then(|v| v.as_i64())
            .ok_or("Missing character_id")? as i32;

        let reason = arguments
            .get("reason")
            .and_then(|v| v.as_str())
            .unwrap_or("Currency updated");

        let mut conn = self
            .db_service
            .get_connection()
            .map_err(|e| format!("Database error: {}", e))?;

        let mut char_service = CharacterService::new(&mut conn);
        let (_character, mut char_data) = char_service
            .get_character(character_id)
            .map_err(|e| format!("Character not found: {}", e))?;

        let mut changes = Vec::new();

        // Update copper
        if let Some(cp) = arguments.get("copper").and_then(|v| v.as_i64()) {
            let new_val = (char_data.currency.copper + cp as i32).max(0);
            changes.push(format!("copper: {} -> {}", char_data.currency.copper, new_val));
            char_data.currency.copper = new_val;
        }

        // Update silver
        if let Some(sp) = arguments.get("silver").and_then(|v| v.as_i64()) {
            let new_val = (char_data.currency.silver + sp as i32).max(0);
            changes.push(format!("silver: {} -> {}", char_data.currency.silver, new_val));
            char_data.currency.silver = new_val;
        }

        // Update electrum
        if let Some(ep) = arguments.get("electrum").and_then(|v| v.as_i64()) {
            let new_val = (char_data.currency.electrum + ep as i32).max(0);
            changes.push(format!("electrum: {} -> {}", char_data.currency.electrum, new_val));
            char_data.currency.electrum = new_val;
        }

        // Update gold
        if let Some(gp) = arguments.get("gold").and_then(|v| v.as_i64()) {
            let new_val = (char_data.currency.gold + gp as i32).max(0);
            changes.push(format!("gold: {} -> {}", char_data.currency.gold, new_val));
            char_data.currency.gold = new_val;
        }

        // Update platinum
        if let Some(pp) = arguments.get("platinum").and_then(|v| v.as_i64()) {
            let new_val = (char_data.currency.platinum + pp as i32).max(0);
            changes.push(format!("platinum: {} -> {}", char_data.currency.platinum, new_val));
            char_data.currency.platinum = new_val;
        }

        if changes.is_empty() {
            return Ok(json!({
                "success": true,
                "message": "No currency changes specified"
            }).to_string());
        }

        let snapshot_reason = format!("{}: {}", reason, changes.join(", "));
        char_service
            .update_character(character_id, char_data.clone(), Some(snapshot_reason))
            .map_err(|e| format!("Failed to update character: {}", e))?;

        let result = json!({
            "success": true,
            "character_id": character_id,
            "character_name": char_data.character_name,
            "currency": {
                "copper": char_data.currency.copper,
                "silver": char_data.currency.silver,
                "electrum": char_data.currency.electrum,
                "gold": char_data.currency.gold,
                "platinum": char_data.currency.platinum
            },
            "message": format!("Updated currency for {}", char_data.character_name)
        });

        debug!("Updated currency for character {}", character_id);
        Ok(serde_json::to_string_pretty(&result)?)
    }
}
