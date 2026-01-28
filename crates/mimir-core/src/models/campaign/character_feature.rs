//! CharacterFeature Model
//!
//! Tracks class feature choices: Fighting Style, Metamagic, Maneuvers, Invocations, Pact Boon.

use crate::schema::character_features;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

/// A class feature choice made by a character.
#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize, Deserialize)]
#[diesel(table_name = character_features)]
pub struct CharacterFeature {
    /// Unique ID (UUID)
    pub id: String,
    /// Character who has this feature
    pub character_id: String,
    /// Feature type: 'fighting_style', 'metamagic', 'maneuver', 'invocation', 'pact_boon'
    pub feature_type: String,
    /// Feature name (e.g., "Defense", "Quickened Spell", "Riposte")
    pub feature_name: String,
    /// Feature source (e.g., "PHB", "XGE", "TCE")
    pub feature_source: String,
    /// Which class granted this feature
    pub source_class: String,
}

impl CharacterFeature {
    /// Check if this is a fighting style.
    pub fn is_fighting_style(&self) -> bool {
        self.feature_type == "fighting_style"
    }

    /// Check if this is a metamagic option.
    pub fn is_metamagic(&self) -> bool {
        self.feature_type == "metamagic"
    }

    /// Check if this is a battle master maneuver.
    pub fn is_maneuver(&self) -> bool {
        self.feature_type == "maneuver"
    }

    /// Check if this is a warlock invocation.
    pub fn is_invocation(&self) -> bool {
        self.feature_type == "invocation"
    }

    /// Check if this is a pact boon.
    pub fn is_pact_boon(&self) -> bool {
        self.feature_type == "pact_boon"
    }
}

/// Type of class feature choice.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FeatureType {
    /// Fighter/Paladin/Ranger fighting style
    FightingStyle,
    /// Sorcerer metamagic options
    Metamagic,
    /// Battle Master maneuvers
    Maneuver,
    /// Warlock eldritch invocations
    Invocation,
    /// Warlock pact boon
    PactBoon,
}

impl FeatureType {
    /// Convert to string for database storage.
    pub fn as_str(&self) -> &'static str {
        match self {
            FeatureType::FightingStyle => "fighting_style",
            FeatureType::Metamagic => "metamagic",
            FeatureType::Maneuver => "maneuver",
            FeatureType::Invocation => "invocation",
            FeatureType::PactBoon => "pact_boon",
        }
    }

    /// Parse from string.
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "fighting_style" => Some(FeatureType::FightingStyle),
            "metamagic" => Some(FeatureType::Metamagic),
            "maneuver" => Some(FeatureType::Maneuver),
            "invocation" => Some(FeatureType::Invocation),
            "pact_boon" => Some(FeatureType::PactBoon),
            _ => None,
        }
    }
}

/// Data for inserting a new character feature.
#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = character_features)]
pub struct NewCharacterFeature<'a> {
    pub id: &'a str,
    pub character_id: &'a str,
    pub feature_type: &'a str,
    pub feature_name: &'a str,
    pub feature_source: &'a str,
    pub source_class: &'a str,
}

impl<'a> NewCharacterFeature<'a> {
    /// Create a new feature entry.
    pub fn new(
        id: &'a str,
        character_id: &'a str,
        feature_type: FeatureType,
        feature_name: &'a str,
        feature_source: &'a str,
        source_class: &'a str,
    ) -> Self {
        Self {
            id,
            character_id,
            feature_type: feature_type.as_str(),
            feature_name,
            feature_source,
            source_class,
        }
    }

    /// Create a fighting style feature.
    pub fn fighting_style(
        id: &'a str,
        character_id: &'a str,
        name: &'a str,
        source: &'a str,
        class: &'a str,
    ) -> Self {
        Self::new(id, character_id, FeatureType::FightingStyle, name, source, class)
    }

    /// Create a metamagic option.
    pub fn metamagic(
        id: &'a str,
        character_id: &'a str,
        name: &'a str,
        source: &'a str,
    ) -> Self {
        Self::new(id, character_id, FeatureType::Metamagic, name, source, "Sorcerer")
    }

    /// Create a battle master maneuver.
    pub fn maneuver(
        id: &'a str,
        character_id: &'a str,
        name: &'a str,
        source: &'a str,
    ) -> Self {
        Self::new(id, character_id, FeatureType::Maneuver, name, source, "Fighter")
    }

    /// Create a warlock invocation.
    pub fn invocation(
        id: &'a str,
        character_id: &'a str,
        name: &'a str,
        source: &'a str,
    ) -> Self {
        Self::new(id, character_id, FeatureType::Invocation, name, source, "Warlock")
    }

    /// Create a warlock pact boon.
    pub fn pact_boon(
        id: &'a str,
        character_id: &'a str,
        name: &'a str,
        source: &'a str,
    ) -> Self {
        Self::new(id, character_id, FeatureType::PactBoon, name, source, "Warlock")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feature_type_as_str() {
        assert_eq!(FeatureType::FightingStyle.as_str(), "fighting_style");
        assert_eq!(FeatureType::Metamagic.as_str(), "metamagic");
        assert_eq!(FeatureType::Maneuver.as_str(), "maneuver");
        assert_eq!(FeatureType::Invocation.as_str(), "invocation");
        assert_eq!(FeatureType::PactBoon.as_str(), "pact_boon");
    }

    #[test]
    fn test_feature_type_from_str() {
        assert_eq!(FeatureType::from_str("fighting_style"), Some(FeatureType::FightingStyle));
        assert_eq!(FeatureType::from_str("metamagic"), Some(FeatureType::Metamagic));
        assert_eq!(FeatureType::from_str("maneuver"), Some(FeatureType::Maneuver));
        assert_eq!(FeatureType::from_str("invocation"), Some(FeatureType::Invocation));
        assert_eq!(FeatureType::from_str("pact_boon"), Some(FeatureType::PactBoon));
        assert_eq!(FeatureType::from_str("invalid"), None);
    }

    #[test]
    fn test_new_fighting_style() {
        let feature = NewCharacterFeature::fighting_style(
            "feat-1", "char-1", "Defense", "PHB", "Fighter"
        );
        assert_eq!(feature.feature_type, "fighting_style");
        assert_eq!(feature.feature_name, "Defense");
        assert_eq!(feature.source_class, "Fighter");
    }

    #[test]
    fn test_new_metamagic() {
        let feature = NewCharacterFeature::metamagic(
            "feat-1", "char-1", "Quickened Spell", "PHB"
        );
        assert_eq!(feature.feature_type, "metamagic");
        assert_eq!(feature.source_class, "Sorcerer");
    }

    #[test]
    fn test_new_maneuver() {
        let feature = NewCharacterFeature::maneuver(
            "feat-1", "char-1", "Riposte", "PHB"
        );
        assert_eq!(feature.feature_type, "maneuver");
        assert_eq!(feature.source_class, "Fighter");
    }

    #[test]
    fn test_new_invocation() {
        let feature = NewCharacterFeature::invocation(
            "feat-1", "char-1", "Agonizing Blast", "PHB"
        );
        assert_eq!(feature.feature_type, "invocation");
        assert_eq!(feature.source_class, "Warlock");
    }

    #[test]
    fn test_new_pact_boon() {
        let feature = NewCharacterFeature::pact_boon(
            "feat-1", "char-1", "Pact of the Blade", "PHB"
        );
        assert_eq!(feature.feature_type, "pact_boon");
        assert_eq!(feature.source_class, "Warlock");
    }
}
