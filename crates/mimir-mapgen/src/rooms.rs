//! Declarative room and corridor layout types and generation.
//!
//! Rooms are defined by grid position and dimensions, with per-wall toggles
//! and portal (door/window) declarations. Corridors connect rooms by ID.
//! All coordinates are in grid squares — converted to pixels internally (× 256).

use serde::{Deserialize, Serialize};
use std::f64::consts::FRAC_PI_2;

use crate::format::entities::{MapPortal, MapWall};
use crate::format::godot_types::Vector2;
use crate::format::NodeIdAllocator;

/// Which side of a room a wall, portal, or corridor attaches to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum WallSide {
    North,
    South,
    East,
    West,
}

/// Semantic portal type — maps to Dungeondraft portal textures.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PortalType {
    Door,
    Window,
    Archway,
    SecretDoor,
}

impl PortalType {
    /// Return the default Dungeondraft texture path for this portal type.
    pub fn default_texture(&self) -> &'static str {
        match self {
            PortalType::Door => "res://textures/portals/door_00.png",
            PortalType::Window => "res://textures/portals/window_03.png",
            PortalType::Archway => "res://textures/portals/archway_00.png",
            PortalType::SecretDoor => "res://textures/portals/secret_00.png",
        }
    }

    /// Return the default radius (half-width in pixels) for this portal type.
    pub fn default_radius(&self) -> f64 {
        match self {
            PortalType::Door => 128.0,      // 1 grid square
            PortalType::Window => 99.5,     // slightly smaller
            PortalType::Archway => 128.0,   // 1 grid square
            PortalType::SecretDoor => 128.0, // 1 grid square
        }
    }
}

/// A portal (door/window/archway) declaration on a room wall.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortalConfig {
    /// Which wall this portal is on.
    pub wall: WallSide,
    /// Offset in grid squares along the wall (from left/top of that wall).
    pub position: u32,
    /// Portal type — determines texture and default size.
    #[serde(rename = "type")]
    pub portal_type: PortalType,
    /// Portal width in grid squares (default: 1).
    #[serde(default = "default_portal_width")]
    pub width: u32,
}

fn default_portal_width() -> u32 {
    1
}

/// Per-wall toggle — which sides of the room have walls.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WallToggles {
    #[serde(default = "default_true")]
    pub north: bool,
    #[serde(default = "default_true")]
    pub south: bool,
    #[serde(default = "default_true")]
    pub east: bool,
    #[serde(default = "default_true")]
    pub west: bool,
}

fn default_true() -> bool {
    true
}

impl Default for WallToggles {
    fn default() -> Self {
        Self {
            north: true,
            south: true,
            east: true,
            west: true,
        }
    }
}

impl WallToggles {
    /// Check if a specific wall side is enabled.
    pub fn is_enabled(&self, side: WallSide) -> bool {
        match side {
            WallSide::North => self.north,
            WallSide::South => self.south,
            WallSide::East => self.east,
            WallSide::West => self.west,
        }
    }
}

/// A room declaration — a rectangular area on the grid.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoomConfig {
    /// Unique identifier for this room (referenced by corridors).
    pub id: String,
    /// Grid X position (left edge, in grid squares).
    pub x: u32,
    /// Grid Y position (top edge, in grid squares).
    pub y: u32,
    /// Room width in grid squares.
    pub width: u32,
    /// Room height in grid squares.
    pub height: u32,
    /// Which terrain texture slot fills the floor (0–3, indexes into terrain textures).
    #[serde(default)]
    pub terrain_slot: Option<usize>,
    /// Per-wall toggles — defaults to all walls enabled.
    #[serde(default)]
    pub walls: WallToggles,
    /// Portal declarations on this room's walls.
    #[serde(default)]
    pub portals: Vec<PortalConfig>,
}

/// A corridor connecting two rooms.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorridorConfig {
    /// Source room ID.
    pub from: String,
    /// Which wall of the source room the corridor exits from.
    pub from_wall: WallSide,
    /// Destination room ID.
    pub to: String,
    /// Which wall of the destination room the corridor enters.
    pub to_wall: WallSide,
    /// Corridor width in grid squares (default: 2).
    #[serde(default = "default_corridor_width")]
    pub width: u32,
    /// Terrain texture slot for corridor floor.
    #[serde(default)]
    pub terrain_slot: Option<usize>,
    /// Portals at corridor ends (doors at entrances/exits).
    #[serde(default)]
    pub portals: Vec<CorridorPortalConfig>,
}

fn default_corridor_width() -> u32 {
    2
}

/// A portal declaration at a corridor end.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorridorPortalConfig {
    /// Which end of the corridor: "from" or "to".
    pub end: CorridorEnd,
    /// Portal type.
    #[serde(rename = "type")]
    pub portal_type: PortalType,
    /// Portal width in grid squares (default: 1).
    #[serde(default = "default_portal_width")]
    pub width: u32,
}

/// Which end of a corridor a portal sits at.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CorridorEnd {
    From,
    To,
}

/// Pixels per grid square in Dungeondraft maps.
pub const PIXELS_PER_GRID: f64 = 256.0;

