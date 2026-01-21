//! Spell List Models
//!
//! Models for spell-class and spell-subclass relationships.
//! These represent the many-to-many relationships between spells and the
//! classes/subclasses that have access to them.

use crate::schema::{spell_classes, spell_subclasses};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

/// A spell-class association indicating a spell is on a class's spell list.
#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize, Deserialize)]
#[diesel(table_name = spell_classes)]
#[diesel(primary_key(id))]
pub struct SpellClass {
    /// Unique database ID.
    pub id: Option<i32>,
    /// Foreign key to the spell.
    pub spell_id: i32,
    /// The class name (e.g., "Wizard", "Cleric").
    pub class_name: String,
    /// Source book code where this association is defined.
    pub source: String,
}

/// Data for inserting a new spell-class association.
#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = spell_classes)]
pub struct NewSpellClass<'a> {
    pub spell_id: i32,
    pub class_name: &'a str,
    pub source: &'a str,
}

impl<'a> NewSpellClass<'a> {
    /// Create a new spell-class association.
    pub fn new(spell_id: i32, class_name: &'a str, source: &'a str) -> Self {
        Self {
            spell_id,
            class_name,
            source,
        }
    }
}

/// A spell-subclass association indicating a spell is on a subclass's expanded spell list.
#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize, Deserialize)]
#[diesel(table_name = spell_subclasses)]
#[diesel(primary_key(id))]
pub struct SpellSubclass {
    /// Unique database ID.
    pub id: Option<i32>,
    /// Foreign key to the spell.
    pub spell_id: i32,
    /// The subclass name (e.g., "Arcane Trickster", "Eldritch Knight").
    pub subclass_name: String,
    /// The parent class name (e.g., "Rogue", "Fighter").
    pub class_name: String,
    /// Source book code where this association is defined.
    pub source: String,
}

/// Data for inserting a new spell-subclass association.
#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = spell_subclasses)]
pub struct NewSpellSubclass<'a> {
    pub spell_id: i32,
    pub subclass_name: &'a str,
    pub class_name: &'a str,
    pub source: &'a str,
}

impl<'a> NewSpellSubclass<'a> {
    /// Create a new spell-subclass association.
    pub fn new(
        spell_id: i32,
        subclass_name: &'a str,
        class_name: &'a str,
        source: &'a str,
    ) -> Self {
        Self {
            spell_id,
            subclass_name,
            class_name,
            source,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_spell_class() {
        let spell_class = NewSpellClass::new(1, "Wizard", "PHB");
        assert_eq!(spell_class.spell_id, 1);
        assert_eq!(spell_class.class_name, "Wizard");
        assert_eq!(spell_class.source, "PHB");
    }

    #[test]
    fn test_new_spell_subclass() {
        let spell_subclass = NewSpellSubclass::new(1, "Arcane Trickster", "Rogue", "PHB");
        assert_eq!(spell_subclass.spell_id, 1);
        assert_eq!(spell_subclass.subclass_name, "Arcane Trickster");
        assert_eq!(spell_subclass.class_name, "Rogue");
        assert_eq!(spell_subclass.source, "PHB");
    }
}
