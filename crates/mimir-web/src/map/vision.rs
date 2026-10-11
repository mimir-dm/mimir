//! Sight on a map: walls, light and what tokens see. Pure functions, so
//! the DM map and the player display share them and the host tests them.
//! A port of the desktop app's `useVisibilityPolygon` and
//! `useVisionCalculation` (their tests are ported below).
//!
//! Units: pixels of the served map image. 5 ft is one grid cell.

use mimir_wire::{Light, MapGeometry, Token};

/// A point in pixels.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

/// A wall segment.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Wall {
    pub p1: Point,
    pub p2: Point,
}

/// A door: a wall when closed.
#[derive(Debug, Clone, PartialEq)]
pub struct Door {
    pub wall: Wall,
    pub closed: bool,
}

/// A light from the map file, in pixels.
#[derive(Debug, Clone, PartialEq)]
pub struct FileLight {
    pub position: Point,
    pub range_px: f64,
    pub intensity: f64,
    /// CSS colour.
    pub color: String,
    pub shadows: bool,
}

/// Ambient light (5e).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LightLevel {
    Bright,
    Dim,
    Darkness,
}

impl LightLevel {
    /// The map's lighting mode ("bright", "dim", "dark").
    pub fn from_mode(mode: &str) -> Self {
        match mode {
            "dim" => LightLevel::Dim,
            "dark" | "darkness" => LightLevel::Darkness,
            _ => LightLevel::Bright,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            LightLevel::Bright => "bright",
            LightLevel::Dim => "dim",
            LightLevel::Darkness => "darkness",
        }
    }
}

/// The light level of a UVTT ambient colour (ARGB hex), by brightness.
pub fn ambient_level(argb: Option<&str>) -> LightLevel {
    let Some(argb) = argb else {
        return LightLevel::Bright;
    };
    let hex = argb.trim_start_matches('#');
    if hex.len() < 6 {
        return LightLevel::Bright;
    }
    let byte = |i: usize| {
        hex.get(i..i + 2)
            .and_then(|s| u8::from_str_radix(s, 16).ok())
            .map(f64::from)
            .unwrap_or(0.0)
    };
    let (r, g, b) = (byte(2), byte(4), byte(6));
    let brightness = (r * 299.0 + g * 587.0 + b * 114.0) / 1000.0;
    if brightness > 170.0 {
        LightLevel::Bright
    } else if brightness > 85.0 {
        LightLevel::Dim
    } else {
        LightLevel::Darkness
    }
}

/// The walls of the map file in pixels (each polyline → segments).
pub fn walls_px(g: &MapGeometry) -> Vec<Wall> {
    let ppg = ppg(g);
    g.walls
        .iter()
        .flat_map(|line| {
            line.windows(2).map(move |w| Wall {
                p1: Point {
                    x: w[0].x * ppg,
                    y: w[0].y * ppg,
                },
                p2: Point {
                    x: w[1].x * ppg,
                    y: w[1].y * ppg,
                },
            })
        })
        .collect()
}

/// The doors of the map file in pixels.
pub fn doors_px(g: &MapGeometry) -> Vec<Door> {
    let ppg = ppg(g);
    g.portals
        .iter()
        .map(|p| Door {
            wall: Wall {
                p1: Point {
                    x: p.bounds[0].x * ppg,
                    y: p.bounds[0].y * ppg,
                },
                p2: Point {
                    x: p.bounds[1].x * ppg,
                    y: p.bounds[1].y * ppg,
                },
            },
            closed: p.closed,
        })
        .collect()
}

/// The lights of the map file in pixels.
pub fn file_lights_px(g: &MapGeometry) -> Vec<FileLight> {
    let ppg = ppg(g);
    g.lights
        .iter()
        .map(|l| FileLight {
            position: Point {
                x: l.position.x * ppg,
                y: l.position.y * ppg,
            },
            range_px: l.range * ppg,
            intensity: l.intensity,
            color: argb_to_css(&l.color),
            shadows: l.shadows,
        })
        .collect()
}

fn ppg(g: &MapGeometry) -> f64 {
    if g.grid_size_px > 0.0 {
        g.grid_size_px
    } else {
        70.0
    }
}