/// An axis-aligned rectangular exclusion zone in pixel coordinates.
#[derive(Debug, Clone)]
pub struct ExclusionZone {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

impl ExclusionZone {
    /// Check if a point (in pixel coordinates) falls inside this zone.
    pub fn contains(&self, px: f64, py: f64) -> bool {
        px >= self.x && px <= self.x + self.width && py >= self.y && py <= self.y + self.height
    }
}

/// Build exclusion zones from room configs (rooms + corridors).
pub fn build_exclusion_zones(rooms: &[RoomConfig], corridors: &[CorridorConfig]) -> Vec<ExclusionZone> {
    let mut zones = Vec::new();

    // Pad exclusion zones by 1 grid square so trees/clutter don't overlap walls.
    let padding = PIXELS_PER_GRID;

    for room in rooms {
        zones.push(ExclusionZone {
            x: room.x as f64 * PIXELS_PER_GRID - padding,
            y: room.y as f64 * PIXELS_PER_GRID - padding,
            width: room.width as f64 * PIXELS_PER_GRID + padding * 2.0,
            height: room.height as f64 * PIXELS_PER_GRID + padding * 2.0,
        });
    }

    // Corridor zones are approximated from room connection points
    for corridor in corridors {
        let from_room = rooms.iter().find(|r| r.id == corridor.from);
        let to_room = rooms.iter().find(|r| r.id == corridor.to);
        if let (Some(from), Some(to)) = (from_room, to_room) {
            let (fx, fy) = wall_connection_point(from, corridor.from_wall, corridor.width);
            let (tx, ty) = wall_connection_point(to, corridor.to_wall, corridor.width);
            let half_w = (corridor.width as f64 * PIXELS_PER_GRID) / 2.0;

            // For straight corridors, one zone. For L-shaped, two zones.
            let min_x = fx.min(tx);
            let max_x = fx.max(tx);
            let min_y = fy.min(ty);
            let max_y = fy.max(ty);

            if (fy - ty).abs() < 1.0 {
                // Straight horizontal
                zones.push(ExclusionZone {
                    x: min_x,
                    y: fy - half_w,
                    width: max_x - min_x,
                    height: half_w * 2.0,
                });
            } else if (fx - tx).abs() < 1.0 {
                // Straight vertical
                zones.push(ExclusionZone {
                    x: fx - half_w,
                    y: min_y,
                    width: half_w * 2.0,
                    height: max_y - min_y,
                });
            } else {
                // L-shaped: horizontal then vertical
                zones.push(ExclusionZone {
                    x: min_x,
                    y: fy - half_w,
                    width: max_x - min_x + half_w,
                    height: half_w * 2.0,
                });
                zones.push(ExclusionZone {
                    x: tx - half_w,
                    y: min_y,
                    width: half_w * 2.0,
                    height: max_y - min_y,
                });
            }
        }
    }

    zones
}

/// Check if a point falls inside any exclusion zone.
pub fn is_excluded(zones: &[ExclusionZone], px: f64, py: f64) -> bool {
    zones.iter().any(|z| z.contains(px, py))
}

/// Result of laying out rooms: walls, portals, shapes, and terrain overrides.
#[derive(Debug, Clone)]
pub struct RoomLayoutResult {
    /// Wall segments generated from room boundaries.
    pub walls: Vec<MapWall>,
    /// Freestanding portals (if any).
    pub portals: Vec<MapPortal>,
    /// Shape wall IDs as decimal integers (parallel with `shape_polygons`).
    pub shape_wall_ids: Vec<serde_json::Value>,
    /// Shape polygons as PoolVector2Array values (parallel with `shape_wall_ids`).
    pub shape_polygons: Vec<serde_json::Value>,
    /// Terrain override regions — rectangular areas to fill with a specific slot.
    pub terrain_overrides: Vec<TerrainOverride>,
}

/// A rectangular region to override in the terrain splat map.
#[derive(Debug, Clone)]
pub struct TerrainOverride {
    /// Top-left X in splat-map cells (grid_x × 4).
    pub cell_x: usize,
    /// Top-left Y in splat-map cells (grid_y × 4).
    pub cell_y: usize,
    /// Width in splat-map cells (grid_width × 4).
    pub cell_width: usize,
    /// Height in splat-map cells (grid_height × 4).
    pub cell_height: usize,
    /// Which terrain slot (0–3) to fill.
    pub slot: usize,
}

impl TerrainOverride {
    /// Apply this override to a terrain splat map.
    ///
    /// `splat_data` is a flat array of 4-byte cells, row-major,
    /// with `map_cells_x` cells per row (= map_width_grids × 4).
    pub fn apply(&self, splat_data: &mut [u8], map_cells_x: usize) {
        let mut weights = [0u8; 4];
        weights[self.slot] = 255;

        for dy in 0..self.cell_height {
            let row = self.cell_y + dy;
            for dx in 0..self.cell_width {
                let col = self.cell_x + dx;
                let idx = (row * map_cells_x + col) * 4;
                if idx + 4 <= splat_data.len() {
                    splat_data[idx..idx + 4].copy_from_slice(&weights);
                }
            }
        }
    }
}

/// Generate the full room and corridor layout.
///
/// Each room becomes a closed-polygon `MapWall` with embedded portals.
/// Corridors generate parallel wall segments connecting rooms.
/// Returns walls, portals, shapes data, and terrain overrides.
pub fn generate_room_layout(
    rooms: &[RoomConfig],
    corridors: &[CorridorConfig],
    alloc: &NodeIdAllocator,
) -> RoomLayoutResult {
    let mut result = RoomLayoutResult {
        walls: Vec::new(),
        portals: Vec::new(),
        shape_wall_ids: Vec::new(),
        shape_polygons: Vec::new(),
        terrain_overrides: Vec::new(),
    };

    for room in rooms {
        generate_single_room(room, alloc, &mut result);
    }

    for corridor in corridors {
        generate_corridor(corridor, rooms, alloc, &mut result);
    }

    result
}

/// Generate walls, portals, shapes, and terrain override for a single room.
fn generate_single_room(
    room: &RoomConfig,
    alloc: &NodeIdAllocator,
    result: &mut RoomLayoutResult,
) {
    let px = room.x as f64 * PIXELS_PER_GRID;
    let py = room.y as f64 * PIXELS_PER_GRID;
    let pw = room.width as f64 * PIXELS_PER_GRID;
    let ph = room.height as f64 * PIXELS_PER_GRID;

    // Room corners: TL, TR, BR, BL (clockwise for closed polygon)
    let corners = [
        Vector2::new(px, py),           // top-left (NW)
        Vector2::new(px + pw, py),      // top-right (NE)
        Vector2::new(px + pw, py + ph), // bottom-right (SE)
        Vector2::new(px, py + ph),      // bottom-left (SW)
    ];

    // Segments: North (0→1), East (1→2), South (2→3), West (3→0)
    let sides = [WallSide::North, WallSide::East, WallSide::South, WallSide::West];

    // Build the wall points — only include sides that have walls enabled
    let mut points = Vec::new();
    let mut enabled_sides = Vec::new();

    for (i, &side) in sides.iter().enumerate() {
        if room.walls.is_enabled(side) {
            points.push(corners[i]);
            enabled_sides.push((i, side));
        }
    }

    // If no walls enabled, skip wall generation (but still do terrain)
    if !points.is_empty() {
        let wall_id = alloc.next();

        // Collect portals for this room
        let mut wall_portals = Vec::new();
        for portal_cfg in &room.portals {
            if let Some(portal) = make_portal(room, portal_cfg, &wall_id, &corners, alloc) {
                wall_portals.push(portal);
            }
        }

        // Build closed-polygon wall with all room corners (even disabled sides
        // still contribute corners to the polygon shape)
        let wall = MapWall::new_room(
            corners.to_vec(),
            "res://textures/walls/battlements.png",
            &wall_id,
        )
        .with_portals(wall_portals);

        // Add shapes entry: wall ID as decimal integer, polygon as the room outline
        let wall_id_decimal: i64 = wall_id.parse().unwrap_or(0);
        result.shape_wall_ids.push(serde_json::Value::Number(
            serde_json::Number::from(wall_id_decimal),
        ));

        // Build polygon value as a PoolVector2Array string
        let polygon_str = format!(
            "PoolVector2Array( {} )",
            corners
                .iter()
                .map(|c| format!("{}, {}", c.x, c.y))
                .collect::<Vec<_>>()
                .join(", ")
        );
        result
            .shape_polygons
            .push(serde_json::Value::String(polygon_str));

        result.walls.push(wall);
    }

    // Terrain override
    if let Some(slot) = room.terrain_slot {
        result.terrain_overrides.push(TerrainOverride {
            cell_x: (room.x * 4) as usize,
            cell_y: (room.y * 4) as usize,
            cell_width: (room.width * 4) as usize,
            cell_height: (room.height * 4) as usize,
            slot,
        });
    }
}

/// Create a MapPortal from a room's PortalConfig.
fn make_portal(
    room: &RoomConfig,
    portal: &PortalConfig,
    wall_id: &str,
    _corners: &[Vector2; 4],
    alloc: &NodeIdAllocator,
) -> Option<MapPortal> {
    let px = room.x as f64 * PIXELS_PER_GRID;
    let py = room.y as f64 * PIXELS_PER_GRID;
    let pw = room.width as f64 * PIXELS_PER_GRID;
    let ph = room.height as f64 * PIXELS_PER_GRID;

    // Portal center offset along the wall (in pixels from wall start)
    let portal_center_offset =
        (portal.position as f64 + portal.width as f64 / 2.0) * PIXELS_PER_GRID;

    // Compute position, rotation, direction, and point_index based on wall side.
    // Direction = wall segment direction (NOT outward normal).
    // Rotation = angle of the wall segment.
    let (position, rotation, direction, point_index) = match portal.wall {
        WallSide::North => {
            // North wall segment goes east: corners[0]→corners[1]
            let pos = Vector2::new(px + portal_center_offset, py);
            (pos, 0.0, Vector2::new(1.0, 0.0), 0)
        }
        WallSide::East => {
            // East wall segment goes south: corners[1]→corners[2]
            let pos = Vector2::new(px + pw, py + portal_center_offset);
            (pos, FRAC_PI_2, Vector2::new(0.0, 1.0), 1)
        }
        WallSide::South => {
            // South wall segment goes west: corners[2]→corners[3]
            let pos = Vector2::new(px + pw - portal_center_offset, py + ph);
            (pos, std::f64::consts::PI, Vector2::new(-1.0, 0.0), 2)
        }
        WallSide::West => {
            // West wall segment goes north: corners[3]→corners[0]
            let pos = Vector2::new(px, py + ph - portal_center_offset);
            (pos, -FRAC_PI_2, Vector2::new(0.0, -1.0), 3)
        }
    };

    // wall_distance = point_index + fraction along the segment
    let segment_length = match portal.wall {
        WallSide::North | WallSide::South => pw,
        WallSide::East | WallSide::West => ph,
    };
    let fraction = portal_center_offset / segment_length;
    let wall_distance = point_index as f64 + fraction;

    let radius = portal.portal_type.default_radius();

    Some(MapPortal::new(
        position,
        rotation,
        direction,
        portal.portal_type.default_texture(),
        radius,
        point_index,
        wall_id,
        wall_distance,
        &alloc.next(),
    ))
}

/// Find a room by ID in the rooms list.
fn find_room<'a>(rooms: &'a [RoomConfig], id: &str) -> Option<&'a RoomConfig> {
    rooms.iter().find(|r| r.id == id)
}

