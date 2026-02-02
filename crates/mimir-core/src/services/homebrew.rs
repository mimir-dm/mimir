//! Homebrew Service
//!
//! Business logic for homebrew items, monsters, and spells.
//! Centralizes UUID generation, timestamp management, and JSON validation
//! that was previously duplicated across Tauri commands and MCP tools.

use chrono::Utc;
use diesel::SqliteConnection;
use serde_json::Value;
use uuid::Uuid;

use crate::dal::campaign as dal;
use crate::models::campaign::{
    CampaignHomebrewItem, CampaignHomebrewMonster, CampaignHomebrewSpell,
    NewCampaignHomebrewItem, NewCampaignHomebrewMonster, NewCampaignHomebrewSpell,
    UpdateCampaignHomebrewItem, UpdateCampaignHomebrewMonster, UpdateCampaignHomebrewSpell,
};
use crate::services::{ServiceError, ServiceResult};

// ── Input structs ──────────────────────────────────────────────────────

/// Input for creating a homebrew item.
#[derive(Debug, Clone)]
pub struct CreateHomebrewItemInput {
    pub campaign_id: String,
    pub name: String,
    pub data: String,
    pub item_type: Option<String>,
    pub rarity: Option<String>,
    pub cloned_from_name: Option<String>,
    pub cloned_from_source: Option<String>,
}

/// Input for updating a homebrew item.
#[derive(Debug, Clone, Default)]
pub struct UpdateHomebrewItemInput {
    pub name: Option<String>,
    pub data: Option<String>,
    pub item_type: Option<Option<String>>,
    pub rarity: Option<Option<String>>,
}

/// Input for creating a homebrew monster.
#[derive(Debug, Clone)]
pub struct CreateHomebrewMonsterInput {
    pub campaign_id: String,
    pub name: String,
    pub data: String,
    pub cr: Option<String>,
    pub creature_type: Option<String>,
    pub size: Option<String>,
    pub cloned_from_name: Option<String>,
    pub cloned_from_source: Option<String>,
}

/// Input for updating a homebrew monster.
#[derive(Debug, Clone, Default)]
pub struct UpdateHomebrewMonsterInput {
    pub name: Option<String>,
    pub data: Option<String>,
    pub cr: Option<Option<String>>,
    pub creature_type: Option<Option<String>>,
    pub size: Option<Option<String>>,
}

/// Input for creating a homebrew spell.
#[derive(Debug, Clone)]
pub struct CreateHomebrewSpellInput {
    pub campaign_id: String,
    pub name: String,
    pub data: String,
    pub level: Option<i32>,
    pub school: Option<String>,
    pub cloned_from_name: Option<String>,
    pub cloned_from_source: Option<String>,
}

/// Input for updating a homebrew spell.
#[derive(Debug, Clone, Default)]
pub struct UpdateHomebrewSpellInput {
    pub name: Option<String>,
    pub data: Option<String>,
    pub level: Option<Option<i32>>,
    pub school: Option<Option<String>>,
}

// ── Service ────────────────────────────────────────────────────────────

/// Service for homebrew content management.
pub struct HomebrewService<'a> {
    conn: &'a mut SqliteConnection,
}

impl<'a> HomebrewService<'a> {
    /// Create a new homebrew service.
    pub fn new(conn: &'a mut SqliteConnection) -> Self {
        Self { conn }
    }

    // ── Items ──────────────────────────────────────────────────────

    /// List all homebrew items for a campaign.
    pub fn list_items(&mut self, campaign_id: &str) -> ServiceResult<Vec<CampaignHomebrewItem>> {
        dal::list_campaign_homebrew_items(self.conn, campaign_id).map_err(ServiceError::from)
    }

    /// Get a homebrew item by ID.
    pub fn get_item(&mut self, id: &str) -> ServiceResult<CampaignHomebrewItem> {
        dal::get_campaign_homebrew_item(self.conn, id).map_err(ServiceError::from)
    }

    /// Get a homebrew item by campaign ID and name.
    pub fn get_item_by_name(
        &mut self,
        campaign_id: &str,
        name: &str,
    ) -> ServiceResult<Option<CampaignHomebrewItem>> {
        dal::get_campaign_homebrew_item_by_name(self.conn, campaign_id, name)
            .map_err(ServiceError::from)
    }

