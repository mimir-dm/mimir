//! Module frontmatter schema and parsing
//!
//! Provides structured YAML front matter parsing for module documents,
//! including references to monsters, NPCs, and items from the catalog.

use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

/// Complete frontmatter structure for module documents.
///
/// This represents the "machine interface" for modules - structured YAML
/// that can be parsed, validated, and synced to database tables.
///
/// # Example YAML
/// ```yaml
/// ---
/// id: module_overview
/// title: "Module 1: The Brittle Steel Mystery"
/// type: module_overview
/// level: module
/// purpose: Complete runnable adventure module
/// author: DM Name
///
/// module_number: 1
/// theme: "Mystery"
/// tone: "Dark Fantasy"
/// estimated_hours: 6
///
/// monsters:
///   - encounter: entrance_fight
///     name: "Guard"
///     source: MM
///     quantity: 2
///     notes: "Protect the entrance"
///   - encounter: boss_fight
///     name: "Hobgoblin Warlord"
///     source: MM
///     quantity: 1
///     notes: "Main antagonist"
///
/// npcs:
///   - role: quest_giver
///     name: "Elder Miriam"
///     source: campaign
///     location: "Village Square"
///     notes: "Provides the hook"
///   - role: antagonist
///     name: "Grimnar Goldbeard"
///     source: campaign
///     location: "Goldbeard Manor"
///     notes: "Secret villain"
///
/// items:
///   - location: boss_chamber
///     name: "+1 Longsword"
///     source: DMG
///     quantity: 1
///     notes: "Primary reward"
///   - location: hidden_cache
///     name: "Potion of Healing"
///     source: PHB
///     quantity: 3
///     notes: "Optional discovery"
///
/// variables:
///   - name: module_name
///     type: string
///     description: Name of the module
///     default: "[Module Name]"
///     required: true
/// ---
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleFrontmatter {
    /// Unique identifier for this module
    pub id: String,

    /// Human-readable title
    pub title: String,

    /// Template type (module_overview, module_mystery, etc.)
    #[serde(rename = "type")]
    pub module_type: String,

    /// Document level (always "module" for modules)
    #[serde(default = "default_level")]
    pub level: String,

    /// Brief description of the module's purpose
    #[serde(default)]
    pub purpose: Option<String>,

    /// Author of the module
    #[serde(default = "default_author")]
    pub author: String,

    // Module-specific metadata

    /// Module number in the campaign sequence
    #[serde(default)]
    pub module_number: Option<i32>,

    /// Theme of the module (Mystery, Adventure, Horror, etc.)
    #[serde(default)]
    pub theme: Option<String>,

    /// Tone of the module (Dark, Light, Serious, etc.)
    #[serde(default)]
    pub tone: Option<String>,

    /// Estimated hours of play time
    #[serde(default)]
    pub estimated_hours: Option<f32>,

    // Catalog references

    /// Monsters referenced in this module, organized by encounter
    #[serde(default)]
    pub monsters: Vec<MonsterReference>,

    /// NPCs referenced in this module, organized by role
    #[serde(default)]
    pub npcs: Vec<NpcReference>,

    /// Items/treasure referenced in this module, organized by location
    #[serde(default)]
    pub items: Vec<ItemReference>,

    /// Template variables for customization
    #[serde(default)]
    pub variables: Vec<ModuleVariable>,
}

fn default_level() -> String {
    "module".to_string()
}

fn default_author() -> String {
    "Mimir Team".to_string()
}

/// Reference to a monster from the catalog.
///
/// The `encounter` field groups monsters by their use in the adventure
/// (e.g., "entrance_fight", "boss_fight", "random_encounter").
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonsterReference {
    /// Encounter tag for grouping (matches Adventure Content sections)
    #[serde(default)]
    pub encounter: Option<String>,

    /// Monster name as it appears in the catalog
    pub name: String,

    /// Source book abbreviation (MM, PHB, DMG, etc.)
    pub source: String,

    /// Number of creatures in this encounter
    #[serde(default = "default_quantity")]
    pub quantity: i32,

    /// Notes about this monster's role in the encounter
    #[serde(default)]
    pub notes: Option<String>,
}

/// Reference to an NPC (from catalog or campaign-specific).
///
/// The `role` field categorizes NPCs by their narrative function
/// (e.g., "quest_giver", "antagonist", "ally", "informant").
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NpcReference {
    /// NPC role for categorization (quest_giver, antagonist, ally, etc.)
    #[serde(default)]
    pub role: Option<String>,

    /// NPC name
    pub name: String,

    /// Source: "campaign" for custom NPCs, or source book abbreviation
    pub source: String,

    /// Where this NPC is typically found
    #[serde(default)]
    pub location: Option<String>,

    /// Notes about this NPC's role in the module
    #[serde(default)]
    pub notes: Option<String>,
}

