//! Token management service for Visual Display System.
//!
//! Provides business logic for managing map tokens - monsters, PCs, NPCs,
//! traps, and markers placed on battle maps.

use crate::connection::DbConnection;
use crate::error::Result;
use crate::models::campaign::{NewToken, Token, TokenSummary, UpdateToken};
use crate::schema::tokens;
use diesel::prelude::*;

/// Service for managing map tokens
pub struct TokenService<'a> {
    conn: &'a mut DbConnection,
}

impl<'a> TokenService<'a> {
    /// Create a new token service.
    pub fn new(conn: &'a mut DbConnection) -> Self {
        Self { conn }
    }

    /// Create a new token on a map.
    ///
    /// # Arguments
    /// * `new_token` - The token data to insert
    ///
    /// # Returns
    /// * `Ok(Token)` - The created token record
    pub fn create_token(&mut self, new_token: NewToken) -> Result<Token> {
        diesel::insert_into(tokens::table)
            .values(&new_token)
            .returning(Token::as_returning())
            .get_result(self.conn)
            .map_err(Into::into)
    }

    /// Get a token by ID.
    ///
    /// # Arguments
    /// * `id` - Database ID of the token
    ///
    /// # Returns
    /// * `Ok(Some(Token))` - If found
    /// * `Ok(None)` - If no token exists with that ID
    pub fn get_token(&mut self, id: i32) -> Result<Option<Token>> {
        tokens::table
            .find(id)
            .first(self.conn)
            .optional()
            .map_err(Into::into)
    }

    /// List all tokens for a map.
    ///
    /// # Arguments
    /// * `map_id` - Database ID of the map
    ///
    /// # Returns
    /// * `Ok(Vec<Token>)` - All tokens on the map
    pub fn list_tokens_for_map(&mut self, map_id: i32) -> Result<Vec<Token>> {
        tokens::table
            .filter(tokens::map_id.eq(map_id))
            .order(tokens::name.asc())
            .load(self.conn)
            .map_err(Into::into)
    }

    /// List only visible tokens for a map (for player display).
    ///
    /// # Arguments
    /// * `map_id` - Database ID of the map
    ///
    /// # Returns
    /// * `Ok(Vec<Token>)` - Visible tokens on the map
    pub fn list_visible_tokens_for_map(&mut self, map_id: i32) -> Result<Vec<Token>> {
        tokens::table
            .filter(tokens::map_id.eq(map_id))
            .filter(tokens::visible_to_players.eq(true))
            .order(tokens::name.asc())
            .load(self.conn)
            .map_err(Into::into)
    }

    /// Get token summaries with resolved monster/character names.
    ///
    /// # Arguments
    /// * `map_id` - Database ID of the map
    ///
    /// # Returns
    /// * `Ok(Vec<TokenSummary>)` - Token summaries with linked entity names
    pub fn list_token_summaries(&mut self, map_id: i32) -> Result<Vec<TokenSummary>> {
        use crate::schema::{catalog_monsters, characters};

        // Get all tokens for the map
        let all_tokens: Vec<Token> = tokens::table
            .filter(tokens::map_id.eq(map_id))
            .order(tokens::name.asc())
            .load(self.conn)?;

        // Get monster names for tokens that have monster_id
        let monster_ids: Vec<i32> = all_tokens
            .iter()
            .filter_map(|t| t.monster_id)
            .collect();

        let monster_names: Vec<(i32, String)> = if !monster_ids.is_empty() {
            catalog_monsters::table
                .filter(catalog_monsters::id.eq_any(&monster_ids))
                .select((catalog_monsters::id, catalog_monsters::name))
                .load(self.conn)?
        } else {
            vec![]
        };

        // Get character names for tokens that have character_id
        let character_ids: Vec<i32> = all_tokens
            .iter()
            .filter_map(|t| t.character_id)
            .collect();

        let character_names: Vec<(i32, String)> = if !character_ids.is_empty() {
            characters::table
                .filter(characters::id.eq_any(&character_ids))
                .select((characters::id, characters::character_name))
                .load(self.conn)?
        } else {
            vec![]
        };

        // Build summaries
        let summaries = all_tokens
            .into_iter()
            .map(|t| {
                let monster_name = t.monster_id.and_then(|mid| {
                    monster_names
                        .iter()
                        .find(|(id, _)| *id == mid)
                        .map(|(_, name)| name.clone())
                });

                let character_name = t.character_id.and_then(|cid| {
                    character_names
                        .iter()
                        .find(|(id, _)| *id == cid)
                        .map(|(_, name)| name.clone())
                });

                TokenSummary {
                    id: t.id,
                    map_id: t.map_id,
                    name: t.name,
                    token_type: t.token_type,
                    size: t.size,
                    x: t.x,
                    y: t.y,
                    visible_to_players: t.visible_to_players,
                    color: t.color,
                    image_path: t.image_path,
                    monster_id: t.monster_id,
                    monster_name,
                    character_id: t.character_id,
                    character_name,
                    vision_type: t.vision_type,
                    vision_range_ft: t.vision_range_ft,
                }
            })
            .collect();

        Ok(summaries)
    }

