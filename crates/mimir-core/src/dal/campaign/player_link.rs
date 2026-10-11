//! Player link data access (MIMIR-T-0716).

use diesel::prelude::*;
use diesel::SqliteConnection;

use crate::models::campaign::{NewPlayerLink, PlayerLink};
use crate::schema::player_links;

/// Replace the link of a character (a new token).
pub fn replace_player_link(
    conn: &mut SqliteConnection,
    link: &NewPlayerLink,
) -> QueryResult<usize> {
    diesel::replace_into(player_links::table)
        .values(link)
        .execute(conn)
}

pub fn get_player_link(
    conn: &mut SqliteConnection,
    character_id: &str,
) -> QueryResult<Option<PlayerLink>> {
    player_links::table
        .find(character_id)
        .first(conn)
        .optional()
}

pub fn find_player_link_by_hash(
    conn: &mut SqliteConnection,
    token_hash: &str,
) -> QueryResult<Option<PlayerLink>> {
    player_links::table
        .filter(player_links::token_hash.eq(token_hash))
        .first(conn)
        .optional()
}

pub fn delete_player_link(conn: &mut SqliteConnection, character_id: &str) -> QueryResult<usize> {
    diesel::delete(player_links::table.find(character_id)).execute(conn)
}
