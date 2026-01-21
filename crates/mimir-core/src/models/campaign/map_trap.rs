//! MapTrap Model
//!
//! Trap placements on maps for encounter building.

use crate::schema::map_traps;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

/// A trap placed on a map.
#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize, Deserialize)]
#[diesel(table_name = map_traps)]
pub struct MapTrap {
    /// Unique ID (UUID)
    pub id: String,
    /// Map this trap is placed on
    pub map_id: String,
    /// Grid X coordinate
    pub grid_x: i32,
    /// Grid Y coordinate
    pub grid_y: i32,
    /// Trap name
    pub name: String,
    /// Description
    pub description: Option<String>,
    /// How the trap is triggered
    pub trigger_description: Option<String>,
    /// What happens when triggered
    pub effect_description: Option<String>,
    /// Detection/disarm DC
    pub dc: Option<i32>,
    /// Has been triggered (0=armed, 1=triggered)
    pub triggered: i32,
    /// Visible to players (0=hidden, 1=visible)
    pub visible: i32,
    /// ISO8601 timestamp of creation
    pub created_at: String,
    /// ISO8601 timestamp of last update
    pub updated_at: String,
}

impl MapTrap {
    /// Check if this trap has been triggered.
    pub fn is_triggered(&self) -> bool {
        self.triggered != 0
    }

    /// Check if this trap is visible to players.
    pub fn is_visible(&self) -> bool {
        self.visible != 0
    }

    /// Check if this trap is still armed (not triggered).
    pub fn is_armed(&self) -> bool {
        self.triggered == 0
    }
}

/// Data for inserting a new map trap.
#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = map_traps)]
pub struct NewMapTrap<'a> {
    pub id: &'a str,
    pub map_id: &'a str,
    pub grid_x: i32,
    pub grid_y: i32,
    pub name: &'a str,
    pub description: Option<&'a str>,
    pub trigger_description: Option<&'a str>,
    pub effect_description: Option<&'a str>,
    pub dc: Option<i32>,
    pub triggered: i32,
    pub visible: i32,
}

impl<'a> NewMapTrap<'a> {
    /// Create a new map trap.
    pub fn new(id: &'a str, map_id: &'a str, name: &'a str, grid_x: i32, grid_y: i32) -> Self {
        Self {
            id,
            map_id,
            grid_x,
            grid_y,
            name,
            description: None,
            trigger_description: None,
            effect_description: None,
            dc: None,
            triggered: 0,
            visible: 0,
        }
    }

    /// Set description.
    pub fn with_description(mut self, description: &'a str) -> Self {
        self.description = Some(description);
        self
    }

    /// Set trigger description.
    pub fn with_trigger(mut self, trigger: &'a str) -> Self {
        self.trigger_description = Some(trigger);
        self
    }

    /// Set effect description.
    pub fn with_effect(mut self, effect: &'a str) -> Self {
        self.effect_description = Some(effect);
        self
    }

    /// Set detection/disarm DC.
    pub fn with_dc(mut self, dc: i32) -> Self {
        self.dc = Some(dc);
        self
    }

    /// Mark as visible to players.
    pub fn visible(mut self) -> Self {
        self.visible = 1;
        self
    }
}

/// Data for updating a map trap.
#[derive(Debug, Clone, Default, AsChangeset)]
#[diesel(table_name = map_traps)]
pub struct UpdateMapTrap<'a> {
    pub grid_x: Option<i32>,
    pub grid_y: Option<i32>,
    pub name: Option<&'a str>,
    pub description: Option<Option<&'a str>>,
    pub trigger_description: Option<Option<&'a str>>,
    pub effect_description: Option<Option<&'a str>>,
    pub dc: Option<Option<i32>>,
    pub triggered: Option<i32>,
    pub visible: Option<i32>,
    pub updated_at: Option<&'a str>,
}

impl<'a> UpdateMapTrap<'a> {
    /// Update position.
    pub fn set_position(grid_x: i32, grid_y: i32, updated_at: &'a str) -> Self {
        Self {
            grid_x: Some(grid_x),
            grid_y: Some(grid_y),
            updated_at: Some(updated_at),
            ..Default::default()
        }
    }

    /// Mark trap as triggered.
    pub fn trigger(updated_at: &'a str) -> Self {
        Self {
            triggered: Some(1),
            updated_at: Some(updated_at),
            ..Default::default()
        }
    }

    /// Reset trap to armed state.
    pub fn reset(updated_at: &'a str) -> Self {
        Self {
            triggered: Some(0),
            updated_at: Some(updated_at),
            ..Default::default()
        }
    }

    /// Set visibility to players.
    pub fn set_visible(visible: bool, updated_at: &'a str) -> Self {
        Self {
            visible: Some(if visible { 1 } else { 0 }),
            updated_at: Some(updated_at),
            ..Default::default()
        }
    }

    /// Update DC.
    pub fn set_dc(dc: Option<i32>, updated_at: &'a str) -> Self {
        Self {
            dc: Some(dc),
            updated_at: Some(updated_at),
            ..Default::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_map_trap() {
        let trap = NewMapTrap::new("trap-1", "map-1", "Pit Trap", 5, 10);
        assert_eq!(trap.name, "Pit Trap");
        assert_eq!(trap.grid_x, 5);
        assert_eq!(trap.grid_y, 10);
        assert_eq!(trap.triggered, 0);
        assert_eq!(trap.visible, 0);
    }

    #[test]
    fn test_with_descriptions() {
        let trap = NewMapTrap::new("trap-1", "map-1", "Arrow Trap", 0, 0)
            .with_description("A pressure plate triggers arrows from the wall")
            .with_trigger("Stepping on the plate")
            .with_effect("2d6 piercing damage");
        assert!(trap.description.is_some());
        assert!(trap.trigger_description.is_some());
        assert!(trap.effect_description.is_some());
    }

    #[test]
    fn test_with_dc() {
        let trap = NewMapTrap::new("trap-1", "map-1", "Hidden Spike", 0, 0).with_dc(15);
        assert_eq!(trap.dc, Some(15));
    }

    #[test]
    fn test_visible() {
        let trap = NewMapTrap::new("trap-1", "map-1", "Obvious Trap", 0, 0).visible();
        assert_eq!(trap.visible, 1);
    }

    #[test]
    fn test_update_trigger() {
        let update = UpdateMapTrap::trigger("2024-01-20T12:00:00Z");
        assert_eq!(update.triggered, Some(1));
    }

    #[test]
    fn test_update_reset() {
        let update = UpdateMapTrap::reset("2024-01-20T12:00:00Z");
        assert_eq!(update.triggered, Some(0));
    }

    #[test]
    fn test_update_visible() {
        let update = UpdateMapTrap::set_visible(true, "2024-01-20T12:00:00Z");
        assert_eq!(update.visible, Some(1));

        let update = UpdateMapTrap::set_visible(false, "2024-01-20T12:00:00Z");
        assert_eq!(update.visible, Some(0));
    }
}
