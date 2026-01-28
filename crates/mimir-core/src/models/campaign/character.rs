//! Character Model
//!
//! Player characters and NPCs for campaigns.

use crate::schema::characters;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

use super::{CharacterClass, CharacterProficiency};

/// A character - either a player character or NPC in a campaign.
/// This is the database model - use `CharacterResponse` for API responses with classes.
#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize, Deserialize)]
#[diesel(table_name = characters)]
pub struct Character {
    /// Unique character ID (UUID)
    pub id: String,
    /// Campaign this character belongs to
    pub campaign_id: String,
    /// Character name
    pub name: String,
    /// Whether this is an NPC (1) or PC (0)
    pub is_npc: i32,
    /// Player name (for PCs)
    pub player_name: Option<String>,

    // Race and background (catalog references)
    /// Race name (e.g., "Elf", "Human")
    pub race_name: Option<String>,
    /// Race source (e.g., "PHB", "VGtM")
    pub race_source: Option<String>,
    /// Background name (e.g., "Acolyte", "Criminal")
    pub background_name: Option<String>,
    /// Background source (e.g., "PHB")
    pub background_source: Option<String>,

    // Ability scores
    pub strength: i32,
    pub dexterity: i32,
    pub constitution: i32,
    pub intelligence: i32,
    pub wisdom: i32,
    pub charisma: i32,

    // Currency
    pub cp: i32,
    pub sp: i32,
    pub ep: i32,
    pub gp: i32,
    pub pp: i32,

    // Roleplay elements
    pub traits: Option<String>,
    pub ideals: Option<String>,
    pub bonds: Option<String>,
    pub flaws: Option<String>,

    // NPC-specific fields
    pub role: Option<String>,
    pub location: Option<String>,
    pub faction: Option<String>,

    /// ISO8601 timestamp of creation
    pub created_at: String,
    /// ISO8601 timestamp of last update
    pub updated_at: String,
}

impl Character {
    /// Check if this character is an NPC.
    pub fn is_npc(&self) -> bool {
        self.is_npc != 0
    }

    /// Check if this is a player character.
    pub fn is_pc(&self) -> bool {
        self.is_npc == 0
    }

    /// Calculate ability modifier for a given score.
    /// Uses floor division to match D&D 5e rules.
    pub fn ability_modifier(score: i32) -> i32 {
        (score - 10).div_euclid(2)
    }

    /// Get strength modifier.
    pub fn str_mod(&self) -> i32 {
        Self::ability_modifier(self.strength)
    }

    /// Get dexterity modifier.
    pub fn dex_mod(&self) -> i32 {
        Self::ability_modifier(self.dexterity)
    }

    /// Get constitution modifier.
    pub fn con_mod(&self) -> i32 {
        Self::ability_modifier(self.constitution)
    }

    /// Get intelligence modifier.
    pub fn int_mod(&self) -> i32 {
        Self::ability_modifier(self.intelligence)
    }

    /// Get wisdom modifier.
    pub fn wis_mod(&self) -> i32 {
        Self::ability_modifier(self.wisdom)
    }

    /// Get charisma modifier.
    pub fn cha_mod(&self) -> i32 {
        Self::ability_modifier(self.charisma)
    }

    /// Calculate total gold value of all currency.
    pub fn total_gold_value(&self) -> f64 {
        (self.cp as f64 / 100.0)
            + (self.sp as f64 / 10.0)
            + (self.ep as f64 / 2.0)
            + (self.gp as f64)
            + (self.pp as f64 * 10.0)
    }
}

/// Data for inserting a new character.
#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = characters)]
pub struct NewCharacter<'a> {
    pub id: &'a str,
    pub campaign_id: &'a str,
    pub name: &'a str,
    pub is_npc: i32,
    pub player_name: Option<&'a str>,
    pub race_name: Option<&'a str>,
    pub race_source: Option<&'a str>,
    pub background_name: Option<&'a str>,
    pub background_source: Option<&'a str>,
    pub strength: i32,
    pub dexterity: i32,
    pub constitution: i32,
    pub intelligence: i32,
    pub wisdom: i32,
    pub charisma: i32,
    pub cp: i32,
    pub sp: i32,
    pub ep: i32,
    pub gp: i32,
    pub pp: i32,
    pub traits: Option<&'a str>,
    pub ideals: Option<&'a str>,
    pub bonds: Option<&'a str>,
    pub flaws: Option<&'a str>,
    pub role: Option<&'a str>,
    pub location: Option<&'a str>,
    pub faction: Option<&'a str>,
}