    /// Create a homebrew item.
    pub fn create_item(
        &mut self,
        input: CreateHomebrewItemInput,
    ) -> ServiceResult<CampaignHomebrewItem> {
        validate_json(&input.data)?;

        let id = Uuid::new_v4().to_string();
        let mut new_item = NewCampaignHomebrewItem::new(&id, &input.campaign_id, &input.name, &input.data);

        if let Some(ref t) = input.item_type {
            new_item = new_item.with_item_type(t);
        }
        if let Some(ref r) = input.rarity {
            new_item = new_item.with_rarity(r);
        }
        if let (Some(ref n), Some(ref s)) = (&input.cloned_from_name, &input.cloned_from_source) {
            new_item = new_item.cloned_from(n, s);
        }

        dal::insert_campaign_homebrew_item(self.conn, &new_item)?;
        dal::get_campaign_homebrew_item(self.conn, &id).map_err(ServiceError::from)
    }

    /// Update a homebrew item.
    pub fn update_item(
        &mut self,
        id: &str,
        input: UpdateHomebrewItemInput,
    ) -> ServiceResult<CampaignHomebrewItem> {
        if let Some(ref data) = input.data {
            validate_json(data)?;
        }

        let now = Utc::now().to_rfc3339();
        let name_ref = input.name.as_deref();
        let data_ref = input.data.as_deref();
        let item_type_ref = input.item_type.as_ref().map(|v| v.as_deref());
        let rarity_ref = input.rarity.as_ref().map(|v| v.as_deref());

        let update = UpdateCampaignHomebrewItem {
            name: name_ref,
            data: data_ref,
            item_type: item_type_ref,
            rarity: rarity_ref,
            updated_at: Some(&now),
        };

        let rows = dal::update_campaign_homebrew_item(self.conn, id, &update)?;
        if rows == 0 {
            return Err(ServiceError::not_found("HomebrewItem", id));
        }

        dal::get_campaign_homebrew_item(self.conn, id).map_err(ServiceError::from)
    }

    /// Delete a homebrew item.
    pub fn delete_item(&mut self, id: &str) -> ServiceResult<()> {
        let rows = dal::delete_campaign_homebrew_item(self.conn, id)?;
        if rows == 0 {
            return Err(ServiceError::not_found("HomebrewItem", id));
        }
        Ok(())
    }

    // ── Monsters ───────────────────────────────────────────────────

    /// List all homebrew monsters for a campaign.
    pub fn list_monsters(
        &mut self,
        campaign_id: &str,
    ) -> ServiceResult<Vec<CampaignHomebrewMonster>> {
        dal::list_campaign_homebrew_monsters(self.conn, campaign_id).map_err(ServiceError::from)
    }

    /// Get a homebrew monster by ID.
    pub fn get_monster(&mut self, id: &str) -> ServiceResult<CampaignHomebrewMonster> {
        dal::get_campaign_homebrew_monster(self.conn, id).map_err(ServiceError::from)
    }

    /// Get a homebrew monster by campaign ID and name.
    pub fn get_monster_by_name(
        &mut self,
        campaign_id: &str,
        name: &str,
    ) -> ServiceResult<Option<CampaignHomebrewMonster>> {
        dal::get_campaign_homebrew_monster_by_name(self.conn, campaign_id, name)
            .map_err(ServiceError::from)
    }

    /// Create a homebrew monster.
    pub fn create_monster(
        &mut self,
        input: CreateHomebrewMonsterInput,
    ) -> ServiceResult<CampaignHomebrewMonster> {
        validate_json(&input.data)?;

        let id = Uuid::new_v4().to_string();
        let mut new_monster =
            NewCampaignHomebrewMonster::new(&id, &input.campaign_id, &input.name, &input.data);

        if let Some(ref cr) = input.cr {
            new_monster = new_monster.with_cr(cr);
        }
        if let Some(ref ct) = input.creature_type {
            new_monster = new_monster.with_creature_type(ct);
        }
        if let Some(ref sz) = input.size {
            new_monster = new_monster.with_size(sz);
        }
        if let (Some(ref n), Some(ref s)) = (&input.cloned_from_name, &input.cloned_from_source) {
            new_monster = new_monster.cloned_from(n, s);
        }

        dal::insert_campaign_homebrew_monster(self.conn, &new_monster)?;
        dal::get_campaign_homebrew_monster(self.conn, &id).map_err(ServiceError::from)
    }

