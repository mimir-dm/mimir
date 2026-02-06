//! CharacterProficiency Model
//!
//! Tracks character proficiencies in skills, saves, tools, weapons, armor, and languages.

use crate::schema::character_proficiencies;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[cfg(test)]
use ts_rs::TS;

/// A proficiency held by a character.
#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize, Deserialize)]
#[cfg_attr(test, derive(TS))]
#[cfg_attr(test, ts(export, export_to = "bindings/"))]
#[diesel(table_name = character_proficiencies)]
pub struct CharacterProficiency {
    /// Unique ID (UUID)
    pub id: String,
    /// Character who has this proficiency
    pub character_id: String,
    /// Type of proficiency
    pub proficiency_type: String,
    /// Name of the proficiency
    pub name: String,
    /// Whether character has expertise (double proficiency)
    pub expertise: i32,
}

impl CharacterProficiency {
    /// Check if this is expertise (double proficiency).
    pub fn has_expertise(&self) -> bool {
        self.expertise != 0
    }

    /// Check if this is a skill proficiency.
    pub fn is_skill(&self) -> bool {
        self.proficiency_type == "skill"
    }

    /// Check if this is a saving throw proficiency.
    pub fn is_save(&self) -> bool {
        self.proficiency_type == "save"
    }

    /// Check if this is a tool proficiency.
    pub fn is_tool(&self) -> bool {
        self.proficiency_type == "tool"
    }

    /// Check if this is a weapon proficiency.
    pub fn is_weapon(&self) -> bool {
        self.proficiency_type == "weapon"
    }

    /// Check if this is an armor proficiency.
    pub fn is_armor(&self) -> bool {
        self.proficiency_type == "armor"
    }

    /// Check if this is a language.
    pub fn is_language(&self) -> bool {
        self.proficiency_type == "language"
    }
}

/// Type of proficiency.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProficiencyType {
    Skill,
    Save,
    Tool,
    Weapon,
    Armor,
    Language,
}

impl ProficiencyType {
    /// Convert to string for database storage.
    pub fn as_str(&self) -> &'static str {
        match self {
            ProficiencyType::Skill => "skill",
            ProficiencyType::Save => "save",
            ProficiencyType::Tool => "tool",
            ProficiencyType::Weapon => "weapon",
            ProficiencyType::Armor => "armor",
            ProficiencyType::Language => "language",
        }
    }
}

/// Data for inserting a new proficiency.
#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = character_proficiencies)]
pub struct NewCharacterProficiency<'a> {
    pub id: &'a str,
    pub character_id: &'a str,
    pub proficiency_type: &'a str,
    pub name: &'a str,
    pub expertise: i32,
}

impl<'a> NewCharacterProficiency<'a> {
    /// Create a new proficiency.
    pub fn new(
        id: &'a str,
        character_id: &'a str,
        proficiency_type: ProficiencyType,
        name: &'a str,
    ) -> Self {
        Self {
            id,
            character_id,
            proficiency_type: proficiency_type.as_str(),
            name,
            expertise: 0,
        }
    }

    /// Create a skill proficiency.
    pub fn skill(id: &'a str, character_id: &'a str, name: &'a str) -> Self {
        Self::new(id, character_id, ProficiencyType::Skill, name)
    }

    /// Create a saving throw proficiency.
    pub fn save(id: &'a str, character_id: &'a str, name: &'a str) -> Self {
        Self::new(id, character_id, ProficiencyType::Save, name)
    }

    /// Create a tool proficiency.
    pub fn tool(id: &'a str, character_id: &'a str, name: &'a str) -> Self {
        Self::new(id, character_id, ProficiencyType::Tool, name)
    }

    /// Create a weapon proficiency.
    pub fn weapon(id: &'a str, character_id: &'a str, name: &'a str) -> Self {
        Self::new(id, character_id, ProficiencyType::Weapon, name)
    }

    /// Create an armor proficiency.
    pub fn armor(id: &'a str, character_id: &'a str, name: &'a str) -> Self {
        Self::new(id, character_id, ProficiencyType::Armor, name)
    }

    /// Create a language.
    pub fn language(id: &'a str, character_id: &'a str, name: &'a str) -> Self {
        Self::new(id, character_id, ProficiencyType::Language, name)
    }

    /// Mark as expertise.
    pub fn with_expertise(mut self) -> Self {
        self.expertise = 1;
        self
    }
}

/// Data for updating a proficiency.
#[derive(Debug, Clone, Default, AsChangeset)]
#[diesel(table_name = character_proficiencies)]
pub struct UpdateCharacterProficiency {
    pub expertise: Option<i32>,
}

impl UpdateCharacterProficiency {
    /// Set expertise status.
    pub fn set_expertise(has_expertise: bool) -> Self {
        Self {
            expertise: Some(if has_expertise { 1 } else { 0 }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_skill_proficiency() {
        let prof = NewCharacterProficiency::skill("prof-1", "char-1", "Perception");
        assert_eq!(prof.proficiency_type, "skill");
        assert_eq!(prof.name, "Perception");
        assert_eq!(prof.expertise, 0);
    }

    #[test]
    fn test_new_save_proficiency() {
        let prof = NewCharacterProficiency::save("prof-1", "char-1", "Constitution");
        assert_eq!(prof.proficiency_type, "save");
        assert_eq!(prof.name, "Constitution");
    }

    #[test]
    fn test_skill_with_expertise() {
        let prof = NewCharacterProficiency::skill("prof-1", "char-1", "Stealth")
            .with_expertise();
        assert_eq!(prof.expertise, 1);
    }

    #[test]
    fn test_tool_proficiency() {
        let prof = NewCharacterProficiency::tool("prof-1", "char-1", "Thieves' Tools");
        assert_eq!(prof.proficiency_type, "tool");
    }

    #[test]
    fn test_weapon_proficiency() {
        let prof = NewCharacterProficiency::weapon("prof-1", "char-1", "Longbow");
        assert_eq!(prof.proficiency_type, "weapon");
    }

    #[test]
    fn test_armor_proficiency() {
        let prof = NewCharacterProficiency::armor("prof-1", "char-1", "Heavy Armor");
        assert_eq!(prof.proficiency_type, "armor");
    }

    #[test]
    fn test_language() {
        let prof = NewCharacterProficiency::language("prof-1", "char-1", "Elvish");
        assert_eq!(prof.proficiency_type, "language");
    }

    #[test]
    fn test_proficiency_type_as_str() {
        assert_eq!(ProficiencyType::Skill.as_str(), "skill");
        assert_eq!(ProficiencyType::Save.as_str(), "save");
        assert_eq!(ProficiencyType::Tool.as_str(), "tool");
        assert_eq!(ProficiencyType::Weapon.as_str(), "weapon");
        assert_eq!(ProficiencyType::Armor.as_str(), "armor");
        assert_eq!(ProficiencyType::Language.as_str(), "language");
    }

    #[test]
    fn test_update_expertise() {
        let add = UpdateCharacterProficiency::set_expertise(true);
        assert_eq!(add.expertise, Some(1));

        let remove = UpdateCharacterProficiency::set_expertise(false);
        assert_eq!(remove.expertise, Some(0));
    }
}
