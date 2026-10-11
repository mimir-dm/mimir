//! Door states DAL (MIMIR-T-0728).

use diesel::prelude::*;

use crate::models::campaign::MapDoorState;
use crate::schema::map_door_states;

/// The doors of a map the DM set, by index.
pub fn list_map_door_states(
    conn: &mut SqliteConnection,
    map_id: &str,
) -> QueryResult<Vec<MapDoorState>> {
    map_door_states::table
        .filter(map_door_states::map_id.eq(map_id))
        .order(map_door_states::portal_index.asc())
        .load(conn)
}

/// Set a door open or closed (insert or replace).
pub fn set_map_door_state(conn: &mut SqliteConnection, state: &MapDoorState) -> QueryResult<usize> {
    diesel::replace_into(map_door_states::table)
        .values(state)
        .execute(conn)
}

/// Put every door of a map back as the file has it.
pub fn delete_map_door_states(conn: &mut SqliteConnection, map_id: &str) -> QueryResult<usize> {
    diesel::delete(map_door_states::table.filter(map_door_states::map_id.eq(map_id))).execute(conn)
}
