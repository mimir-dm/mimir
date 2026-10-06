//! Character Service
//!
//! Business logic for character management (PCs and NPCs).

use diesel::SqliteConnection;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::dal::campaign as dal;
use crate::dal::catalog as catalog_dal;
use crate::models::campaign::{
    Character, CharacterClass, CharacterFeat, CharacterInventory, CharacterSpell, FeatSourceType,
    NewCharacterClass, NewCharacterFeat, NewCharacterFeature, NewCharacterInventory,
    NewCharacterProficiency, NewCharacterSpell, ProficiencyType, UpdateCharacter,
    UpdateCharacterClass, UpdateCharacterInventory, UpdateCharacterProficiency,
    UpdateCharacterSpell,
};
use crate::services::catalog::CatalogEntityService;
use crate::services::{ClassService, ServiceError, ServiceResult};
use crate::utils::now_rfc3339;

mod core;
#[cfg(test)]
mod test_support;

pub use core::{CreateCharacterInput, UpdateCharacterInput};

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

// =============================================================================
// Proficiency Extraction Helpers
// =============================================================================

/// A proficiency to be inserted during character creation.
struct ProficiencyEntry {
    prof_type: ProficiencyType,
    name: String,
}

/// Extract deterministic (non-choice) proficiencies from a keyed JSON object.
///
/// Handles the common 5etools format: `{ "perception": true, "stealth": true }`
/// Ignores choice keys like "choose", "any", "anyStandard", etc.
fn extract_keyed_proficiencies(
    items: &[serde_json::Value],
    prof_type: ProficiencyType,
) -> Vec<ProficiencyEntry> {
    let mut result = Vec::new();
    for item in items {
        if let Some(obj) = item.as_object() {
            for (key, val) in obj {
                // Skip choice/meta keys
                if key == "choose" || key == "any" || key.starts_with("any") {
                    continue;
                }
                // Boolean true means granted proficiency
                if val.as_bool() == Some(true) {
                    // Clean up key: remove source suffix (e.g., "longsword|phb" -> "longsword")
                    let name = key.split('|').next().unwrap_or(key);
                    result.push(ProficiencyEntry {
                        prof_type: prof_type.clone(),
                        name: capitalize_proficiency(name),
                    });
                }
            }
        } else if let Some(s) = item.as_str() {
            // Simple string proficiency (e.g., armor types: "light", "medium", "heavy")
            result.push(ProficiencyEntry {
                prof_type: prof_type.clone(),
                name: capitalize_proficiency(s),
            });
        }
    }
    result
}

/// Capitalize a proficiency name for display.
fn capitalize_proficiency(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => {
            let upper: String = first.to_uppercase().collect();
            upper + chars.as_str()
        }
    }
}

/// Extract proficiencies from class catalog data.
fn extract_class_proficiencies(
    class_data: &serde_json::Value,
    selected_skills: &[String],
) -> Vec<ProficiencyEntry> {
    let mut profs = Vec::new();

    // Saving throws from "proficiency" field: ["str", "con"]
    if let Some(saves) = class_data.get("proficiency").and_then(|v| v.as_array()) {
        for save in saves {
            if let Some(s) = save.as_str() {
                let full_name = match s {
                    "str" => "Strength",
                    "dex" => "Dexterity",
                    "con" => "Constitution",
                    "int" => "Intelligence",
                    "wis" => "Wisdom",
                    "cha" => "Charisma",
                    other => other,
                };
                profs.push(ProficiencyEntry {
                    prof_type: ProficiencyType::Save,
                    name: full_name.to_string(),
                });
            }
        }
    }

    // Starting proficiencies
    if let Some(sp) = class_data.get("startingProficiencies") {
        // Armor proficiencies
        if let Some(armor) = sp.get("armor").and_then(|v| v.as_array()) {
            profs.extend(extract_keyed_proficiencies(armor, ProficiencyType::Armor));
        }

        // Weapon proficiencies
        if let Some(weapons) = sp.get("weapons").and_then(|v| v.as_array()) {
            profs.extend(extract_keyed_proficiencies(weapons, ProficiencyType::Weapon));
        }

        // Tool proficiencies
        if let Some(tools) = sp.get("tools").and_then(|v| v.as_array()) {
            profs.extend(extract_keyed_proficiencies(tools, ProficiencyType::Tool));
        }
    }

    // Skill proficiencies (user-selected)
    for skill in selected_skills {
        profs.push(ProficiencyEntry {
            prof_type: ProficiencyType::Skill,
            name: skill.clone(),
        });
    }

    profs
}

