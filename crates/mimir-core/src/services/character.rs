//! Character Service
//!
//! Business logic for character management (PCs and NPCs).

use chrono::Utc;
use diesel::SqliteConnection;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::dal::campaign as dal;
use crate::models::campaign::{
    Character, CharacterClass, CharacterInventory, FeatSourceType, NewCharacter,
    NewCharacterClass, NewCharacterFeat, NewCharacterFeature, NewCharacterInventory,
    NewCharacterProficiency, NewCharacterSpell, ProficiencyType, UpdateCharacter,
    UpdateCharacterClass, UpdateCharacterInventory, UpdateCharacterProficiency,
};
use crate::services::catalog::CatalogEntityService;
use crate::services::{ClassService, ServiceError, ServiceResult};

/// Input for creating a new character.
#[derive(Debug, Clone)]
pub struct CreateCharacterInput {
    /// Campaign this character belongs to
    pub campaign_id: String,
    /// Character name
    pub name: String,
    /// Whether this is an NPC
    pub is_npc: bool,
    /// Player name (for PCs)
    pub player_name: Option<String>,
    /// Race name (e.g., "Elf")
    pub race_name: Option<String>,
    /// Race source (e.g., "PHB")
    pub race_source: Option<String>,
    /// Background name (e.g., "Acolyte")
    pub background_name: Option<String>,
    /// Background source (e.g., "PHB")
    pub background_source: Option<String>,
    /// Ability scores [STR, DEX, CON, INT, WIS, CHA]
    pub ability_scores: Option<[i32; 6]>,
}

impl CreateCharacterInput {
    /// Create input for a new player character.
    pub fn new_pc(
        campaign_id: impl Into<String>,
        name: impl Into<String>,
        player_name: impl Into<String>,
    ) -> Self {
        Self {
            campaign_id: campaign_id.into(),
            name: name.into(),
            is_npc: false,
            player_name: Some(player_name.into()),
            race_name: None,
            race_source: None,
            background_name: None,
            background_source: None,
            ability_scores: None,
        }
    }

    /// Create input for a new NPC.
    pub fn new_npc(campaign_id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            campaign_id: campaign_id.into(),
            name: name.into(),
            is_npc: true,
            player_name: None,
            race_name: None,
            race_source: None,
            background_name: None,
            background_source: None,
            ability_scores: None,
        }
    }

    /// Set the race.
    pub fn with_race(mut self, name: impl Into<String>, source: impl Into<String>) -> Self {
        self.race_name = Some(name.into());
        self.race_source = Some(source.into());
        self
    }

    /// Set the background.
    pub fn with_background(mut self, name: impl Into<String>, source: impl Into<String>) -> Self {
        self.background_name = Some(name.into());
        self.background_source = Some(source.into());
        self
    }

    /// Set ability scores [STR, DEX, CON, INT, WIS, CHA].
    pub fn with_ability_scores(mut self, scores: [i32; 6]) -> Self {
        self.ability_scores = Some(scores);
        self
    }
}

/// Input for updating a character.
#[derive(Debug, Clone, Default)]
pub struct UpdateCharacterInput {
    pub name: Option<String>,
    pub player_name: Option<Option<String>>,
    pub race_name: Option<Option<String>>,
    pub race_source: Option<Option<String>>,
    pub background_name: Option<Option<String>>,
    pub background_source: Option<Option<String>>,
    pub ability_scores: Option<[i32; 6]>,
    pub currency: Option<[i32; 5]>,
    pub traits: Option<Option<String>>,
    pub ideals: Option<Option<String>>,
    pub bonds: Option<Option<String>>,
    pub flaws: Option<Option<String>>,
    pub role: Option<Option<String>>,
    pub location: Option<Option<String>>,
    pub faction: Option<Option<String>>,
}

impl UpdateCharacterInput {
    /// Update character name.
    pub fn set_name(name: impl Into<String>) -> Self {
        Self {
            name: Some(name.into()),
            ..Default::default()
        }
    }

    /// Update race.
    pub fn set_race(name: Option<String>, source: Option<String>) -> Self {
        Self {
            race_name: Some(name),
            race_source: Some(source),
            ..Default::default()
        }
    }

    /// Update background.
    pub fn set_background(name: Option<String>, source: Option<String>) -> Self {
        Self {
            background_name: Some(name),
            background_source: Some(source),
            ..Default::default()
        }
    }

    /// Update ability scores [STR, DEX, CON, INT, WIS, CHA].
    pub fn set_ability_scores(scores: [i32; 6]) -> Self {
        Self {
            ability_scores: Some(scores),
            ..Default::default()
        }
    }

    /// Update currency [CP, SP, EP, GP, PP].
    pub fn set_currency(currency: [i32; 5]) -> Self {
        Self {
            currency: Some(currency),
            ..Default::default()
        }
    }

    /// Update roleplay elements.
    pub fn set_roleplay(
        traits: Option<String>,
        ideals: Option<String>,
        bonds: Option<String>,
        flaws: Option<String>,
    ) -> Self {
        Self {
            traits: Some(traits),
            ideals: Some(ideals),
            bonds: Some(bonds),
            flaws: Some(flaws),
            ..Default::default()
        }
    }

    /// Update NPC info.
    pub fn set_npc_info(
        role: Option<String>,
        location: Option<String>,
        faction: Option<String>,
    ) -> Self {
        Self {
            role: Some(role),
            location: Some(location),
            faction: Some(faction),
            ..Default::default()
        }
    }
}

/// Input for adding an item to inventory.
#[derive(Debug, Clone)]
pub struct AddInventoryInput {
    /// Item name from catalog
    pub item_name: String,
    /// Item source (e.g., "PHB")
    pub item_source: String,
    /// Quantity (default 1)
    pub quantity: Option<i32>,
    /// Whether equipped
    pub equipped: bool,
    /// Whether attuned
    pub attuned: bool,
    /// Notes about the item
    pub notes: Option<String>,
}

impl AddInventoryInput {
    /// Create input for adding an item.
    pub fn new(name: impl Into<String>, source: impl Into<String>) -> Self {
        Self {
            item_name: name.into(),
            item_source: source.into(),
            quantity: None,
            equipped: false,
            attuned: false,
            notes: None,
        }
    }

    /// Set quantity.
    pub fn with_quantity(mut self, quantity: i32) -> Self {
        self.quantity = Some(quantity);
        self
    }

    /// Mark as equipped.
    pub fn equipped(mut self) -> Self {
        self.equipped = true;
        self
    }

    /// Mark as attuned.
    pub fn attuned(mut self) -> Self {
        self.attuned = true;
        self
    }

    /// Add notes.
    pub fn with_notes(mut self, notes: impl Into<String>) -> Self {
        self.notes = Some(notes.into());
        self
    }
}

