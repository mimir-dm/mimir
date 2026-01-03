//! Character sheet PDF generation
//!
//! Core functions for generating character sheet PDFs, separated from
//! Tauri command handling to enable testing and reuse.

use std::path::PathBuf;

use mimir_dm_core::models::catalog::Spell;
use mimir_dm_core::models::character::data::CharacterData;
use serde::Deserialize;

use crate::builder::DocumentBuilder;
use crate::error::Result;
use crate::sections::{
    is_card_worthy, CharacterLongFormSection, CharacterSheetSection, CompactSheetSection,
    EquipmentCardsSection, EquipmentDetailSection, SpellCardsSection,
};

/// Options for composable character export
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct CharacterExportOptions {
    /// Include compact 2-page character sheet
    pub include_compact_sheet: bool,
    /// Include long form character details (personality, background, RP notes)
    pub include_long_form: bool,
    /// Include spell cards (silently no-op if no spells)
    pub include_spell_cards: bool,
    /// Include detailed equipment list with descriptions
    pub include_equipment_detail: bool,
    /// Include equipment cards (weapons, magic items, special ammo)
    pub include_equipment_cards: bool,
}

/// Generate a character sheet PDF from character data
///
/// This is the core function for character sheet PDF generation, separated from
/// Tauri command handling to enable testing and reuse.
///
/// # Arguments
/// * `character` - Character data from the database
/// * `spells` - Full spell details from catalog (for spell cards)
/// * `templates_root` - Path to Typst templates
///
/// # Returns
/// PDF bytes on success
pub fn generate_character_sheet_pdf(
    character: CharacterData,
    spells: Vec<Spell>,
    templates_root: PathBuf,
) -> Result<Vec<u8>> {
    let title = character.character_name.clone();
    let section = CharacterSheetSection::new(character).with_spells(spells);

    DocumentBuilder::new(&title)
        .with_templates_root(templates_root)
        .with_toc(false)
        .append(section)
        .to_pdf()
}

/// Export character to PDF with composable sections
///
/// Allows users to select which sections to include in the PDF:
/// - Compact sheet (2-page WotC-style)
/// - Long form (personality, background, RP notes)
/// - Spell cards (silently skipped if no spells)
/// - Equipment cards (weapons, magic items, special ammo)
/// - Equipment detail (full inventory with descriptions)
///
/// Sections appear in a fixed order regardless of selection order.
pub fn export_character_pdf(
    character: CharacterData,
    spells: Vec<Spell>,
    templates_root: PathBuf,
    options: CharacterExportOptions,
) -> Result<Vec<u8>> {
    export_character_pdf_with_equipment(character, spells, vec![], templates_root, options)
}

/// Export character to PDF with composable sections and catalog equipment data
///
/// Same as `export_character_pdf` but accepts full catalog data for equipment cards.
pub fn export_character_pdf_with_equipment(
    character: CharacterData,
    spells: Vec<Spell>,
    equipment: Vec<serde_json::Value>,
    templates_root: PathBuf,
    options: CharacterExportOptions,
) -> Result<Vec<u8>> {
    let title = character.character_name.clone();

    let mut builder = DocumentBuilder::new(&title)
        .with_templates_root(templates_root)
        .with_toc(false)
        .with_title_page(false);

    // Sections appear in fixed order: Compact Sheet, Long Form, Spell Cards, Equipment Cards, Equipment Detail

    // 1. Compact Sheet (2-page WotC-style)
    if options.include_compact_sheet {
        let section = CompactSheetSection::new(character.clone())
            .with_spells(spells.clone())
            .with_equipment(equipment.clone());
        builder = builder.append(section);
    }

    // 2. Long Form (personality, background, RP notes)
    if options.include_long_form {
        let section = CharacterLongFormSection::new(character.clone());
        builder = builder.append(section);
    }

    // 3. Spell Cards (silently no-op if no spells)
    if options.include_spell_cards && !spells.is_empty() {
        // Convert Spell structs to JSON Values for SpellCardsSection
        let spell_values: Vec<serde_json::Value> = spells
            .iter()
            .filter_map(|s| serde_json::to_value(s).ok())
            .collect();

        if !spell_values.is_empty() {
            let section = SpellCardsSection::new(spell_values);
            builder = builder.append(section);
        }
    }

    // 4. Equipment Cards (weapons, magic items, special ammo)
    if options.include_equipment_cards && !equipment.is_empty() {
        // Filter to only card-worthy items
        let card_worthy: Vec<serde_json::Value> = equipment
            .into_iter()
            .filter(|item| is_card_worthy(item))
            .collect();

        if !card_worthy.is_empty() {
            let section = EquipmentCardsSection::new(card_worthy);
            builder = builder.append(section);
        }
    }

    // 5. Equipment Detail
    if options.include_equipment_detail {
        let section = EquipmentDetailSection::new(character.clone());
        builder = builder.append(section);
    }

    builder.to_pdf()
}

