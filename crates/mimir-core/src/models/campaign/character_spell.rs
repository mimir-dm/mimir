//! CharacterSpell Model
//!
//! Tracks spells known or prepared by characters.

use crate::schema::character_spells;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

/// A spell known or prepared by a character.
#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize, Deserialize)]
#[diesel(table_name = character_spells)]
pub struct CharacterSpell {
    /// Unique ID (UUID)
    pub id: String,
    /// Character who knows this spell
    pub character_id: String,
    /// Spell name
    pub spell_name: String,
    /// Spell source (e.g., "PHB", "XGE")
    pub spell_source: String,
    /// Class that grants this spell
    pub source_class: String,
    /// Whether the spell is prepared
    pub prepared: i32,
}

impl CharacterSpell {
    /// Check if this spell is prepared.
    pub fn is_prepared(&self) -> bool {
        self.prepared != 0
    }
}

/// Data for inserting a new character spell.
#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = character_spells)]
pub struct NewCharacterSpell<'a> {
    pub id: &'a str,
    pub character_id: &'a str,
    pub spell_name: &'a str,
    pub spell_source: &'a str,
    pub source_class: &'a str,
    pub prepared: i32,
}

impl<'a> NewCharacterSpell<'a> {
    /// Create a new spell entry.
    pub fn new(
        id: &'a str,
        character_id: &'a str,
        spell_name: &'a str,
        spell_source: &'a str,
        source_class: &'a str,
    ) -> Self {
        Self {
            id,
            character_id,
            spell_name,
            spell_source,
            source_class,
            prepared: 0,
        }
    }

    /// Mark as prepared.
    pub fn prepared(mut self) -> Self {
        self.prepared = 1;
        self
    }
}

/// Data for updating a character spell.
#[derive(Debug, Clone, Default, AsChangeset)]
#[diesel(table_name = character_spells)]
pub struct UpdateCharacterSpell {
    pub prepared: Option<i32>,
}

impl UpdateCharacterSpell {
    /// Set prepared status.
    pub fn set_prepared(is_prepared: bool) -> Self {
        Self {
            prepared: Some(if is_prepared { 1 } else { 0 }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_spell() {
        let spell = NewCharacterSpell::new("spell-1", "char-1", "Fireball", "PHB", "Wizard");
        assert_eq!(spell.spell_name, "Fireball");
        assert_eq!(spell.spell_source, "PHB");
        assert_eq!(spell.source_class, "Wizard");
        assert_eq!(spell.prepared, 0);
    }

    #[test]
    fn test_prepared_spell() {
        let spell = NewCharacterSpell::new("spell-1", "char-1", "Shield", "PHB", "Wizard")
            .prepared();
        assert_eq!(spell.prepared, 1);
    }

    #[test]
    fn test_update_prepared() {
        let prepare = UpdateCharacterSpell::set_prepared(true);
        assert_eq!(prepare.prepared, Some(1));

        let unprepare = UpdateCharacterSpell::set_prepared(false);
        assert_eq!(unprepare.prepared, Some(0));
    }
}