/// Get the connection point on a room's wall for a corridor.
///
/// Returns the center point of the wall side in pixel coordinates
/// and the direction the corridor extends from the room.
fn wall_connection_point(room: &RoomConfig, side: WallSide, _corridor_width: u32) -> (f64, f64) {
    let px = room.x as f64 * PIXELS_PER_GRID;
    let py = room.y as f64 * PIXELS_PER_GRID;
    let pw = room.width as f64 * PIXELS_PER_GRID;
    let ph = room.height as f64 * PIXELS_PER_GRID;

    match side {
        WallSide::North => (px + pw / 2.0, py),
        WallSide::South => (px + pw / 2.0, py + ph),
        WallSide::East => (px + pw, py + ph / 2.0),
        WallSide::West => (px, py + ph / 2.0),
    }
}

/// Generate corridor walls and terrain between two rooms.
fn generate_corridor(
    corridor: &CorridorConfig,
    rooms: &[RoomConfig],
    alloc: &NodeIdAllocator,
    result: &mut RoomLayoutResult,
) {
    let from_room = match find_room(rooms, &corridor.from) {
        Some(r) => r,
        None => return,
    };
    let to_room = match find_room(rooms, &corridor.to) {
        Some(r) => r,
        None => return,
    };

    let (from_x, from_y) = wall_connection_point(from_room, corridor.from_wall, corridor.width);
    let (to_x, to_y) = wall_connection_point(to_room, corridor.to_wall, corridor.width);

    let half_width = (corridor.width as f64 * PIXELS_PER_GRID) / 2.0;

    // Determine if this is a straight or L-shaped corridor
    let is_horizontal = matches!(
        (corridor.from_wall, corridor.to_wall),
        (WallSide::East, WallSide::West) | (WallSide::West, WallSide::East)
    );
    let is_vertical = matches!(
        (corridor.from_wall, corridor.to_wall),
        (WallSide::North, WallSide::South) | (WallSide::South, WallSide::North)
    );

    if is_horizontal && (from_y - to_y).abs() < 1.0 {
        // Straight horizontal corridor
        let min_x = from_x.min(to_x);
        let max_x = from_x.max(to_x);
        let center_y = from_y;

        generate_straight_corridor_walls(
            min_x, center_y - half_width,
            max_x, center_y + half_width,
            true, // horizontal
            corridor, alloc, result,
        );
    } else if is_vertical && (from_x - to_x).abs() < 1.0 {
        // Straight vertical corridor
        let min_y = from_y.min(to_y);
        let max_y = from_y.max(to_y);
        let center_x = from_x;

        generate_straight_corridor_walls(
            center_x - half_width, min_y,
            center_x + half_width, max_y,
            false, // vertical
            corridor, alloc, result,
        );
    } else {
        // L-shaped corridor: horizontal from source, then vertical to destination
        // (or vice versa depending on wall sides)
        generate_l_corridor(from_x, from_y, to_x, to_y, half_width, corridor, alloc, result);
    }
}

