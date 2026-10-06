//! Character Service
//!
//! Business logic for character management (PCs and NPCs).

use diesel::SqliteConnection;

mod core;
mod feats;
mod inventory;
mod levelup;
mod proficiencies;
mod spells;
#[cfg(test)]
mod test_support;

pub use core::{CreateCharacterInput, UpdateCharacterInput};
pub use inventory::AddInventoryInput;
pub use levelup::{
    AsiOrFeat, FeatureChoices, FeatureReference, HpGainMethod, InvocationChoices, LevelUpRequest,
    LevelUpResult, ManeuverChoices, SpellChanges, SpellReference, SubclassChoice,
};

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
}