// =============================================================================
// Level Up Types
// =============================================================================

/// Request for leveling up a character.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LevelUpRequest {
    /// Class to level up in (allows multiclassing)
    pub class_name: String,
    /// Source book for the class (e.g., "PHB", "XGE")
    pub class_source: String,
    /// HP gain method
    pub hit_points_method: HpGainMethod,
    /// Subclass choice (if this is the level where subclass is chosen)
    pub subclass: Option<SubclassChoice>,
    /// Ability score improvement or feat selection (if applicable at this level)
    pub asi_or_feat: Option<AsiOrFeat>,
    /// Spell changes (new spells, cantrips, swaps) for spellcasters
    pub spell_changes: Option<SpellChanges>,
    /// Feature choices (fighting style, metamagic, maneuvers, invocations, etc.)
    pub feature_choices: Option<FeatureChoices>,
}

/// Spell changes during level up.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpellChanges {
    /// New spells learned this level (for Spells Known casters or Wizard spellbook)
    pub new_spells: Vec<SpellReference>,
    /// New cantrips learned this level
    pub new_cantrips: Vec<SpellReference>,
    /// Spell to remove (for Spells Known swap)
    pub swap_out: Option<SpellReference>,
    /// Spell to add in place of swapped spell
    pub swap_in: Option<SpellReference>,
}

/// Reference to a spell from the catalog.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpellReference {
    /// Spell name (e.g., "Fireball")
    pub name: String,
    /// Spell source (e.g., "PHB", "XGE")
    pub source: String,
}

/// Class feature choices during level up.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureChoices {
    /// Fighting Style selection (Fighter 1, Paladin 2, Ranger 2)
    pub fighting_style: Option<FeatureReference>,
    /// Metamagic options (Sorcerer 3, 10, 17)
    pub metamagic: Option<Vec<FeatureReference>>,
    /// Battle Master maneuvers (Fighter/Battle Master 3, 7, 10, 15)
    pub maneuvers: Option<ManeuverChoices>,
    /// Warlock Eldritch Invocations (Warlock 2, 5, 7, 9, 12, 15, 18)
    pub invocations: Option<InvocationChoices>,
    /// Warlock Pact Boon (Warlock 3)
    pub pact_boon: Option<FeatureReference>,
    /// Expertise skills (Rogue 1/6, Bard 3/10)
    pub expertise_skills: Option<Vec<String>>,
}

/// Reference to a class feature option from the catalog.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureReference {
    /// Feature name (e.g., "Defense", "Quickened Spell", "Riposte")
    pub name: String,
    /// Feature source (e.g., "PHB", "TCE")
    pub source: String,
}

/// Maneuver choices with optional swap for Battle Master.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManeuverChoices {
    /// New maneuvers to learn
    pub new_maneuvers: Vec<FeatureReference>,
    /// Maneuver to swap out (optional, one per level)
    pub swap_out: Option<FeatureReference>,
    /// Maneuver to swap in (required if swap_out is provided)
    pub swap_in: Option<FeatureReference>,
}

/// Invocation choices with optional swap for Warlock.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvocationChoices {
    /// New invocations to learn
    pub new_invocations: Vec<FeatureReference>,
    /// Invocation to swap out (optional, one per level)
    pub swap_out: Option<FeatureReference>,
    /// Invocation to swap in (required if swap_out is provided)
    pub swap_in: Option<FeatureReference>,
}

/// Method for gaining HP on level up.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum HpGainMethod {
    /// Take the average (rounded up): (hit_die / 2) + 1
    Average,
    /// Roll the hit die (value is the roll result, must be 1-hit_die)
    Roll(i32),
    /// Manual HP entry (any positive value)
    Manual(i32),
}

/// Ability Score Improvement or Feat selection.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum AsiOrFeat {
    /// Improve ability scores (total increase must be 2)
    AbilityScoreImprovement {
        /// First ability to increase
        ability1: String,
        /// Amount to increase first ability (1 or 2)
        increase1: i32,
        /// Optional second ability to increase
        ability2: Option<String>,
        /// Amount to increase second ability (1)
        increase2: Option<i32>,
    },
    /// Take a feat instead of ASI
    Feat {
        /// Feat name
        name: String,
        /// Feat source (e.g., "PHB")
        source: String,
    },
}

/// Subclass choice for level up.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubclassChoice {
    /// Subclass name (e.g., "Champion", "School of Evocation")
    pub name: String,
    /// Subclass source (e.g., "PHB", "XGE")
    pub source: String,
}

/// Response from level up operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LevelUpResult {
    /// Updated character data
    pub character: Character,
    /// Updated class entry
    pub class: CharacterClass,
    /// HP gained this level
    pub hp_gained: i32,
    /// New total level
    pub new_total_level: i32,
    /// Whether this was a multiclass (new class added)
    pub is_multiclass: bool,
}

// =============================================================================
// Multiclass Prerequisites
// =============================================================================

/// Multiclass prerequisites for D&D 5e classes.
/// Returns required ability scores as (ability_name, minimum_score) pairs.
/// For OR requirements (like Fighter), any one of the abilities meeting the requirement suffices.
fn get_multiclass_prerequisites(class_name: &str) -> Option<Vec<(&'static str, i32, bool)>> {
    // Returns: Vec<(ability, min_score, is_or_requirement)>
    // is_or_requirement = true means this is part of an OR group (only one needs to pass)
    match class_name.to_lowercase().as_str() {
        "barbarian" => Some(vec![("strength", 13, false)]),
        "bard" => Some(vec![("charisma", 13, false)]),
        "cleric" => Some(vec![("wisdom", 13, false)]),
        "druid" => Some(vec![("wisdom", 13, false)]),
        "fighter" => Some(vec![("strength", 13, true), ("dexterity", 13, true)]), // STR OR DEX
        "monk" => Some(vec![("dexterity", 13, false), ("wisdom", 13, false)]),    // DEX AND WIS
        "paladin" => Some(vec![("strength", 13, false), ("charisma", 13, false)]), // STR AND CHA
        "ranger" => Some(vec![("dexterity", 13, false), ("wisdom", 13, false)]),  // DEX AND WIS
        "rogue" => Some(vec![("dexterity", 13, false)]),
        "sorcerer" => Some(vec![("charisma", 13, false)]),
        "warlock" => Some(vec![("charisma", 13, false)]),
        "wizard" => Some(vec![("intelligence", 13, false)]),
        // Artificer from Tasha's
        "artificer" => Some(vec![("intelligence", 13, false)]),
        // Blood Hunter from Critical Role
        "blood hunter" => Some(vec![("strength", 13, true), ("dexterity", 13, true), ("intelligence", 13, false)]),
        _ => None, // Unknown class - allow without prerequisite check
    }
}

