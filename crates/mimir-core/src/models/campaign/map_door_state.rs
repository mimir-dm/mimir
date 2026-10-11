//! The doors of a map that the DM opened or closed (MIMIR-T-0728).

use crate::schema::map_door_states;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

/// A door of the map file in the state the DM set. A door with no row is
/// as the file has it.
#[derive(Debug, Clone, PartialEq, Queryable, Selectable, Insertable, Serialize, Deserialize)]
#[diesel(table_name = map_door_states)]
pub struct MapDoorState {
    pub map_id: String,
    /// The door's index in the file's portals.
    pub portal_index: i32,
    /// 1: closed, 0: open.
    pub closed: i32,
}
