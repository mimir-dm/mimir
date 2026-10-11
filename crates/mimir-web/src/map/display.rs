//! `/display/:campaign`: the player display. A TV, a tablet or a player's
//! device; no app shell. It shows the map the DM shows, from the server's
//! player view only (hidden tokens, unlit lights, hidden markers and trap
//! details never reach it), and follows the DM over the live socket. The
//! viewer pans and zooms; a button asks for full screen.

use std::collections::HashMap;

use aurora_leptos::components::*;
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::hooks::use_params_map;
use mimir_wire::{
    DisplayState, Light, MapGeometry, PlayerLight, PlayerToken, PlayerView, ServerMsg, Token,
};

use super::dm::{cells_for_size, grid_path};
use super::viewport::Viewport;
use super::vision::{self, LightLevel, Point};
use crate::api;
use crate::live::Live;

/// A player token as the vision functions take it.
pub fn as_token(t: &PlayerToken) -> Token {
    Token {
        id: t.id.clone(),
        map_id: String::new(),
        name: t.name.clone(),
        token_type: t.token_type.clone(),
        size: t.size.clone(),
        grid_x: t.grid_x,
        grid_y: t.grid_y,
        x: t.x,
        y: t.y,
        visible_to_players: true,
        color: t.color.clone(),
        monster_id: None,
        npc_id: None,
        vision_bright_ft: t.vision_bright_ft,
        vision_dim_ft: t.vision_dim_ft,
        vision_dark_ft: t.vision_dark_ft,
        light_radius_ft: t.light_radius_ft,
    }
}

/// A lit light as the vision functions take it.
pub fn as_light(l: &PlayerLight) -> Light {
    Light {
        id: l.id.clone(),
        map_id: String::new(),
        name: None,
        grid_x: l.grid_x,
        grid_y: l.grid_y,
        x: l.x,
        y: l.y,
        bright_radius_ft: l.bright_radius_ft,
        dim_radius_ft: l.dim_radius_ft,
        color: l.color.clone(),
        active: true,
    }
}

/// What the players see of the map: the sight of the party (with walls),
/// the revealed fog, and how dark the rest is.
#[derive(Debug, Clone, PartialEq)]
pub struct Sight {
    /// Areas seen: SVG paths (party sight, then revealed fog areas).
    pub holes: Vec<String>,
    /// The party sight polygons (for token visibility).
    pub polygons: Vec<Vec<Point>>,
    /// Opacity of the cover over the rest: 1 under fog, less for darkness,
    /// 0 for none.
    pub cover: f64,
}

/// Work out the sight for a player view and the map geometry.
pub fn sight(view: &PlayerView, geometry: Option<&MapGeometry>) -> Sight {
    let m = &view.map;
    let g = m.grid_size_px;
    let (w, h) = (f64::from(m.width_px), f64::from(m.height_px));
    let tokens: Vec<Token> = view.tokens.iter().map(as_token).collect();
    let lights: Vec<Light> = view.lights.iter().map(as_light).collect();
    let ambient = LightLevel::from_mode(&m.lighting_mode);
    let zones = vision::light_zones(&lights, &tokens, g);
    let party = vision::party_vision(&tokens, &zones, ambient, g);
    let walls = geometry
        .map(|geo| vision::blocking(&vision::walls_px(geo), &vision::doors_px(geo)))
        .unwrap_or_default();
    let cap = w.max(h) * 2.0;
    let polygons: Vec<Vec<Point>> = party
        .iter()
        .filter(|v| v.radius_px > 0.0)
        .map(|v| {
            vision::visibility_polygon(Point { x: v.x, y: v.y }, &walls, v.radius_px.min(cap), w, h)
        })
        .collect();
    let mut holes: Vec<String> = polygons.iter().map(|p| vision::svg_path(p)).collect();
    let fog_on = view.fog.enabled;
    if fog_on {
        holes.extend(
            view.fog
                .revealed
                .iter()
                .map(|a| format!("M{} {}h{}v{}h{}Z", a.x, a.y, a.width, a.height, -a.width)),
        );
    }
    let cover = if fog_on {
        1.0
    } else if vision::needs_vision_overlay(ambient, &party) {
        match ambient {
            LightLevel::Darkness => 0.92,
            _ => 0.75,
        }
    } else {
        0.0
    };
    Sight {
        holes,
        polygons,
        cover,
    }
}