    /// Update a homebrew monster.
    pub fn update_monster(
        &mut self,
        id: &str,
        input: UpdateHomebrewMonsterInput,
    ) -> ServiceResult<CampaignHomebrewMonster> {
        if let Some(ref data) = input.data {
            validate_json(data)?;
        }

        let now = Utc::now().to_rfc3339();
        let name_ref = input.name.as_deref();
        let data_ref = input.data.as_deref();
        let cr_ref = input.cr.as_ref().map(|v| v.as_deref());
        let creature_type_ref = input.creature_type.as_ref().map(|v| v.as_deref());
        let size_ref = input.size.as_ref().map(|v| v.as_deref());

        let update = UpdateCampaignHomebrewMonster {
            name: name_ref,
            data: data_ref,
            cr: cr_ref,
            creature_type: creature_type_ref,
            size: size_ref,
            updated_at: Some(&now),
        };

        let rows = dal::update_campaign_homebrew_monster(self.conn, id, &update)?;
        if rows == 0 {
            return Err(ServiceError::not_found("HomebrewMonster", id));
        }

        dal::get_campaign_homebrew_monster(self.conn, id).map_err(ServiceError::from)
    }

    /// Delete a homebrew monster.
    pub fn delete_monster(&mut self, id: &str) -> ServiceResult<()> {
        let rows = dal::delete_campaign_homebrew_monster(self.conn, id)?;
        if rows == 0 {
            return Err(ServiceError::not_found("HomebrewMonster", id));
        }
        Ok(())
    }

    // ── Spells ─────────────────────────────────────────────────────

    /// List all homebrew spells for a campaign.
    pub fn list_spells(
        &mut self,
        campaign_id: &str,
    ) -> ServiceResult<Vec<CampaignHomebrewSpell>> {
        dal::list_campaign_homebrew_spells(self.conn, campaign_id).map_err(ServiceError::from)
    }

    /// Get a homebrew spell by ID.
    pub fn get_spell(&mut self, id: &str) -> ServiceResult<CampaignHomebrewSpell> {
        dal::get_campaign_homebrew_spell(self.conn, id).map_err(ServiceError::from)
    }

    /// Get a homebrew spell by campaign ID and name.
    pub fn get_spell_by_name(
        &mut self,
        campaign_id: &str,
        name: &str,
    ) -> ServiceResult<Option<CampaignHomebrewSpell>> {
        dal::get_campaign_homebrew_spell_by_name(self.conn, campaign_id, name)
            .map_err(ServiceError::from)
    }

    /// Create a homebrew spell.
    pub fn create_spell(
        &mut self,
        input: CreateHomebrewSpellInput,
    ) -> ServiceResult<CampaignHomebrewSpell> {
        validate_json(&input.data)?;

        let id = Uuid::new_v4().to_string();
        let mut new_spell =
            NewCampaignHomebrewSpell::new(&id, &input.campaign_id, &input.name, &input.data);

        if let Some(level) = input.level {
            new_spell = new_spell.with_level(level);
        }
        if let Some(ref school) = input.school {
            new_spell = new_spell.with_school(school);
        }
        if let (Some(ref n), Some(ref s)) = (&input.cloned_from_name, &input.cloned_from_source) {
            new_spell = new_spell.cloned_from(n, s);
        }

        dal::insert_campaign_homebrew_spell(self.conn, &new_spell)?;
        dal::get_campaign_homebrew_spell(self.conn, &id).map_err(ServiceError::from)
    }

    /// Update a homebrew spell.
    pub fn update_spell(
        &mut self,
        id: &str,
        input: UpdateHomebrewSpellInput,
    ) -> ServiceResult<CampaignHomebrewSpell> {
        if let Some(ref data) = input.data {
            validate_json(data)?;
        }

        let now = Utc::now().to_rfc3339();
        let name_ref = input.name.as_deref();
        let data_ref = input.data.as_deref();
        let level_ref = input.level.as_ref().copied();
        let school_ref = input.school.as_ref().map(|v| v.as_deref());

        let update = UpdateCampaignHomebrewSpell {
            name: name_ref,
            data: data_ref,
            level: level_ref,
            school: school_ref,
            updated_at: Some(&now),
        };

        let rows = dal::update_campaign_homebrew_spell(self.conn, id, &update)?;
        if rows == 0 {
            return Err(ServiceError::not_found("HomebrewSpell", id));
        }

        dal::get_campaign_homebrew_spell(self.conn, id).map_err(ServiceError::from)
    }

    /// Delete a homebrew spell.
    pub fn delete_spell(&mut self, id: &str) -> ServiceResult<()> {
        let rows = dal::delete_campaign_homebrew_spell(self.conn, id)?;
        if rows == 0 {
            return Err(ServiceError::not_found("HomebrewSpell", id));
        }
        Ok(())
    }
}

// ── Helpers ────────────────────────────────────────────────────────────

/// Validate that a string is valid JSON.
fn validate_json(data: &str) -> ServiceResult<()> {
    serde_json::from_str::<Value>(data)
        .map(|_| ())
        .map_err(|e| ServiceError::validation(format!("Invalid JSON data: {e}")))
}
