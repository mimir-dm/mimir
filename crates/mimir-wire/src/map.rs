//! Maps and what is on them (MIMIR-T-0711): the DM's full view, the
//! changes the DM makes, and the player view.
//!
//! Positions are grid cells (`grid_x`, `grid_y`, from 0 at the top left);
//! responses also give the pixel centre of the cell (`x`, `y`) in the served
//! map image. Geometry (walls, doors) is in grid units. Fog areas are in
//! pixels of the served image. Radii are in feet.

use serde::{Deserialize, Serialize};

use crate::patch::double;

/// A map, with its size and grid.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MapDetail {
    pub id: String,
    pub campaign_id: String,
    pub module_id: Option<String>,
    pub name: String,
    pub description: Option<String>,
    pub sort_order: i32,
    /// "bright", "dim" or "dark".
    pub lighting_mode: String,
    pub fog_enabled: bool,
    /// Pixels per grid cell in the served image.
    pub grid_size_px: f64,
    pub columns: f64,
    pub rows: f64,
    /// Size of the served image.
    pub width_px: i32,
    pub height_px: i32,
}

/// A point in grid units.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct GridPoint {
    pub x: f64,
    pub y: f64,
}

/// A door from the map file.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Portal {
    pub position: GridPoint,
    pub bounds: [GridPoint; 2],
    pub rotation: f64,
    pub closed: bool,
    pub freestanding: bool,
}

/// A light drawn into the map file.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MapFileLight {
    pub position: GridPoint,
    /// Grid units.
    pub range: f64,
    pub intensity: f64,
    /// RRGGBBAA hex.
    pub color: String,
    pub shadows: bool,
}

/// `GET /maps/{id}/geometry`: what blocks sight, from the map file.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MapGeometry {
    pub grid_size_px: f64,
    pub columns: f64,
    pub rows: f64,
    /// Each wall is a polyline.
    pub walls: Vec<Vec<GridPoint>>,
    pub portals: Vec<Portal>,
    pub lights: Vec<MapFileLight>,
    pub ambient_light: Option<String>,
    pub baked_lighting: bool,
}

/// A token on a map (DM view).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Token {
    pub id: String,
    pub map_id: String,
    pub name: String,
    /// "monster", "npc" or "pc".
    pub token_type: String,
    /// "tiny" … "gargantuan".
    pub size: String,
    pub grid_x: i32,
    pub grid_y: i32,
    pub x: f64,
    pub y: f64,
    /// Players see it (not hidden).
    pub visible_to_players: bool,
    pub color: Option<String>,
    /// The module monster, for a monster token.
    pub monster_id: Option<String>,
    /// The module NPC, for an NPC token.
    pub npc_id: Option<String>,
    pub vision_bright_ft: Option<i32>,
    pub vision_dim_ft: Option<i32>,
    pub vision_dark_ft: i32,
    pub light_radius_ft: i32,
}

/// `POST /maps/{id}/tokens`. A monster token names `monster_id`, an NPC
/// token `npc_id`; with neither it is a PC token and needs a `label`.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct NewToken {
    #[serde(default)]
    pub monster_id: Option<String>,
    #[serde(default)]
    pub npc_id: Option<String>,
    pub grid_x: i32,
    pub grid_y: i32,
    #[serde(default)]
    pub label: Option<String>,
    #[serde(default)]
    pub color: Option<String>,
    #[serde(default)]
    pub hidden: bool,
}

/// `PATCH /tokens/{id}`: move, rename, hide or show, change vision.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct TokenPatch {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub grid_x: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub grid_y: Option<i32>,
    #[serde(
        default,
        deserialize_with = "double",
        skip_serializing_if = "Option::is_none"
    )]
    pub label: Option<Option<String>>,
    #[serde(
        default,
        deserialize_with = "double",
        skip_serializing_if = "Option::is_none"
    )]
    pub color: Option<Option<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hidden: Option<bool>,
    #[serde(
        default,
        deserialize_with = "double",
        skip_serializing_if = "Option::is_none"
    )]
    pub vision_bright_ft: Option<Option<i32>>,
    #[serde(
        default,
        deserialize_with = "double",
        skip_serializing_if = "Option::is_none"
    )]
    pub vision_dim_ft: Option<Option<i32>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub vision_dark_ft: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub light_radius_ft: Option<i32>,
}

/// A revealed area of the fog, in pixels of the served image.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FogArea {
    pub id: String,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

/// `GET /maps/{id}/fog`
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Fog {
    pub enabled: bool,
    pub revealed: Vec<FogArea>,
}

/// `PUT /maps/{id}/fog`: fog on or off.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FogSetting {
    pub enabled: bool,
}

/// `POST /maps/{id}/fog/reveal`, in pixels of the served image. A circle is
/// kept as its bounding box.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "shape", rename_all = "snake_case")]
pub enum Reveal {
    Rect {
        x: f64,
        y: f64,
        width: f64,
        height: f64,
    },
    Circle {
        center_x: f64,
        center_y: f64,
        radius: f64,
    },
    /// The whole map.
    All,
}

/// A light the DM placed.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Light {
    pub id: String,
    pub map_id: String,
    pub name: Option<String>,
    pub grid_x: i32,
    pub grid_y: i32,
    pub x: f64,
    pub y: f64,
    pub bright_radius_ft: i32,
    pub dim_radius_ft: i32,
    pub color: Option<String>,
    pub active: bool,
}

/// `POST /maps/{id}/lights`. With a `preset` ("torch" or "lantern") the
/// preset sets the name, radii and colour.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct NewLight {
    pub grid_x: i32,
    pub grid_y: i32,
    #[serde(default)]
    pub preset: Option<String>,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub bright_radius_ft: Option<i32>,
    #[serde(default)]
    pub dim_radius_ft: Option<i32>,
    #[serde(default)]
    pub color: Option<String>,
    #[serde(default)]
    pub active: Option<bool>,
}

