//! Item Attunement Models
//!
//! Models for class-specific item attunement requirements.
//! Some magic items require attunement by a specific class (e.g., "requires
//! attunement by a Cleric or Paladin").

use crate::schema::item_attunement_classes;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

/// An item-class attunement association.
#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize, Deserialize)]
#[diesel(table_name = item_attunement_classes)]
#[diesel(primary_key(id))]
pub struct ItemAttunementClass {
    /// Unique database ID.
    pub id: Option<i32>,
    /// Foreign key to the item.
    pub item_id: i32,
    /// The class name that can attune to this item.
    pub class_name: String,
}

/// Data for inserting a new item attunement class requirement.
#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = item_attunement_classes)]
pub struct NewItemAttunementClass<'a> {
    pub item_id: i32,
    pub class_name: &'a str,
}

impl<'a> NewItemAttunementClass<'a> {
    /// Create a new item attunement class requirement.
    pub fn new(item_id: i32, class_name: &'a str) -> Self {
        Self { item_id, class_name }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_item_attunement_class() {
        let attunement = NewItemAttunementClass::new(1, "Cleric");
        assert_eq!(attunement.item_id, 1);
        assert_eq!(attunement.class_name, "Cleric");
    }
}
