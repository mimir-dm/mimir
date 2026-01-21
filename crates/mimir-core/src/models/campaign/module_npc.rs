//! ModuleNpc Model
//!
//! Custom NPCs created by the DM for use in modules.

use crate::schema::module_npcs;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

/// A custom NPC created by the DM for a module.
#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize, Deserialize)]
#[diesel(table_name = module_npcs)]
pub struct ModuleNpc {
    /// Unique ID (UUID)
    pub id: String,
    /// Module this NPC belongs to
    pub module_id: String,
    /// NPC name
    pub name: String,
    /// Role (e.g., "Quest Giver", "Merchant", "Villain")
    pub role: Option<String>,
    /// Description
    pub description: Option<String>,
    /// Physical appearance
    pub appearance: Option<String>,
    /// Personality traits
    pub personality: Option<String>,
    /// Motivation
    pub motivation: Option<String>,
    /// Secrets (DM-only info)
    pub secrets: Option<String>,
    /// JSON stat block (optional)
    pub stat_block: Option<String>,
    /// Token image asset ID
    pub token_asset_id: Option<String>,
    /// ISO8601 timestamp of creation
    pub created_at: String,
    /// ISO8601 timestamp of last update
    pub updated_at: String,
}

impl ModuleNpc {
    /// Check if this NPC has a stat block.
    pub fn has_stat_block(&self) -> bool {
        self.stat_block.is_some()
    }

    /// Check if this NPC has a custom token.
    pub fn has_token(&self) -> bool {
        self.token_asset_id.is_some()
    }
}

/// Data for inserting a new module NPC.
#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = module_npcs)]
pub struct NewModuleNpc<'a> {
    pub id: &'a str,
    pub module_id: &'a str,
    pub name: &'a str,
    pub role: Option<&'a str>,
    pub description: Option<&'a str>,
    pub appearance: Option<&'a str>,
    pub personality: Option<&'a str>,
    pub motivation: Option<&'a str>,
    pub secrets: Option<&'a str>,
    pub stat_block: Option<&'a str>,
    pub token_asset_id: Option<&'a str>,
}

impl<'a> NewModuleNpc<'a> {
    /// Create a new module NPC with just a name.
    pub fn new(id: &'a str, module_id: &'a str, name: &'a str) -> Self {
        Self {
            id,
            module_id,
            name,
            role: None,
            description: None,
            appearance: None,
            personality: None,
            motivation: None,
            secrets: None,
            stat_block: None,
            token_asset_id: None,
        }
    }

    /// Set the role.
    pub fn with_role(mut self, role: &'a str) -> Self {
        self.role = Some(role);
        self
    }

    /// Set the description.
    pub fn with_description(mut self, description: &'a str) -> Self {
        self.description = Some(description);
        self
    }

    /// Set the appearance.
    pub fn with_appearance(mut self, appearance: &'a str) -> Self {
        self.appearance = Some(appearance);
        self
    }

    /// Set personality traits.
    pub fn with_personality(mut self, personality: &'a str) -> Self {
        self.personality = Some(personality);
        self
    }

    /// Set motivation.
    pub fn with_motivation(mut self, motivation: &'a str) -> Self {
        self.motivation = Some(motivation);
        self
    }

    /// Set secrets (DM-only).
    pub fn with_secrets(mut self, secrets: &'a str) -> Self {
        self.secrets = Some(secrets);
        self
    }

    /// Set the stat block JSON.
    pub fn with_stat_block(mut self, stat_block: &'a str) -> Self {
        self.stat_block = Some(stat_block);
        self
    }

    /// Set the token asset.
    pub fn with_token(mut self, token_asset_id: &'a str) -> Self {
        self.token_asset_id = Some(token_asset_id);
        self
    }
}

/// Data for updating a module NPC.
#[derive(Debug, Clone, Default, AsChangeset)]
#[diesel(table_name = module_npcs)]
pub struct UpdateModuleNpc<'a> {
    pub name: Option<&'a str>,
    pub role: Option<Option<&'a str>>,
    pub description: Option<Option<&'a str>>,
    pub appearance: Option<Option<&'a str>>,
    pub personality: Option<Option<&'a str>>,
    pub motivation: Option<Option<&'a str>>,
    pub secrets: Option<Option<&'a str>>,
    pub stat_block: Option<Option<&'a str>>,
    pub token_asset_id: Option<Option<&'a str>>,
    pub updated_at: Option<&'a str>,
}

impl<'a> UpdateModuleNpc<'a> {
    /// Update the name.
    pub fn set_name(name: &'a str, updated_at: &'a str) -> Self {
        Self {
            name: Some(name),
            updated_at: Some(updated_at),
            ..Default::default()
        }
    }

    /// Update the role.
    pub fn set_role(role: Option<&'a str>, updated_at: &'a str) -> Self {
        Self {
            role: Some(role),
            updated_at: Some(updated_at),
            ..Default::default()
        }
    }

    /// Update the description.
    pub fn set_description(description: Option<&'a str>, updated_at: &'a str) -> Self {
        Self {
            description: Some(description),
            updated_at: Some(updated_at),
            ..Default::default()
        }
    }

    /// Update secrets.
    pub fn set_secrets(secrets: Option<&'a str>, updated_at: &'a str) -> Self {
        Self {
            secrets: Some(secrets),
            updated_at: Some(updated_at),
            ..Default::default()
        }
    }

    /// Update the token asset.
    pub fn set_token(token_asset_id: Option<&'a str>, updated_at: &'a str) -> Self {
        Self {
            token_asset_id: Some(token_asset_id),
            updated_at: Some(updated_at),
            ..Default::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_module_npc() {
        let npc = NewModuleNpc::new("npc-1", "mod-1", "Sildar Hallwinter");
        assert_eq!(npc.name, "Sildar Hallwinter");
        assert!(npc.role.is_none());
    }

    #[test]
    fn test_with_role() {
        let npc = NewModuleNpc::new("npc-1", "mod-1", "Gundren Rockseeker")
            .with_role("Quest Giver");
        assert_eq!(npc.role, Some("Quest Giver"));
    }

    #[test]
    fn test_with_description() {
        let npc = NewModuleNpc::new("npc-1", "mod-1", "Innkeeper")
            .with_description("A friendly halfling woman who runs the Stonehill Inn.");
        assert!(npc.description.unwrap().contains("halfling"));
    }

    #[test]
    fn test_with_personality_and_motivation() {
        let npc = NewModuleNpc::new("npc-1", "mod-1", "Villain")
            .with_personality("Cruel and calculating")
            .with_motivation("Seeks ultimate power");
        assert_eq!(npc.personality, Some("Cruel and calculating"));
        assert_eq!(npc.motivation, Some("Seeks ultimate power"));
    }

    #[test]
    fn test_with_secrets() {
        let npc = NewModuleNpc::new("npc-1", "mod-1", "Mysterious Stranger")
            .with_secrets("Is actually a dragon in disguise");
        assert!(npc.secrets.is_some());
    }

    #[test]
    fn test_update_name() {
        let update = UpdateModuleNpc::set_name("New Name", "2024-01-20T12:00:00Z");
        assert_eq!(update.name, Some("New Name"));
    }
}