impl<'a> NewCharacter<'a> {
    /// Create a new player character with default ability scores.
    pub fn new_pc(id: &'a str, campaign_id: &'a str, name: &'a str, player_name: &'a str) -> Self {
        Self {
            id,
            campaign_id,
            name,
            is_npc: 0,
            player_name: Some(player_name),
            race_name: None,
            race_source: None,
            background_name: None,
            background_source: None,
            strength: 10,
            dexterity: 10,
            constitution: 10,
            intelligence: 10,
            wisdom: 10,
            charisma: 10,
            cp: 0,
            sp: 0,
            ep: 0,
            gp: 0,
            pp: 0,
            traits: None,
            ideals: None,
            bonds: None,
            flaws: None,
            role: None,
            location: None,
            faction: None,
        }
    }

    /// Create a new NPC with default ability scores.
    pub fn new_npc(id: &'a str, campaign_id: &'a str, name: &'a str) -> Self {
        Self {
            id,
            campaign_id,
            name,
            is_npc: 1,
            player_name: None,
            race_name: None,
            race_source: None,
            background_name: None,
            background_source: None,
            strength: 10,
            dexterity: 10,
            constitution: 10,
            intelligence: 10,
            wisdom: 10,
            charisma: 10,
            cp: 0,
            sp: 0,
            ep: 0,
            gp: 0,
            pp: 0,
            traits: None,
            ideals: None,
            bonds: None,
            flaws: None,
            role: None,
            location: None,
            faction: None,
        }
    }

    /// Set the race.
    pub fn with_race(mut self, name: &'a str, source: &'a str) -> Self {
        self.race_name = Some(name);
        self.race_source = Some(source);
        self
    }

    /// Set the background.
    pub fn with_background(mut self, name: &'a str, source: &'a str) -> Self {
        self.background_name = Some(name);
        self.background_source = Some(source);
        self
    }

    /// Set ability scores.
    pub fn with_ability_scores(
        mut self,
        str: i32,
        dex: i32,
        con: i32,
        int: i32,
        wis: i32,
        cha: i32,
    ) -> Self {
        self.strength = str;
        self.dexterity = dex;
        self.constitution = con;
        self.intelligence = int;
        self.wisdom = wis;
        self.charisma = cha;
        self
    }

    /// Set starting currency.
    pub fn with_currency(mut self, cp: i32, sp: i32, ep: i32, gp: i32, pp: i32) -> Self {
        self.cp = cp;
        self.sp = sp;
        self.ep = ep;
        self.gp = gp;
        self.pp = pp;
        self
    }

    /// Set roleplay elements.
    pub fn with_roleplay(
        mut self,
        traits: Option<&'a str>,
        ideals: Option<&'a str>,
        bonds: Option<&'a str>,
        flaws: Option<&'a str>,
    ) -> Self {
        self.traits = traits;
        self.ideals = ideals;
        self.bonds = bonds;
        self.flaws = flaws;
        self
    }

    /// Set NPC-specific fields.
    pub fn with_npc_info(
        mut self,
        role: Option<&'a str>,
        location: Option<&'a str>,
        faction: Option<&'a str>,
    ) -> Self {
        self.role = role;
        self.location = location;
        self.faction = faction;
        self
    }
}

/// Data for updating a character.
#[derive(Debug, Clone, Default, AsChangeset)]
#[diesel(table_name = characters)]
pub struct UpdateCharacter<'a> {
    pub name: Option<&'a str>,
    pub player_name: Option<Option<&'a str>>,
    pub race_name: Option<Option<&'a str>>,
    pub race_source: Option<Option<&'a str>>,
    pub background_name: Option<Option<&'a str>>,
    pub background_source: Option<Option<&'a str>>,
    pub strength: Option<i32>,
    pub dexterity: Option<i32>,
    pub constitution: Option<i32>,
    pub intelligence: Option<i32>,
    pub wisdom: Option<i32>,
    pub charisma: Option<i32>,
    pub cp: Option<i32>,
    pub sp: Option<i32>,
    pub ep: Option<i32>,
    pub gp: Option<i32>,
    pub pp: Option<i32>,
    pub traits: Option<Option<&'a str>>,
    pub ideals: Option<Option<&'a str>>,
    pub bonds: Option<Option<&'a str>>,
    pub flaws: Option<Option<&'a str>>,
    pub role: Option<Option<&'a str>>,
    pub location: Option<Option<&'a str>>,
    pub faction: Option<Option<&'a str>>,
    pub updated_at: Option<&'a str>,
}

