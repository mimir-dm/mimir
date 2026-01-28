//! Character Source Data Access Layer
//!
//! Database operations for character sources (allowed books per character).

use crate::models::campaign::{CharacterSource, NewCharacterSource};
use crate::schema::character_sources;
use diesel::prelude::*;
use diesel::SqliteConnection;

/// Insert a new character source.
pub fn insert_character_source(
    conn: &mut SqliteConnection,
    source: &NewCharacterSource,
) -> QueryResult<String> {
    diesel::insert_into(character_sources::table)
        .values(source)
        .execute(conn)?;

    Ok(source.id.to_string())
}

/// Get a character source by ID.
pub fn get_character_source(conn: &mut SqliteConnection, id: &str) -> QueryResult<CharacterSource> {
    character_sources::table.find(id).first(conn)
}

/// List all sources for a character.
pub fn list_character_sources(
    conn: &mut SqliteConnection,
    character_id: &str,
) -> QueryResult<Vec<CharacterSource>> {
    character_sources::table
        .filter(character_sources::character_id.eq(character_id))
        .order(character_sources::source_code.asc())
        .load(conn)
}

/// List source codes for a character (just the codes, not full objects).
pub fn list_character_source_codes(
    conn: &mut SqliteConnection,
    character_id: &str,
) -> QueryResult<Vec<String>> {
    character_sources::table
        .filter(character_sources::character_id.eq(character_id))
        .select(character_sources::source_code)
        .order(character_sources::source_code.asc())
        .load(conn)
}

/// Delete a character source by ID.
pub fn delete_character_source(conn: &mut SqliteConnection, id: &str) -> QueryResult<usize> {
    diesel::delete(character_sources::table.find(id)).execute(conn)
}

/// Delete a character source by character and source code.
pub fn delete_character_source_by_code(
    conn: &mut SqliteConnection,
    character_id: &str,
    source_code: &str,
) -> QueryResult<usize> {
    diesel::delete(
        character_sources::table
            .filter(character_sources::character_id.eq(character_id))
            .filter(character_sources::source_code.eq(source_code)),
    )
    .execute(conn)
}

/// Delete all sources for a character.
pub fn delete_all_character_sources(
    conn: &mut SqliteConnection,
    character_id: &str,
) -> QueryResult<usize> {
    diesel::delete(
        character_sources::table.filter(character_sources::character_id.eq(character_id)),
    )
    .execute(conn)
}

/// Check if a character has a specific source enabled.
pub fn character_has_source(
    conn: &mut SqliteConnection,
    character_id: &str,
    source_code: &str,
) -> QueryResult<bool> {
    use diesel::dsl::exists;
    use diesel::select;

    select(exists(
        character_sources::table
            .filter(character_sources::character_id.eq(character_id))
            .filter(character_sources::source_code.eq(source_code)),
    ))
    .get_result(conn)
}

/// Count sources for a character.
pub fn count_character_sources(conn: &mut SqliteConnection, character_id: &str) -> QueryResult<i64> {
    character_sources::table
        .filter(character_sources::character_id.eq(character_id))
        .count()
        .get_result(conn)
}
