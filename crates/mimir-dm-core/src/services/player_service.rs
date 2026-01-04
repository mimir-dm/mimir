//! Player service for business logic operations
//!
//! This service handles all player-related business logic including:
//! - Player CRUD operations
//! - Campaign association management
//! - Player listing and filtering

use crate::{
    connection::DbConnection,
    dal::player::{CampaignPlayerRepository, PlayerRepository},
    error::{DbError, Result},
    models::player::{NewCampaignPlayer, NewPlayer, Player, UpdatePlayer},
};

/// Service for player-related business logic operations.
pub struct PlayerService<'a> {
    conn: &'a mut DbConnection,
}

impl<'a> PlayerService<'a> {
    /// Create a new player service
    pub fn new(conn: &'a mut DbConnection) -> Self {
        Self { conn }
    }

    /// Create a new player
    pub fn create_player(
        &mut self,
        name: &str,
        email: Option<String>,
        notes: Option<String>,
    ) -> Result<Player> {
        // Validate inputs
        if name.trim().is_empty() {
            return Err(DbError::InvalidData(
                "Player name cannot be empty".to_string(),
            ));
        }

        let mut repo = PlayerRepository::new(self.conn);
        let new_player = NewPlayer {
            name: name.to_string(),
            email,
            notes,
        };

        repo.create(new_player)
    }

    /// Get a player by ID
    pub fn get_player(&mut self, player_id: i32) -> Result<Player> {
        let mut repo = PlayerRepository::new(self.conn);
        repo.find_by_id(player_id)?
            .ok_or_else(|| DbError::NotFound {
                entity_type: "Player".to_string(),
                id: player_id.to_string(),
            })
    }

    /// Update a player
    pub fn update_player(
        &mut self,
        player_id: i32,
        name: Option<String>,
        email: Option<Option<String>>,
        notes: Option<Option<String>>,
    ) -> Result<Player> {
        // Validate name if provided
        if let Some(ref n) = name {
            if n.trim().is_empty() {
                return Err(DbError::InvalidData(
                    "Player name cannot be empty".to_string(),
                ));
            }
        }

        let mut repo = PlayerRepository::new(self.conn);
        let update = UpdatePlayer { name, email, notes };
        repo.update(player_id, update)
    }

    /// Delete a player
    pub fn delete_player(&mut self, player_id: i32) -> Result<()> {
        // Note: Due to ON DELETE CASCADE, this will also remove:
        // - campaign_players entries
        // - characters entries
        // - character_versions entries (via characters cascade)
        let mut repo = PlayerRepository::new(self.conn);
        repo.delete(player_id)
    }

    /// List all players
    pub fn list_players(&mut self) -> Result<Vec<Player>> {
        let mut repo = PlayerRepository::new(self.conn);
        repo.list()
    }

    /// Add a player to a campaign
    pub fn add_player_to_campaign(&mut self, campaign_id: i32, player_id: i32) -> Result<()> {
        let mut cp_repo = CampaignPlayerRepository::new(self.conn);

        // Check if association already exists
        if cp_repo.is_player_in_campaign(campaign_id, player_id)? {
            return Err(DbError::ConstraintViolation {
                field: "campaign_player".to_string(),
                message: format!(
                    "Player {} already associated with campaign {}",
                    player_id, campaign_id
                ),
            });
        }

        let new_association = NewCampaignPlayer {
            campaign_id,
            player_id,
        };

        cp_repo.add(new_association)?;
        Ok(())
    }

    /// Remove a player from a campaign
    pub fn remove_player_from_campaign(&mut self, campaign_id: i32, player_id: i32) -> Result<()> {
        let mut cp_repo = CampaignPlayerRepository::new(self.conn);
        cp_repo.remove(campaign_id, player_id)
    }

    /// Set a player's active status in a campaign
    pub fn set_player_active_status(
        &mut self,
        campaign_id: i32,
        player_id: i32,
        active: bool,
    ) -> Result<()> {
        let mut cp_repo = CampaignPlayerRepository::new(self.conn);
        let update = crate::models::player::UpdateCampaignPlayer {
            active: Some(active),
        };
        cp_repo.update(campaign_id, player_id, update)?;
        Ok(())
    }

    /// List all players for a campaign
    pub fn list_players_for_campaign(&mut self, campaign_id: i32) -> Result<Vec<Player>> {
        let mut cp_repo = CampaignPlayerRepository::new(self.conn);
        cp_repo.list_for_campaign(campaign_id)
    }