/// Does the party see this token? PCs always; others inside the sight or a
/// revealed area. With no PCs on the map: all, unless the fog is on.
pub fn token_seen(t: &PlayerToken, view: &PlayerView, sight: &Sight) -> bool {
    if t.token_type == "pc" {
        return true;
    }
    let has_party = view.tokens.iter().any(|t| t.token_type == "pc");
    if !has_party {
        return !view.fog.enabled;
    }
    let p = Point { x: t.x, y: t.y };
    sight
        .polygons
        .iter()
        .any(|poly| vision::point_in_polygon(p, poly))
        || (view.fog.enabled
            && view
                .fog
                .revealed
                .iter()
                .any(|a| t.x >= a.x && t.x <= a.x + a.width && t.y >= a.y && t.y <= a.y + a.height))
}

#[component]
pub fn DisplayPage() -> impl IntoView {
    let params = use_params_map();
    let campaign = Memo::new(move |_| params.get().get("campaign").unwrap_or_default());

    let display = RwSignal::new(None::<DisplayState>);
    let view = RwSignal::new(None::<PlayerView>);
    let shown = Memo::new(move |_| {
        display.with(|d| {
            d.as_ref()
                .filter(|d| !d.blackout)
                .and_then(|d| d.map_id.clone())
        })
    });

    // With the DM token the socket sends notices; read the player view
    // then. A player's socket sends the view itself.
    let fetch = move |map_id: String| {
        spawn_local(async move {
            if let Ok(v) = api::get::<PlayerView>(format!("/maps/{map_id}/player-view")).await {
                if shown.get_untracked().as_deref() == Some(v.map.id.as_str()) {
                    view.set(Some(v));
                }
            }
        });
    };
    let live = StoredValue::new_local(None::<Live>);
    Effect::new(move |_| {
        let c = campaign.get();
        if c.is_empty() {
            return;
        }
        live.set_value(Some(Live::watch(c, move |msg| match msg {
            ServerMsg::Display { display: d } => {
                let map = (!d.blackout).then(|| d.map_id.clone()).flatten();
                display.set(Some(d));
                match map {
                    Some(m) => fetch(m),
                    None => view.set(None),
                }
            }
            ServerMsg::MapChanged { map_id, .. } => {
                if shown.get_untracked().as_deref() == Some(map_id.as_str()) {
                    fetch(map_id);
                }
            }
            ServerMsg::PlayerView { view: v } => view.set(Some(*v)),
            _ => {}
        })));
    });

    let geometry = LocalResource::new(move || {
        let m = shown.get();
        async move {
            match m {
                Some(m) => api::get::<MapGeometry>(format!("/maps/{m}/geometry"))
                    .await
                    .ok(),
                None => None,
            }
        }
    });
    let image = LocalResource::new(move || {
        let m = shown.get();
        async move {
            match m {
                Some(m) => api::image_url(format!("/maps/{m}/image"))
                    .await
                    .ok()
                    .flatten(),
                None => None,
            }
        }
    });
    let art = RwSignal::new(HashMap::<String, Option<String>>::new());
    Effect::new(move |_| {
        let Some(v) = view.get() else { return };
        for t in v.tokens.into_iter().filter(|t| t.token_type == "monster") {
            if art.with_untracked(|m| m.contains_key(&t.id)) {
                continue;
            }
            art.update(|m| {
                m.insert(t.id.clone(), None);
            });
            spawn_local(async move {
                if let Ok(Some(url)) = api::image_url(format!("/tokens/{}/image", t.id)).await {
                    art.update(|m| {
                        m.insert(t.id, Some(url));
                    });
                }
            });
        }
    });

    let vp = Viewport::new();
    let fitted_for = RwSignal::new(String::new());
    Effect::new(move |_| {
        let Some(v) = view.get() else { return };
        if vp.scene.get().is_none() || fitted_for.get_untracked() == v.map.id {
            return;
        }
        vp.fit(f64::from(v.map.width_px), f64::from(v.map.height_px));
        fitted_for.set(v.map.id);
    });

    let full_screen = move |_| {
        if let Some(el) = vp.scene.get_untracked() {
            let _ = el.request_fullscreen();
        }
    };

    let scene = move || {
        let blackout = display.with(|d| d.as_ref().is_some_and(|d| d.blackout));
        if blackout {
            return view! {
                <div class="mimir-display__message" role="status">
                    <Text size="lg">"Paused"</Text>
                </div>
            }
            .into_any();
        }
        let Some(v) = view.get() else {
            return view! {
                <div class="mimir-display__message" role="status">
                    <Text size="lg" dimmed=true>"Waiting for the DM to show a map."</Text>
                </div>
            }
            .into_any();
        };
        let m = v.map.clone();
        let (w, h, g) = (
            f64::from(m.width_px),
            f64::from(m.height_px),
            m.grid_size_px,
        );
        let geo = geometry.get().flatten();
        let s = sight(&v, geo.as_ref());
        let cover = s.cover;
        let holes = s.holes.clone();
        let tokens: Vec<PlayerToken> = v
            .tokens
            .iter()
            .filter(|t| token_seen(t, &v, &s))
            .cloned()
            .collect();
        let grid = grid_path(m.columns, m.rows, g);
        view! {
            <svg class="mimir-display__svg" role="img" aria-label=format!("Map: {}", m.name)>
                <defs>
                    <clipPath id="mimir-display-clip">
                        <rect x="0" y="0" width=w height=h />
                    </clipPath>
                    <mask id="mimir-sight-mask" maskUnits="userSpaceOnUse" x="0" y="0" width=w height=h>
                        <rect x="0" y="0" width=w height=h fill="white" />
                        {holes.into_iter().map(|d| view! { <path d=d fill="black" /> }).collect_view()}
                    </mask>
                </defs>
                <g transform=move || vp.view.get().transform()>
                    {move || image.get().flatten().map(|href| view! { <image href=href x="0" y="0" width=w height=h /> })}
                    <path class="mimir-map__grid" d=grid />
                    <g clip-path="url(#mimir-display-clip)">
                    {v.lights.iter().map(|l| {
                        let color = l.color.clone().unwrap_or_else(|| "#ffcc66".into());
                        let dim = vision::feet_to_px(f64::from(l.dim_radius_ft), g);
                        let bright = vision::feet_to_px(f64::from(l.bright_radius_ft), g);
                        view! {
                            <circle cx=l.x cy=l.y r=dim fill=color.clone() opacity="0.10" />
                            <circle cx=l.x cy=l.y r=bright fill=color opacity="0.14" />
                        }
                    }).collect_view()}
                    </g>
                    {v.markers.iter().map(|mk| {
                        let (x, y) = ((f64::from(mk.grid_x) + 0.5) * g, (f64::from(mk.grid_y) + 0.5) * g);
                        let s = g * 0.3;
                        let shape = if mk.kind == "trap" {
                            view! { <path class="mimir-display__trap" d=format!("M{x} {}L{} {}L{} {}Z", y - s, x + s, y + s, x - s, y + s) /> }.into_any()
                        } else {
                            let color = mk.color.clone().unwrap_or_else(|| "var(--ice)".into());
                            view! { <circle cx=x cy=y r=g * 0.25 style=format!("fill:{color}") /> }.into_any()
                        };
                        view! {
                            <g class="mimir-map__marker" aria-label=mk.name.clone()>
                                {shape}
                                <text x=x y=y + s + g * 0.3 text-anchor="middle">{mk.name.clone()}</text>
                            </g>
                        }
                    }).collect_view()}
                    {tokens.into_iter().map(|t| {
                        let side = cells_for_size(&t.size) * g;
                        let (x, y) = (f64::from(t.grid_x) * g, f64::from(t.grid_y) * g);
                        let color = t.color.clone().unwrap_or_else(|| match t.token_type.as_str() {
                            "pc" => "var(--ok)".to_string(),
                            "npc" => "var(--ice)".to_string(),
                            _ => "var(--bad)".to_string(),
                        });
                        let initial: String = t.name.chars().next().map(String::from).unwrap_or_default();
                        let clip = format!("mimir-dtok-{}", t.id);
                        let tid = t.id.clone();
                        view! {
                            <g class="mimir-map__token" aria-label=t.name.clone()>
                                <clipPath id=clip.clone()>
                                    <circle cx=x + side / 2.0 cy=y + side / 2.0 r=side * 0.45 />
                                </clipPath>
                                <circle class="mimir-map__token-ring" cx=x + side / 2.0 cy=y + side / 2.0 r=side * 0.47 style=format!("stroke:{color}") />
                                {move || match art.with(|a| a.get(&tid).cloned().flatten()) {
                                    Some(href) => view! { <image href=href x=x y=y width=side height=side clip-path=format!("url(#{clip})") /> }.into_any(),
                                    None => view! { <text class="mimir-map__token-initial" x=x + side / 2.0 y=y + side / 2.0 text-anchor="middle" dominant-baseline="central" font-size=side * 0.45>{initial.clone()}</text> }.into_any(),
                                }}
                                <text class="mimir-map__token-name" x=x + side / 2.0 y=y + side + g * 0.28 text-anchor="middle">{t.name.clone()}</text>
                            </g>
                        }
                    }).collect_view()}
                    {(cover > 0.0).then(|| view! {
                        <rect class="mimir-display__cover" x="0" y="0" width=w height=h opacity=cover mask="url(#mimir-sight-mask)" />
                    })}
                </g>
            </svg>
        }
            .into_any()
    };

    view! {
        <div class="mimir-display">
            <div
                class="mimir-display__scene"
                node_ref=vp.scene
                on:pointerdown=move |ev: web_sys::PointerEvent| vp.down(&ev)
                on:pointermove=move |ev: web_sys::PointerEvent| vp.moved(&ev)
                on:pointerup=move |ev: web_sys::PointerEvent| {
                    vp.up(&ev);
                }
                on:pointercancel=move |ev: web_sys::PointerEvent| {
                    vp.up(&ev);
                }
                on:wheel=move |ev: web_sys::WheelEvent| vp.wheel(&ev)
            >
                {scene}
            </div>
            <div class="mimir-display__controls">
                <ActionIcon title="Zoom in" on_click=Callback::new(move |_| vp.zoom(1.25))>"+"</ActionIcon>
                <ActionIcon title="Zoom out" on_click=Callback::new(move |_| vp.zoom(0.8))>"−"</ActionIcon>
                <ActionIcon title="Fit the map" on_click=Callback::new(move |_| {
                    if let Some(v) = view.get_untracked() {
                        vp.fit(f64::from(v.map.width_px), f64::from(v.map.height_px));
                    }
                })>"⤢"</ActionIcon>
                <ActionIcon title="Full screen" on_click=Callback::new(full_screen)>"⛶"</ActionIcon>
            </div>
        </div>
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mimir_wire::{Fog, FogArea, MapDetail, PlayerMarker};

    fn map(mode: &str) -> MapDetail {
        MapDetail {
            id: "m1".into(),
            campaign_id: "c1".into(),
            module_id: None,
            name: "Cave".into(),
            description: None,
            sort_order: 0,
            lighting_mode: mode.into(),
            fog_enabled: false,
            grid_size_px: 70.0,
            columns: 10.0,
            rows: 10.0,
            width_px: 700,
            height_px: 700,
        }
    }

    fn tok(id: &str, kind: &str, gx: i32, gy: i32) -> PlayerToken {
        PlayerToken {
            id: id.into(),
            name: id.into(),
            token_type: kind.into(),
            size: "medium".into(),
            grid_x: gx,
            grid_y: gy,
            x: (f64::from(gx) + 0.5) * 70.0,
            y: (f64::from(gy) + 0.5) * 70.0,
            color: None,
            vision_bright_ft: Some(30),
            vision_dim_ft: Some(30),
            vision_dark_ft: 0,
            light_radius_ft: 0,
        }
    }

    fn view(mode: &str, fog: bool, tokens: Vec<PlayerToken>) -> PlayerView {
        PlayerView {
            map: map(mode),
            fog: Fog {
                enabled: fog,
                revealed: vec![],
            },
            tokens,
            lights: vec![],
            markers: vec![] as Vec<PlayerMarker>,
            initiative: None,
        }
    }

    #[test]
    fn fog_covers_all_but_the_party_sight() {
        let v = view(
            "bright",
            true,
            vec![
                tok("pc", "pc", 1, 1),
                tok("orc", "monster", 8, 8),
                tok("rat", "monster", 2, 1),
            ],
        );
        let s = sight(&v, None);
        assert_eq!(s.cover, 1.0);
        assert_eq!(s.holes.len(), 1, "one PC, one hole");
        let seen: Vec<&str> = v
            .tokens
            .iter()
            .filter(|t| token_seen(t, &v, &s))
            .map(|t| t.id.as_str())
            .collect();
        assert_eq!(seen, vec!["pc", "rat"], "the orc is beyond 30 ft");
    }

    #[test]
    fn revealed_areas_show_and_their_tokens_too() {
        let mut v = view(
            "bright",
            true,
            vec![tok("pc", "pc", 0, 0), tok("orc", "monster", 8, 8)],
        );
        v.fog.revealed = vec![FogArea {
            id: "a".into(),
            x: 500.0,
            y: 500.0,
            width: 200.0,
            height: 200.0,
        }];
        let s = sight(&v, None);
        assert_eq!(s.holes.len(), 2);
        assert!(token_seen(&v.tokens[1], &v, &s));
    }

    #[test]
    fn without_fog_bright_unlimited_sight_has_no_cover() {
        let mut pc = tok("pc", "pc", 1, 1);
        pc.vision_bright_ft = None;
        let v = view("bright", false, vec![pc]);
        assert_eq!(sight(&v, None).cover, 0.0);
        let dark = view("dark", false, vec![tok("pc", "pc", 1, 1)]);
        assert_eq!(sight(&dark, None).cover, 0.92);
        let dim = view("dim", false, vec![tok("pc", "pc", 1, 1)]);
        assert_eq!(sight(&dim, None).cover, 0.75);
    }

    #[test]
    fn no_pcs_shows_all_unless_fog() {
        let v = view("bright", false, vec![tok("orc", "monster", 8, 8)]);
        let s = sight(&v, None);
        assert!(token_seen(&v.tokens[0], &v, &s));
        let f = view("bright", true, vec![tok("orc", "monster", 8, 8)]);
        let s = sight(&f, None);
        assert!(!token_seen(&f.tokens[0], &f, &s));
    }

    #[test]
    fn walls_limit_the_sight() {
        let v = view(
            "bright",
            true,
            vec![tok("pc", "pc", 1, 1), tok("orc", "monster", 4, 1)],
        );
        let mut geo = MapGeometry {
            grid_size_px: 70.0,
            columns: 10.0,
            rows: 10.0,
            walls: vec![],
            portals: vec![],
            lights: vec![],
            ambient_light: None,
            baked_lighting: false,
        };
        let mut far_sight = v.clone();
        far_sight.tokens[0].vision_bright_ft = None;
        let open = sight(&far_sight, Some(&geo));
        assert!(token_seen(&far_sight.tokens[1], &far_sight, &open));
        // A wall between them at x = 3 cells.
        geo.walls = vec![vec![
            mimir_wire::GridPoint { x: 3.0, y: 0.0 },
            mimir_wire::GridPoint { x: 3.0, y: 10.0 },
        ]];
        let walled = sight(&far_sight, Some(&geo));
        assert!(!token_seen(&far_sight.tokens[1], &far_sight, &walled));
    }
}