/// Extract proficiencies from background catalog data.
fn extract_background_proficiencies(bg_data: &serde_json::Value) -> Vec<ProficiencyEntry> {
    let mut profs = Vec::new();

    // Skill proficiencies (usually deterministic for backgrounds)
    if let Some(skills) = bg_data.get("skillProficiencies").and_then(|v| v.as_array()) {
        profs.extend(extract_keyed_proficiencies(skills, ProficiencyType::Skill));
    }

    // Tool proficiencies
    if let Some(tools) = bg_data.get("toolProficiencies").and_then(|v| v.as_array()) {
        profs.extend(extract_keyed_proficiencies(tools, ProficiencyType::Tool));
    }

    // Language proficiencies
    if let Some(langs) = bg_data.get("languageProficiencies").and_then(|v| v.as_array()) {
        profs.extend(extract_keyed_proficiencies(langs, ProficiencyType::Language));
    }

    profs
}

/// Extract proficiencies from race catalog data.
fn extract_race_proficiencies(race_data: &serde_json::Value) -> Vec<ProficiencyEntry> {
    let mut profs = Vec::new();

    // Skill proficiencies
    if let Some(skills) = race_data.get("skillProficiencies").and_then(|v| v.as_array()) {
        profs.extend(extract_keyed_proficiencies(skills, ProficiencyType::Skill));
    }

    // Weapon proficiencies
    if let Some(weapons) = race_data.get("weaponProficiencies").and_then(|v| v.as_array()) {
        profs.extend(extract_keyed_proficiencies(weapons, ProficiencyType::Weapon));
    }

    // Armor proficiencies
    if let Some(armor) = race_data.get("armorProficiencies").and_then(|v| v.as_array()) {
        profs.extend(extract_keyed_proficiencies(armor, ProficiencyType::Armor));
    }

    // Tool proficiencies
    if let Some(tools) = race_data.get("toolProficiencies").and_then(|v| v.as_array()) {
        profs.extend(extract_keyed_proficiencies(tools, ProficiencyType::Tool));
    }

    // Language proficiencies
    if let Some(langs) = race_data.get("languageProficiencies").and_then(|v| v.as_array()) {
        profs.extend(extract_keyed_proficiencies(langs, ProficiencyType::Language));
    }

    profs
}

/// Extract multiclass proficiencies from class catalog data.
fn extract_multiclass_proficiencies(class_data: &serde_json::Value) -> Vec<ProficiencyEntry> {
    let mut profs = Vec::new();

    if let Some(mc) = class_data.get("multiclassing") {
        if let Some(gained) = mc.get("proficienciesGained") {
            if let Some(armor) = gained.get("armor").and_then(|v| v.as_array()) {
                profs.extend(extract_keyed_proficiencies(armor, ProficiencyType::Armor));
            }
            if let Some(weapons) = gained.get("weapons").and_then(|v| v.as_array()) {
                profs.extend(extract_keyed_proficiencies(weapons, ProficiencyType::Weapon));
            }
            if let Some(tools) = gained.get("tools").and_then(|v| v.as_array()) {
                profs.extend(extract_keyed_proficiencies(tools, ProficiencyType::Tool));
            }
        }
    }

    profs
}