impl<'a> UpdateCharacter<'a> {
    /// Update character name.
    pub fn set_name(name: &'a str, updated_at: &'a str) -> Self {
        Self {
            name: Some(name),
            updated_at: Some(updated_at),
            ..Default::default()
        }
    }

    /// Update ability scores.
    pub fn set_ability_scores(
        str: i32,
        dex: i32,
        con: i32,
        int: i32,
        wis: i32,
        cha: i32,
        updated_at: &'a str,
    ) -> Self {
        Self {
            strength: Some(str),
            dexterity: Some(dex),
            constitution: Some(con),
            intelligence: Some(int),
            wisdom: Some(wis),
            charisma: Some(cha),
            updated_at: Some(updated_at),
            ..Default::default()
        }
    }

    /// Update currency.
    pub fn set_currency(cp: i32, sp: i32, ep: i32, gp: i32, pp: i32, updated_at: &'a str) -> Self {
        Self {
            cp: Some(cp),
            sp: Some(sp),
            ep: Some(ep),
            gp: Some(gp),
            pp: Some(pp),
            updated_at: Some(updated_at),
            ..Default::default()
        }
    }

    /// Update roleplay elements.
    pub fn set_roleplay(
        traits: Option<&'a str>,
        ideals: Option<&'a str>,
        bonds: Option<&'a str>,
        flaws: Option<&'a str>,
        updated_at: &'a str,
    ) -> Self {
        Self {
            traits: Some(traits),
            ideals: Some(ideals),
            bonds: Some(bonds),
            flaws: Some(flaws),
            updated_at: Some(updated_at),
            ..Default::default()
        }
    }

    /// Update NPC info.
    pub fn set_npc_info(
        role: Option<&'a str>,
        location: Option<&'a str>,
        faction: Option<&'a str>,
        updated_at: &'a str,
    ) -> Self {
        Self {
            role: Some(role),
            location: Some(location),
            faction: Some(faction),
            updated_at: Some(updated_at),
            ..Default::default()
        }
    }

    /// Update race.
    pub fn set_race(name: Option<&'a str>, source: Option<&'a str>, updated_at: &'a str) -> Self {
        Self {
            race_name: Some(name),
            race_source: Some(source),
            updated_at: Some(updated_at),
            ..Default::default()
        }
    }

