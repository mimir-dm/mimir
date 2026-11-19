//! Player notes and sheet shares data access (MIMIR-T-0718).

use diesel::prelude::*;
use diesel::SqliteConnection;

use crate::models::campaign::{NewPlayerNote, NewSheetShare, PlayerNote, UpdatePlayerNote};
use crate::schema::{characters, player_notes, sheet_shares};

pub fn insert_player_note(conn: &mut SqliteConnection, note: &NewPlayerNote) -> QueryResult<usize> {
    diesel::insert_into(player_notes::table)
        .values(note)
        .execute(conn)
}

pub fn get_player_note(conn: &mut SqliteConnection, id: &str) -> QueryResult<Option<PlayerNote>> {
    player_notes::table.find(id).first(conn).optional()
}

/// A character's notes, newest change first.
pub fn list_player_notes(
    conn: &mut SqliteConnection,
    character_id: &str,
) -> QueryResult<Vec<PlayerNote>> {
    player_notes::table
        .filter(player_notes::character_id.eq(character_id))
        .order(player_notes::updated_at.desc())
        .load(conn)
}

/// The notes about a module, with the name of each note's character.
pub fn list_module_player_notes(
    conn: &mut SqliteConnection,
    module_id: &str,
) -> QueryResult<Vec<(PlayerNote, String)>> {
    player_notes::table
        .inner_join(characters::table)
        .filter(player_notes::module_id.eq(module_id))
        .order(player_notes::updated_at.desc())
        .select((PlayerNote::as_select(), characters::name))
        .load(conn)
}

pub fn update_player_note(
    conn: &mut SqliteConnection,
    id: &str,
    update: &UpdatePlayerNote,
) -> QueryResult<usize> {
    diesel::update(player_notes::table.find(id))
        .set(update)
        .execute(conn)
}

pub fn delete_player_note(conn: &mut SqliteConnection, id: &str) -> QueryResult<usize> {
    diesel::delete(player_notes::table.find(id)).execute(conn)
}

pub fn is_sheet_shared(conn: &mut SqliteConnection, character_id: &str) -> QueryResult<bool> {
    use diesel::dsl::exists;
    diesel::select(exists(sheet_shares::table.find(character_id))).get_result(conn)
}

pub fn share_sheet(conn: &mut SqliteConnection, character_id: &str) -> QueryResult<usize> {
    diesel::insert_or_ignore_into(sheet_shares::table)
        .values(&NewSheetShare { character_id })
        .execute(conn)
}

pub fn unshare_sheet(conn: &mut SqliteConnection, character_id: &str) -> QueryResult<usize> {
    diesel::delete(sheet_shares::table.find(character_id)).execute(conn)
}

/// The ids of the shared sheets among these characters.
pub fn shared_among(conn: &mut SqliteConnection, ids: &[String]) -> QueryResult<Vec<String>> {
    sheet_shares::table
        .filter(sheet_shares::character_id.eq_any(ids))
        .select(sheet_shares::character_id)
        .load(conn)
}