/// Insert proficiency entries for a character, skipping duplicates.
fn insert_proficiencies(
    conn: &mut SqliteConnection,
    character_id: &str,
    entries: &[ProficiencyEntry],
) -> ServiceResult<()> {
    for entry in entries {
        // Skip if character already has this proficiency
        if dal::character_has_proficiency(
            conn,
            character_id,
            &entry.prof_type.as_str(),
            &entry.name,
        )? {
            continue;
        }
        let prof_id = Uuid::new_v4().to_string();
        let new_prof = NewCharacterProficiency::new(
            &prof_id,
            character_id,
            entry.prof_type.clone(),
            &entry.name,
        );
        dal::insert_character_proficiency(conn, &new_prof)?;
    }
    Ok(())
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
                    let now = now_rfc3339();
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

            // Add multiclass proficiencies if this is a multiclass level-up
            if is_multiclass {
                if let Ok(Some(class_row)) = catalog_dal::get_class_by_name(
                    self.conn,
                    &request.class_name,
                    &request.class_source,
                ) {
                    if let Ok(class_data) =
                        serde_json::from_str::<serde_json::Value>(&class_row.data)
                    {
                        let mc_profs = extract_multiclass_proficiencies(&class_data);
                        let _ = insert_proficiencies(self.conn, character_id, &mc_profs);
                    }
                }
            }

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

    /// List all spells a character knows.
    pub fn list_spells(&mut self, character_id: &str) -> ServiceResult<Vec<CharacterSpell>> {
        dal::list_character_spells(self.conn, character_id).map_err(ServiceError::from)
    }

    /// Add a spell to a character's known spells.
    ///
    /// Rejects the add if the character already knows the spell from the same
    /// class. Single implementation shared by the desktop UI and MCP server.
    pub fn add_spell(
        &mut self,
        character_id: &str,
        spell_name: &str,
        spell_source: &str,
        source_class: &str,
        prepared: bool,
    ) -> ServiceResult<CharacterSpell> {
        if dal::find_character_spell_by_name(self.conn, character_id, spell_name, source_class)?
            .is_some()
        {
            return Err(ServiceError::Validation(format!(
                "Character already knows {} from {}",
                spell_name, source_class
            )));
        }

        let id = Uuid::new_v4().to_string();
        let mut spell =
            NewCharacterSpell::new(&id, character_id, spell_name, spell_source, source_class);
        if prepared {
            spell = spell.prepared();
        }
        dal::insert_character_spell(self.conn, &spell)?;

        Ok(CharacterSpell {
            id,
            character_id: character_id.to_string(),
            spell_name: spell_name.to_string(),
            spell_source: spell_source.to_string(),
            source_class: source_class.to_string(),
            prepared: if prepared { 1 } else { 0 },
        })
    }

    /// Remove a spell from a character's known spells.
    ///
    /// With `source_class`, removes that class's instance only. Without it,
    /// removes every instance matching the name case-insensitively.
    pub fn remove_spell(
        &mut self,
        character_id: &str,
        spell_name: &str,
        source_class: Option<&str>,
    ) -> ServiceResult<()> {
        if let Some(class) = source_class {
            let spell =
                dal::find_character_spell_by_name(self.conn, character_id, spell_name, class)?
                    .ok_or_else(|| ServiceError::Validation(format!(
                        "Character doesn't know {} from {}",
                        spell_name, class
                    )))?;
            dal::delete_character_spell(self.conn, &spell.id)?;
        } else {
            let spells = dal::list_character_spells(self.conn, character_id)?;
            let matching: Vec<_> = spells
                .iter()
                .filter(|s| s.spell_name.to_lowercase() == spell_name.to_lowercase())
                .collect();
            if matching.is_empty() {
                return Err(ServiceError::Validation(format!(
                    "Character doesn't know {}",
                    spell_name
                )));
            }
            for spell in matching {
                dal::delete_character_spell(self.conn, &spell.id)?;
            }
        }
        Ok(())
    }

    /// List the feats a character has, ordered by name.
    pub fn list_feats(&mut self, character_id: &str) -> ServiceResult<Vec<CharacterFeat>> {
        if !dal::character_exists(self.conn, character_id)? {
            return Err(ServiceError::not_found("Character", character_id));
        }
        dal::list_character_feats(self.conn, character_id).map_err(ServiceError::from)
    }

    /// Record a feat for a character outside level-up (e.g. a deferred ASI choice).
    ///
    /// The feat must exist in the catalog under `feat_name` (case-insensitive)
    /// and `feat_source`; it is stored with the catalog's spelling. A character
    /// may hold a feat once, unless the catalog marks it repeatable.
    /// Single implementation shared by the desktop UI and MCP server.
    pub fn add_feat(
        &mut self,
        character_id: &str,
        feat_name: &str,
        feat_source: &str,
        source_type: FeatSourceType,
    ) -> ServiceResult<CharacterFeat> {
        if !dal::character_exists(self.conn, character_id)? {
            return Err(ServiceError::not_found("Character", character_id));
        }

        let catalog_feat = catalog_dal::get_feat_by_name(self.conn, feat_name, feat_source)?
            .ok_or_else(|| {
                ServiceError::Validation(format!(
                    "Feat '{}' from {} is not in the catalog",
                    feat_name, feat_source
                ))
            })?;

        let repeatable = serde_json::from_str::<serde_json::Value>(&catalog_feat.data)
            .ok()
            .and_then(|d| d.get("repeatable").and_then(|r| r.as_bool()))
            .unwrap_or(false);
        // Compare case-insensitively: level-up stores the name as the caller
        // typed it, so an existing row may not use the catalog's spelling.
        let wanted = catalog_feat.name.to_lowercase();
        let already_has = dal::list_character_feats(self.conn, character_id)?
            .iter()
            .any(|f| f.feat_name.to_lowercase() == wanted);
        if !repeatable && already_has {
            return Err(ServiceError::Validation(format!(
                "Character already has {}",
                catalog_feat.name
            )));
        }

        let id = Uuid::new_v4().to_string();
        let new_feat = NewCharacterFeat::new(
            &id,
            character_id,
            &catalog_feat.name,
            &catalog_feat.source,
            source_type,
        );
        dal::insert_character_feat(self.conn, &new_feat)?;

        Ok(CharacterFeat {
            id,
            character_id: character_id.to_string(),
            feat_name: catalog_feat.name,
            feat_source: catalog_feat.source,
            source_type: source_type.as_str().to_string(),
        })
    }

    /// Remove a feat from a character by name (case-insensitive).
    ///
    /// Removes every instance of the feat, so a repeatable feat taken twice is
    /// removed twice. Returns how many were removed.
    pub fn remove_feat(&mut self, character_id: &str, feat_name: &str) -> ServiceResult<usize> {
        let wanted = feat_name.to_lowercase();
        let matching: Vec<_> = self
            .list_feats(character_id)?
            .into_iter()
            .filter(|f| f.feat_name.to_lowercase() == wanted)
            .collect();
        if matching.is_empty() {
            return Err(ServiceError::Validation(format!(
                "Character doesn't have {}",
                feat_name
            )));
        }
        for feat in &matching {
            dal::delete_character_feat(self.conn, &feat.id)?;
        }
        Ok(matching.len())
    }

    /// Toggle a spell's prepared status.
    pub fn toggle_spell_prepared(&mut self, spell_id: &str) -> ServiceResult<CharacterSpell> {
        let spell = dal::get_character_spell_optional(self.conn, spell_id)?
            .ok_or_else(|| ServiceError::not_found("CharacterSpell", spell_id))?;

        let new_prepared = !spell.is_prepared();
        let update = UpdateCharacterSpell::set_prepared(new_prepared);
        dal::update_character_spell(self.conn, spell_id, &update)?;

        Ok(CharacterSpell {
            prepared: if new_prepared { 1 } else { 0 },
            ..spell
        })
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
    
    
    use crate::test_utils::setup_test_db;
    use super::test_support::create_test_campaign;

    #[test]
    fn test_add_to_inventory() {
        let mut conn = setup_test_db();
        let campaign_id = create_test_campaign(&mut conn);

        let mut service = CharacterService::new(&mut conn);

        let input = CreateCharacterInput::new_pc(Some(&campaign_id), "Hero", "John");
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

        let input = CreateCharacterInput::new_pc(Some(&campaign_id), "Hero", "John");
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

        let input = CreateCharacterInput::new_pc(Some(&campaign_id), "Hero", "John");
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

        let input = CreateCharacterInput::new_pc(Some(&campaign_id), "Hero", "John");
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

        let input = CreateCharacterInput::new_pc(Some(&campaign_id), "Hero", "John");
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

    // ── Proficiency Extraction Tests ────────────────────────────────

    #[test]
    fn test_extract_keyed_proficiencies_boolean_keys() {
        let items = vec![serde_json::json!({"perception": true, "stealth": true})];
        let profs = extract_keyed_proficiencies(&items, ProficiencyType::Skill);
        let names: Vec<&str> = profs.iter().map(|p| p.name.as_str()).collect();
        assert!(names.contains(&"Perception"));
        assert!(names.contains(&"Stealth"));
        assert_eq!(profs.len(), 2);
    }

    #[test]
    fn test_extract_keyed_proficiencies_ignores_choice_keys() {
        let items = vec![serde_json::json!({"choose": {"from": ["a", "b"]}, "any": 2})];
        let profs = extract_keyed_proficiencies(&items, ProficiencyType::Skill);
        assert_eq!(profs.len(), 0);
    }

    #[test]
    fn test_extract_keyed_proficiencies_ignores_any_prefix_keys() {
        let items = vec![serde_json::json!({"anyStandard": 2, "anyMusicalInstrument": 1})];
        let profs = extract_keyed_proficiencies(&items, ProficiencyType::Language);
        assert_eq!(profs.len(), 0);
    }

    #[test]
    fn test_extract_keyed_proficiencies_string_values() {
        let items = vec![
            serde_json::Value::String("light".to_string()),
            serde_json::Value::String("medium".to_string()),
            serde_json::Value::String("{@item shield|phb}".to_string()),
        ];
        let profs = extract_keyed_proficiencies(&items, ProficiencyType::Armor);
        assert_eq!(profs.len(), 3);
        assert_eq!(profs[0].name, "Light");
        assert_eq!(profs[1].name, "Medium");
    }

    #[test]
    fn test_extract_keyed_proficiencies_strips_source_suffix() {
        let items = vec![serde_json::json!({"longsword|phb": true, "shortbow|phb": true})];
        let profs = extract_keyed_proficiencies(&items, ProficiencyType::Weapon);
        let names: Vec<&str> = profs.iter().map(|p| p.name.as_str()).collect();
        assert!(names.contains(&"Longsword"));
        assert!(names.contains(&"Shortbow"));
    }

    #[test]
    fn test_extract_class_proficiencies_saves() {
        let class_data = serde_json::json!({
            "proficiency": ["str", "con"],
            "startingProficiencies": {
                "armor": ["light", "medium", "heavy", "{@item shield|phb}"],
                "weapons": [{"simple weapons": true, "martial weapons": true}]
            }
        });
        let profs = extract_class_proficiencies(&class_data, &[]);
        let saves: Vec<&str> = profs
            .iter()
            .filter(|p| matches!(p.prof_type, ProficiencyType::Save))
            .map(|p| p.name.as_str())
            .collect();
        assert!(saves.contains(&"Strength"));
        assert!(saves.contains(&"Constitution"));
    }

    #[test]
    fn test_extract_class_proficiencies_with_skills() {
        let class_data = serde_json::json!({
            "proficiency": ["dex", "int"],
            "startingProficiencies": {}
        });
        let selected = vec!["Stealth".to_string(), "Perception".to_string()];
        let profs = extract_class_proficiencies(&class_data, &selected);
        let skills: Vec<&str> = profs
            .iter()
            .filter(|p| matches!(p.prof_type, ProficiencyType::Skill))
            .map(|p| p.name.as_str())
            .collect();
        assert!(skills.contains(&"Stealth"));
        assert!(skills.contains(&"Perception"));
    }

    #[test]
    fn test_extract_background_proficiencies() {
        let bg_data = serde_json::json!({
            "skillProficiencies": [{"insight": true, "religion": true}],
            "languageProficiencies": [{"common": true, "elvish": true}],
            "toolProficiencies": [{"disguise kit": true}]
        });
        let profs = extract_background_proficiencies(&bg_data);
        let skills: Vec<&str> = profs
            .iter()
            .filter(|p| matches!(p.prof_type, ProficiencyType::Skill))
            .map(|p| p.name.as_str())
            .collect();
        let langs: Vec<&str> = profs
            .iter()
            .filter(|p| matches!(p.prof_type, ProficiencyType::Language))
            .map(|p| p.name.as_str())
            .collect();
        let tools: Vec<&str> = profs
            .iter()
            .filter(|p| matches!(p.prof_type, ProficiencyType::Tool))
            .map(|p| p.name.as_str())
            .collect();
        assert!(skills.contains(&"Insight"));
        assert!(skills.contains(&"Religion"));
        assert!(langs.contains(&"Common"));
        assert!(langs.contains(&"Elvish"));
        assert!(tools.contains(&"Disguise kit"));
    }

    #[test]
    fn test_extract_race_proficiencies() {
        let race_data = serde_json::json!({
            "skillProficiencies": [{"perception": true}],
            "languageProficiencies": [{"common": true, "elvish": true}],
            "weaponProficiencies": [{"longsword|phb": true, "shortbow|phb": true}]
        });
        let profs = extract_race_proficiencies(&race_data);
        let skills: Vec<&str> = profs
            .iter()
            .filter(|p| matches!(p.prof_type, ProficiencyType::Skill))
            .map(|p| p.name.as_str())
            .collect();
        let weapons: Vec<&str> = profs
            .iter()
            .filter(|p| matches!(p.prof_type, ProficiencyType::Weapon))
            .map(|p| p.name.as_str())
            .collect();
        assert!(skills.contains(&"Perception"));
        assert!(weapons.contains(&"Longsword"));
        assert!(weapons.contains(&"Shortbow"));
    }

    #[test]
    fn test_extract_multiclass_proficiencies() {
        let class_data = serde_json::json!({
            "multiclassing": {
                "proficienciesGained": {
                    "armor": ["light", "medium"],
                    "weapons": [{"simple weapons": true}]
                }
            }
        });
        let profs = extract_multiclass_proficiencies(&class_data);
        let armor: Vec<&str> = profs
            .iter()
            .filter(|p| matches!(p.prof_type, ProficiencyType::Armor))
            .map(|p| p.name.as_str())
            .collect();
        let weapons: Vec<&str> = profs
            .iter()
            .filter(|p| matches!(p.prof_type, ProficiencyType::Weapon))
            .map(|p| p.name.as_str())
            .collect();
        assert!(armor.contains(&"Light"));
        assert!(armor.contains(&"Medium"));
        assert!(weapons.contains(&"Simple weapons"));
    }

    #[test]
    fn test_extract_multiclass_proficiencies_no_multiclass() {
        let class_data = serde_json::json!({});
        let profs = extract_multiclass_proficiencies(&class_data);
        assert_eq!(profs.len(), 0);
    }

    #[test]
    fn test_capitalize_proficiency() {
        assert_eq!(capitalize_proficiency("perception"), "Perception");
        assert_eq!(capitalize_proficiency("stealth"), "Stealth");
        assert_eq!(capitalize_proficiency(""), "");
        assert_eq!(capitalize_proficiency("a"), "A");
    }

    // -- Feats (MIMIR-T-0658) -------------------------------------------------

    fn feat_fixture() -> (SqliteConnection, String) {
        use crate::dal::catalog::insert_feat;
        use crate::models::catalog::NewFeat;

        let mut conn = crate::test_utils::setup_test_db_with_sources();
        insert_feat(
            &mut conn,
            &NewFeat::new("Alert", "PHB", r#"{"name":"Alert"}"#),
        )
        .expect("insert Alert");
        insert_feat(
            &mut conn,
            &NewFeat::new(
                "Elemental Adept",
                "PHB",
                r#"{"name":"Elemental Adept","repeatable":true}"#,
            ),
        )
        .expect("insert Elemental Adept");

        let campaign_id = create_test_campaign(&mut conn);
        let mut service = CharacterService::new(&mut conn);
        let character = service
            .create(CreateCharacterInput::new_pc(
                Some(&campaign_id),
                "Hero",
                "John",
            ))
            .expect("create character");
        (conn, character.id)
    }

    #[test]
    fn test_add_feat_uses_catalog_name_and_source_type() {
        let (mut conn, character_id) = feat_fixture();
        let mut service = CharacterService::new(&mut conn);

        let feat = service
            .add_feat(&character_id, "alert", "PHB", FeatSourceType::Asi)
            .expect("add feat");
        assert_eq!(
            feat.feat_name, "Alert",
            "stored with the catalog's spelling"
        );
        assert_eq!(feat.feat_source, "PHB");
        assert_eq!(feat.source_type, "asi");

        let feats = service.list_feats(&character_id).expect("list feats");
        assert_eq!(feats.len(), 1);
        assert_eq!(feats[0].id, feat.id);
    }

    #[test]
    fn test_add_feat_rejects_duplicate_unless_repeatable() {
        let (mut conn, character_id) = feat_fixture();
        let mut service = CharacterService::new(&mut conn);

        service
            .add_feat(&character_id, "Alert", "PHB", FeatSourceType::Asi)
            .expect("first add");
        let err = service
            .add_feat(&character_id, "ALERT", "PHB", FeatSourceType::Bonus)
            .expect_err("duplicate must be rejected");
        assert!(matches!(err, ServiceError::Validation(ref m) if m.contains("already has")));

        service
            .add_feat(&character_id, "Elemental Adept", "PHB", FeatSourceType::Asi)
            .expect("first Elemental Adept");
        service
            .add_feat(&character_id, "Elemental Adept", "PHB", FeatSourceType::Asi)
            .expect("repeatable feat may be taken again");

        assert_eq!(service.list_feats(&character_id).unwrap().len(), 3);
    }

    #[test]
    fn test_add_feat_duplicate_check_ignores_case_of_existing_rows() {
        let (mut conn, character_id) = feat_fixture();

        // A level-up stores the feat name as typed, e.g. lowercase.
        let id = Uuid::new_v4().to_string();
        dal::insert_character_feat(
            &mut conn,
            &NewCharacterFeat::new(&id, &character_id, "alert", "PHB", FeatSourceType::Asi),
        )
        .unwrap();

        let mut service = CharacterService::new(&mut conn);
        let err = service
            .add_feat(&character_id, "Alert", "PHB", FeatSourceType::Asi)
            .expect_err("lowercase row must count as a duplicate");
        assert!(matches!(err, ServiceError::Validation(ref m) if m.contains("already has")));
    }

    #[test]
    fn test_add_feat_rejects_catalog_miss_and_missing_character() {
        let (mut conn, character_id) = feat_fixture();
        let mut service = CharacterService::new(&mut conn);

        let err = service
            .add_feat(&character_id, "Not A Feat", "PHB", FeatSourceType::Asi)
            .expect_err("unknown feat");
        assert!(matches!(err, ServiceError::Validation(ref m) if m.contains("not in the catalog")));

        let err = service
            .add_feat(&character_id, "Alert", "XGE", FeatSourceType::Asi)
            .expect_err("right name, wrong source");
        assert!(matches!(err, ServiceError::Validation(_)));

        let err = service
            .add_feat("no-such-character", "Alert", "PHB", FeatSourceType::Asi)
            .expect_err("missing character");
        assert!(matches!(err, ServiceError::NotFound { .. }));

        let err = service
            .list_feats("no-such-character")
            .expect_err("list on missing character");
        assert!(matches!(err, ServiceError::NotFound { .. }));

        assert!(service.list_feats(&character_id).unwrap().is_empty());
    }

    #[test]
    fn test_remove_feat_is_case_insensitive_and_reports_misses() {
        let (mut conn, character_id) = feat_fixture();
        let mut service = CharacterService::new(&mut conn);

        service
            .add_feat(&character_id, "Alert", "PHB", FeatSourceType::Asi)
            .unwrap();
        service
            .add_feat(&character_id, "Elemental Adept", "PHB", FeatSourceType::Asi)
            .unwrap();
        service
            .add_feat(&character_id, "Elemental Adept", "PHB", FeatSourceType::Asi)
            .unwrap();

        assert_eq!(service.remove_feat(&character_id, "alert").unwrap(), 1);
        assert_eq!(
            service
                .remove_feat(&character_id, "elemental adept")
                .unwrap(),
            2,
            "every instance of a repeatable feat is removed"
        );
        assert!(service.list_feats(&character_id).unwrap().is_empty());

        let err = service
            .remove_feat(&character_id, "Alert")
            .expect_err("nothing left to remove");
        assert!(matches!(err, ServiceError::Validation(ref m) if m.contains("doesn't have")));
    }
}