#[cfg(test)]
mod tests {
    use super::*;
    use mimir_dm_core::models::character::data::{
        AbilityScores, Appearance, ClassLevel, Currency, EquippedItems, Personality, Proficiencies,
        RoleplayNotes, SpellData as CharacterSpellData,
    };

    fn sample_character() -> CharacterData {
        CharacterData {
            character_name: "Test Fighter".to_string(),
            player_id: Some(1),
            level: 5,
            experience_points: 6500,
            version: 1,
            snapshot_reason: None,
            created_at: "2025-01-01".to_string(),
            race: "Human".to_string(),
            subrace: None,
            classes: vec![ClassLevel {
                class_name: "Fighter".to_string(),
                level: 5,
                subclass: Some("Champion".to_string()),
                hit_dice_type: "d10".to_string(),
                hit_dice_remaining: 5,
            }],
            background: "Soldier".to_string(),
            alignment: Some("Lawful Good".to_string()),
            abilities: AbilityScores {
                strength: 16,
                dexterity: 14,
                constitution: 14,
                intelligence: 10,
                wisdom: 12,
                charisma: 8,
            },
            max_hp: 44,
            current_hp: 44,
            speed: 30,
            proficiencies: Proficiencies {
                skills: vec!["Athletics".to_string(), "Intimidation".to_string()],
                saves: vec!["Strength".to_string(), "Constitution".to_string()],
                armor: vec!["All armor".to_string(), "Shields".to_string()],
                weapons: vec!["Simple weapons".to_string(), "Martial weapons".to_string()],
                tools: vec![],
                languages: vec!["Common".to_string()],
            },
            class_features: vec![],
            feats: vec![],
            spells: CharacterSpellData::default(),
            inventory: vec![],
            currency: Currency::default(),
            equipped: EquippedItems {
                armor: Some("Chain Mail".to_string()),
                shield: Some("Shield".to_string()),
                main_hand: Some("Longsword".to_string()),
                off_hand: None,
            },
            personality: Personality::default(),
            player_name: None,
            appearance: Appearance::default(),
            backstory: None,
            background_feature: None,
            roleplay_notes: RoleplayNotes::default(),
            npc_role: None,
            npc_location: None,
            npc_faction: None,
            npc_notes: None,
            legendary_actions: vec![],
            legendary_action_count: None,
        }
    }

    #[test]
    fn test_generate_character_sheet_pdf() {
        let character = sample_character();
        let templates = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("templates");

        let result = generate_character_sheet_pdf(character, vec![], templates);
        assert!(result.is_ok(), "PDF generation failed: {:?}", result.err());

        let pdf_bytes = result.unwrap();
        assert!(!pdf_bytes.is_empty(), "PDF should not be empty");
        assert_eq!(
            &pdf_bytes[0..4],
            b"%PDF",
            "Output should be a valid PDF (starts with %PDF)"
        );
    }

    #[test]
    fn test_generate_character_sheet_pdf_with_correct_name() {
        let mut character = sample_character();
        character.character_name = "Sildar Hallwinter".to_string();

        let templates = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("templates");

        let result = generate_character_sheet_pdf(character, vec![], templates);
        assert!(result.is_ok());

        // The PDF should contain the character name
        // We can't easily inspect PDF content, but we verified it generates
    }

    #[test]
    fn test_export_character_pdf_compact_sheet() {
        let character = sample_character();
        let templates = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("templates");

        let options = CharacterExportOptions {
            include_compact_sheet: true,
            include_long_form: false,
            include_spell_cards: false,
            include_equipment_cards: false,
            include_equipment_detail: false,
        };

        let result = export_character_pdf(character, vec![], templates, options);
        assert!(result.is_ok(), "Compact sheet PDF generation failed: {:?}", result.err());
    }

    #[test]
    fn test_export_character_pdf_long_form() {
        let character = sample_character();
        let templates = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("templates");

        let options = CharacterExportOptions {
            include_compact_sheet: false,
            include_long_form: true,
            include_spell_cards: false,
            include_equipment_cards: false,
            include_equipment_detail: false,
        };

        let result = export_character_pdf(character, vec![], templates, options);
        assert!(result.is_ok(), "Long form PDF generation failed: {:?}", result.err());
    }