/// Reference to an item/treasure from the catalog.
///
/// The `location` field indicates where the item is found in the module
/// (e.g., "boss_chamber", "hidden_cache", "reward").
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemReference {
    /// Location where this item is found (matches Adventure Content sections)
    #[serde(default)]
    pub location: Option<String>,

    /// Item name as it appears in the catalog
    pub name: String,

    /// Source book abbreviation (DMG, PHB, etc.) or "campaign" for custom items
    pub source: String,

    /// Number of this item
    #[serde(default = "default_quantity")]
    pub quantity: i32,

    /// Notes about this item's role in the module
    #[serde(default)]
    pub notes: Option<String>,
}

fn default_quantity() -> i32 {
    1
}

/// Module variable definition with required default value.
///
/// Same structure as template variables for consistency.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleVariable {
    /// Variable name (without delimiters)
    pub name: String,

    /// Data type (string, number, boolean, list, object, etc.)
    #[serde(rename = "type")]
    pub var_type: String,

    /// Description of what this variable represents
    pub description: String,

    /// Default value for this variable
    pub default: JsonValue,

    /// Whether this variable is required
    #[serde(default = "default_true")]
    pub required: bool,
}

fn default_true() -> bool {
    true
}

impl ModuleFrontmatter {
    /// Parse frontmatter from a markdown document using gray_matter.
    ///
    /// Returns `None` if the document has no frontmatter or parsing fails.
    pub fn parse_from_markdown(content: &str) -> Option<Self> {
        let matter = gray_matter::Matter::<gray_matter::engine::YAML>::new();
        let parsed = matter.parse::<serde_yaml::Value>(content).ok()?;

        // If there's no frontmatter data, return None
        parsed.data.as_ref()?;

        // Convert the parsed YAML Value to our ModuleFrontmatter struct
        let yaml_value = parsed.data?;
        serde_yaml::from_value(yaml_value).ok()
    }

    /// Extract the content after frontmatter using gray_matter.
    ///
    /// Returns the original content if parsing fails.
    pub fn extract_content(markdown: &str) -> String {
        let matter = gray_matter::Matter::<gray_matter::engine::YAML>::new();
        match matter.parse::<serde_yaml::Value>(markdown) {
            Ok(parsed) => parsed.content,
            Err(_) => markdown.to_string(),
        }
    }

    /// Check if this frontmatter has any catalog references.
    pub fn has_catalog_references(&self) -> bool {
        !self.monsters.is_empty() || !self.npcs.is_empty() || !self.items.is_empty()
    }

    /// Get monster references grouped by encounter tag.
    pub fn monsters_by_encounter(&self) -> Vec<(Option<String>, Vec<&MonsterReference>)> {
        let mut groups: Vec<(Option<String>, Vec<&MonsterReference>)> = Vec::new();

        for monster in &self.monsters {
            let tag = monster.encounter.clone();
            if let Some(group) = groups.iter_mut().find(|(t, _)| *t == tag) {
                group.1.push(monster);
            } else {
                groups.push((tag, vec![monster]));
            }
        }

        groups
    }

    /// Get NPC references grouped by role.
    pub fn npcs_by_role(&self) -> Vec<(Option<String>, Vec<&NpcReference>)> {
        let mut groups: Vec<(Option<String>, Vec<&NpcReference>)> = Vec::new();

        for npc in &self.npcs {
            let role = npc.role.clone();
            if let Some(group) = groups.iter_mut().find(|(r, _)| *r == role) {
                group.1.push(npc);
            } else {
                groups.push((role, vec![npc]));
            }
        }

        groups
    }

    /// Get item references grouped by location.
    pub fn items_by_location(&self) -> Vec<(Option<String>, Vec<&ItemReference>)> {
        let mut groups: Vec<(Option<String>, Vec<&ItemReference>)> = Vec::new();

        for item in &self.items {
            let location = item.location.clone();
            if let Some(group) = groups.iter_mut().find(|(l, _)| *l == location) {
                group.1.push(item);
            } else {
                groups.push((location, vec![item]));
            }
        }

        groups
    }

    /// Convert to JSON for database storage.
    pub fn to_json(&self) -> serde_json::Result<String> {
        serde_json::to_string(self)
    }