    /// Update background.
    pub fn set_background(
        name: Option<&'a str>,
        source: Option<&'a str>,
        updated_at: &'a str,
    ) -> Self {
        Self {
            background_name: Some(name),
            background_source: Some(source),
            updated_at: Some(updated_at),
            ..Default::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_pc() {
        let pc = NewCharacter::new_pc("char-1", "camp-1", "Gandalf", "John");
        assert_eq!(pc.id, "char-1");
        assert_eq!(pc.name, "Gandalf");
        assert_eq!(pc.is_npc, 0);
        assert_eq!(pc.player_name, Some("John"));
        assert_eq!(pc.strength, 10);
    }

    #[test]
    fn test_new_npc() {
        let npc = NewCharacter::new_npc("char-1", "camp-1", "Shopkeeper");
        assert_eq!(npc.name, "Shopkeeper");
        assert_eq!(npc.is_npc, 1);
        assert!(npc.player_name.is_none());
    }

    #[test]
    fn test_with_ability_scores() {
        let pc = NewCharacter::new_pc("char-1", "camp-1", "Fighter", "John")
            .with_ability_scores(16, 14, 15, 8, 10, 12);
        assert_eq!(pc.strength, 16);
        assert_eq!(pc.dexterity, 14);
        assert_eq!(pc.constitution, 15);
        assert_eq!(pc.intelligence, 8);
        assert_eq!(pc.wisdom, 10);
        assert_eq!(pc.charisma, 12);
    }

    #[test]
    fn test_with_currency() {
        let pc = NewCharacter::new_pc("char-1", "camp-1", "Rogue", "Jane")
            .with_currency(50, 20, 0, 15, 1);
        assert_eq!(pc.cp, 50);
        assert_eq!(pc.sp, 20);
        assert_eq!(pc.ep, 0);
        assert_eq!(pc.gp, 15);
        assert_eq!(pc.pp, 1);
    }

    #[test]
    fn test_with_npc_info() {
        let npc = NewCharacter::new_npc("char-1", "camp-1", "Guard Captain")
            .with_npc_info(Some("military"), Some("Waterdeep"), Some("City Watch"));
        assert_eq!(npc.role, Some("military"));
        assert_eq!(npc.location, Some("Waterdeep"));
        assert_eq!(npc.faction, Some("City Watch"));
    }

    #[test]
    fn test_ability_modifier() {
        assert_eq!(Character::ability_modifier(10), 0);
        assert_eq!(Character::ability_modifier(11), 0);
        assert_eq!(Character::ability_modifier(12), 1);
        assert_eq!(Character::ability_modifier(14), 2);
        assert_eq!(Character::ability_modifier(8), -1);
        assert_eq!(Character::ability_modifier(6), -2);
        assert_eq!(Character::ability_modifier(20), 5);
        assert_eq!(Character::ability_modifier(1), -5);
    }

    #[test]
    fn test_update_name() {
        let update = UpdateCharacter::set_name("New Name", "2024-01-20T12:00:00Z");
        assert_eq!(update.name, Some("New Name"));
        assert!(update.strength.is_none());
    }

    #[test]
    fn test_update_ability_scores() {
        let update =
            UpdateCharacter::set_ability_scores(18, 16, 14, 12, 10, 8, "2024-01-20T12:00:00Z");
        assert_eq!(update.strength, Some(18));
        assert_eq!(update.charisma, Some(8));
    }

    #[test]
    fn test_update_currency() {
        let update = UpdateCharacter::set_currency(100, 50, 25, 10, 5, "2024-01-20T12:00:00Z");
        assert_eq!(update.cp, Some(100));
        assert_eq!(update.pp, Some(5));
    }
}

// =============================================================================
// API Response Type
// =============================================================================

/// Character with classes included - used for API responses.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterResponse {
    pub id: String,
    pub campaign_id: String,
    pub name: String,
    pub is_npc: i32,
    pub player_name: Option<String>,
    pub race_name: Option<String>,
    pub race_source: Option<String>,
    pub background_name: Option<String>,
    pub background_source: Option<String>,
    pub strength: i32,
    pub dexterity: i32,
    pub constitution: i32,
    pub intelligence: i32,
    pub wisdom: i32,
    pub charisma: i32,
    pub cp: i32,
    pub sp: i32,
    pub ep: i32,
    pub gp: i32,
    pub pp: i32,
    pub traits: Option<String>,
    pub ideals: Option<String>,
    pub bonds: Option<String>,
    pub flaws: Option<String>,
    pub role: Option<String>,
    pub location: Option<String>,
    pub faction: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    /// Character classes (populated from character_classes table)
    pub classes: Vec<CharacterClass>,
    /// Character proficiencies (populated from character_proficiencies table)
    pub proficiencies: Vec<CharacterProficiency>,
}

impl CharacterResponse {
    /// Create from a Character, its classes, and proficiencies.
    pub fn from_character(
        character: Character,
        classes: Vec<CharacterClass>,
        proficiencies: Vec<CharacterProficiency>,
    ) -> Self {
        Self {
            id: character.id,
            campaign_id: character.campaign_id,
            name: character.name,
            is_npc: character.is_npc,
            player_name: character.player_name,
            race_name: character.race_name,
            race_source: character.race_source,
            background_name: character.background_name,
            background_source: character.background_source,
            strength: character.strength,
            dexterity: character.dexterity,
            constitution: character.constitution,
            intelligence: character.intelligence,
            wisdom: character.wisdom,
            charisma: character.charisma,
            cp: character.cp,
            sp: character.sp,
            ep: character.ep,
            gp: character.gp,
            pp: character.pp,
            traits: character.traits,
            ideals: character.ideals,
            bonds: character.bonds,
            flaws: character.flaws,
            role: character.role,
            location: character.location,
            faction: character.faction,
            created_at: character.created_at,
            updated_at: character.updated_at,
            classes,
            proficiencies,
        }
    }

    /// Get total character level across all classes.
    pub fn total_level(&self) -> i32 {
        self.classes.iter().map(|c| c.level).sum()
    }

    /// Format class string (e.g., "Fighter 5 / Rogue 3").
    pub fn class_string(&self) -> String {
        if self.classes.is_empty() {
            return "No Class".to_string();
        }
        self.classes
            .iter()
            .map(|c| {
                if let Some(ref sub) = c.subclass_name {
                    format!("{} ({}) {}", c.class_name, sub, c.level)
                } else {
                    format!("{} {}", c.class_name, c.level)
                }
            })
            .collect::<Vec<_>>()
            .join(" / ")
    }
}
