//! Player data access layer

use crate::connection::DbConnection;
use crate::error::Result;
use crate::models::player::{
    CampaignPlayer, NewCampaignPlayer, NewPlayer, Player, UpdateCampaignPlayer, UpdatePlayer,
};
use crate::schema::{campaign_players, players};
use diesel::prelude::*;

/// Repository for player operations
pub struct PlayerRepository<'a> {
    conn: &'a mut DbConnection,
}

impl<'a> PlayerRepository<'a> {
    /// Creates a new player repository with the given database connection.
    pub fn new(conn: &'a mut DbConnection) -> Self {
        Self { conn }
    }

    /// Create a new player
    pub fn create(&mut self, new_player: NewPlayer) -> Result<Player> {
        diesel::insert_into(players::table)
            .values(&new_player)
            .returning(Player::as_returning())
            .get_result(self.conn)
            .map_err(Into::into)
    }

    /// Find a player by ID
    pub fn find_by_id(&mut self, id: i32) -> Result<Option<Player>> {
        players::table
            .find(id)
            .first(self.conn)
            .optional()
            .map_err(Into::into)
    }

    /// Update a player
    pub fn update(&mut self, id: i32, update: UpdatePlayer) -> Result<Player> {
        diesel::update(players::table.find(id))
            .set(&update)
            .returning(Player::as_returning())
            .get_result(self.conn)
            .map_err(Into::into)
    }

    /// Delete a player
    pub fn delete(&mut self, id: i32) -> Result<()> {
        diesel::delete(players::table.find(id)).execute(self.conn)?;
        Ok(())
    }

    /// List all players
    pub fn list(&mut self) -> Result<Vec<Player>> {
        players::table
            .order_by(players::name)
            .load(self.conn)
            .map_err(Into::into)
    }
}

/// Repository for campaign-player association operations
pub struct CampaignPlayerRepository<'a> {
    conn: &'a mut DbConnection,
}

impl<'a> CampaignPlayerRepository<'a> {
    /// Creates a new campaign-player repository with the given database connection.
    pub fn new(conn: &'a mut DbConnection) -> Self {
        Self { conn }
    }

    /// Add a player to a campaign
    pub fn add(&mut self, new_association: NewCampaignPlayer) -> Result<CampaignPlayer> {
        diesel::insert_into(campaign_players::table)
            .values(&new_association)
            .returning(CampaignPlayer::as_returning())
            .get_result(self.conn)
            .map_err(Into::into)
    }

    /// Remove a player from a campaign
    pub fn remove(&mut self, campaign_id: i32, player_id: i32) -> Result<()> {
        diesel::delete(
            campaign_players::table
                .filter(campaign_players::campaign_id.eq(campaign_id))
                .filter(campaign_players::player_id.eq(player_id)),
        )
        .execute(self.conn)?;
        Ok(())
    }

    /// Update campaign-player association (e.g., set active status)
    pub fn update(
        &mut self,
        campaign_id: i32,
        player_id: i32,
        update: UpdateCampaignPlayer,
    ) -> Result<CampaignPlayer> {
        diesel::update(
            campaign_players::table
                .filter(campaign_players::campaign_id.eq(campaign_id))
                .filter(campaign_players::player_id.eq(player_id)),
        )
        .set(&update)
        .returning(CampaignPlayer::as_returning())
        .get_result(self.conn)
        .map_err(Into::into)
    }

    /// List all players for a campaign
    pub fn list_for_campaign(&mut self, campaign_id: i32) -> Result<Vec<Player>> {
        campaign_players::table
            .inner_join(players::table)
            .filter(campaign_players::campaign_id.eq(campaign_id))
            .select(Player::as_select())
            .load(self.conn)
            .map_err(Into::into)
    }

    /// List active players for a campaign
    pub fn list_active_for_campaign(&mut self, campaign_id: i32) -> Result<Vec<Player>> {
        campaign_players::table
            .inner_join(players::table)
            .filter(campaign_players::campaign_id.eq(campaign_id))
            .filter(campaign_players::active.eq(true))
            .select(Player::as_select())
            .load(self.conn)
            .map_err(Into::into)
    }

    /// Check if a player is associated with a campaign
    pub fn is_player_in_campaign(&mut self, campaign_id: i32, player_id: i32) -> Result<bool> {
        let count: i64 = campaign_players::table
            .filter(campaign_players::campaign_id.eq(campaign_id))
            .filter(campaign_players::player_id.eq(player_id))
            .count()
            .get_result(self.conn)?;
        Ok(count > 0)
    }
}
