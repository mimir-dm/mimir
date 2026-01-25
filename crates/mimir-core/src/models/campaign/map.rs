//! Map Model
//!
//! Map metadata and initial play state. UVTT files remain the source of truth
//! for grid, walls, and lighting geometry.

use crate::schema::maps;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

/// A map in a campaign or module.
#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize, Deserialize)]
#[diesel(table_name = maps)]
pub struct Map {
    /// Unique ID (UUID)
    pub id: String,
    /// Campaign this map belongs to
    pub campaign_id: String,
    /// Module this map belongs to (optional - None for campaign-level maps)
    pub module_id: Option<String>,
    /// Display name
    pub name: String,
    /// Description or notes
    pub description: Option<String>,
    /// Sort order within campaign/module
    pub sort_order: i32,
    /// Reference to the UVTT asset in blob storage
    pub uvtt_asset_id: String,
    /// Initial lighting mode: bright, dim, or dark
    pub lighting_mode: String,
    /// Whether fog of war is enabled
    pub fog_enabled: i32,
    /// ISO8601 timestamp of creation
    pub created_at: String,
    /// ISO8601 timestamp of last update
    pub updated_at: String,
}

impl Map {
    /// Check if this is a module-level map.
    pub fn is_module_map(&self) -> bool {
        self.module_id.is_some()
    }

    /// Check if this is a campaign-level map (e.g., world map).
    pub fn is_campaign_map(&self) -> bool {
        self.module_id.is_none()
    }

    /// Check if lighting mode is bright.
    pub fn is_bright(&self) -> bool {
        self.lighting_mode == "bright"
    }

    /// Check if lighting mode is dim.
    pub fn is_dim(&self) -> bool {
        self.lighting_mode == "dim"
    }

    /// Check if lighting mode is dark.
    pub fn is_dark(&self) -> bool {
        self.lighting_mode == "dark"
    }

    /// Check if fog of war is enabled.
    pub fn is_fog_enabled(&self) -> bool {
        self.fog_enabled != 0
    }
}

/// Lighting mode for initial play state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LightingMode {
    /// Full visibility
    Bright,
    /// Partial visibility
    Dim,
    /// No visibility (fog of war)
    Dark,
}

impl LightingMode {
    /// Convert to string for database storage.
    pub fn as_str(&self) -> &'static str {
        match self {
            LightingMode::Bright => "bright",
            LightingMode::Dim => "dim",
            LightingMode::Dark => "dark",
        }
    }

    /// Parse from string.
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "bright" => Some(LightingMode::Bright),
            "dim" => Some(LightingMode::Dim),
            "dark" => Some(LightingMode::Dark),
            _ => None,
        }
    }
}

impl Default for LightingMode {
    fn default() -> Self {
        LightingMode::Bright
    }
}

/// Data for inserting a new map.
#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = maps)]
pub struct NewMap<'a> {
    pub id: &'a str,
    pub campaign_id: &'a str,
    pub module_id: Option<&'a str>,
    pub name: &'a str,
    pub description: Option<&'a str>,
    pub sort_order: i32,
    pub uvtt_asset_id: &'a str,
    pub lighting_mode: &'a str,
    pub fog_enabled: i32,
}

impl<'a> NewMap<'a> {
    /// Create a new campaign-level map (e.g., world map).
    pub fn for_campaign(
        id: &'a str,
        campaign_id: &'a str,
        name: &'a str,
        uvtt_asset_id: &'a str,
    ) -> Self {
        Self {
            id,
            campaign_id,
            module_id: None,
            name,
            description: None,
            sort_order: 0,
            uvtt_asset_id,
            lighting_mode: LightingMode::default().as_str(),
            fog_enabled: 0,
        }
    }

    /// Create a new module-level map (e.g., dungeon map).
    pub fn for_module(
        id: &'a str,
        campaign_id: &'a str,
        module_id: &'a str,
        name: &'a str,
        uvtt_asset_id: &'a str,
    ) -> Self {
        Self {
            id,
            campaign_id,
            module_id: Some(module_id),
            name,
            description: None,
            sort_order: 0,
            uvtt_asset_id,
            lighting_mode: LightingMode::default().as_str(),
            fog_enabled: 0,
        }
    }

    /// Set description.
    pub fn with_description(mut self, description: &'a str) -> Self {
        self.description = Some(description);
        self
    }

    /// Set sort order.
    pub fn with_sort_order(mut self, sort_order: i32) -> Self {
        self.sort_order = sort_order;
        self
    }

    /// Set lighting mode.
    pub fn with_lighting_mode(mut self, mode: LightingMode) -> Self {
        self.lighting_mode = mode.as_str();
        self
    }
}

