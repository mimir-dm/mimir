//! Combat tracker data access (COLLIERY-I-0468).

use crate::models::campaign::{CombatEntry, CombatSession, NewCombatEntry, NewCombatSession};
use crate::schema::{combat_entries, combat_sessions};
use diesel::prelude::*;
use diesel::SqliteConnection;

/// Insert a combat session.
pub fn insert_combat_session(
    conn: &mut SqliteConnection,
    session: &NewCombatSession,
) -> QueryResult<usize> {
    diesel::insert_into(combat_sessions::table)
        .values(session)
        .execute(conn)
}

/// Get a combat session by ID.
pub fn get_combat_session_optional(
    conn: &mut SqliteConnection,
    id: &str,
) -> QueryResult<Option<CombatSession>> {
    combat_sessions::table.find(id).first(conn).optional()
}

/// The module's active combat session, if any.
pub fn get_active_combat_session(
    conn: &mut SqliteConnection,
    module_id: &str,
) -> QueryResult<Option<CombatSession>> {
    combat_sessions::table
        .filter(combat_sessions::module_id.eq(module_id))
        .filter(combat_sessions::status.eq("active"))
        .first(conn)
        .optional()
}

/// Write every column of a session.
pub fn save_combat_session(
    conn: &mut SqliteConnection,
    session: &CombatSession,
) -> QueryResult<usize> {
    diesel::update(session).set(session).execute(conn)
}

/// Insert a combat entry.
pub fn insert_combat_entry(
    conn: &mut SqliteConnection,
    entry: &NewCombatEntry,
) -> QueryResult<usize> {
    diesel::insert_into(combat_entries::table)
        .values(entry)
        .execute(conn)
}

/// Get a combat entry by ID.
pub fn get_combat_entry_optional(
    conn: &mut SqliteConnection,
    id: &str,
) -> QueryResult<Option<CombatEntry>> {
    combat_entries::table.find(id).first(conn).optional()
}

/// All entries of a session (unordered; the service orders them).
pub fn list_combat_entries(
    conn: &mut SqliteConnection,
    session_id: &str,
) -> QueryResult<Vec<CombatEntry>> {
    combat_entries::table
        .filter(combat_entries::session_id.eq(session_id))
        .load(conn)
}

/// Write every column of an entry.
pub fn save_combat_entry(conn: &mut SqliteConnection, entry: &CombatEntry) -> QueryResult<usize> {
    diesel::update(entry).set(entry).execute(conn)
}

/// Delete an entry.
pub fn delete_combat_entry(conn: &mut SqliteConnection, id: &str) -> QueryResult<usize> {
    diesel::delete(combat_entries::table.find(id)).execute(conn)
}
