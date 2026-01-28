//! Character Source Model
//!
//! Links characters to allowed source books.

use crate::schema::character_sources;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

/// A character source - links a character to an allowed source book.
#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize, Deserialize)]
#[diesel(table_name = character_sources)]
pub struct CharacterSource {
    /// Unique ID (UUID)
    pub id: String,
    /// Character this source belongs to
    pub character_id: String,
    /// Source book code (e.g., "PHB", "XPHB")
    pub source_code: String,
}

/// Data for inserting a new character source.
#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = character_sources)]
pub struct NewCharacterSource<'a> {
    pub id: &'a str,
    pub character_id: &'a str,
    pub source_code: &'a str,
}

impl<'a> NewCharacterSource<'a> {
    /// Create a new character source link.
    pub fn new(id: &'a str, character_id: &'a str, source_code: &'a str) -> Self {
        Self {
            id,
            character_id,
            source_code,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_character_source() {
        let source = NewCharacterSource::new("source-id", "char-id", "PHB");
        assert_eq!(source.id, "source-id");
        assert_eq!(source.character_id, "char-id");
        assert_eq!(source.source_code, "PHB");
    }
}