/// Generate walls for a straight corridor (axis-aligned rectangle).
fn generate_straight_corridor_walls(
    x1: f64, y1: f64, // top-left
    x2: f64, y2: f64, // bottom-right
    horizontal: bool,
    corridor: &CorridorConfig,
    alloc: &NodeIdAllocator,
    result: &mut RoomLayoutResult,
) {
    if horizontal {
        // Two horizontal wall segments (top and bottom of corridor)
        let wall1_id = alloc.next();
        let wall1 = MapWall::new_open(
            vec![Vector2::new(x1, y1), Vector2::new(x2, y1)],
            "res://textures/walls/battlements.png",
            &wall1_id,
        );

        let wall2_id = alloc.next();
        let wall2 = MapWall::new_open(
            vec![Vector2::new(x1, y2), Vector2::new(x2, y2)],
            "res://textures/walls/battlements.png",
            &wall2_id,
        );

        result.walls.push(wall1);
        result.walls.push(wall2);
    } else {
        // Two vertical wall segments (left and right of corridor)
        let wall1_id = alloc.next();
        let wall1 = MapWall::new_open(
            vec![Vector2::new(x1, y1), Vector2::new(x1, y2)],
            "res://textures/walls/battlements.png",
            &wall1_id,
        );

        let wall2_id = alloc.next();
        let wall2 = MapWall::new_open(
            vec![Vector2::new(x2, y1), Vector2::new(x2, y2)],
            "res://textures/walls/battlements.png",
            &wall2_id,
        );

        result.walls.push(wall1);
        result.walls.push(wall2);
    }

    // Terrain override for corridor floor
    if let Some(slot) = corridor.terrain_slot {
        let grid_x = (x1 / PIXELS_PER_GRID).round() as usize;
        let grid_y = (y1 / PIXELS_PER_GRID).round() as usize;
        let grid_w = ((x2 - x1) / PIXELS_PER_GRID).round() as usize;
        let grid_h = ((y2 - y1) / PIXELS_PER_GRID).round() as usize;

        if grid_w > 0 && grid_h > 0 {
            result.terrain_overrides.push(TerrainOverride {
                cell_x: grid_x * 4,
                cell_y: grid_y * 4,
                cell_width: grid_w * 4,
                cell_height: grid_h * 4,
                slot,
            });
        }
    }

    // Add corridor portals
    // Direction = corridor segment direction (not outward normal).
    for portal_cfg in &corridor.portals {
        let (pos, rotation, direction, _wall_id_ref) = match portal_cfg.end {
            CorridorEnd::From => {
                if horizontal {
                    let center_y = (y1 + y2) / 2.0;
                    (
                        Vector2::new(x1, center_y),
                        FRAC_PI_2,
                        Vector2::new(0.0, 1.0),
                        "ffffffff",
                    )
                } else {
                    let center_x = (x1 + x2) / 2.0;
                    (
                        Vector2::new(center_x, y1),
                        0.0,
                        Vector2::new(1.0, 0.0),
                        "ffffffff",
                    )
                }
            }
            CorridorEnd::To => {
                if horizontal {
                    let center_y = (y1 + y2) / 2.0;
                    (
                        Vector2::new(x2, center_y),
                        FRAC_PI_2,
                        Vector2::new(0.0, 1.0),
                        "ffffffff",
                    )
                } else {
                    let center_x = (x1 + x2) / 2.0;
                    (
                        Vector2::new(center_x, y2),
                        0.0,
                        Vector2::new(1.0, 0.0),
                        "ffffffff",
                    )
                }
            }
        };

        let radius = portal_cfg.portal_type.default_radius();
        let portal = MapPortal::new_freestanding(
            pos,
            rotation,
            direction,
            portal_cfg.portal_type.default_texture(),
            radius,
            &alloc.next(),
        );
        result.portals.push(portal);
    }
}