/// Check if a character meets multiclass prerequisites for a class.
fn check_multiclass_prerequisites(
    character: &Character,
    class_name: &str,
) -> Result<(), ServiceError> {
    let prereqs = match get_multiclass_prerequisites(class_name) {
        Some(p) => p,
        None => return Ok(()), // Unknown class - skip check
    };

    // Separate AND requirements from OR requirements
    let and_reqs: Vec<_> = prereqs.iter().filter(|(_, _, is_or)| !is_or).collect();
    let or_reqs: Vec<_> = prereqs.iter().filter(|(_, _, is_or)| *is_or).collect();

    // Check AND requirements - all must pass
    for (ability, min_score, _) in and_reqs {
        let score = get_ability_score(character, ability);
        if score < *min_score {
            return Err(ServiceError::validation(format!(
                "Multiclass prerequisite not met: {} requires {} {} (character has {})",
                class_name, ability, min_score, score
            )));
        }
    }

    // Check OR requirements - at least one must pass
    if !or_reqs.is_empty() {
        let any_pass = or_reqs.iter().any(|(ability, min_score, _)| {
            get_ability_score(character, ability) >= *min_score
        });
        if !any_pass {
            let reqs_str = or_reqs
                .iter()
                .map(|(a, s, _)| format!("{} {}", a, s))
                .collect::<Vec<_>>()
                .join(" or ");
            return Err(ServiceError::validation(format!(
                "Multiclass prerequisite not met: {} requires {}",
                class_name, reqs_str
            )));
        }
    }

    Ok(())
}

/// Get an ability score by name.
fn get_ability_score(character: &Character, ability: &str) -> i32 {
    match ability.to_lowercase().as_str() {
        "strength" | "str" => character.strength,
        "dexterity" | "dex" => character.dexterity,
        "constitution" | "con" => character.constitution,
        "intelligence" | "int" => character.intelligence,
        "wisdom" | "wis" => character.wisdom,
        "charisma" | "cha" => character.charisma,
        _ => 0,
    }
}

/// Set an ability score by name, returning the new scores array.
fn set_ability_score(character: &Character, ability: &str, new_value: i32) -> [i32; 6] {
    let mut scores = [
        character.strength,
        character.dexterity,
        character.constitution,
        character.intelligence,
        character.wisdom,
        character.charisma,
    ];
    match ability.to_lowercase().as_str() {
        "strength" | "str" => scores[0] = new_value,
        "dexterity" | "dex" => scores[1] = new_value,
        "constitution" | "con" => scores[2] = new_value,
        "intelligence" | "int" => scores[3] = new_value,
        "wisdom" | "wis" => scores[4] = new_value,
        "charisma" | "cha" => scores[5] = new_value,
        _ => {}
    }
    scores
}

/// Calculate HP gain for a level up.
fn calculate_hp_gain(method: &HpGainMethod, hit_die: i32, con_mod: i32) -> i32 {
    let base = match method {
        HpGainMethod::Average => (hit_die / 2) + 1,
        HpGainMethod::Roll(roll) => *roll,
        HpGainMethod::Manual(value) => *value,
    };
    // Minimum 1 HP per level even with negative CON
    (base + con_mod).max(1)
}

/// Get hit die value for a class from catalog, returns d8 as default.
fn get_class_hit_die(conn: &mut SqliteConnection, class_name: &str, class_source: &str) -> i32 {
    // Try to get from catalog
    if let Ok(Some(class)) = ClassService::new(conn).get_by_name_and_source(class_name, class_source) {
        // Parse hit die from JSON data
        if let Ok(data) = class.parse_data() {
            if let Some(hd) = data.get("hd") {
                if let Some(faces) = hd.get("faces").and_then(|f| f.as_i64()) {
                    return faces as i32;
                }
            }
        }
    }
    // Default hit die values by class name
    match class_name.to_lowercase().as_str() {
        "barbarian" => 12,
        "fighter" | "paladin" | "ranger" => 10,
        "sorcerer" | "wizard" => 6,
        _ => 8, // Default for most classes
    }
}

/// Service for character management.
///
/// Handles character CRUD operations and inventory management.
pub struct CharacterService<'a> {
    conn: &'a mut SqliteConnection,
}

impl<'a> CharacterService<'a> {
    /// Create a new character service.
    pub fn new(conn: &'a mut SqliteConnection) -> Self {
        Self { conn }
    }

    /// Create a new character.
    pub fn create(&mut self, input: CreateCharacterInput) -> ServiceResult<Character> {
        let char_id = Uuid::new_v4().to_string();

        let race_name = input.race_name.as_deref();
        let race_source = input.race_source.as_deref();
        let background_name = input.background_name.as_deref();
        let background_source = input.background_source.as_deref();
        let player_name = input.player_name.as_deref();

        let mut new_char = if input.is_npc {
            NewCharacter::new_npc(&char_id, &input.campaign_id, &input.name)
        } else {
            NewCharacter::new_pc(
                &char_id,
                &input.campaign_id,
                &input.name,
                player_name.unwrap_or(""),
            )
        };

        // Set race if provided
        if let (Some(name), Some(source)) = (race_name, race_source) {
            new_char = new_char.with_race(name, source);
        }

        // Set background if provided
        if let (Some(name), Some(source)) = (background_name, background_source) {
            new_char = new_char.with_background(name, source);
        }

        // Set ability scores if provided
        if let Some([str, dex, con, int, wis, cha]) = input.ability_scores {
            new_char = new_char.with_ability_scores(str, dex, con, int, wis, cha);
        }

        dal::insert_character(self.conn, &new_char)?;
        dal::get_character(self.conn, &char_id).map_err(ServiceError::from)
    }

    /// List all characters for a campaign.
    pub fn list_for_campaign(&mut self, campaign_id: &str) -> ServiceResult<Vec<Character>> {
        dal::list_campaign_characters(self.conn, campaign_id).map_err(ServiceError::from)
    }

    /// List only player characters for a campaign.
    pub fn list_pcs(&mut self, campaign_id: &str) -> ServiceResult<Vec<Character>> {
        dal::list_pcs(self.conn, campaign_id).map_err(ServiceError::from)
    }

    /// List only NPCs for a campaign.
    pub fn list_npcs(&mut self, campaign_id: &str) -> ServiceResult<Vec<Character>> {
        dal::list_npcs(self.conn, campaign_id).map_err(ServiceError::from)
    }

    /// List NPCs by location.
    pub fn list_npcs_by_location(
        &mut self,
        campaign_id: &str,
        location: &str,
    ) -> ServiceResult<Vec<Character>> {
        dal::list_npcs_by_location(self.conn, campaign_id, location)
            .map_err(ServiceError::from)
    }