/// ARGB hex ("ffeccd8b") → CSS `rgba(…)`; RGB hex → `rgb(…)`.
pub fn argb_to_css(argb: &str) -> String {
    let hex = argb.trim_start_matches('#');
    let byte = |i: usize| {
        hex.get(i..i + 2)
            .and_then(|s| u8::from_str_radix(s, 16).ok())
    };
    match hex.len() {
        8 => match (byte(0), byte(2), byte(4), byte(6)) {
            (Some(a), Some(r), Some(g), Some(b)) => {
                let a = f64::from(a) / 255.0;
                format!("rgba({r}, {g}, {b}, {})", trim_float(a))
            }
            _ => "#ffcc66".into(),
        },
        6 => match (byte(0), byte(2), byte(4)) {
            (Some(r), Some(g), Some(b)) => format!("rgb({r}, {g}, {b})"),
            _ => "#ffcc66".into(),
        },
        _ => "#ffcc66".into(),
    }
}

fn trim_float(v: f64) -> String {
    let s = format!("{v:.3}");
    let s = s.trim_end_matches('0').trim_end_matches('.');
    s.to_string()
}

/// The walls that block sight: the file's walls and the closed doors.
pub fn blocking(walls: &[Wall], doors: &[Door]) -> Vec<Wall> {
    walls
        .iter()
        .copied()
        .chain(doors.iter().filter(|d| d.closed).map(|d| d.wall))
        .collect()
}

/// Visibility extends this far past a wall line, to show the wall's art.
const WALL_EXTENSION_PX: f64 = 12.0;

fn ray_hit(origin: Point, angle: f64, seg: &Wall, extend: bool) -> Option<Point> {
    let (dx, dy) = (angle.cos(), angle.sin());
    let (x1, y1, x2, y2) = (seg.p1.x, seg.p1.y, seg.p2.x, seg.p2.y);
    let (x3, y3) = (origin.x, origin.y);
    let (x4, y4) = (origin.x + dx, origin.y + dy);
    let denom = (x1 - x2) * (y3 - y4) - (y1 - y2) * (x3 - x4);
    if denom.abs() < 1e-10 {
        return None;
    }
    let t = ((x1 - x3) * (y3 - y4) - (y1 - y3) * (x3 - x4)) / denom;
    let u = -((x1 - x2) * (y1 - y3) - (y1 - y2) * (x1 - x3)) / denom;
    if (0.0..=1.0).contains(&t) && u > 0.0 {
        let mut p = Point {
            x: x1 + t * (x2 - x1),
            y: y1 + t * (y2 - y1),
        };
        if extend {
            p.x += dx * WALL_EXTENSION_PX;
            p.y += dy * WALL_EXTENSION_PX;
        }
        Some(p)
    } else {
        None
    }
}

fn dist(a: Point, b: Point) -> f64 {
    ((b.x - a.x).powi(2) + (b.y - a.y).powi(2)).sqrt()
}

