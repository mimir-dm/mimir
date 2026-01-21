//! TokenPlacement Model
//!
//! Token placements for monsters and NPCs on maps.
//! PCs are placed at runtime via frontend state, not persisted here.

use crate::schema::token_placements;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

/// A token placement on a map, representing either a monster or NPC.
#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize, Deserialize)]
#[diesel(table_name = token_placements)]
pub struct TokenPlacement {
    /// Unique ID (UUID)
    pub id: String,
    /// Map this token is placed on
    pub map_id: String,
    /// Module monster reference (mutually exclusive with module_npc_id)
    pub module_monster_id: Option<String>,
    /// Module NPC reference (mutually exclusive with module_monster_id)
    pub module_npc_id: Option<String>,
    /// Grid X coordinate
    pub grid_x: i32,
    /// Grid Y coordinate
    pub grid_y: i32,
    /// Optional override label
    pub label: Option<String>,
    /// Faction color (hex, e.g., "#FF0000" for enemy)
    pub faction_color: Option<String>,
    /// Hidden from players initially (0=visible, 1=hidden)
    pub hidden: i32,
    /// ISO8601 timestamp of creation
    pub created_at: String,
}

impl TokenPlacement {
    /// Check if this is a monster token.
    pub fn is_monster(&self) -> bool {
        self.module_monster_id.is_some()
    }

    /// Check if this is an NPC token.
    pub fn is_npc(&self) -> bool {
        self.module_npc_id.is_some()
    }

    /// Check if this token is hidden from players.
    pub fn is_hidden(&self) -> bool {
        self.hidden != 0
    }
}

/// Data for inserting a new token placement.
#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = token_placements)]
pub struct NewTokenPlacement<'a> {
    pub id: &'a str,
    pub map_id: &'a str,
    pub module_monster_id: Option<&'a str>,
    pub module_npc_id: Option<&'a str>,
    pub grid_x: i32,
    pub grid_y: i32,
    pub label: Option<&'a str>,
    pub faction_color: Option<&'a str>,
    pub hidden: i32,
}

impl<'a> NewTokenPlacement<'a> {
    /// Create a new token placement for a monster.
    pub fn for_monster(
        id: &'a str,
        map_id: &'a str,
        module_monster_id: &'a str,
        grid_x: i32,
        grid_y: i32,
    ) -> Self {
        Self {
            id,
            map_id,
            module_monster_id: Some(module_monster_id),
            module_npc_id: None,
            grid_x,
            grid_y,
            label: None,
            faction_color: None,
            hidden: 0,
        }
    }

    /// Create a new token placement for an NPC.
    pub fn for_npc(
        id: &'a str,
        map_id: &'a str,
        module_npc_id: &'a str,
        grid_x: i32,
        grid_y: i32,
    ) -> Self {
        Self {
            id,
            map_id,
            module_monster_id: None,
            module_npc_id: Some(module_npc_id),
            grid_x,
            grid_y,
            label: None,
            faction_color: None,
            hidden: 0,
        }
    }

    /// Set a custom label.
    pub fn with_label(mut self, label: &'a str) -> Self {
        self.label = Some(label);
        self
    }

    /// Set faction color.
    pub fn with_faction_color(mut self, color: &'a str) -> Self {
        self.faction_color = Some(color);
        self
    }

    /// Mark as hidden from players.
    pub fn hidden(mut self) -> Self {
        self.hidden = 1;
        self
    }
}

/// Data for updating a token placement.
#[derive(Debug, Clone, Default, AsChangeset)]
#[diesel(table_name = token_placements)]
pub struct UpdateTokenPlacement<'a> {
    pub grid_x: Option<i32>,
    pub grid_y: Option<i32>,
    pub label: Option<Option<&'a str>>,
    pub faction_color: Option<Option<&'a str>>,
    pub hidden: Option<i32>,
}

impl<'a> UpdateTokenPlacement<'a> {
    /// Update position.
    pub fn set_position(grid_x: i32, grid_y: i32) -> Self {
        Self {
            grid_x: Some(grid_x),
            grid_y: Some(grid_y),
            ..Default::default()
        }
    }

    /// Update label.
    pub fn set_label(label: Option<&'a str>) -> Self {
        Self {
            label: Some(label),
            ..Default::default()
        }
    }

    /// Update faction color.
    pub fn set_faction_color(color: Option<&'a str>) -> Self {
        Self {
            faction_color: Some(color),
            ..Default::default()
        }
    }

    /// Update hidden state.
    pub fn set_hidden(hidden: bool) -> Self {
        Self {
            hidden: Some(if hidden { 1 } else { 0 }),
            ..Default::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_monster_placement() {
        let placement = NewTokenPlacement::for_monster("tp-1", "map-1", "mm-1", 5, 10);
        assert!(placement.module_monster_id.is_some());
        assert!(placement.module_npc_id.is_none());
        assert_eq!(placement.grid_x, 5);
        assert_eq!(placement.grid_y, 10);
        assert_eq!(placement.hidden, 0);
    }

    #[test]
    fn test_new_npc_placement() {
        let placement = NewTokenPlacement::for_npc("tp-1", "map-1", "npc-1", 3, 7);
        assert!(placement.module_monster_id.is_none());
        assert!(placement.module_npc_id.is_some());
    }

    #[test]
    fn test_with_label() {
        let placement =
            NewTokenPlacement::for_monster("tp-1", "map-1", "mm-1", 0, 0).with_label("Goblin Boss");
        assert_eq!(placement.label, Some("Goblin Boss"));
    }

    #[test]
    fn test_with_faction_color() {
        let placement = NewTokenPlacement::for_monster("tp-1", "map-1", "mm-1", 0, 0)
            .with_faction_color("#FF0000");
        assert_eq!(placement.faction_color, Some("#FF0000"));
    }

    #[test]
    fn test_hidden() {
        let placement = NewTokenPlacement::for_monster("tp-1", "map-1", "mm-1", 0, 0).hidden();
        assert_eq!(placement.hidden, 1);
    }

    #[test]
    fn test_update_position() {
        let update = UpdateTokenPlacement::set_position(15, 20);
        assert_eq!(update.grid_x, Some(15));
        assert_eq!(update.grid_y, Some(20));
    }

    #[test]
    fn test_update_hidden() {
        let update = UpdateTokenPlacement::set_hidden(true);
        assert_eq!(update.hidden, Some(1));

        let update = UpdateTokenPlacement::set_hidden(false);
        assert_eq!(update.hidden, Some(0));
    }
}