    /// List NPCs by faction.
    pub fn list_npcs_by_faction(
        &mut self,
        campaign_id: &str,
        faction: &str,
    ) -> ServiceResult<Vec<Character>> {
        dal::list_npcs_by_faction(self.conn, campaign_id, faction).map_err(ServiceError::from)
    }

    /// Get a character by ID.
    pub fn get(&mut self, id: &str) -> ServiceResult<Option<Character>> {
        dal::get_character_optional(self.conn, id).map_err(ServiceError::from)
    }

    /// Update a character.
    pub fn update(&mut self, id: &str, input: UpdateCharacterInput) -> ServiceResult<Character> {
        let now = Utc::now().to_rfc3339();

        // Build the update changeset
        let name_ref = input.name.as_deref();
        let player_name_ref = input.player_name.as_ref().map(|o| o.as_deref());
        let race_name_ref = input.race_name.as_ref().map(|o| o.as_deref());
        let race_source_ref = input.race_source.as_ref().map(|o| o.as_deref());
        let background_name_ref = input.background_name.as_ref().map(|o| o.as_deref());
        let background_source_ref = input.background_source.as_ref().map(|o| o.as_deref());
        let traits_ref = input.traits.as_ref().map(|o| o.as_deref());
        let ideals_ref = input.ideals.as_ref().map(|o| o.as_deref());
        let bonds_ref = input.bonds.as_ref().map(|o| o.as_deref());
        let flaws_ref = input.flaws.as_ref().map(|o| o.as_deref());
        let role_ref = input.role.as_ref().map(|o| o.as_deref());
        let location_ref = input.location.as_ref().map(|o| o.as_deref());
        let faction_ref = input.faction.as_ref().map(|o| o.as_deref());

        let update = UpdateCharacter {
            name: name_ref,
            player_name: player_name_ref,
            race_name: race_name_ref,
            race_source: race_source_ref,
            background_name: background_name_ref,
            background_source: background_source_ref,
            strength: input.ability_scores.map(|s| s[0]),
            dexterity: input.ability_scores.map(|s| s[1]),
            constitution: input.ability_scores.map(|s| s[2]),
            intelligence: input.ability_scores.map(|s| s[3]),
            wisdom: input.ability_scores.map(|s| s[4]),
            charisma: input.ability_scores.map(|s| s[5]),
            cp: input.currency.map(|c| c[0]),
            sp: input.currency.map(|c| c[1]),
            ep: input.currency.map(|c| c[2]),
            gp: input.currency.map(|c| c[3]),
            pp: input.currency.map(|c| c[4]),
            traits: traits_ref,
            ideals: ideals_ref,
            bonds: bonds_ref,
            flaws: flaws_ref,
            role: role_ref,
            location: location_ref,
            faction: faction_ref,
            updated_at: Some(&now),
        };

        let rows = dal::update_character(self.conn, id, &update)?;
        if rows == 0 {
            return Err(ServiceError::not_found("Character", id));
        }

        dal::get_character(self.conn, id).map_err(ServiceError::from)
    }

    /// Delete a character and all related data.
    pub fn delete(&mut self, id: &str) -> ServiceResult<()> {
        let rows = dal::delete_character(self.conn, id)?;
        if rows == 0 {
            return Err(ServiceError::not_found("Character", id));
        }
        Ok(())
    }

    /// Check if a character exists.
    pub fn exists(&mut self, id: &str) -> ServiceResult<bool> {
        dal::character_exists(self.conn, id).map_err(ServiceError::from)
    }

    /// Count all characters for a campaign.
    pub fn count_for_campaign(&mut self, campaign_id: &str) -> ServiceResult<i64> {
        dal::count_campaign_characters(self.conn, campaign_id).map_err(ServiceError::from)
    }

    /// Count PCs for a campaign.
    pub fn count_pcs(&mut self, campaign_id: &str) -> ServiceResult<i64> {
        dal::count_pcs(self.conn, campaign_id).map_err(ServiceError::from)
    }

    /// Count NPCs for a campaign.
    pub fn count_npcs(&mut self, campaign_id: &str) -> ServiceResult<i64> {
        dal::count_npcs(self.conn, campaign_id).map_err(ServiceError::from)
    }

    // --- Level Up ---