/// The area seen from `origin` (raycasting to each wall end), as a polygon
/// sorted by angle. Bounded by `max_radius` and the map.
pub fn visibility_polygon(
    origin: Point,
    walls: &[Wall],
    max_radius: f64,
    map_width: f64,
    map_height: f64,
) -> Vec<Point> {
    let corners = [
        Point {
            x: (origin.x - max_radius).max(0.0),
            y: (origin.y - max_radius).max(0.0),
        },
        Point {
            x: (origin.x + max_radius).min(map_width),
            y: (origin.y - max_radius).max(0.0),
        },
        Point {
            x: (origin.x + max_radius).min(map_width),
            y: (origin.y + max_radius).min(map_height),
        },
        Point {
            x: (origin.x - max_radius).max(0.0),
            y: (origin.y + max_radius).min(map_height),
        },
    ];
    let bounds = [
        Wall {
            p1: corners[0],
            p2: corners[1],
        },
        Wall {
            p1: corners[1],
            p2: corners[2],
        },
        Wall {
            p1: corners[2],
            p2: corners[3],
        },
        Wall {
            p1: corners[3],
            p2: corners[0],
        },
    ];
    let mut targets: Vec<Point> = Vec::with_capacity(walls.len() * 2 + 4);
    for p in walls.iter().flat_map(|w| [w.p1, w.p2]).chain(corners) {
        if !targets.iter().any(|q| q.x == p.x && q.y == p.y) {
            targets.push(p);
        }
    }

    let mut hits: Vec<(f64, Point)> = Vec::with_capacity(targets.len() * 3);
    for target in targets {
        let base = (target.y - origin.y).atan2(target.x - origin.x);
        for offset in [-1e-4, 0.0, 1e-4] {
            let angle = base + offset;
            let mut best: Option<(f64, Point)> = None;
            let candidates = walls
                .iter()
                .map(|w| (w, true))
                .chain(bounds.iter().map(|w| (w, false)));
            for (wall, extend) in candidates {
                if let Some(p) = ray_hit(origin, angle, wall, extend) {
                    let d = dist(origin, p);
                    if d <= max_radius && best.is_none_or(|(bd, _)| d < bd) {
                        best = Some((d, p));
                    }
                }
            }
            let point = match best {
                Some((_, p)) => p,
                None => Point {
                    x: (origin.x + angle.cos() * max_radius).clamp(0.0, map_width),
                    y: (origin.y + angle.sin() * max_radius).clamp(0.0, map_height),
                },
            };
            hits.push((angle, point));
        }
    }
    hits.sort_by(|a, b| a.0.total_cmp(&b.0));
    let mut out: Vec<Point> = Vec::with_capacity(hits.len());
    for (_, p) in hits {
        if !out
            .iter()
            .any(|q| (q.x - p.x).abs() < 0.5 && (q.y - p.y).abs() < 0.5)
        {
            out.push(p);
        }
    }
    out
}

/// SVG path data for a polygon ("" for fewer than 3 points).
pub fn svg_path(points: &[Point]) -> String {
    if points.len() < 3 {
        return String::new();
    }
    let mut s = String::new();
    for (i, p) in points.iter().enumerate() {
        s.push_str(if i == 0 { "M " } else { " L " });
        s.push_str(&format!("{:.1} {:.1}", p.x, p.y));
    }
    s.push_str(" Z");
    s
}

/// Is the point inside the polygon (even-odd)?
pub fn point_in_polygon(p: Point, poly: &[Point]) -> bool {
    let mut inside = false;
    let n = poly.len();
    if n < 3 {
        return false;
    }
    let mut j = n - 1;
    for i in 0..n {
        let (a, b) = (poly[i], poly[j]);
        if (a.y > p.y) != (b.y > p.y) && p.x < (b.x - a.x) * (p.y - a.y) / (b.y - a.y) + a.x {
            inside = !inside;
        }
        j = i;
    }
    inside
}

// ---- Vision (5e) -----------------------------------------------------------

/// Vision with no limit.
pub const UNLIMITED: f64 = 100_000.0;

pub fn feet_to_px(feet: f64, grid_px: f64) -> f64 {
    feet / 5.0 * grid_px
}

/// A lit area: a placed light or a token's own light.
#[derive(Debug, Clone, PartialEq)]
pub struct LightZone {
    pub source_id: String,
    pub x: f64,
    pub y: f64,
    pub bright_px: f64,
    pub dim_px: f64,
}

/// The lit areas: active placed lights and tokens that carry light (bright
/// to half the radius, dim to the radius).
pub fn light_zones(lights: &[Light], tokens: &[Token], grid_px: f64) -> Vec<LightZone> {
    let placed = lights.iter().filter(|l| l.active).map(|l| LightZone {
        source_id: format!("map-{}", l.id),
        x: l.x,
        y: l.y,
        bright_px: feet_to_px(f64::from(l.bright_radius_ft), grid_px),
        dim_px: feet_to_px(f64::from(l.dim_radius_ft), grid_px),
    });
    let carried = tokens.iter().filter(|t| t.light_radius_ft > 0).map(|t| {
        let dim = f64::from(t.light_radius_ft);
        LightZone {
            source_id: format!("token-{}", t.id),
            x: t.x,
            y: t.y,
            bright_px: feet_to_px(dim / 2.0, grid_px),
            dim_px: feet_to_px(dim, grid_px),
        }
    });
    placed.chain(carried).collect()
}