/// Data for updating a map.
#[derive(Debug, Clone, Default, AsChangeset)]
#[diesel(table_name = maps)]
pub struct UpdateMap<'a> {
    pub name: Option<&'a str>,
    pub description: Option<Option<&'a str>>,
    pub sort_order: Option<i32>,
    pub lighting_mode: Option<&'a str>,
    pub fog_enabled: Option<i32>,
    pub module_id: Option<Option<&'a str>>,
    pub updated_at: Option<&'a str>,
}

impl<'a> UpdateMap<'a> {
    /// Update the name.
    pub fn set_name(name: &'a str, updated_at: &'a str) -> Self {
        Self {
            name: Some(name),
            updated_at: Some(updated_at),
            ..Default::default()
        }
    }

    /// Update the description.
    pub fn set_description(description: Option<&'a str>, updated_at: &'a str) -> Self {
        Self {
            description: Some(description),
            updated_at: Some(updated_at),
            ..Default::default()
        }
    }

    /// Update the sort order.
    pub fn set_sort_order(sort_order: i32, updated_at: &'a str) -> Self {
        Self {
            sort_order: Some(sort_order),
            updated_at: Some(updated_at),
            ..Default::default()
        }
    }

    /// Update the lighting mode.
    pub fn set_lighting_mode(mode: LightingMode, updated_at: &'a str) -> Self {
        Self {
            lighting_mode: Some(mode.as_str()),
            updated_at: Some(updated_at),
            ..Default::default()
        }
    }

    /// Enable fog of war.
    pub fn enable_fog(updated_at: &'a str) -> Self {
        Self {
            fog_enabled: Some(1),
            updated_at: Some(updated_at),
            ..Default::default()
        }
    }

    /// Disable fog of war.
    pub fn disable_fog(updated_at: &'a str) -> Self {
        Self {
            fog_enabled: Some(0),
            updated_at: Some(updated_at),
            ..Default::default()
        }
    }

    /// Set fog enabled state.
    pub fn set_fog_enabled(enabled: bool, updated_at: &'a str) -> Self {
        Self {
            fog_enabled: Some(if enabled { 1 } else { 0 }),
            updated_at: Some(updated_at),
            ..Default::default()
        }
    }

    /// Move map to a module.
    pub fn move_to_module(module_id: &'a str, updated_at: &'a str) -> Self {
        Self {
            module_id: Some(Some(module_id)),
            updated_at: Some(updated_at),
            ..Default::default()
        }
    }

    /// Move map to campaign level (out of module).
    pub fn move_to_campaign(updated_at: &'a str) -> Self {
        Self {
            module_id: Some(None),
            updated_at: Some(updated_at),
            ..Default::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_campaign_map() {
        let map = NewMap::for_campaign("map-1", "camp-1", "World Map", "asset-1");
        assert_eq!(map.name, "World Map");
        assert!(map.module_id.is_none());
        assert_eq!(map.lighting_mode, "bright");
    }

    #[test]
    fn test_new_module_map() {
        let map = NewMap::for_module("map-1", "camp-1", "mod-1", "Goblin Cave", "asset-1");
        assert_eq!(map.name, "Goblin Cave");
        assert_eq!(map.module_id, Some("mod-1"));
    }

    #[test]
    fn test_with_lighting_mode() {
        let map = NewMap::for_module("map-1", "camp-1", "mod-1", "Dark Dungeon", "asset-1")
            .with_lighting_mode(LightingMode::Dark);
        assert_eq!(map.lighting_mode, "dark");
    }

    #[test]
    fn test_with_sort_order() {
        let map = NewMap::for_module("map-1", "camp-1", "mod-1", "Floor 2", "asset-1")
            .with_sort_order(2);
        assert_eq!(map.sort_order, 2);
    }

    #[test]
    fn test_lighting_mode_as_str() {
        assert_eq!(LightingMode::Bright.as_str(), "bright");
        assert_eq!(LightingMode::Dim.as_str(), "dim");
        assert_eq!(LightingMode::Dark.as_str(), "dark");
    }

    #[test]
    fn test_lighting_mode_from_str() {
        assert_eq!(LightingMode::from_str("bright"), Some(LightingMode::Bright));
        assert_eq!(LightingMode::from_str("dim"), Some(LightingMode::Dim));
        assert_eq!(LightingMode::from_str("dark"), Some(LightingMode::Dark));
        assert_eq!(LightingMode::from_str("invalid"), None);
    }

    #[test]
    fn test_update_lighting_mode() {
        let update = UpdateMap::set_lighting_mode(LightingMode::Dim, "2024-01-20T12:00:00Z");
        assert_eq!(update.lighting_mode, Some("dim"));
    }

    #[test]
    fn test_update_move_to_module() {
        let update = UpdateMap::move_to_module("mod-1", "2024-01-20T12:00:00Z");
        assert_eq!(update.module_id, Some(Some("mod-1")));
    }

    #[test]
    fn test_update_move_to_campaign() {
        let update = UpdateMap::move_to_campaign("2024-01-20T12:00:00Z");
        assert_eq!(update.module_id, Some(None));
    }
}