    /// List active players for a campaign
    pub fn list_active_players_for_campaign(&mut self, campaign_id: i32) -> Result<Vec<Player>> {
        let mut cp_repo = CampaignPlayerRepository::new(self.conn);
        cp_repo.list_active_for_campaign(campaign_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::establish_connection;
    use diesel::prelude::*;

    fn setup_test_db() -> DbConnection {
        let mut conn = establish_connection(":memory:").expect("Failed to create in-memory DB");
        crate::run_migrations(&mut conn).expect("Failed to run migrations");
        conn
    }

    #[test]
    fn test_create_player() {
        let mut conn = setup_test_db();
        let mut service = PlayerService::new(&mut conn);

        let player = service
            .create_player("Test Player", Some("test@example.com".to_string()), None)
            .expect("Failed to create player");

        assert_eq!(player.name, "Test Player");
        assert_eq!(player.email, Some("test@example.com".to_string()));
    }

    #[test]
    fn test_create_player_empty_name() {
        let mut conn = setup_test_db();
        let mut service = PlayerService::new(&mut conn);

        let result = service.create_player("", None, None);
        assert!(result.is_err());
    }

    #[test]
    fn test_get_player() {
        let mut conn = setup_test_db();
        let mut service = PlayerService::new(&mut conn);

        let created = service
            .create_player("Test Player", None, None)
            .expect("Failed to create player");

        let fetched = service
            .get_player(created.id)
            .expect("Failed to get player");
        assert_eq!(fetched.id, created.id);
        assert_eq!(fetched.name, "Test Player");
    }

    #[test]
    fn test_update_player() {
        let mut conn = setup_test_db();
        let mut service = PlayerService::new(&mut conn);

        let player = service
            .create_player("Original Name", None, None)
            .expect("Failed to create player");

        let updated = service
            .update_player(player.id, Some("Updated Name".to_string()), None, None)
            .expect("Failed to update player");

        assert_eq!(updated.name, "Updated Name");
    }

    #[test]
    fn test_delete_player() {
        let mut conn = setup_test_db();
        let mut service = PlayerService::new(&mut conn);

        let player = service
            .create_player("To Delete", None, None)
            .expect("Failed to create player");

        service
            .delete_player(player.id)
            .expect("Failed to delete player");

        let result = service.get_player(player.id);
        assert!(result.is_err());
    }

    #[test]
    fn test_list_players() {
        let mut conn = setup_test_db();
        let mut service = PlayerService::new(&mut conn);

        service.create_player("Player 1", None, None).unwrap();
        service.create_player("Player 2", None, None).unwrap();

        let players = service.list_players().expect("Failed to list players");
        assert_eq!(players.len(), 2);
    }

    #[test]
    fn test_add_player_to_campaign() {
        let mut conn = setup_test_db();

        // Create a campaign first
        let campaign = diesel::insert_into(crate::schema::campaigns::table)
            .values((
                crate::schema::campaigns::name.eq("Test Campaign"),
                crate::schema::campaigns::status.eq("concept"),
                crate::schema::campaigns::directory_path.eq("/test"),
            ))
            .returning(crate::models::campaign::Campaign::as_returning())
            .get_result(&mut conn)
            .expect("Failed to create campaign");

        let mut service = PlayerService::new(&mut conn);
        let player = service.create_player("Test Player", None, None).unwrap();

        service
            .add_player_to_campaign(campaign.id, player.id)
            .expect("Failed to add player to campaign");

        let players = service
            .list_players_for_campaign(campaign.id)
            .expect("Failed to list players for campaign");
        assert_eq!(players.len(), 1);
        assert_eq!(players[0].id, player.id);
    }

    #[test]
    fn test_add_player_to_campaign_duplicate() {
        let mut conn = setup_test_db();

        let campaign = diesel::insert_into(crate::schema::campaigns::table)
            .values((
                crate::schema::campaigns::name.eq("Test Campaign"),
                crate::schema::campaigns::status.eq("concept"),
                crate::schema::campaigns::directory_path.eq("/test"),
            ))
            .returning(crate::models::campaign::Campaign::as_returning())
            .get_result(&mut conn)
            .expect("Failed to create campaign");

        let mut service = PlayerService::new(&mut conn);
        let player = service.create_player("Test Player", None, None).unwrap();

        service
            .add_player_to_campaign(campaign.id, player.id)
            .unwrap();

        // Try to add again - should fail
        let result = service.add_player_to_campaign(campaign.id, player.id);
        assert!(result.is_err());
    }

    #[test]
    fn test_remove_player_from_campaign() {
        let mut conn = setup_test_db();

        let campaign = diesel::insert_into(crate::schema::campaigns::table)
            .values((
                crate::schema::campaigns::name.eq("Test Campaign"),
                crate::schema::campaigns::status.eq("concept"),
                crate::schema::campaigns::directory_path.eq("/test"),
            ))
            .returning(crate::models::campaign::Campaign::as_returning())
            .get_result(&mut conn)
            .expect("Failed to create campaign");

        let mut service = PlayerService::new(&mut conn);
        let player = service.create_player("Test Player", None, None).unwrap();

        service
            .add_player_to_campaign(campaign.id, player.id)
            .unwrap();
        service
            .remove_player_from_campaign(campaign.id, player.id)
            .expect("Failed to remove player from campaign");

        let players = service.list_players_for_campaign(campaign.id).unwrap();
        assert_eq!(players.len(), 0);
    }

    #[test]
    fn test_set_player_active_status() {
        let mut conn = setup_test_db();

        let campaign = diesel::insert_into(crate::schema::campaigns::table)
            .values((
                crate::schema::campaigns::name.eq("Test Campaign"),
                crate::schema::campaigns::status.eq("concept"),
                crate::schema::campaigns::directory_path.eq("/test"),
            ))
            .returning(crate::models::campaign::Campaign::as_returning())
            .get_result(&mut conn)
            .expect("Failed to create campaign");

        let mut service = PlayerService::new(&mut conn);
        let player = service.create_player("Test Player", None, None).unwrap();

        service
            .add_player_to_campaign(campaign.id, player.id)
            .unwrap();

        // Initially active (default)
        let active_players = service
            .list_active_players_for_campaign(campaign.id)
            .unwrap();
        assert_eq!(active_players.len(), 1);

        // Set to inactive
        service
            .set_player_active_status(campaign.id, player.id, false)
            .expect("Failed to set inactive");

        let active_players = service
            .list_active_players_for_campaign(campaign.id)
            .unwrap();
        assert_eq!(active_players.len(), 0);
    }
}