    /// Extract defaults as a map from variables.
    pub fn defaults_map(&self) -> serde_json::Map<String, JsonValue> {
        let mut defaults = serde_json::Map::new();
        for var in &self.variables {
            defaults.insert(var.name.clone(), var.default.clone());
        }
        defaults
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_FRONTMATTER: &str = r#"---
id: module_mystery
title: "Module 1: The Brittle Steel Mystery"
type: module_mystery
level: module
purpose: Investigation-focused adventure
author: Test Author

module_number: 1
theme: "Mystery"
tone: "Dark Fantasy"
estimated_hours: 6

monsters:
  - encounter: entrance_fight
    name: Guard
    source: MM
    quantity: 2
    notes: "Protect the entrance"
  - encounter: boss_fight
    name: Hobgoblin Warlord
    source: MM
    quantity: 1
    notes: "Main antagonist"

npcs:
  - role: quest_giver
    name: Elder Miriam
    source: campaign
    location: Village Square
    notes: "Provides the hook"
  - role: antagonist
    name: Grimnar Goldbeard
    source: campaign
    location: Goldbeard Manor
    notes: "Secret villain"

items:
  - location: boss_chamber
    name: "+1 Longsword"
    source: DMG
    quantity: 1
    notes: "Primary reward"
  - location: hidden_cache
    name: Potion of Healing
    source: PHB
    quantity: 3
    notes: "Optional discovery"

variables:
  - name: module_name
    type: string
    description: Name of the module
    default: "The Brittle Steel Mystery"
    required: true
---

# Module Content Here

This is the adventure content after the frontmatter.
"#;

    #[test]
    fn test_parse_frontmatter() {
        let frontmatter = ModuleFrontmatter::parse_from_markdown(SAMPLE_FRONTMATTER);
        assert!(frontmatter.is_some());

        let fm = frontmatter.unwrap();
        assert_eq!(fm.id, "module_mystery");
        assert_eq!(fm.title, "Module 1: The Brittle Steel Mystery");
        assert_eq!(fm.module_type, "module_mystery");
        assert_eq!(fm.module_number, Some(1));
        assert_eq!(fm.theme, Some("Mystery".to_string()));
    }

    #[test]
    fn test_monster_references() {
        let fm = ModuleFrontmatter::parse_from_markdown(SAMPLE_FRONTMATTER).unwrap();
        assert_eq!(fm.monsters.len(), 2);

        let first_monster = &fm.monsters[0];
        assert_eq!(first_monster.encounter, Some("entrance_fight".to_string()));
        assert_eq!(first_monster.name, "Guard");
        assert_eq!(first_monster.source, "MM");
        assert_eq!(first_monster.quantity, 2);
    }

    #[test]
    fn test_npc_references() {
        let fm = ModuleFrontmatter::parse_from_markdown(SAMPLE_FRONTMATTER).unwrap();
        assert_eq!(fm.npcs.len(), 2);

        let quest_giver = &fm.npcs[0];
        assert_eq!(quest_giver.role, Some("quest_giver".to_string()));
        assert_eq!(quest_giver.name, "Elder Miriam");
        assert_eq!(quest_giver.source, "campaign");
    }

    #[test]
    fn test_item_references() {
        let fm = ModuleFrontmatter::parse_from_markdown(SAMPLE_FRONTMATTER).unwrap();
        assert_eq!(fm.items.len(), 2);

        let reward = &fm.items[0];
        assert_eq!(reward.location, Some("boss_chamber".to_string()));
        assert_eq!(reward.name, "+1 Longsword");
        assert_eq!(reward.source, "DMG");
    }

    #[test]
    fn test_monsters_by_encounter() {
        let fm = ModuleFrontmatter::parse_from_markdown(SAMPLE_FRONTMATTER).unwrap();
        let groups = fm.monsters_by_encounter();

        assert_eq!(groups.len(), 2);

        // Find entrance_fight group
        let entrance = groups
            .iter()
            .find(|(t, _)| *t == Some("entrance_fight".to_string()));
        assert!(entrance.is_some());
        assert_eq!(entrance.unwrap().1.len(), 1);
    }

    #[test]
    fn test_extract_content() {
        let content = ModuleFrontmatter::extract_content(SAMPLE_FRONTMATTER);
        assert!(content.contains("# Module Content Here"));
        assert!(content.contains("This is the adventure content"));
        assert!(!content.contains("monsters:"));
    }

    #[test]
    fn test_has_catalog_references() {
        let fm = ModuleFrontmatter::parse_from_markdown(SAMPLE_FRONTMATTER).unwrap();
        assert!(fm.has_catalog_references());
    }

    #[test]
    fn test_empty_frontmatter() {
        let empty = r#"---
id: empty_module
title: Empty Module
type: module_overview
level: module
---

# Content
"#;
        let fm = ModuleFrontmatter::parse_from_markdown(empty).unwrap();
        assert!(fm.monsters.is_empty());
        assert!(fm.npcs.is_empty());
        assert!(fm.items.is_empty());
        assert!(!fm.has_catalog_references());
    }
}