/// Generate an L-shaped corridor between two non-aligned points.
///
/// Routes horizontally first to align X coordinates, then vertically.
fn generate_l_corridor(
    from_x: f64, from_y: f64,
    to_x: f64, to_y: f64,
    half_width: f64,
    corridor: &CorridorConfig,
    alloc: &NodeIdAllocator,
    result: &mut RoomLayoutResult,
) {
    // Determine the bend point: go horizontal first, then vertical
    let bend_x = to_x;
    let bend_y = from_y;

    // Horizontal segment: from (from_x, from_y) to (bend_x, bend_y)
    let h_min_x = from_x.min(bend_x);
    let h_max_x = from_x.max(bend_x);

    if (h_max_x - h_min_x).abs() > 1.0 {
        // Top wall of horizontal segment
        let wall_id = alloc.next();
        result.walls.push(MapWall::new_open(
            vec![
                Vector2::new(h_min_x, bend_y - half_width),
                Vector2::new(h_max_x + half_width, bend_y - half_width),
            ],
            "res://textures/walls/battlements.png",
            &wall_id,
        ));

        // Bottom wall of horizontal segment
        let wall_id = alloc.next();
        result.walls.push(MapWall::new_open(
            vec![
                Vector2::new(h_min_x, bend_y + half_width),
                Vector2::new(h_max_x, bend_y + half_width),
            ],
            "res://textures/walls/battlements.png",
            &wall_id,
        ));

        // Terrain for horizontal leg
        if let Some(slot) = corridor.terrain_slot {
            let grid_x = (h_min_x / PIXELS_PER_GRID).round() as usize;
            let grid_y = ((bend_y - half_width) / PIXELS_PER_GRID).round() as usize;
            let grid_w = (((h_max_x + half_width) - h_min_x) / PIXELS_PER_GRID).round() as usize;
            let grid_h = ((half_width * 2.0) / PIXELS_PER_GRID).round() as usize;
            if grid_w > 0 && grid_h > 0 {
                result.terrain_overrides.push(TerrainOverride {
                    cell_x: grid_x * 4,
                    cell_y: grid_y * 4,
                    cell_width: grid_w * 4,
                    cell_height: grid_h * 4,
                    slot,
                });
            }
        }
    }

    // Vertical segment: from (bend_x, bend_y) to (to_x, to_y)
    let v_min_y = bend_y.min(to_y);
    let v_max_y = bend_y.max(to_y);

    if (v_max_y - v_min_y).abs() > 1.0 {
        // Left wall of vertical segment
        let wall_id = alloc.next();
        result.walls.push(MapWall::new_open(
            vec![
                Vector2::new(bend_x - half_width, v_min_y),
                Vector2::new(bend_x - half_width, v_max_y),
            ],
            "res://textures/walls/battlements.png",
            &wall_id,
        ));

        // Right wall of vertical segment
        let wall_id = alloc.next();
        result.walls.push(MapWall::new_open(
            vec![
                Vector2::new(bend_x + half_width, v_min_y + half_width),
                Vector2::new(bend_x + half_width, v_max_y),
            ],
            "res://textures/walls/battlements.png",
            &wall_id,
        ));

        // Terrain for vertical leg
        if let Some(slot) = corridor.terrain_slot {
            let grid_x = ((bend_x - half_width) / PIXELS_PER_GRID).round() as usize;
            let grid_y = (v_min_y / PIXELS_PER_GRID).round() as usize;
            let grid_w = ((half_width * 2.0) / PIXELS_PER_GRID).round() as usize;
            let grid_h = ((v_max_y - v_min_y) / PIXELS_PER_GRID).round() as usize;
            if grid_w > 0 && grid_h > 0 {
                result.terrain_overrides.push(TerrainOverride {
                    cell_x: grid_x * 4,
                    cell_y: grid_y * 4,
                    cell_width: grid_w * 4,
                    cell_height: grid_h * 4,
                    slot,
                });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wall_toggles_default() {
        let toggles = WallToggles::default();
        assert!(toggles.north);
        assert!(toggles.south);
        assert!(toggles.east);
        assert!(toggles.west);
    }

    #[test]
    fn test_wall_side_serde() {
        let side: WallSide = serde_json::from_str("\"north\"").unwrap();
        assert_eq!(side, WallSide::North);
        let json = serde_json::to_string(&WallSide::West).unwrap();
        assert_eq!(json, "\"west\"");
    }

    #[test]
    fn test_portal_type_textures() {
        assert!(PortalType::Door.default_texture().contains("door"));
        assert!(PortalType::Window.default_texture().contains("window"));
        assert!(PortalType::Archway.default_texture().contains("archway"));
        assert!(PortalType::SecretDoor.default_texture().contains("secret"));
    }

    #[test]
    fn test_portal_type_serde() {
        let pt: PortalType = serde_json::from_str("\"secret_door\"").unwrap();
        assert_eq!(pt, PortalType::SecretDoor);
        let json = serde_json::to_string(&PortalType::SecretDoor).unwrap();
        assert_eq!(json, "\"secret_door\"");
    }

    #[test]
    fn test_room_config_yaml() {
        let yaml = r#"
id: "guard_room"
x: 4
y: 6
width: 5
height: 4
terrain_slot: 3
walls:
  north: true
  south: true
  east: true
  west: false
portals:
  - wall: "north"
    position: 2
    type: "door"
    width: 1
  - wall: "east"
    position: 1
    type: "window"
    width: 1
"#;
        let room: RoomConfig = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(room.id, "guard_room");
        assert_eq!(room.x, 4);
        assert_eq!(room.y, 6);
        assert_eq!(room.width, 5);
        assert_eq!(room.height, 4);
        assert_eq!(room.terrain_slot, Some(3));
        assert!(!room.walls.west);
        assert!(room.walls.north);
        assert_eq!(room.portals.len(), 2);
        assert_eq!(room.portals[0].portal_type, PortalType::Door);
        assert_eq!(room.portals[1].portal_type, PortalType::Window);
        assert_eq!(room.portals[1].position, 1);
    }

    #[test]
    fn test_room_config_minimal_yaml() {
        let yaml = r#"
id: "simple_room"
x: 0
y: 0
width: 3
height: 3
"#;
        let room: RoomConfig = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(room.id, "simple_room");
        assert!(room.walls.north);
        assert!(room.walls.south);
        assert!(room.walls.east);
        assert!(room.walls.west);
        assert!(room.portals.is_empty());
        assert_eq!(room.terrain_slot, None);
    }

    #[test]
    fn test_corridor_config_yaml() {
        let yaml = r#"
from: "guard_room"
from_wall: "west"
to: "throne_room"
to_wall: "east"
width: 2
terrain_slot: 3
portals:
  - end: "from"
    type: "door"
    width: 1
"#;
        let corridor: CorridorConfig = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(corridor.from, "guard_room");
        assert_eq!(corridor.from_wall, WallSide::West);
        assert_eq!(corridor.to, "throne_room");
        assert_eq!(corridor.to_wall, WallSide::East);
        assert_eq!(corridor.width, 2);
        assert_eq!(corridor.terrain_slot, Some(3));
        assert_eq!(corridor.portals.len(), 1);
        assert_eq!(corridor.portals[0].end, CorridorEnd::From);
        assert_eq!(corridor.portals[0].portal_type, PortalType::Door);
    }

    #[test]
    fn test_corridor_config_minimal_yaml() {
        let yaml = r#"
from: "room_a"
from_wall: "south"
to: "room_b"
to_wall: "north"
"#;
        let corridor: CorridorConfig = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(corridor.width, 2); // default
        assert!(corridor.portals.is_empty());
        assert_eq!(corridor.terrain_slot, None);
    }

    #[test]
    fn test_wall_toggles_is_enabled() {
        let toggles = WallToggles {
            north: true,
            south: false,
            east: true,
            west: false,
        };
        assert!(toggles.is_enabled(WallSide::North));
        assert!(!toggles.is_enabled(WallSide::South));
        assert!(toggles.is_enabled(WallSide::East));
        assert!(!toggles.is_enabled(WallSide::West));
    }

    // --- Layout engine tests ---

    use crate::format::NodeIdAllocator;

    #[test]
    fn test_generate_single_room_4_walls() {
        let alloc = NodeIdAllocator::new(1);
        let room = RoomConfig {
            id: "test_room".to_string(),
            x: 4,
            y: 6,
            width: 5,
            height: 4,
            terrain_slot: None,
            walls: WallToggles::default(),
            portals: vec![],
        };

        let result = generate_room_layout(&[room], &[], &alloc);
        assert_eq!(result.walls.len(), 1);

        let wall = &result.walls[0];
        assert!(wall.is_loop);
        assert_eq!(wall.points.0.len(), 4); // 4 corners
        assert!(wall.portals.is_empty());

        // Check corners in pixel coordinates
        assert_eq!(wall.points.0[0].x, 1024.0); // 4 * 256
        assert_eq!(wall.points.0[0].y, 1536.0); // 6 * 256
        assert_eq!(wall.points.0[1].x, 2304.0); // (4+5) * 256
        assert_eq!(wall.points.0[1].y, 1536.0);
        assert_eq!(wall.points.0[2].x, 2304.0);
        assert_eq!(wall.points.0[2].y, 2560.0); // (6+4) * 256
        assert_eq!(wall.points.0[3].x, 1024.0);
        assert_eq!(wall.points.0[3].y, 2560.0);

        // Shapes entry
        assert_eq!(result.shape_wall_ids.len(), 1);
        assert_eq!(result.shape_polygons.len(), 1);

        // No terrain override
        assert!(result.terrain_overrides.is_empty());
    }

    #[test]
    fn test_generate_room_with_portal() {
        let alloc = NodeIdAllocator::new(1);
        let room = RoomConfig {
            id: "room_with_door".to_string(),
            x: 2,
            y: 2,
            width: 4,
            height: 3,
            terrain_slot: None,
            walls: WallToggles::default(),
            portals: vec![PortalConfig {
                wall: WallSide::North,
                position: 1,
                portal_type: PortalType::Door,
                width: 1,
            }],
        };

        let result = generate_room_layout(&[room], &[], &alloc);
        assert_eq!(result.walls.len(), 1);

        let wall = &result.walls[0];
        assert_eq!(wall.portals.len(), 1);

        let portal = &wall.portals[0];
        // Portal center: x = (2 + 1 + 0.5) * 256 = 896
        assert_eq!(portal.position.x, 896.0);
        // Portal on north wall: y = 2 * 256 = 512
        assert_eq!(portal.position.y, 512.0);
        // North wall rotation = 0
        assert_eq!(portal.rotation, 0.0);
        // Direction = segment direction (east)
        assert_eq!(portal.direction.x, 1.0);
        assert_eq!(portal.direction.y, 0.0);
        // Point index 0 (north wall = segment 0)
        assert_eq!(portal.point_index, 0);
        // wall_id matches wall
        assert_eq!(portal.wall_id, wall.node_id);
        // Texture is door
        assert!(portal.texture.contains("door"));
        // Radius for 1-grid door
        assert_eq!(portal.radius, 128.0);
    }

    #[test]
    fn test_generate_room_with_terrain_override() {
        let alloc = NodeIdAllocator::new(1);
        let room = RoomConfig {
            id: "stone_room".to_string(),
            x: 3,
            y: 5,
            width: 4,
            height: 3,
            terrain_slot: Some(2),
            walls: WallToggles::default(),
            portals: vec![],
        };

        let result = generate_room_layout(&[room], &[], &alloc);
        assert_eq!(result.terrain_overrides.len(), 1);

        let ovr = &result.terrain_overrides[0];
        assert_eq!(ovr.cell_x, 12);  // 3 * 4
        assert_eq!(ovr.cell_y, 20);  // 5 * 4
        assert_eq!(ovr.cell_width, 16);  // 4 * 4
        assert_eq!(ovr.cell_height, 12); // 3 * 4
        assert_eq!(ovr.slot, 2);
    }

    #[test]
    fn test_terrain_override_apply() {
        // 2x2 grid map = 8x8 splat cells
        let map_cells_x = 8;
        let mut splat = vec![0u8; 8 * 8 * 4];

        let ovr = TerrainOverride {
            cell_x: 2,
            cell_y: 2,
            cell_width: 4,
            cell_height: 4,
            slot: 3,
        };
        ovr.apply(&mut splat, map_cells_x);

        // Check a cell inside the override region
        let idx = (2 * map_cells_x + 2) * 4;
        assert_eq!(splat[idx], 0);     // slot 0
        assert_eq!(splat[idx + 1], 0); // slot 1
        assert_eq!(splat[idx + 2], 0); // slot 2
        assert_eq!(splat[idx + 3], 255); // slot 3

        // Check a cell outside the override region
        let idx_outside = (0 * map_cells_x + 0) * 4;
        assert_eq!(splat[idx_outside], 0);
        assert_eq!(splat[idx_outside + 3], 0);
    }

    #[test]
    fn test_generate_multiple_rooms() {
        let alloc = NodeIdAllocator::new(1);
        let rooms = vec![
            RoomConfig {
                id: "room_a".to_string(),
                x: 0,
                y: 0,
                width: 3,
                height: 3,
                terrain_slot: Some(0),
                walls: WallToggles::default(),
                portals: vec![],
            },
            RoomConfig {
                id: "room_b".to_string(),
                x: 5,
                y: 0,
                width: 3,
                height: 3,
                terrain_slot: Some(2),
                walls: WallToggles::default(),
                portals: vec![],
            },
        ];

        let result = generate_room_layout(&rooms, &[], &alloc);
        assert_eq!(result.walls.len(), 2);
        assert_eq!(result.terrain_overrides.len(), 2);
        // Each wall gets a unique node_id
        assert_ne!(result.walls[0].node_id, result.walls[1].node_id);
    }

    #[test]
    fn test_east_wall_portal() {
        let alloc = NodeIdAllocator::new(1);
        let room = RoomConfig {
            id: "east_door".to_string(),
            x: 0,
            y: 0,
            width: 4,
            height: 4,
            terrain_slot: None,
            walls: WallToggles::default(),
            portals: vec![PortalConfig {
                wall: WallSide::East,
                position: 1,
                portal_type: PortalType::Window,
                width: 1,
            }],
        };

        let result = generate_room_layout(&[room], &[], &alloc);
        let portal = &result.walls[0].portals[0];

        // East wall: x = (0+4)*256 = 1024, y = (0 + 1 + 0.5)*256 = 384
        assert_eq!(portal.position.x, 1024.0);
        assert_eq!(portal.position.y, 384.0);
        // East wall rotation = π/2
        assert!((portal.rotation - FRAC_PI_2).abs() < 1e-10);
        // Direction = segment direction (south)
        assert_eq!(portal.direction.x, 0.0);
        assert_eq!(portal.direction.y, 1.0);
        // Point index 1 (east wall)
        assert_eq!(portal.point_index, 1);
        // Window texture
        assert!(portal.texture.contains("window"));
    }

    // --- Corridor tests ---

    fn two_rooms_horizontal() -> Vec<RoomConfig> {
        vec![
            RoomConfig {
                id: "room_a".to_string(),
                x: 2, y: 4, width: 4, height: 4,
                terrain_slot: None,
                walls: WallToggles::default(),
                portals: vec![],
            },
            RoomConfig {
                id: "room_b".to_string(),
                x: 10, y: 4, width: 4, height: 4,
                terrain_slot: None,
                walls: WallToggles::default(),
                portals: vec![],
            },
        ]
    }

    #[test]
    fn test_straight_horizontal_corridor() {
        let alloc = NodeIdAllocator::new(1);
        let rooms = two_rooms_horizontal();
        let corridors = vec![CorridorConfig {
            from: "room_a".to_string(),
            from_wall: WallSide::East,
            to: "room_b".to_string(),
            to_wall: WallSide::West,
            width: 2,
            terrain_slot: Some(1),
            portals: vec![],
        }];

        let result = generate_room_layout(&rooms, &corridors, &alloc);
        // 2 room walls + 2 corridor walls
        assert_eq!(result.walls.len(), 4);

        // Corridor walls are open (not loops)
        assert!(!result.walls[2].is_loop);
        assert!(!result.walls[3].is_loop);

        // Terrain override for corridor
        assert!(result.terrain_overrides.iter().any(|t| t.slot == 1));
    }

    #[test]
    fn test_straight_vertical_corridor() {
        let alloc = NodeIdAllocator::new(1);
        let rooms = vec![
            RoomConfig {
                id: "top".to_string(),
                x: 4, y: 2, width: 4, height: 4,
                terrain_slot: None,
                walls: WallToggles::default(),
                portals: vec![],
            },
            RoomConfig {
                id: "bottom".to_string(),
                x: 4, y: 10, width: 4, height: 4,
                terrain_slot: None,
                walls: WallToggles::default(),
                portals: vec![],
            },
        ];
        let corridors = vec![CorridorConfig {
            from: "top".to_string(),
            from_wall: WallSide::South,
            to: "bottom".to_string(),
            to_wall: WallSide::North,
            width: 2,
            terrain_slot: None,
            portals: vec![],
        }];

        let result = generate_room_layout(&rooms, &corridors, &alloc);
        // 2 room walls + 2 corridor walls
        assert_eq!(result.walls.len(), 4);
        assert!(!result.walls[2].is_loop);
        assert!(!result.walls[3].is_loop);
    }

    #[test]
    fn test_l_shaped_corridor() {
        let alloc = NodeIdAllocator::new(1);
        let rooms = vec![
            RoomConfig {
                id: "left".to_string(),
                x: 2, y: 2, width: 4, height: 4,
                terrain_slot: None,
                walls: WallToggles::default(),
                portals: vec![],
            },
            RoomConfig {
                id: "lower_right".to_string(),
                x: 10, y: 10, width: 4, height: 4,
                terrain_slot: None,
                walls: WallToggles::default(),
                portals: vec![],
            },
        ];
        let corridors = vec![CorridorConfig {
            from: "left".to_string(),
            from_wall: WallSide::East,
            to: "lower_right".to_string(),
            to_wall: WallSide::North,
            width: 2,
            terrain_slot: Some(2),
            portals: vec![],
        }];

        let result = generate_room_layout(&rooms, &corridors, &alloc);
        // 2 room walls + 4 corridor walls (2 per leg of L)
        assert!(result.walls.len() >= 4);
        // Should have terrain overrides for both legs
        let corridor_terrains: Vec<_> = result.terrain_overrides.iter().filter(|t| t.slot == 2).collect();
        assert!(corridor_terrains.len() >= 1);
    }

    #[test]
    fn test_corridor_with_door() {
        let alloc = NodeIdAllocator::new(1);
        let rooms = two_rooms_horizontal();
        let corridors = vec![CorridorConfig {
            from: "room_a".to_string(),
            from_wall: WallSide::East,
            to: "room_b".to_string(),
            to_wall: WallSide::West,
            width: 2,
            terrain_slot: None,
            portals: vec![
                CorridorPortalConfig {
                    end: CorridorEnd::From,
                    portal_type: PortalType::Door,
                    width: 1,
                },
            ],
        }];

        let result = generate_room_layout(&rooms, &corridors, &alloc);
        // Freestanding portal at corridor entrance
        assert_eq!(result.portals.len(), 1);
        assert!(result.portals[0].texture.contains("door"));
        assert_eq!(result.portals[0].wall_id, "ffffffff");
    }
}