    /// Update a token.
    ///
    /// # Arguments
    /// * `id` - Database ID of the token
    /// * `update` - Fields to update
    ///
    /// # Returns
    /// * `Ok(Token)` - The updated token
    pub fn update_token(&mut self, id: i32, mut update: UpdateToken) -> Result<Token> {
        // Set updated_at timestamp
        update.updated_at = Some(chrono::Utc::now().to_rfc3339());

        diesel::update(tokens::table.find(id))
            .set(&update)
            .returning(Token::as_returning())
            .get_result(self.conn)
            .map_err(Into::into)
    }

    /// Update just the position of a token (for drag operations).
    ///
    /// # Arguments
    /// * `id` - Database ID of the token
    /// * `x` - New X position (grid coordinates)
    /// * `y` - New Y position (grid coordinates)
    ///
    /// # Returns
    /// * `Ok(Token)` - The updated token
    pub fn update_token_position(&mut self, id: i32, x: f32, y: f32) -> Result<Token> {
        let update = UpdateToken::position(x, y);
        self.update_token(id, update)
    }

    /// Toggle token visibility.
    ///
    /// # Arguments
    /// * `id` - Database ID of the token
    ///
    /// # Returns
    /// * `Ok(Token)` - The updated token with toggled visibility
    pub fn toggle_token_visibility(&mut self, id: i32) -> Result<Token> {
        // First get current visibility
        let token = self.get_token(id)?
            .ok_or_else(|| crate::error::DbError::NotFound {
                entity_type: "Token".to_string(),
                id: id.to_string(),
            })?;

        let update = UpdateToken::visibility(!token.visible_to_players);
        self.update_token(id, update)
    }

    /// Set token visibility explicitly.
    ///
    /// # Arguments
    /// * `id` - Database ID of the token
    /// * `visible` - Whether token should be visible to players
    ///
    /// # Returns
    /// * `Ok(Token)` - The updated token
    pub fn set_token_visibility(&mut self, id: i32, visible: bool) -> Result<Token> {
        let update = UpdateToken::visibility(visible);
        self.update_token(id, update)
    }

    /// Update positions of multiple tokens (for group moves).
    ///
    /// # Arguments
    /// * `updates` - Vec of (token_id, x, y) tuples
    ///
    /// # Returns
    /// * `Ok(Vec<Token>)` - The updated tokens
    pub fn bulk_update_positions(&mut self, updates: Vec<(i32, f32, f32)>) -> Result<Vec<Token>> {
        let mut results = Vec::with_capacity(updates.len());
        for (id, x, y) in updates {
            results.push(self.update_token_position(id, x, y)?);
        }
        Ok(results)
    }

    /// Delete a token.
    ///
    /// # Arguments
    /// * `id` - Database ID of the token to delete
    ///
    /// # Returns
    /// * `Ok(())` - If deletion succeeds
    pub fn delete_token(&mut self, id: i32) -> Result<()> {
        diesel::delete(tokens::table.find(id))
            .execute(self.conn)?;
        Ok(())
    }

    /// Delete all tokens for a map.
    ///
    /// Useful when deleting a map or resetting token positions.
    ///
    /// # Arguments
    /// * `map_id` - Database ID of the map
    ///
    /// # Returns
    /// * `Ok(usize)` - Number of tokens deleted
    pub fn delete_tokens_for_map(&mut self, map_id: i32) -> Result<usize> {
        let count = diesel::delete(tokens::table.filter(tokens::map_id.eq(map_id)))
            .execute(self.conn)?;
        Ok(count)
    }

    /// Count tokens on a map.
    ///
    /// # Arguments
    /// * `map_id` - Database ID of the map
    ///
    /// # Returns
    /// * `Ok(i64)` - Number of tokens on the map
    pub fn count_tokens_for_map(&mut self, map_id: i32) -> Result<i64> {
        tokens::table
            .filter(tokens::map_id.eq(map_id))
            .count()
            .get_result(self.conn)
            .map_err(Into::into)
    }
}
