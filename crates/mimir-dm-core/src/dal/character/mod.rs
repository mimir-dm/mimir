//! Character data access layer

use crate::connection::DbConnection;
use crate::error::Result;
use crate::models::character::{
    Character, CharacterVersion, NewCharacter, NewCharacterVersion, UpdateCharacter,
};
use crate::schema::{character_versions, characters};
use diesel::prelude::*;

/// Repository for character operations
pub struct CharacterRepository<'a> {
    conn: &'a mut DbConnection,
}

impl<'a> CharacterRepository<'a> {
    /// Creates a new character repository with the given database connection.
    pub fn new(conn: &'a mut DbConnection) -> Self {
        Self { conn }
    }

    /// Create a new character
    pub fn create(&mut self, new_character: NewCharacter) -> Result<Character> {
        diesel::insert_into(characters::table)
            .values(&new_character)
            .returning(Character::as_returning())
            .get_result(self.conn)
            .map_err(Into::into)
    }

    /// Find a character by ID
    pub fn find_by_id(&mut self, id: i32) -> Result<Option<Character>> {
        characters::table
            .find(id)
            .first(self.conn)
            .optional()
            .map_err(Into::into)
    }

    /// Update a character
    pub fn update(&mut self, id: i32, update: UpdateCharacter) -> Result<Character> {
        diesel::update(characters::table.find(id))
            .set(&update)
            .returning(Character::as_returning())
            .get_result(self.conn)
            .map_err(Into::into)
    }

    /// Delete a character
    pub fn delete(&mut self, id: i32) -> Result<()> {
        diesel::delete(characters::table.find(id)).execute(self.conn)?;
        Ok(())
    }

    /// List all characters (including unassigned)
    pub fn list_all(&mut self) -> Result<Vec<Character>> {
        characters::table
            .order_by(characters::character_name)
            .load(self.conn)
            .map_err(Into::into)
    }

    /// List all characters for a campaign
    pub fn list_for_campaign(&mut self, campaign_id: i32) -> Result<Vec<Character>> {
        characters::table
            .filter(characters::campaign_id.eq(campaign_id))
            .order_by(characters::character_name)
            .load(self.conn)
            .map_err(Into::into)
    }

    /// List characters for a specific player in a campaign
    pub fn list_for_player(&mut self, campaign_id: i32, player_id: i32) -> Result<Vec<Character>> {
        characters::table
            .filter(characters::campaign_id.eq(campaign_id))
            .filter(characters::player_id.eq(player_id))
            .order_by(characters::character_name)
            .load(self.conn)
            .map_err(Into::into)
    }

    /// List NPCs for a campaign
    pub fn list_npcs(&mut self, campaign_id: i32) -> Result<Vec<Character>> {
        characters::table
            .filter(characters::campaign_id.eq(campaign_id))
            .filter(characters::is_npc.eq(1))
            .order_by(characters::character_name)
            .load(self.conn)
            .map_err(Into::into)
    }

    /// List PCs for a campaign
    pub fn list_pcs(&mut self, campaign_id: i32) -> Result<Vec<Character>> {
        characters::table
            .filter(characters::campaign_id.eq(campaign_id))
            .filter(characters::is_npc.eq(0))
            .order_by(characters::character_name)
            .load(self.conn)
            .map_err(Into::into)
    }

    /// Find a character by name in a campaign
    pub fn find_by_name_in_campaign(
        &mut self,
        campaign_id: i32,
        name: &str,
    ) -> Result<Option<Character>> {
        characters::table
            .filter(characters::campaign_id.eq(campaign_id))
            .filter(characters::character_name.eq(name))
            .first(self.conn)
            .optional()
            .map_err(Into::into)
    }

    /// Find an NPC by name in a campaign
    pub fn find_npc_by_name_in_campaign(
        &mut self,
        campaign_id: i32,
        name: &str,
    ) -> Result<Option<Character>> {
        characters::table
            .filter(characters::campaign_id.eq(campaign_id))
            .filter(characters::character_name.eq(name))
            .filter(characters::is_npc.eq(1))
            .first(self.conn)
            .optional()
            .map_err(Into::into)
    }
}

/// Repository for character version operations
pub struct CharacterVersionRepository<'a> {
    conn: &'a mut DbConnection,
}

impl<'a> CharacterVersionRepository<'a> {
    /// Creates a new character version repository with the given database connection.
    pub fn new(conn: &'a mut DbConnection) -> Self {
        Self { conn }
    }

    /// Create a new character version
    pub fn create(&mut self, new_version: NewCharacterVersion) -> Result<CharacterVersion> {
        diesel::insert_into(character_versions::table)
            .values(&new_version)
            .returning(CharacterVersion::as_returning())
            .get_result(self.conn)
            .map_err(Into::into)
    }

    /// Find a specific version by character ID and version number
    pub fn find_by_character_and_version(
        &mut self,
        character_id: i32,
        version_number: i32,
    ) -> Result<Option<CharacterVersion>> {
        character_versions::table
            .filter(character_versions::character_id.eq(character_id))
            .filter(character_versions::version_number.eq(version_number))
            .first(self.conn)
            .optional()
            .map_err(Into::into)
    }

    /// Get the latest version for a character
    pub fn find_latest(&mut self, character_id: i32) -> Result<Option<CharacterVersion>> {
        character_versions::table
            .filter(character_versions::character_id.eq(character_id))
            .order_by(character_versions::version_number.desc())
            .first(self.conn)
            .optional()
            .map_err(Into::into)
    }

    /// List all versions for a character
    pub fn list_for_character(&mut self, character_id: i32) -> Result<Vec<CharacterVersion>> {
        character_versions::table
            .filter(character_versions::character_id.eq(character_id))
            .order_by(character_versions::version_number.asc())
            .load(self.conn)
            .map_err(Into::into)
    }

    /// Get the next version number for a character
    pub fn get_next_version_number(&mut self, character_id: i32) -> Result<i32> {
        let latest = self.find_latest(character_id)?;
        Ok(latest.map(|v| v.version_number + 1).unwrap_or(1))
    }

    /// Update the file path for a specific version
    pub fn update_file_path(&mut self, version_id: i32, file_path: String) -> Result<()> {
        diesel::update(character_versions::table.find(version_id))
            .set(character_versions::file_path.eq(file_path))
            .execute(self.conn)?;
        Ok(())
    }
}