/// `PATCH /lights/{id}`
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct LightPatch {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub grid_x: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub grid_y: Option<i32>,
    #[serde(
        default,
        deserialize_with = "double",
        skip_serializing_if = "Option::is_none"
    )]
    pub name: Option<Option<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bright_radius_ft: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dim_radius_ft: Option<i32>,
    #[serde(
        default,
        deserialize_with = "double",
        skip_serializing_if = "Option::is_none"
    )]
    pub color: Option<Option<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub active: Option<bool>,
}

/// A trap (DM view: all of it).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Trap {
    pub id: String,
    pub map_id: String,
    pub grid_x: i32,
    pub grid_y: i32,
    pub name: String,
    pub description: Option<String>,
    pub trigger_description: Option<String>,
    pub effect_description: Option<String>,
    pub dc: Option<i32>,
    pub triggered: bool,
    /// Players see its marker.
    pub visible: bool,
}

/// `POST /maps/{id}/traps`
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct NewTrap {
    pub name: String,
    pub grid_x: i32,
    pub grid_y: i32,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub trigger_description: Option<String>,
    #[serde(default)]
    pub effect_description: Option<String>,
    #[serde(default)]
    pub dc: Option<i32>,
    #[serde(default)]
    pub visible: bool,
}

/// `PATCH /traps/{id}`. Text fields change when given (they cannot be
/// cleared yet); `triggered` false re-arms the trap.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct TrapPatch {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub grid_x: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub grid_y: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trigger_description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub effect_description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dc: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub visible: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub triggered: Option<bool>,
}

/// A point of interest.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Poi {
    pub id: String,
    pub map_id: String,
    pub grid_x: i32,
    pub grid_y: i32,
    pub name: String,
    pub description: Option<String>,
    pub icon: String,
    pub color: Option<String>,
    pub visible: bool,
}

/// `POST /maps/{id}/pois`
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct NewPoi {
    pub name: String,
    pub grid_x: i32,
    pub grid_y: i32,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub icon: Option<String>,
    #[serde(default)]
    pub color: Option<String>,
    #[serde(default)]
    pub visible: bool,
}

/// `PATCH /pois/{id}`. Text fields change when given.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct PoiPatch {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub grid_x: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub grid_y: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub visible: Option<bool>,
}

/// A token as the players see it: no monster or NPC link, no DM fields.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlayerToken {
    pub id: String,
    pub name: String,
    pub token_type: String,
    pub size: String,
    pub grid_x: i32,
    pub grid_y: i32,
    pub x: f64,
    pub y: f64,
    pub color: Option<String>,
    pub vision_bright_ft: Option<i32>,
    pub vision_dim_ft: Option<i32>,
    pub vision_dark_ft: i32,
    pub light_radius_ft: i32,
}

/// A lit light, as the players see it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlayerLight {
    pub id: String,
    pub grid_x: i32,
    pub grid_y: i32,
    pub x: f64,
    pub y: f64,
    pub bright_radius_ft: i32,
    pub dim_radius_ft: i32,
    pub color: Option<String>,
}

/// A marker the players see: a shown trap (name only) or a shown point of
/// interest.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlayerMarker {
    pub id: String,
    /// "trap" or "poi".
    pub kind: String,
    pub grid_x: i32,
    pub grid_y: i32,
    pub name: String,
    pub icon: String,
    pub color: Option<String>,
}

/// `GET /maps/{id}/player-view`: all that the player display may show.
/// Hidden tokens, unlit lights, hidden markers and trap details are not in
/// it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlayerView {
    pub map: MapDetail,
    pub fog: Fog,
    pub tokens: Vec<PlayerToken>,
    pub lights: Vec<PlayerLight>,
    pub markers: Vec<PlayerMarker>,
    /// The turn order, when the DM shows it and a combat runs.
    #[serde(default)]
    pub initiative: Option<crate::combat::PlayerInitiative>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_patch_tells_absent_from_null() {
        let p: TokenPatch = serde_json::from_str(r#"{"grid_x": 3, "label": null}"#).unwrap();
        assert_eq!(p.grid_x, Some(3));
        assert_eq!(p.grid_y, None);
        assert_eq!(p.label, Some(None), "null clears");
        assert_eq!(p.color, None, "absent keeps");
        let p: TokenPatch = serde_json::from_str(r#"{"color": "red"}"#).unwrap();
        assert_eq!(p.color, Some(Some("red".into())));
        // Serializing a patch drops absent fields and round-trips.
        let back: TokenPatch = serde_json::from_str(&serde_json::to_string(&p).unwrap()).unwrap();
        assert_eq!(back, p);
    }

    #[test]
    fn reveal_shapes_are_tagged() {
        let r: Reveal =
            serde_json::from_str(r#"{"shape":"circle","center_x":1,"center_y":2,"radius":3}"#)
                .unwrap();
        assert_eq!(
            r,
            Reveal::Circle {
                center_x: 1.0,
                center_y: 2.0,
                radius: 3.0
            }
        );
        assert_eq!(
            serde_json::to_string(&Reveal::All).unwrap(),
            r#"{"shape":"all"}"#
        );
    }

    #[test]
    fn a_new_token_needs_only_a_cell() {
        let t: NewToken =
            serde_json::from_str(r#"{"grid_x":1,"grid_y":2,"label":"Robin"}"#).unwrap();
        assert_eq!((t.grid_x, t.grid_y, t.hidden), (1, 2, false));
        assert_eq!(t.label.as_deref(), Some("Robin"));
    }
}
