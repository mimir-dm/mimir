//! A player link (MIMIR-T-0716): the hash of a character's link token.

use diesel::prelude::*;
use serde::{Deserialize, Serialize};

use crate::schema::player_links;

#[derive(Debug, Clone, PartialEq, Queryable, Selectable, Identifiable, Serialize, Deserialize)]
#[diesel(table_name = player_links)]
#[diesel(primary_key(character_id))]
pub struct PlayerLink {
    pub character_id: String,
    /// hex SHA-256 of the token.
    pub token_hash: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = player_links)]
pub struct NewPlayerLink<'a> {
    pub character_id: &'a str,
    pub token_hash: &'a str,
}