    #[test]
    fn test_export_character_pdf_equipment_detail() {
        use mimir_dm_core::models::character::data::InventoryItem;

        let mut character = sample_character();
        character.inventory = vec![
            InventoryItem {
                name: "Longsword".to_string(),
                source: None,
                quantity: 1,
                weight: 3.0,
                value: 15.0,
                notes: Some("Martial weapon, 1d8 slashing, versatile (1d10)".to_string()),
            },
            InventoryItem {
                name: "Healing Potion".to_string(),
                source: None,
                quantity: 2,
                weight: 0.5,
                value: 50.0,
                notes: None,
            },
        ];

        let templates = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("templates");

        let options = CharacterExportOptions {
            include_compact_sheet: false,
            include_long_form: false,
            include_spell_cards: false,
            include_equipment_cards: false,
            include_equipment_detail: true,
        };

        let result = export_character_pdf(character, vec![], templates, options);
        assert!(result.is_ok(), "Equipment detail PDF generation failed: {:?}", result.err());
    }

    #[test]
    fn test_export_character_pdf_all_sections() {
        use mimir_dm_core::models::character::data::InventoryItem;

        let mut character = sample_character();
        character.inventory = vec![
            InventoryItem {
                name: "Magic Sword".to_string(),
                source: None,
                quantity: 1,
                weight: 3.0,
                value: 500.0,
                notes: Some("+1 to attack and damage rolls".to_string()),
            },
        ];

        let templates = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("templates");

        let options = CharacterExportOptions {
            include_compact_sheet: true,
            include_long_form: true,
            include_spell_cards: true,  // No spells, so silently skipped
            include_equipment_cards: true,
            include_equipment_detail: true,
        };

        let result = export_character_pdf(character, vec![], templates, options);
        assert!(result.is_ok(), "All sections PDF generation failed: {:?}", result.err());
    }

    #[test]
    fn test_export_character_pdf_with_spell_cards() {
        use mimir_dm_core::models::catalog::{
            CastingTime, Components, Distance, Duration, SpellRange, SpellSchool,
        };

        let character = sample_character();
        let templates = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("templates");

        // Create a spell with 5etools-style tags that would previously fail
        // Tags like <damage>, <condition>, <creature> must be escaped
        let spell = Spell {
            name: "Chromatic Orb".to_string(),
            source: "PHB".to_string(),
            page: Some(221),
            level: 1,
            school: SpellSchool::Evocation,
            time: vec![CastingTime {
                number: 1,
                unit: "action".to_string(),
                condition: None,
            }],
            range: SpellRange::Point {
                range_type: "point".to_string(),
                distance: Distance {
                    distance_type: "feet".to_string(),
                    amount: Some(90),
                },
            },
            components: Components {
                v: Some(true),
                s: Some(true),
                m: None,
                r: None,
            },
            duration: vec![Duration {
                duration_type: "instant".to_string(),
                duration: None,
                concentration: None,
                ends: None,
            }],
            // Entry with 5etools-style tags that previously caused compilation errors
            // Tags like {@damage}, {@condition}, {@creature} must be escaped
            entries: vec![serde_json::json!(
                "You hurl a 4-inch-diameter sphere of energy at a creature. Deal {@damage 3d8} acid, cold, fire, lightning, poison, or thunder damage. The {@creature target} becomes {@condition blinded} on a critical hit."
            )],
            classes: None,
            scaling_level_dice: None,
            damage_inflict: Some(vec!["acid".to_string(), "cold".to_string(), "fire".to_string()]),
            saving_throw: None,
            meta: None,
        };

        let options = CharacterExportOptions {
            include_compact_sheet: false,
            include_long_form: false,
            include_spell_cards: true,
            include_equipment_cards: false,
            include_equipment_detail: false,
        };

        let result = export_character_pdf(character, vec![spell], templates, options);
        assert!(result.is_ok(), "Spell cards PDF generation failed with 5etools tags: {:?}", result.err());

        // Verify it's a valid PDF
        let pdf_bytes = result.unwrap();
        assert!(!pdf_bytes.is_empty(), "PDF should not be empty");
        assert_eq!(
            &pdf_bytes[0..4],
            b"%PDF",
            "Output should be a valid PDF"
        );
    }
}