    /// Level up a character.
    ///
    /// Handles HP calculation, multiclass validation, and class level updates.
    /// All updates occur in a single transaction.
    pub fn level_up(
        &mut self,
        character_id: &str,
        request: LevelUpRequest,
    ) -> ServiceResult<LevelUpResult> {
        // 1. Get the character
        let character = dal::get_character_optional(self.conn, character_id)?
            .ok_or_else(|| ServiceError::not_found("Character", character_id))?;

        // 2. Get existing classes for this character
        let existing_classes = dal::list_character_classes(self.conn, character_id)?;
        let has_existing_class = !existing_classes.is_empty();

        // 3. Check if character already has this class
        let existing_class_entry = dal::find_character_class_by_name(
            self.conn,
            character_id,
            &request.class_name,
            &request.class_source,
        )?;

        let is_multiclass = has_existing_class && existing_class_entry.is_none();

        // 4. Validate multiclass prerequisites if this is a multiclass
        if is_multiclass {
            // Check prerequisites for target class
            check_multiclass_prerequisites(&character, &request.class_name)?;

            // Also check prerequisites for all current classes (character must meet
            // the multiclass requirements for classes they already have)
            for existing in &existing_classes {
                check_multiclass_prerequisites(&character, &existing.class_name)?;
            }
        }

        // 5. Validate HP roll if applicable
        let hit_die = get_class_hit_die(self.conn, &request.class_name, &request.class_source);
        if let HpGainMethod::Roll(roll) = &request.hit_points_method {
            if *roll < 1 || *roll > hit_die {
                return Err(ServiceError::validation(format!(
                    "HP roll {} is invalid for hit die d{} (must be 1-{})",
                    roll, hit_die, hit_die
                )));
            }
        }

        // 6. Calculate HP gain
        let con_mod = Character::ability_modifier(character.constitution);
        let hp_gained = calculate_hp_gain(&request.hit_points_method, hit_die, con_mod);

        // 7. Handle ASI or Feat if provided
        let mut updated_character = character.clone();
        if let Some(ref asi_or_feat) = request.asi_or_feat {
            match asi_or_feat {
                AsiOrFeat::AbilityScoreImprovement {
                    ability1,
                    increase1,
                    ability2,
                    increase2,
                } => {
                    // Validate total increase is exactly 2
                    let total_increase = increase1 + increase2.unwrap_or(0);
                    if total_increase != 2 {
                        return Err(ServiceError::validation(format!(
                            "ASI total increase must be exactly 2, got {}",
                            total_increase
                        )));
                    }

                    // Apply first ability increase (cap at 20)
                    let current1 = get_ability_score(&updated_character, ability1);
                    let new1 = (current1 + increase1).min(20);
                    let scores = set_ability_score(&updated_character, ability1, new1);
                    updated_character.strength = scores[0];
                    updated_character.dexterity = scores[1];
                    updated_character.constitution = scores[2];
                    updated_character.intelligence = scores[3];
                    updated_character.wisdom = scores[4];
                    updated_character.charisma = scores[5];

                    // Apply second ability increase if provided (cap at 20)
                    if let (Some(ability2), Some(increase2)) = (ability2, increase2) {
                        let current2 = get_ability_score(&updated_character, ability2);
                        let new2 = (current2 + increase2).min(20);
                        let scores = set_ability_score(&updated_character, ability2, new2);
                        updated_character.strength = scores[0];
                        updated_character.dexterity = scores[1];
                        updated_character.constitution = scores[2];
                        updated_character.intelligence = scores[3];
                        updated_character.wisdom = scores[4];
                        updated_character.charisma = scores[5];
                    }

                    // Update character ability scores in database
                    let now = Utc::now().to_rfc3339();
                    let update = UpdateCharacter {
                        strength: Some(updated_character.strength),
                        dexterity: Some(updated_character.dexterity),
                        constitution: Some(updated_character.constitution),
                        intelligence: Some(updated_character.intelligence),
                        wisdom: Some(updated_character.wisdom),
                        charisma: Some(updated_character.charisma),
                        updated_at: Some(&now),
                        ..Default::default()
                    };
                    dal::update_character(self.conn, character_id, &update)?;
                }
                AsiOrFeat::Feat { name, source } => {
                    // Add feat to character
                    let feat_id = Uuid::new_v4().to_string();
                    let new_feat =
                        NewCharacterFeat::new(&feat_id, character_id, name, source, FeatSourceType::Asi);
                    dal::insert_character_feat(self.conn, &new_feat)?;
                }
            }
        }

        // 8. Handle spell changes if provided
        if let Some(ref spell_changes) = request.spell_changes {
            // Handle spell swap first (remove old, add new)
            if let (Some(ref swap_out), Some(ref swap_in)) = (&spell_changes.swap_out, &spell_changes.swap_in) {
                // Find and remove the spell being swapped out
                let existing_spell = dal::find_character_spell_by_name(
                    self.conn,
                    character_id,
                    &swap_out.name,
                    &request.class_name,
                )?;

                if let Some(spell) = existing_spell {
                    dal::delete_character_spell(self.conn, &spell.id)?;
                } else {
                    return Err(ServiceError::validation(format!(
                        "Cannot swap out spell '{}' - character doesn't know it from class {}",
                        swap_out.name, request.class_name
                    )));
                }

                // Add the swap-in spell
                let spell_id = Uuid::new_v4().to_string();
                let new_spell = NewCharacterSpell::new(
                    &spell_id,
                    character_id,
                    &swap_in.name,
                    &swap_in.source,
                    &request.class_name,
                );
                dal::insert_character_spell(self.conn, &new_spell)?;
            }

            // Add new spells (Spells Known or Wizard spellbook additions)
            for spell in &spell_changes.new_spells {
                // Check if character already knows this spell from this class
                if dal::character_knows_spell(self.conn, character_id, &spell.name)? {
                    // Skip if already known (could be from different class or previous level)
                    continue;
                }

                let spell_id = Uuid::new_v4().to_string();
                let new_spell = NewCharacterSpell::new(
                    &spell_id,
                    character_id,
                    &spell.name,
                    &spell.source,
                    &request.class_name,
                );
                dal::insert_character_spell(self.conn, &new_spell)?;
            }

            // Add new cantrips
            for cantrip in &spell_changes.new_cantrips {
                // Check if character already knows this cantrip
                if dal::character_knows_spell(self.conn, character_id, &cantrip.name)? {
                    continue;
                }

                let cantrip_id = Uuid::new_v4().to_string();
                let new_cantrip = NewCharacterSpell::new(
                    &cantrip_id,
                    character_id,
                    &cantrip.name,
                    &cantrip.source,
                    &request.class_name,
                );
                dal::insert_character_spell(self.conn, &new_cantrip)?;
            }
        }

        // 9. Handle feature choices if provided
        if let Some(ref feature_choices) = request.feature_choices {
            // Handle Fighting Style
            if let Some(ref fighting_style) = feature_choices.fighting_style {
                // Check if character already has this fighting style
                if !dal::character_has_feature(
                    self.conn,
                    character_id,
                    "fighting_style",
                    &fighting_style.name,
                )? {
                    let feature_id = Uuid::new_v4().to_string();
                    let new_feature = NewCharacterFeature::fighting_style(
                        &feature_id,
                        character_id,
                        &fighting_style.name,
                        &fighting_style.source,
                        &request.class_name,
                    );
                    dal::insert_character_feature(self.conn, &new_feature)?;
                }
            }

            // Handle Metamagic (Sorcerer)
            if let Some(ref metamagic_list) = feature_choices.metamagic {
                for metamagic in metamagic_list {
                    // Skip if already has this metamagic
                    if dal::character_has_feature(
                        self.conn,
                        character_id,
                        "metamagic",
                        &metamagic.name,
                    )? {
                        continue;
                    }
                    let feature_id = Uuid::new_v4().to_string();
                    let new_feature = NewCharacterFeature::metamagic(
                        &feature_id,
                        character_id,
                        &metamagic.name,
                        &metamagic.source,
                    );
                    dal::insert_character_feature(self.conn, &new_feature)?;
                }
            }

            // Handle Maneuvers (Battle Master) with swap support
            if let Some(ref maneuver_choices) = feature_choices.maneuvers {
                // Handle maneuver swap first
                if let (Some(ref swap_out), Some(ref swap_in)) =
                    (&maneuver_choices.swap_out, &maneuver_choices.swap_in)
                {
                    let existing = dal::find_feature_by_name(
                        self.conn,
                        character_id,
                        "maneuver",
                        &swap_out.name,
                    )?;
                    if let Some(feature) = existing {
                        dal::delete_character_feature(self.conn, &feature.id)?;
                    } else {
                        return Err(ServiceError::validation(format!(
                            "Cannot swap out maneuver '{}' - character doesn't know it",
                            swap_out.name
                        )));
                    }

                    let feature_id = Uuid::new_v4().to_string();
                    let new_feature = NewCharacterFeature::maneuver(
                        &feature_id,
                        character_id,
                        &swap_in.name,
                        &swap_in.source,
                    );
                    dal::insert_character_feature(self.conn, &new_feature)?;
                }

                // Add new maneuvers
                for maneuver in &maneuver_choices.new_maneuvers {
                    if dal::character_has_feature(
                        self.conn,
                        character_id,
                        "maneuver",
                        &maneuver.name,
                    )? {
                        continue;
                    }
                    let feature_id = Uuid::new_v4().to_string();
                    let new_feature = NewCharacterFeature::maneuver(
                        &feature_id,
                        character_id,
                        &maneuver.name,
                        &maneuver.source,
                    );
                    dal::insert_character_feature(self.conn, &new_feature)?;
                }
            }

            // Handle Invocations (Warlock) with swap support
            if let Some(ref invocation_choices) = feature_choices.invocations {
                // Handle invocation swap first
                if let (Some(ref swap_out), Some(ref swap_in)) =
                    (&invocation_choices.swap_out, &invocation_choices.swap_in)
                {
                    let existing = dal::find_feature_by_name(
                        self.conn,
                        character_id,
                        "invocation",
                        &swap_out.name,
                    )?;
                    if let Some(feature) = existing {
                        dal::delete_character_feature(self.conn, &feature.id)?;
                    } else {
                        return Err(ServiceError::validation(format!(
                            "Cannot swap out invocation '{}' - character doesn't know it",
                            swap_out.name
                        )));
                    }

                    let feature_id = Uuid::new_v4().to_string();
                    let new_feature = NewCharacterFeature::invocation(
                        &feature_id,
                        character_id,
                        &swap_in.name,
                        &swap_in.source,
                    );
                    dal::insert_character_feature(self.conn, &new_feature)?;
                }

                // Add new invocations
                for invocation in &invocation_choices.new_invocations {
                    if dal::character_has_feature(
                        self.conn,
                        character_id,
                        "invocation",
                        &invocation.name,
                    )? {
                        continue;
                    }
                    let feature_id = Uuid::new_v4().to_string();
                    let new_feature = NewCharacterFeature::invocation(
                        &feature_id,
                        character_id,
                        &invocation.name,
                        &invocation.source,
                    );
                    dal::insert_character_feature(self.conn, &new_feature)?;
                }
            }

            // Handle Pact Boon (Warlock)
            if let Some(ref pact_boon) = feature_choices.pact_boon {
                // Check if character already has a pact boon
                let existing_boons =
                    dal::list_features_by_type(self.conn, character_id, "pact_boon")?;
                if existing_boons.is_empty() {
                    let feature_id = Uuid::new_v4().to_string();
                    let new_feature = NewCharacterFeature::pact_boon(
                        &feature_id,
                        character_id,
                        &pact_boon.name,
                        &pact_boon.source,
                    );
                    dal::insert_character_feature(self.conn, &new_feature)?;
                }
            }

            // Handle Expertise (Rogue/Bard)
            if let Some(ref expertise_skills) = feature_choices.expertise_skills {
                for skill_name in expertise_skills {
                    // Find the existing skill proficiency and upgrade to expertise
                    let proficiencies =
                        dal::list_character_proficiencies(self.conn, character_id)?;
                    let skill_prof = proficiencies
                        .iter()
                        .find(|p| p.proficiency_type == "skill" && p.name == *skill_name);

                    if let Some(prof) = skill_prof {
                        // Already has proficiency, upgrade to expertise
                        if prof.expertise == 0 {
                            let update = UpdateCharacterProficiency::set_expertise(true);
                            dal::update_character_proficiency(self.conn, &prof.id, &update)?;
                        }
                    } else {
                        // Doesn't have proficiency - add with expertise
                        let prof_id = Uuid::new_v4().to_string();
                        let new_prof = NewCharacterProficiency::new(
                            &prof_id,
                            character_id,
                            ProficiencyType::Skill,
                            skill_name,
                        )
                        .with_expertise();
                        dal::insert_character_proficiency(self.conn, &new_prof)?;
                    }
                }
            }
        }

        // 10. Update or insert class entry
        let updated_class = if let Some(existing) = existing_class_entry {
            // Single-class level up - increment existing class level
            let new_level = existing.level + 1;

            // Build update with new level and optional subclass
            let update = if let Some(ref subclass) = request.subclass {
                UpdateCharacterClass::set_level_and_subclass(new_level, &subclass.name, &subclass.source)
            } else {
                UpdateCharacterClass::set_level(new_level)
            };

            dal::update_character_class(self.conn, &existing.id, &update)?;
            dal::get_character_class(self.conn, &existing.id)?
        } else {
            // New class (either first class or multiclass)
            let class_id = Uuid::new_v4().to_string();
            let is_starting = !has_existing_class;

            let mut new_class = if is_starting {
                NewCharacterClass::starting(&class_id, character_id, &request.class_name, &request.class_source)
            } else {
                NewCharacterClass::multiclass(&class_id, character_id, &request.class_name, &request.class_source)
            };

            // Add subclass if provided
            if let Some(ref subclass) = request.subclass {
                new_class = new_class.with_subclass(&subclass.name, &subclass.source);
            }

            dal::insert_character_class(self.conn, &new_class)?;
            dal::get_character_class(self.conn, &class_id)?
        };

        // 11. Calculate new total level
        let new_total_level = dal::get_total_level(self.conn, character_id)? as i32;

        // 12. Refresh character data
        let final_character = dal::get_character(self.conn, character_id)?;

        Ok(LevelUpResult {
            character: final_character,
            class: updated_class,
            hp_gained,
            new_total_level,
            is_multiclass,
        })
    }

