//! CharacterFeat Model
//!
//! Tracks feats selected by characters.

use crate::schema::character_feats;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

/// A feat selected by a character.
#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize, Deserialize)]
#[diesel(table_name = character_feats)]
pub struct CharacterFeat {
    /// Unique ID (UUID)
    pub id: String,
    /// Character who has this feat
    pub character_id: String,
    /// Feat name
    pub feat_name: String,
    /// Feat source (e.g., "PHB", "XGE")
    pub feat_source: String,
    /// How the feat was obtained: 'asi', 'race', 'class', 'bonus'
    pub source_type: String,
}

impl CharacterFeat {
    /// Check if this feat was taken via ASI.
    pub fn is_from_asi(&self) -> bool {
        self.source_type == "asi"
    }

    /// Check if this feat is a racial feat.
    pub fn is_from_race(&self) -> bool {
        self.source_type == "race"
    }

    /// Check if this feat is from a class feature.
    pub fn is_from_class(&self) -> bool {
        self.source_type == "class"
    }

    /// Check if this is a bonus feat (DM granted, etc).
    pub fn is_bonus(&self) -> bool {
        self.source_type == "bonus"
    }
}

/// Source type for how a feat was obtained.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FeatSourceType {
    /// Feat taken instead of Ability Score Improvement
    Asi,
    /// Racial feat (e.g., variant human)
    Race,
    /// Class feature grant
    Class,
    /// Bonus feat (DM granted, background, etc)
    Bonus,
}

impl FeatSourceType {
    /// Convert to string for database storage.
    pub fn as_str(&self) -> &'static str {
        match self {
            FeatSourceType::Asi => "asi",
            FeatSourceType::Race => "race",
            FeatSourceType::Class => "class",
            FeatSourceType::Bonus => "bonus",
        }
    }
}

/// Data for inserting a new character feat.
#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = character_feats)]
pub struct NewCharacterFeat<'a> {
    pub id: &'a str,
    pub character_id: &'a str,
    pub feat_name: &'a str,
    pub feat_source: &'a str,
    pub source_type: &'a str,
}

impl<'a> NewCharacterFeat<'a> {
    /// Create a new feat entry.
    pub fn new(
        id: &'a str,
        character_id: &'a str,
        feat_name: &'a str,
        feat_source: &'a str,
        source_type: FeatSourceType,
    ) -> Self {
        Self {
            id,
            character_id,
            feat_name,
            feat_source,
            source_type: source_type.as_str(),
        }
    }

    /// Create an ASI feat (most common).
    pub fn from_asi(
        id: &'a str,
        character_id: &'a str,
        feat_name: &'a str,
        feat_source: &'a str,
    ) -> Self {
        Self::new(id, character_id, feat_name, feat_source, FeatSourceType::Asi)
    }

    /// Create a racial feat.
    pub fn from_race(
        id: &'a str,
        character_id: &'a str,
        feat_name: &'a str,
        feat_source: &'a str,
    ) -> Self {
        Self::new(id, character_id, feat_name, feat_source, FeatSourceType::Race)
    }

    /// Create a class-granted feat.
    pub fn from_class(
        id: &'a str,
        character_id: &'a str,
        feat_name: &'a str,
        feat_source: &'a str,
    ) -> Self {
        Self::new(id, character_id, feat_name, feat_source, FeatSourceType::Class)
    }

    /// Create a bonus feat (DM granted).
    pub fn bonus(
        id: &'a str,
        character_id: &'a str,
        feat_name: &'a str,
        feat_source: &'a str,
    ) -> Self {
        Self::new(id, character_id, feat_name, feat_source, FeatSourceType::Bonus)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_asi_feat() {
        let feat = NewCharacterFeat::from_asi("feat-1", "char-1", "Great Weapon Master", "PHB");
        assert_eq!(feat.feat_name, "Great Weapon Master");
        assert_eq!(feat.source_type, "asi");
    }

    #[test]
    fn test_new_racial_feat() {
        let feat = NewCharacterFeat::from_race("feat-1", "char-1", "Prodigy", "XGE");
        assert_eq!(feat.source_type, "race");
    }

    #[test]
    fn test_new_class_feat() {
        let feat = NewCharacterFeat::from_class("feat-1", "char-1", "Martial Adept", "PHB");
        assert_eq!(feat.source_type, "class");
    }

    #[test]
    fn test_new_bonus_feat() {
        let feat = NewCharacterFeat::bonus("feat-1", "char-1", "Lucky", "PHB");
        assert_eq!(feat.source_type, "bonus");
    }

    #[test]
    fn test_feat_source_type_as_str() {
        assert_eq!(FeatSourceType::Asi.as_str(), "asi");
        assert_eq!(FeatSourceType::Race.as_str(), "race");
        assert_eq!(FeatSourceType::Class.as_str(), "class");
        assert_eq!(FeatSourceType::Bonus.as_str(), "bonus");
    }
}
