//! CharacterClass Model
//!
//! Tracks character class levels for both single-class and multiclass characters.

use crate::schema::character_classes;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

/// A character's class level entry.
/// Characters can have multiple class entries for multiclassing.
#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize, Deserialize)]
#[diesel(table_name = character_classes)]
pub struct CharacterClass {
    /// Unique ID (UUID)
    pub id: String,
    /// Character this class belongs to
    pub character_id: String,
    /// Class name (e.g., "Fighter", "Wizard")
    pub class_name: String,
    /// Class source (e.g., "PHB", "XGE")
    pub class_source: String,
    /// Level in this class
    pub level: i32,
    /// Subclass name (if selected)
    pub subclass_name: Option<String>,
    /// Subclass source
    pub subclass_source: Option<String>,
    /// Whether this is the starting class (for proficiency and HP purposes)
    pub starting_class: i32,
}

impl CharacterClass {
    /// Check if this is the character's starting class.
    pub fn is_starting_class(&self) -> bool {
        self.starting_class != 0
    }

    /// Check if a subclass has been selected.
    pub fn has_subclass(&self) -> bool {
        self.subclass_name.is_some()
    }
}

/// Data for inserting a new character class.
#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = character_classes)]
pub struct NewCharacterClass<'a> {
    pub id: &'a str,
    pub character_id: &'a str,
    pub class_name: &'a str,
    pub class_source: &'a str,
    pub level: i32,
    pub subclass_name: Option<&'a str>,
    pub subclass_source: Option<&'a str>,
    pub starting_class: i32,
}

impl<'a> NewCharacterClass<'a> {
    /// Create a new starting class entry.
    pub fn starting(
        id: &'a str,
        character_id: &'a str,
        class_name: &'a str,
        class_source: &'a str,
    ) -> Self {
        Self {
            id,
            character_id,
            class_name,
            class_source,
            level: 1,
            subclass_name: None,
            subclass_source: None,
            starting_class: 1,
        }
    }

    /// Create a new multiclass entry.
    pub fn multiclass(
        id: &'a str,
        character_id: &'a str,
        class_name: &'a str,
        class_source: &'a str,
    ) -> Self {
        Self {
            id,
            character_id,
            class_name,
            class_source,
            level: 1,
            subclass_name: None,
            subclass_source: None,
            starting_class: 0,
        }
    }

    /// Set the class level.
    pub fn with_level(mut self, level: i32) -> Self {
        self.level = level;
        self
    }

    /// Set the subclass.
    pub fn with_subclass(mut self, name: &'a str, source: &'a str) -> Self {
        self.subclass_name = Some(name);
        self.subclass_source = Some(source);
        self
    }
}

/// Data for updating a character class.
#[derive(Debug, Clone, Default, AsChangeset)]
#[diesel(table_name = character_classes)]
pub struct UpdateCharacterClass<'a> {
    pub level: Option<i32>,
    pub subclass_name: Option<Option<&'a str>>,
    pub subclass_source: Option<Option<&'a str>>,
}

impl<'a> UpdateCharacterClass<'a> {
    /// Update the class level.
    pub fn set_level(level: i32) -> Self {
        Self {
            level: Some(level),
            ..Default::default()
        }
    }

    /// Set or update the subclass.
    pub fn set_subclass(name: &'a str, source: &'a str) -> Self {
        Self {
            subclass_name: Some(Some(name)),
            subclass_source: Some(Some(source)),
            ..Default::default()
        }
    }

    /// Update level and subclass together.
    pub fn set_level_and_subclass(level: i32, subclass_name: &'a str, subclass_source: &'a str) -> Self {
        Self {
            level: Some(level),
            subclass_name: Some(Some(subclass_name)),
            subclass_source: Some(Some(subclass_source)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_starting_class() {
        let class = NewCharacterClass::starting("class-1", "char-1", "Fighter", "PHB");
        assert_eq!(class.class_name, "Fighter");
        assert_eq!(class.starting_class, 1);
        assert_eq!(class.level, 1);
    }

    #[test]
    fn test_new_multiclass() {
        let class = NewCharacterClass::multiclass("class-1", "char-1", "Rogue", "PHB")
            .with_level(3);
        assert_eq!(class.class_name, "Rogue");
        assert_eq!(class.starting_class, 0);
        assert_eq!(class.level, 3);
    }

    #[test]
    fn test_with_subclass() {
        let class = NewCharacterClass::starting("class-1", "char-1", "Fighter", "PHB")
            .with_level(3)
            .with_subclass("Champion", "PHB");
        assert_eq!(class.subclass_name, Some("Champion"));
        assert_eq!(class.subclass_source, Some("PHB"));
    }

    #[test]
    fn test_update_level() {
        let update = UpdateCharacterClass::set_level(5);
        assert_eq!(update.level, Some(5));
        assert!(update.subclass_name.is_none());
    }

    #[test]
    fn test_update_subclass() {
        let update = UpdateCharacterClass::set_subclass("Battle Master", "PHB");
        assert_eq!(update.subclass_name, Some(Some("Battle Master")));
        assert_eq!(update.subclass_source, Some(Some("PHB")));
    }
}