    // --- Inventory Management ---

    /// Add an item to a character's inventory.
    pub fn add_to_inventory(
        &mut self,
        character_id: &str,
        input: AddInventoryInput,
    ) -> ServiceResult<CharacterInventory> {
        // Verify character exists
        if !dal::character_exists(self.conn, character_id)? {
            return Err(ServiceError::not_found("Character", character_id));
        }

        let inv_id = Uuid::new_v4().to_string();
        let notes_ref = input.notes.as_deref();

        let mut new_item =
            NewCharacterInventory::new(&inv_id, character_id, &input.item_name, &input.item_source);

        if let Some(qty) = input.quantity {
            new_item = new_item.with_quantity(qty);
        }
        if input.equipped {
            new_item = new_item.equipped();
        }
        if input.attuned {
            new_item = new_item.attuned();
        }
        if let Some(notes) = notes_ref {
            new_item = new_item.with_notes(notes);
        }

        dal::insert_character_inventory(self.conn, &new_item)?;
        dal::get_character_inventory(self.conn, &inv_id).map_err(ServiceError::from)
    }

    /// Remove an item from a character's inventory.
    pub fn remove_from_inventory(&mut self, inventory_id: &str) -> ServiceResult<()> {
        let rows = dal::delete_character_inventory(self.conn, inventory_id)?;
        if rows == 0 {
            return Err(ServiceError::not_found("InventoryItem", inventory_id));
        }
        Ok(())
    }