/// The light at a point: the best of the ambient light and the zones.
pub fn light_at(x: f64, y: f64, zones: &[LightZone], ambient: LightLevel) -> LightLevel {
    let mut best = ambient;
    for z in zones {
        let d = ((x - z.x).powi(2) + (y - z.y).powi(2)).sqrt();
        if d <= z.bright_px {
            return LightLevel::Bright;
        } else if d <= z.dim_px && best == LightLevel::Darkness {
            best = LightLevel::Dim;
        }
    }
    best
}

/// How far a token sees, from the light where it stands.
#[derive(Debug, Clone, PartialEq)]
pub struct TokenVision {
    pub token_id: String,
    pub x: f64,
    pub y: f64,
    pub radius_px: f64,
    /// It sees as in dim light (dim light, or darkvision).
    pub dim: bool,
    pub light_px: f64,
}

pub fn token_vision(
    t: &Token,
    zones: &[LightZone],
    ambient: LightLevel,
    grid_px: f64,
) -> TokenVision {
    let (feet, dim) = match light_at(t.x, t.y, zones, ambient) {
        LightLevel::Bright => (t.vision_bright_ft, false),
        LightLevel::Dim => (t.vision_dim_ft, true),
        LightLevel::Darkness => (
            Some(t.vision_dark_ft.max(t.light_radius_ft)),
            t.vision_dark_ft > 0,
        ),
    };
    TokenVision {
        token_id: t.id.clone(),
        x: t.x,
        y: t.y,
        radius_px: feet.map_or(UNLIMITED, |f| feet_to_px(f64::from(f), grid_px)),
        dim,
        light_px: feet_to_px(f64::from(t.light_radius_ft), grid_px),
    }
}

/// The vision of the party: PC tokens the players see.
pub fn party_vision(
    tokens: &[Token],
    zones: &[LightZone],
    ambient: LightLevel,
    grid_px: f64,
) -> Vec<TokenVision> {
    tokens
        .iter()
        .filter(|t| t.token_type == "pc" && t.visible_to_players)
        .map(|t| token_vision(t, zones, ambient, grid_px))
        .collect()
}

pub fn in_vision(x: f64, y: f64, v: &TokenVision) -> bool {
    ((x - v.x).powi(2) + (y - v.y).powi(2)).sqrt() <= v.radius_px
}

/// Can the party see the point, and only as dim?
pub fn party_sees(x: f64, y: f64, party: &[TokenVision]) -> (bool, bool) {
    let seeing: Vec<&TokenVision> = party.iter().filter(|v| in_vision(x, y, v)).collect();
    let can = !seeing.is_empty();
    (can, can && seeing.iter().all(|v| v.dim))
}