    /// Get a character's inventory.
    pub fn get_inventory(&mut self, character_id: &str) -> ServiceResult<Vec<CharacterInventory>> {
        dal::list_character_inventory(self.conn, character_id).map_err(ServiceError::from)
    }

    /// Get equipped items for a character.
    pub fn get_equipped_items(
        &mut self,
        character_id: &str,
    ) -> ServiceResult<Vec<CharacterInventory>> {
        dal::list_equipped_items(self.conn, character_id).map_err(ServiceError::from)
    }

    /// Get attuned items for a character.
    pub fn get_attuned_items(
        &mut self,
        character_id: &str,
    ) -> ServiceResult<Vec<CharacterInventory>> {
        dal::list_attuned_items(self.conn, character_id).map_err(ServiceError::from)
    }

    /// Update an inventory item (quantity, equipped, attuned, notes).
    pub fn update_inventory_item(
        &mut self,
        inventory_id: &str,
        quantity: Option<i32>,
        equipped: Option<bool>,
        attuned: Option<bool>,
    ) -> ServiceResult<CharacterInventory> {
        let update = UpdateCharacterInventory {
            quantity,
            equipped: equipped.map(|e| if e { 1 } else { 0 }),
            attuned: attuned.map(|a| if a { 1 } else { 0 }),
            notes: None,
        };

        let rows = dal::update_character_inventory(self.conn, inventory_id, &update)?;
        if rows == 0 {
            return Err(ServiceError::not_found("InventoryItem", inventory_id));
        }

        dal::get_character_inventory(self.conn, inventory_id).map_err(ServiceError::from)
    }