/// The darkness overlay is needed unless the ambient light is bright and
/// every PC sees without limit.
pub fn needs_vision_overlay(ambient: LightLevel, party: &[TokenVision]) -> bool {
    match ambient {
        LightLevel::Bright => party.iter().any(|v| v.radius_px < UNLIMITED),
        _ => true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mimir_wire::{GridPoint, MapFileLight, Portal};

    const GRID: f64 = 70.0;

    fn geometry(ppg: f64) -> MapGeometry {
        MapGeometry {
            grid_size_px: ppg,
            columns: 10.0,
            rows: 10.0,
            walls: vec![],
            portals: vec![],
            lights: vec![],
            ambient_light: None,
            baked_lighting: false,
        }
    }

    fn gp(x: f64, y: f64) -> GridPoint {
        GridPoint { x, y }
    }

    fn token(id: &str) -> Token {
        Token {
            id: id.into(),
            map_id: "map-1".into(),
            name: "Test Token".into(),
            token_type: "pc".into(),
            size: "medium".into(),
            grid_x: 4,
            grid_y: 4,
            x: 350.0,
            y: 350.0,
            visible_to_players: true,
            color: Some("#ff0000".into()),
            monster_id: None,
            npc_id: None,
            vision_bright_ft: None,
            vision_dim_ft: Some(60),
            vision_dark_ft: 0,
            light_radius_ft: 0,
        }
    }

    fn light(id: &str) -> Light {
        Light {
            id: id.into(),
            map_id: "map-1".into(),
            name: Some("Torch".into()),
            grid_x: 4,
            grid_y: 4,
            x: 350.0,
            y: 350.0,
            bright_radius_ft: 20,
            dim_radius_ft: 40,
            color: Some("#ff9900".into()),
            active: true,
        }
    }

    // ---- uvttAmbientToLevel
    #[test]
    fn ambient_levels() {
        assert_eq!(ambient_level(None), LightLevel::Bright);
        assert_eq!(ambient_level(Some("ffffffff")), LightLevel::Bright);
        assert_eq!(ambient_level(Some("ff000000")), LightLevel::Darkness);
        assert_eq!(ambient_level(Some("ff808080")), LightLevel::Dim);
        assert_eq!(ambient_level(Some("fff")), LightLevel::Bright);
        assert_eq!(ambient_level(Some("#ff000000")), LightLevel::Darkness);
        assert_eq!(LightLevel::from_mode("dark"), LightLevel::Darkness);
        assert_eq!(LightLevel::from_mode("bright"), LightLevel::Bright);
    }

    // ---- uvttWallsToPixels
    #[test]
    fn walls_to_pixels() {
        assert!(walls_px(&geometry(70.0)).is_empty());
        let mut g = geometry(70.0);
        g.walls = vec![vec![gp(1.0, 2.0), gp(3.0, 4.0)]];
        let w = walls_px(&g);
        assert_eq!(w.len(), 1);
        assert_eq!(w[0].p1, Point { x: 70.0, y: 140.0 });
        assert_eq!(w[0].p2, Point { x: 210.0, y: 280.0 });
        g.walls = vec![vec![gp(0.0, 0.0), gp(1.0, 0.0), gp(1.0, 1.0)]];
        let w = walls_px(&g);
        assert_eq!(w.len(), 2, "a polyline is several segments");
        assert_eq!(
            (w[0].p2, w[1].p1),
            (Point { x: 70.0, y: 0.0 }, Point { x: 70.0, y: 0.0 })
        );
        g.walls = vec![
            vec![gp(0.0, 0.0), gp(1.0, 0.0)],
            vec![gp(2.0, 2.0), gp(3.0, 3.0)],
        ];
        assert_eq!(walls_px(&g).len(), 2);
        let mut g0 = geometry(0.0);
        g0.walls = vec![vec![gp(1.0, 0.0), gp(2.0, 0.0)]];
        assert_eq!(walls_px(&g0)[0].p1.x, 70.0, "default 70 px per grid");
    }

    // ---- uvttPortalsToPixels
    #[test]
    fn doors_to_pixels() {
        assert!(doors_px(&geometry(70.0)).is_empty());
        let mut g = geometry(70.0);
        let portal = |closed| Portal {
            position: gp(5.0, 5.0),
            bounds: [gp(4.0, 5.0), gp(6.0, 5.0)],
            rotation: 0.0,
            closed,
            freestanding: false,
        };
        g.portals = vec![portal(false), portal(true)];
        let d = doors_px(&g);
        assert_eq!(d[0].wall.p1, Point { x: 280.0, y: 350.0 });
        assert_eq!(d[0].wall.p2, Point { x: 420.0, y: 350.0 });
        assert!(!d[0].closed && d[1].closed);
        // Only closed doors block.
        assert_eq!(blocking(&[], &d), vec![d[1].wall]);
    }

    // ---- uvttLightsToPixels
    #[test]
    fn file_lights_to_pixels() {
        assert!(file_lights_px(&geometry(70.0)).is_empty());
        let mut g = geometry(70.0);
        g.lights = vec![MapFileLight {
            position: gp(5.0, 3.0),
            range: 4.0,
            intensity: 1.0,
            color: "ffff0000".into(),
            shadows: true,
        }];
        let l = file_lights_px(&g);
        assert_eq!(l[0].position, Point { x: 350.0, y: 210.0 });
        assert_eq!(l[0].range_px, 280.0);
        assert!(l[0].shadows);
        assert_eq!(l[0].color, "rgba(255, 0, 0, 1)");
        assert_eq!(argb_to_css("00ff00"), "rgb(0, 255, 0)");
        assert_eq!(argb_to_css("zz"), "#ffcc66");
    }

    // ---- the raycast (no tests in the desktop app)
    #[test]
    fn open_room_sees_to_the_radius_or_the_map_edge() {
        let poly = visibility_polygon(Point { x: 100.0, y: 100.0 }, &[], 1000.0, 200.0, 200.0);
        assert!(point_in_polygon(Point { x: 190.0, y: 190.0 }, &poly));
        assert!(point_in_polygon(Point { x: 5.0, y: 5.0 }, &poly));
        assert!(poly
            .iter()
            .all(|p| p.x <= 200.0 && p.y <= 200.0 && p.x >= 0.0 && p.y >= 0.0));
    }

    #[test]
    fn a_wall_hides_what_is_behind_it() {
        // A wall across the room at x = 150, from y = 0 to y = 200.
        let wall = Wall {
            p1: Point { x: 150.0, y: 0.0 },
            p2: Point { x: 150.0, y: 200.0 },
        };
        let poly = visibility_polygon(Point { x: 50.0, y: 100.0 }, &[wall], 1000.0, 300.0, 200.0);
        assert!(
            point_in_polygon(Point { x: 100.0, y: 100.0 }, &poly),
            "in front"
        );
        assert!(
            point_in_polygon(Point { x: 155.0, y: 100.0 }, &poly),
            "the wall's art (12 px past the line) shows"
        );
        assert!(
            !point_in_polygon(Point { x: 250.0, y: 100.0 }, &poly),
            "behind"
        );
    }

    #[test]
    fn a_short_wall_casts_a_shadow_only() {
        let wall = Wall {
            p1: Point { x: 150.0, y: 80.0 },
            p2: Point { x: 150.0, y: 120.0 },
        };
        let poly = visibility_polygon(Point { x: 50.0, y: 100.0 }, &[wall], 1000.0, 300.0, 200.0);
        assert!(
            !point_in_polygon(Point { x: 250.0, y: 100.0 }, &poly),
            "in the shadow"
        );
        assert!(
            point_in_polygon(Point { x: 250.0, y: 20.0 }, &poly),
            "beside the shadow"
        );
    }

    #[test]
    fn the_radius_limits_sight() {
        let poly = visibility_polygon(Point { x: 500.0, y: 500.0 }, &[], 100.0, 1000.0, 1000.0);
        assert!(point_in_polygon(Point { x: 550.0, y: 500.0 }, &poly));
        assert!(!point_in_polygon(Point { x: 700.0, y: 500.0 }, &poly));
    }

    #[test]
    fn svg_path_of_a_polygon() {
        assert_eq!(svg_path(&[]), "");
        let p = [
            Point { x: 0.0, y: 0.0 },
            Point { x: 10.0, y: 0.0 },
            Point { x: 10.0, y: 10.0 },
        ];
        assert_eq!(svg_path(&p), "M 0.0 0.0 L 10.0 0.0 L 10.0 10.0 Z");
    }

    // ---- useVisionCalculation
    #[test]
    fn feet_to_pixels() {
        assert_eq!(feet_to_px(30.0, GRID), 420.0);
        assert_eq!(feet_to_px(0.0, GRID), 0.0);
    }

    #[test]
    fn light_zones_from_lights_and_tokens() {
        let z = light_zones(&[light("l1")], &[], GRID);
        assert_eq!(z.len(), 1);
        assert_eq!((z[0].bright_px, z[0].dim_px), (280.0, 560.0));
        let mut off = light("l2");
        off.active = false;
        assert!(
            light_zones(&[off], &[], GRID).is_empty(),
            "inactive lights skipped"
        );
        let mut lit = token("t1");
        lit.light_radius_ft = 40;
        let z = light_zones(&[], &[lit.clone()], GRID);
        assert_eq!(
            (z[0].bright_px, z[0].dim_px),
            (280.0, 560.0),
            "bright to half"
        );
        assert!(light_zones(&[], &[token("t2")], GRID).is_empty());
        assert_eq!(light_zones(&[light("l1")], &[lit], GRID).len(), 2);
    }

    #[test]
    fn light_level_at_a_point() {
        assert_eq!(
            light_at(0.0, 0.0, &[], LightLevel::Darkness),
            LightLevel::Darkness
        );
        let z = light_zones(&[light("l1")], &[], GRID);
        assert_eq!(
            light_at(350.0, 350.0, &z, LightLevel::Darkness),
            LightLevel::Bright
        );
        assert_eq!(
            light_at(350.0 + 400.0, 350.0, &z, LightLevel::Darkness),
            LightLevel::Dim
        );
        assert_eq!(
            light_at(350.0 + 1000.0, 350.0, &z, LightLevel::Darkness),
            LightLevel::Darkness
        );
        assert_eq!(light_at(0.0, 0.0, &[], LightLevel::Dim), LightLevel::Dim);
        assert_eq!(
            light_at(0.0, 0.0, &[], LightLevel::Bright),
            LightLevel::Bright
        );
    }

    #[test]
    fn vision_in_bright_and_dim_light() {
        let t = token("t1");
        assert_eq!(
            token_vision(&t, &[], LightLevel::Bright, GRID).radius_px,
            UNLIMITED
        );
        let mut short = token("t2");
        short.vision_bright_ft = Some(30);
        assert_eq!(
            token_vision(&short, &[], LightLevel::Bright, GRID).radius_px,
            420.0
        );
        let v = token_vision(&t, &[], LightLevel::Dim, GRID);
        assert_eq!(v.radius_px, 840.0);
        assert!(v.dim);
        let mut wide = token("t3");
        wide.vision_dim_ft = None;
        assert_eq!(
            token_vision(&wide, &[], LightLevel::Dim, GRID).radius_px,
            UNLIMITED
        );
    }

    #[test]
    fn vision_in_darkness() {
        let t = token("t1");
        assert_eq!(
            token_vision(&t, &[], LightLevel::Darkness, GRID).radius_px,
            0.0,
            "blind"
        );
        let mut dv = token("t2");
        dv.vision_dark_ft = 60;
        let v = token_vision(&dv, &[], LightLevel::Darkness, GRID);
        assert_eq!(v.radius_px, 840.0);
        assert!(v.dim, "darkvision sees as dim");
        let mut torch = token("t3");
        torch.light_radius_ft = 40;
        // Its own light lights where it stands: bright → unlimited.
        let zones = light_zones(&[], &[torch.clone()], GRID);
        assert_eq!(
            token_vision(&torch, &zones, LightLevel::Darkness, GRID).radius_px,
            UNLIMITED
        );
        // Without the zone, the greater of darkvision and its light.
        let mut both = token("t4");
        both.vision_dark_ft = 30;
        both.light_radius_ft = 40;
        assert_eq!(
            token_vision(&both, &[], LightLevel::Darkness, GRID).radius_px,
            560.0
        );
    }

    #[test]
    fn party_vision_is_visible_pcs_only() {
        let mut monster = token("m");
        monster.token_type = "monster".into();
        let mut hidden = token("h");
        hidden.visible_to_players = false;
        let party = party_vision(
            &[token("pc"), monster, hidden],
            &[],
            LightLevel::Bright,
            GRID,
        );
        assert_eq!(party.len(), 1);
        assert_eq!(party[0].token_id, "pc");
    }

    #[test]
    fn points_in_vision_and_party_sight() {
        let mut t = token("t1");
        t.vision_bright_ft = Some(30);
        let v = token_vision(&t, &[], LightLevel::Bright, GRID);
        assert!(in_vision(350.0, 350.0, &v));
        assert!(in_vision(350.0 + 400.0, 350.0, &v));
        assert!(!in_vision(350.0 + 500.0, 350.0, &v));

        assert_eq!(party_sees(0.0, 0.0, &[]), (false, false));
        let bright = party_vision(&[token("a")], &[], LightLevel::Bright, GRID);
        assert_eq!(party_sees(360.0, 360.0, &bright), (true, false));
        let dim = party_vision(&[token("a")], &[], LightLevel::Dim, GRID);
        assert_eq!(party_sees(360.0, 360.0, &dim), (true, true));
        let mixed = [dim[0].clone(), bright[0].clone()];
        assert_eq!(party_sees(360.0, 360.0, &mixed), (true, false));
    }

    #[test]
    fn when_the_overlay_is_needed() {
        let unlimited = party_vision(&[token("a")], &[], LightLevel::Bright, GRID);
        assert!(!needs_vision_overlay(LightLevel::Bright, &unlimited));
        let mut short = token("b");
        short.vision_bright_ft = Some(30);
        let limited = party_vision(&[short], &[], LightLevel::Bright, GRID);
        assert!(needs_vision_overlay(LightLevel::Bright, &limited));
        assert!(needs_vision_overlay(LightLevel::Dim, &unlimited));
        assert!(needs_vision_overlay(LightLevel::Darkness, &unlimited));
    }
}