    /// Count attuned items for a character (D&D 5e max is 3).
    pub fn count_attuned_items(&mut self, character_id: &str) -> ServiceResult<i64> {
        dal::count_attuned_items(self.conn, character_id).map_err(ServiceError::from)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dal::campaign::insert_campaign;
    use crate::models::campaign::NewCampaign;
    use crate::test_utils::setup_test_db;

    fn create_test_campaign(conn: &mut SqliteConnection) -> String {
        let campaign_id = Uuid::new_v4().to_string();
        let campaign = NewCampaign::new(&campaign_id, "Test Campaign");
        insert_campaign(conn, &campaign).expect("Failed to create campaign");
        campaign_id
    }

    #[test]
    fn test_create_pc() {
        let mut conn = setup_test_db();
        let campaign_id = create_test_campaign(&mut conn);

        let mut service = CharacterService::new(&mut conn);

        let input = CreateCharacterInput::new_pc(&campaign_id, "Gandalf", "John");
        let character = service.create(input).expect("Failed to create character");

        assert_eq!(character.name, "Gandalf");
        assert_eq!(character.is_npc, 0);
        assert_eq!(character.player_name, Some("John".to_string()));
    }

    #[test]
    fn test_create_npc() {
        let mut conn = setup_test_db();
        let campaign_id = create_test_campaign(&mut conn);

        let mut service = CharacterService::new(&mut conn);

        let input = CreateCharacterInput::new_npc(&campaign_id, "Shopkeeper");
        let character = service.create(input).expect("Failed to create character");

        assert_eq!(character.name, "Shopkeeper");
        assert_eq!(character.is_npc, 1);
        assert!(character.player_name.is_none());
    }

    #[test]
    fn test_create_with_race_and_background() {
        let mut conn = setup_test_db();
        let campaign_id = create_test_campaign(&mut conn);

        let mut service = CharacterService::new(&mut conn);

        let input = CreateCharacterInput::new_pc(&campaign_id, "Legolas", "Jane")
            .with_race("Elf", "PHB")
            .with_background("Outlander", "PHB");
        let character = service.create(input).expect("Failed to create character");

        assert_eq!(character.race_name, Some("Elf".to_string()));
        assert_eq!(character.race_source, Some("PHB".to_string()));
        assert_eq!(character.background_name, Some("Outlander".to_string()));
        assert_eq!(character.background_source, Some("PHB".to_string()));
    }

    #[test]
    fn test_create_with_ability_scores() {
        let mut conn = setup_test_db();
        let campaign_id = create_test_campaign(&mut conn);

        let mut service = CharacterService::new(&mut conn);

        let input = CreateCharacterInput::new_pc(&campaign_id, "Fighter", "John")
            .with_ability_scores([16, 14, 15, 8, 10, 12]);
        let character = service.create(input).expect("Failed to create character");

        assert_eq!(character.strength, 16);
        assert_eq!(character.dexterity, 14);
        assert_eq!(character.constitution, 15);
        assert_eq!(character.intelligence, 8);
        assert_eq!(character.wisdom, 10);
        assert_eq!(character.charisma, 12);
    }

    #[test]
    fn test_list_for_campaign() {
        let mut conn = setup_test_db();
        let campaign_id = create_test_campaign(&mut conn);

        let mut service = CharacterService::new(&mut conn);

        let input1 = CreateCharacterInput::new_pc(&campaign_id, "PC1", "John");
        let input2 = CreateCharacterInput::new_pc(&campaign_id, "PC2", "Jane");
        let input3 = CreateCharacterInput::new_npc(&campaign_id, "NPC1");
        service.create(input1).expect("Failed to create character");
        service.create(input2).expect("Failed to create character");
        service.create(input3).expect("Failed to create character");

        let all = service
            .list_for_campaign(&campaign_id)
            .expect("Failed to list");
        assert_eq!(all.len(), 3);

        let pcs = service.list_pcs(&campaign_id).expect("Failed to list");
        assert_eq!(pcs.len(), 2);

        let npcs = service.list_npcs(&campaign_id).expect("Failed to list");
        assert_eq!(npcs.len(), 1);
    }

    #[test]
    fn test_get_character() {
        let mut conn = setup_test_db();
        let campaign_id = create_test_campaign(&mut conn);

        let mut service = CharacterService::new(&mut conn);

        let input = CreateCharacterInput::new_pc(&campaign_id, "Hero", "John");
        let created = service.create(input).expect("Failed to create character");

        let retrieved = service
            .get(&created.id)
            .expect("Failed to get character")
            .expect("Character not found");

        assert_eq!(retrieved.id, created.id);
        assert_eq!(retrieved.name, "Hero");
    }

    #[test]
    fn test_get_character_not_found() {
        let mut conn = setup_test_db();

        let mut service = CharacterService::new(&mut conn);

        let result = service.get("nonexistent").expect("Failed to query");
        assert!(result.is_none());
    }

    #[test]
    fn test_update_character_name() {
        let mut conn = setup_test_db();
        let campaign_id = create_test_campaign(&mut conn);

        let mut service = CharacterService::new(&mut conn);

        let input = CreateCharacterInput::new_pc(&campaign_id, "Original", "John");
        let created = service.create(input).expect("Failed to create character");

        let update = UpdateCharacterInput::set_name("Updated");
        let updated = service
            .update(&created.id, update)
            .expect("Failed to update");

        assert_eq!(updated.name, "Updated");
    }

    #[test]
    fn test_update_character_race() {
        let mut conn = setup_test_db();
        let campaign_id = create_test_campaign(&mut conn);

        let mut service = CharacterService::new(&mut conn);

        let input = CreateCharacterInput::new_pc(&campaign_id, "Hero", "John");
        let created = service.create(input).expect("Failed to create character");
        assert!(created.race_name.is_none());

        let update = UpdateCharacterInput::set_race(Some("Dwarf".to_string()), Some("PHB".to_string()));
        let updated = service
            .update(&created.id, update)
            .expect("Failed to update");

        assert_eq!(updated.race_name, Some("Dwarf".to_string()));
        assert_eq!(updated.race_source, Some("PHB".to_string()));
    }

    #[test]
    fn test_update_character_not_found() {
        let mut conn = setup_test_db();

        let mut service = CharacterService::new(&mut conn);

        let update = UpdateCharacterInput::set_name("Updated");
        let result = service.update("nonexistent", update);

        assert!(matches!(result, Err(ServiceError::NotFound { .. })));
    }

    #[test]
    fn test_delete_character() {
        let mut conn = setup_test_db();
        let campaign_id = create_test_campaign(&mut conn);

        let mut service = CharacterService::new(&mut conn);

        let input = CreateCharacterInput::new_pc(&campaign_id, "Doomed", "John");
        let created = service.create(input).expect("Failed to create character");

        assert!(service.exists(&created.id).expect("Failed to check"));

        service.delete(&created.id).expect("Failed to delete");

        assert!(!service.exists(&created.id).expect("Failed to check"));
    }

    #[test]
    fn test_delete_character_not_found() {
        let mut conn = setup_test_db();

        let mut service = CharacterService::new(&mut conn);

        let result = service.delete("nonexistent");
        assert!(matches!(result, Err(ServiceError::NotFound { .. })));
    }

    #[test]
    fn test_count_characters() {
        let mut conn = setup_test_db();
        let campaign_id = create_test_campaign(&mut conn);

        let mut service = CharacterService::new(&mut conn);

        assert_eq!(
            service
                .count_for_campaign(&campaign_id)
                .expect("Failed to count"),
            0
        );

        let input1 = CreateCharacterInput::new_pc(&campaign_id, "PC", "John");
        let input2 = CreateCharacterInput::new_npc(&campaign_id, "NPC1");
        let input3 = CreateCharacterInput::new_npc(&campaign_id, "NPC2");
        service.create(input1).expect("Failed to create");
        service.create(input2).expect("Failed to create");
        service.create(input3).expect("Failed to create");

        assert_eq!(
            service
                .count_for_campaign(&campaign_id)
                .expect("Failed to count"),
            3
        );
        assert_eq!(
            service.count_pcs(&campaign_id).expect("Failed to count"),
            1
        );
        assert_eq!(
            service.count_npcs(&campaign_id).expect("Failed to count"),
            2
        );
    }

    #[test]
    fn test_add_to_inventory() {
        let mut conn = setup_test_db();
        let campaign_id = create_test_campaign(&mut conn);

        let mut service = CharacterService::new(&mut conn);

        let input = CreateCharacterInput::new_pc(&campaign_id, "Hero", "John");
        let character = service.create(input).expect("Failed to create character");

        let item_input = AddInventoryInput::new("Longsword", "PHB");
        let item = service
            .add_to_inventory(&character.id, item_input)
            .expect("Failed to add item");

        assert_eq!(item.item_name, "Longsword");
        assert_eq!(item.item_source, "PHB");
        assert_eq!(item.quantity, 1);
        assert!(!item.is_equipped());
        assert!(!item.is_attuned());
    }

    #[test]
    fn test_add_to_inventory_with_options() {
        let mut conn = setup_test_db();
        let campaign_id = create_test_campaign(&mut conn);

        let mut service = CharacterService::new(&mut conn);

        let input = CreateCharacterInput::new_pc(&campaign_id, "Hero", "John");
        let character = service.create(input).expect("Failed to create character");

        let item_input = AddInventoryInput::new("Cloak of Protection", "DMG")
            .equipped()
            .attuned()
            .with_notes("Found in dungeon");
        let item = service
            .add_to_inventory(&character.id, item_input)
            .expect("Failed to add item");

        assert!(item.is_equipped());
        assert!(item.is_attuned());
        assert_eq!(item.notes, Some("Found in dungeon".to_string()));
    }

    #[test]
    fn test_get_inventory() {
        let mut conn = setup_test_db();
        let campaign_id = create_test_campaign(&mut conn);

        let mut service = CharacterService::new(&mut conn);

        let input = CreateCharacterInput::new_pc(&campaign_id, "Hero", "John");
        let character = service.create(input).expect("Failed to create character");

        service
            .add_to_inventory(&character.id, AddInventoryInput::new("Sword", "PHB"))
            .expect("Failed to add item");
        service
            .add_to_inventory(&character.id, AddInventoryInput::new("Shield", "PHB"))
            .expect("Failed to add item");

        let inventory = service
            .get_inventory(&character.id)
            .expect("Failed to get inventory");
        assert_eq!(inventory.len(), 2);
    }

    #[test]
    fn test_remove_from_inventory() {
        let mut conn = setup_test_db();
        let campaign_id = create_test_campaign(&mut conn);

        let mut service = CharacterService::new(&mut conn);

        let input = CreateCharacterInput::new_pc(&campaign_id, "Hero", "John");
        let character = service.create(input).expect("Failed to create character");

        let item = service
            .add_to_inventory(&character.id, AddInventoryInput::new("Sword", "PHB"))
            .expect("Failed to add item");

        service
            .remove_from_inventory(&item.id)
            .expect("Failed to remove item");

        let inventory = service
            .get_inventory(&character.id)
            .expect("Failed to get inventory");
        assert_eq!(inventory.len(), 0);
    }

    #[test]
    fn test_update_inventory_item() {
        let mut conn = setup_test_db();
        let campaign_id = create_test_campaign(&mut conn);

        let mut service = CharacterService::new(&mut conn);

        let input = CreateCharacterInput::new_pc(&campaign_id, "Hero", "John");
        let character = service.create(input).expect("Failed to create character");

        let item = service
            .add_to_inventory(
                &character.id,
                AddInventoryInput::new("Arrow", "PHB").with_quantity(20),
            )
            .expect("Failed to add item");

        let updated = service
            .update_inventory_item(&item.id, Some(15), Some(true), None)
            .expect("Failed to update item");

        assert_eq!(updated.quantity, 15);
        assert!(updated.is_equipped());
        assert!(!updated.is_attuned());
    }
}
