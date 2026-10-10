//! `/maps/:id`: the DM map. One SVG scene in map pixels under a pan/zoom
//! transform: the map image, the grid, lights, the fog the players have,
//! markers, doors and tokens. Pointer events drive pan, pinch zoom and
//! token drag, so a mouse and a tablet both work. Each change goes to the
//! API; the live socket tells this page (and the other clients) what
//! changed, and the page reads that part again.

use std::collections::{HashMap, HashSet};

use aurora_leptos::components::*;
use aurora_leptos::frame::Card;
use aurora_leptos::tokens::{token, ApiError};
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::hooks::{use_navigate, use_params_map};
use mimir_wire::{
    self as wire, DisplayState, DisplayUpdate, Fog, Light, MapDetail, MapGeometry, MapPart,
    ModuleMonsterSummary, ModuleNpcSummary, Poi, ServerMsg, Token, Trap,
};
use serde_json::json;

use super::viewport::Viewport;
use super::vision::{self, LightLevel, Point};
use crate::api;
use crate::live::Live;

// ---- Pure helpers (host-tested) ------------------------------------------

/// Grid cells a creature of this size covers (5e).
pub fn cells_for_size(size: &str) -> f64 {
    match size {
        "tiny" => 0.5,
        "large" => 2.0,
        "huge" => 3.0,
        "gargantuan" => 4.0,
        _ => 1.0,
    }
}

/// The grid cell of a map pixel.
pub fn cell_of(px: f64, grid_px: f64) -> i32 {
    if grid_px <= 0.0 {
        return 0;
    }
    (px / grid_px).floor() as i32
}

/// SVG path data for the grid lines of a map.
pub fn grid_path(columns: f64, rows: f64, grid_px: f64) -> String {
    let (w, h) = (columns * grid_px, rows * grid_px);
    let mut d = String::new();
    for c in 0..=(columns.ceil() as i64) {
        let x = c as f64 * grid_px;
        d.push_str(&format!("M{x:.1} 0V{h:.1}"));
    }
    for r in 0..=(rows.ceil() as i64) {
        let y = r as f64 * grid_px;
        d.push_str(&format!("M0 {y:.1}H{w:.1}"));
    }
    d
}

/// What a tap on the map places.
#[derive(Debug, Clone, PartialEq)]
pub enum Place {
    Pc,
    Monster(String),
    Npc(String),
    Torch,
    Lantern,
    Trap,
    Poi,
}

impl Place {
    /// From the value of the place menu ("pc", "monster:<id>", …).
    pub fn parse(value: &str) -> Option<Place> {
        match value.split_once(':') {
            Some(("monster", id)) => Some(Place::Monster(id.to_string())),
            Some(("npc", id)) => Some(Place::Npc(id.to_string())),
            _ => match value {
                "pc" => Some(Place::Pc),
                "torch" => Some(Place::Torch),
                "lantern" => Some(Place::Lantern),
                "trap" => Some(Place::Trap),
                "poi" => Some(Place::Poi),
                _ => None,
            },
        }
    }
}

/// The selected thing.
#[derive(Debug, Clone, PartialEq)]
enum Selected {
    Token(String),
    Light(String),
    Trap(String),
    Poi(String),
}

// ---- The page --------------------------------------------------------------

type Res<T> = LocalResource<Result<T, ApiError>>;

/// A change to send: a boxed future that reports its error.
type Job = std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), ApiError>>>>;

/// A drag of a token: its id and where it is now (map pixels).
#[derive(Debug, Clone, PartialEq)]
struct Drag {
    token_id: String,
    x: f64,
    y: f64,
    moved: bool,
}

#[component]
pub fn DmMapPage() -> impl IntoView {
    let params = use_params_map();
    let id = Memo::new(move |_| params.get().get("id").unwrap_or_default());

    // A version per part: a bump reads that part again.
    let tokens_v = RwSignal::new(0u32);
    let fog_v = RwSignal::new(0u32);
    let lights_v = RwSignal::new(0u32);
    let markers_v = RwSignal::new(0u32);
    let bump = move |part: MapPart| match part {
        MapPart::Tokens => tokens_v.update(|v| *v += 1),
        MapPart::Fog => fog_v.update(|v| *v += 1),
        MapPart::Lights => lights_v.update(|v| *v += 1),
        MapPart::Markers => markers_v.update(|v| *v += 1),
        MapPart::Map => {
            tokens_v.update(|v| *v += 1);
            fog_v.update(|v| *v += 1);
            lights_v.update(|v| *v += 1);
            markers_v.update(|v| *v += 1);
        }
    };

    let map: Res<MapDetail> = LocalResource::new(move || api::get(format!("/maps/{}", id.get())));
    let geometry: Res<MapGeometry> =
        LocalResource::new(move || api::get(format!("/maps/{}/geometry", id.get())));
    let image: Res<Option<String>> =
        LocalResource::new(move || api::image_url(format!("/maps/{}/image", id.get())));
    let tokens: Res<Vec<Token>> = LocalResource::new(move || {
        tokens_v.track();
        api::get(format!("/maps/{}/tokens", id.get()))
    });
    let lights: Res<Vec<Light>> = LocalResource::new(move || {
        lights_v.track();
        api::get(format!("/maps/{}/lights", id.get()))
    });
    let traps: Res<Vec<Trap>> = LocalResource::new(move || {
        markers_v.track();
        api::get(format!("/maps/{}/traps", id.get()))
    });
    let pois: Res<Vec<Poi>> = LocalResource::new(move || {
        markers_v.track();
        api::get(format!("/maps/{}/pois", id.get()))
    });
    let fog: Res<Fog> = LocalResource::new(move || {
        fog_v.track();
        api::get(format!("/maps/{}/fog", id.get()))
    });

    let map_now = move || map.get().and_then(Result::ok);
    let campaign = Memo::new(move |_| map_now().map(|m| m.campaign_id).unwrap_or_default());
    let module = Memo::new(move |_| map_now().and_then(|m| m.module_id));

    // The maps to switch to: the campaign's own and the module's.
    let siblings: Res<Vec<wire::MapSummary>> = LocalResource::new(move || {
        let (c, m) = (campaign.get(), module.get());
        async move {
            if c.is_empty() {
                return Ok(Vec::new());
            }
            let mut all: Vec<wire::MapSummary> = api::campaign_list(c, "maps").await?;
            if let Some(m) = m {
                all.extend(api::module_list::<wire::MapSummary>(m, "maps").await?);
            }
            Ok(all)
        }
    });
    let monsters: Res<Vec<ModuleMonsterSummary>> = LocalResource::new(move || {
        let m = module.get();
        async move {
            match m {
                Some(m) => api::module_list(m, "monsters").await,
                None => Ok(Vec::new()),
            }
        }
    });
    let npcs: Res<Vec<ModuleNpcSummary>> = LocalResource::new(move || {
        let m = module.get();
        async move {
            match m {
                Some(m) => api::module_list(m, "npcs").await,
                None => Ok(Vec::new()),
            }
        }
    });

    // The player display of the campaign, kept by the socket.
    let display = RwSignal::new(DisplayState::default());
    let live = StoredValue::new_local(None::<Live>);
    Effect::new(move |_| {
        let c = campaign.get();
        if c.is_empty() {
            return;
        }
        let here = id.get_untracked();
        live.set_value(Some(Live::watch(c, move |msg| match msg {
            ServerMsg::MapChanged { map_id, part } if map_id == here => bump(part),
            ServerMsg::Display { display: d } => display.set(d),
            _ => {}
        })));
    });

    // DM-side state.
    let vp = Viewport::new();
    let view = vp.view;
    let fitted = RwSignal::new(false);
    let los = RwSignal::new(true);
    let ambient = RwSignal::new(String::from("bright"));
    Effect::new(move |_| {
        if let Some(m) = map_now() {
            ambient.set(LightLevel::from_mode(&m.lighting_mode).as_str().to_string());
        }
    });
    let dead = RwSignal::new(HashSet::<String>::new());
    let doors_flipped = RwSignal::new(HashSet::<usize>::new());
    let selected = RwSignal::new(None::<Selected>);
    let place = RwSignal::new(None::<Place>);
    let place_menu = RwSignal::new(String::new());
    let drag = RwSignal::new(None::<Drag>);
    let error = RwSignal::new(String::new());
    let scene = vp.scene;

    // Token art, fetched once per token.
    let token_art = RwSignal::new(HashMap::<String, Option<String>>::new());
    Effect::new(move |_| {
        let Some(Ok(list)) = tokens.get() else { return };
        for t in list.into_iter().filter(|t| t.monster_id.is_some()) {
            if token_art.with_untracked(|m| m.contains_key(&t.id)) {
                continue;
            }
            token_art.update(|m| {
                m.insert(t.id.clone(), None);
            });
            spawn_local(async move {
                if let Ok(Some(url)) = api::image_url(format!("/tokens/{}/image", t.id)).await {
                    token_art.update(|m| {
                        m.insert(t.id, Some(url));
                    });
                }
            });
        }
    });

    // Fit the map to the box once both are known.
    Effect::new(move |_| {
        let (Some(m), Some(_)) = (map_now(), scene.get()) else {
            return;
        };
        if !fitted.get_untracked() {
            vp.fit(f64::from(m.width_px), f64::from(m.height_px));
            fitted.set(true);
        }
    });

    // A failed change shows in a banner; the next read puts the truth back.
    let report = move |r: Result<(), ApiError>| {
        if let Err(e) = r {
            error.set(match e {
                ApiError::Http { message, .. } => message,
                ApiError::Network => "The server did not answer.".into(),
                ApiError::Unknown(m) => m,
            });
        }
    };
    let run = move |fut: Job| {
        spawn_local(async move { report(fut.await) });
    };

    let grid_px = move || map_now().map(|m| m.grid_size_px).unwrap_or(70.0);

    // Screen point (client) → map point.
    let to_map = move |cx: f64, cy: f64| vp.to_map(cx, cy);

    let place_at = move |kind: Place, mx: f64, my: f64| {
        let g = grid_px();
        let (gx, gy) = (cell_of(mx, g), cell_of(my, g));
        let map_id = id.get_untracked();
        let ask = |what: &str| -> Option<String> {
            web_sys::window()?
                .prompt_with_message(what)
                .ok()
                .flatten()
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
        };
        let body = match &kind {
            Place::Pc => ask("Name of the player character").map(|label| {
                (
                    "tokens",
                    json!({"grid_x": gx, "grid_y": gy, "label": label}),
                )
            }),
            Place::Monster(m) => Some((
                "tokens",
                json!({"grid_x": gx, "grid_y": gy, "monster_id": m}),
            )),
            Place::Npc(n) => Some(("tokens", json!({"grid_x": gx, "grid_y": gy, "npc_id": n}))),
            Place::Torch => Some((
                "lights",
                json!({"grid_x": gx, "grid_y": gy, "preset": "torch"}),
            )),
            Place::Lantern => Some((
                "lights",
                json!({"grid_x": gx, "grid_y": gy, "preset": "lantern"}),
            )),
            Place::Trap => ask("Name of the trap")
                .map(|name| ("traps", json!({"grid_x": gx, "grid_y": gy, "name": name}))),
            Place::Poi => ask("Name of the point of interest")
                .map(|name| ("pois", json!({"grid_x": gx, "grid_y": gy, "name": name}))),
        };
        // Monsters and NPCs can go on the map more than once; the others once.
        if !matches!(kind, Place::Monster(_) | Place::Npc(_)) {
            place.set(None);
            place_menu.set(String::new());
        }
        if let Some((what, body)) = body {
            run(Box::pin(async move {
                api::send::<_, serde_json::Value>("POST", &format!("/maps/{map_id}/{what}"), &body)
                    .await
                    .map(|_| ())
            }));
        }
    };

    // ---- Pointer handling --------------------------------------------------
    let on_down = move |ev: web_sys::PointerEvent| vp.down(&ev);
    let on_move = move |ev: web_sys::PointerEvent| {
        if drag.with_untracked(Option::is_some) {
            let (mx, my) = to_map(f64::from(ev.client_x()), f64::from(ev.client_y()));
            drag.update(|d| {
                if let Some(d) = d {
                    d.x = mx;
                    d.y = my;
                    d.moved = true;
                }
            });
            return;
        }
        vp.moved(&ev);
    };
    let on_up = move |ev: web_sys::PointerEvent| {
        if let Some(d) = drag.get_untracked() {
            drag.set(None);
            if d.moved {
                let g = grid_px();
                let (gx, gy) = (cell_of(d.x, g), cell_of(d.y, g));
                let path = format!("/tokens/{}", d.token_id);
                run(Box::pin(async move {
                    api::send::<_, Token>("PATCH", &path, &json!({"grid_x": gx, "grid_y": gy}))
                        .await
                        .map(|_| ())
                }));
            }
        }
        if vp.up(&ev) {
            if let Some(kind) = place.get_untracked() {
                let (mx, my) = to_map(f64::from(ev.client_x()), f64::from(ev.client_y()));
                place_at(kind, mx, my);
            } else {
                selected.set(None);
            }
        }
    };
    let on_wheel = move |ev: web_sys::WheelEvent| vp.wheel(&ev);
    let zoom_button = move |factor: f64| vp.zoom(factor);
    let fit = move || {
        if let Some(m) = map_now() {
            vp.fit(f64::from(m.width_px), f64::from(m.height_px));
        }
    };

    // ---- Derived scene data ------------------------------------------------
    let token_list = move || tokens.get().and_then(Result::ok).unwrap_or_default();
    let light_list = move || lights.get().and_then(Result::ok).unwrap_or_default();
    let doors = move || {
        let mut d = geometry
            .get()
            .and_then(Result::ok)
            .map(|g| vision::doors_px(&g))
            .unwrap_or_default();
        doors_flipped.with(|f| {
            for i in f {
                if let Some(door) = d.get_mut(*i) {
                    door.closed = !door.closed;
                }
            }
        });
        d
    };
    let walls = move || {
        geometry
            .get()
            .and_then(Result::ok)
            .map(|g| vision::walls_px(&g))
            .unwrap_or_default()
    };
    let level = move || LightLevel::from_mode(&ambient.get());

    // What the players do not see (DM view: a shade over it).
    let fog_shapes =
        Memo::new(move |_| {
            let m = map_now()?;
            let fog_on = fog
                .get()
                .and_then(Result::ok)
                .map(|f| f.enabled)
                .unwrap_or(false);
            let g = m.grid_size_px;
            let toks = token_list();
            let zones = vision::light_zones(&light_list(), &toks, g);
            let party = vision::party_vision(&toks, &zones, level(), g);
            if !fog_on && !vision::needs_vision_overlay(level(), &party) {
                return None;
            }
            let (w, h) = (f64::from(m.width_px), f64::from(m.height_px));
            let blocking = if los.get() {
                vision::blocking(&walls(), &doors())
            } else {
                Vec::new()
            };
            let cap = w.max(h) * 2.0;
            let mut holes: Vec<String> = party
                .iter()
                .filter(|v| v.radius_px > 0.0)
                .map(|v| {
                    let poly = vision::visibility_polygon(
                        Point { x: v.x, y: v.y },
                        &blocking,
                        v.radius_px.min(cap),
                        w,
                        h,
                    );
                    vision::svg_path(&poly)
                })
                .collect();
            if fog_on {
                if let Some(Ok(f)) = fog.get() {
                    holes.extend(f.revealed.iter().map(|a| {
                        format!("M{} {}h{}v{}h{}Z", a.x, a.y, a.width, a.height, -a.width)
                    }));
                }
            }
            Some((w, h, holes))
        });

    let shown = move || display.with(|d| d.map_id.as_deref() == Some(id.get().as_str()));
    let blackout = move || display.with(|d| d.blackout);
    let set_display = move |map_id: Option<String>, black: bool| {
        let c = campaign.get_untracked();
        run(Box::pin(async move {
            let d: DisplayState = api::send(
                "PUT",
                &format!("/campaigns/{c}/display"),
                &DisplayUpdate {
                    map_id,
                    blackout: black,
                },
            )
            .await?;
            display.set(d);
            Ok(())
        }));
    };
    let fog_switch = RwSignal::new(false);
    Effect::new(move |_| {
        if let Some(Ok(f)) = fog.get() {
            fog_switch.set(f.enabled);
        }
    });
    let set_fog = Callback::new(move |on: bool| {
        let path = format!("/maps/{}/fog", id.get_untracked());
        run(Box::pin(async move {
            api::send::<_, Fog>("PUT", &path, &json!({ "enabled": on }))
                .await
                .map(|_| ())
        }));
    });

    let map_select = RwSignal::new(String::new());
    Effect::new(move |_| map_select.set(id.get()));
    let navigate = use_navigate();
    let go_map = Callback::new(move |m: String| {
        if !m.is_empty() && m != id.get_untracked() {
            fitted.set(false);
            selected.set(None);
            navigate(&format!("/maps/{m}"), Default::default());
        }
    });

    let place_options = move || {
        let mut o: Vec<(String, String)> = vec![
            ("pc".into(), "Player character".into()),
            ("torch".into(), "Torch".into()),
            ("lantern".into(), "Lantern".into()),
            ("trap".into(), "Trap".into()),
            ("poi".into(), "Point of interest".into()),
        ];
        if let Some(Ok(list)) = monsters.get() {
            o.extend(
                list.into_iter()
                    .map(|m| (format!("monster:{}", m.id), format!("Monster: {}", m.name))),
            );
        }
        if let Some(Ok(list)) = npcs.get() {
            o.extend(
                list.into_iter()
                    .map(|n| (format!("npc:{}", n.id), format!("NPC: {}", n.name))),
            );
        }
        o
    };
    let on_place = Callback::new(move |v: String| place.set(Place::parse(&v)));

    // ---- View --------------------------------------------------------------
    let header = move || {
        let maps = siblings.get().and_then(Result::ok).unwrap_or_default();
        let pairs: Vec<(String, String)> = maps.into_iter().map(|m| (m.id, m.name)).collect();
        view! {
            <div class="mimir-map__toolbar" role="toolbar" aria-label="Map tools">
                <div class="mimir-map__tool-group">
                    <Select aria_label="Map" option_pairs=pairs value=map_select on_change=go_map />
                </div>
                <div class="mimir-map__tool-group">
                    <Switch label="Fog" checked=fog_switch on_change=set_fog />
                    <Switch label="Line of sight" checked=los />
                    <SegmentedControl
                        options=vec!["bright".into(), "dim".into(), "darkness".into()]
                        value=ambient
                    />
                </div>
                <div class="mimir-map__tool-group">
                    <Select
                        aria_label="Place on the map"
                        placeholder="Place…"
                        option_pairs=place_options()
                        value=place_menu
                        on_change=on_place
                    />
                </div>
                <div class="mimir-map__tool-group">
                    {move || {
                        if shown() {
                            view! {
                                <Button variant="light" on_click=Callback::new(move |_| set_display(None, false))>
                                    "Stop showing"
                                </Button>
                            }
                                .into_any()
                        } else {
                            view! {
                                <Button on_click=Callback::new(move |_| set_display(Some(id.get_untracked()), false))>
                                    "Show to players"
                                </Button>
                            }
                                .into_any()
                        }
                    }}
                    <Button
                        variant="light"
                        bad=true
                        on_click=Callback::new(move |_| {
                            let d = display.get_untracked();
                            set_display(d.map_id, !d.blackout)
                        })
                    >
                        {move || if blackout() { "End blackout" } else { "Blackout" }}
                    </Button>
                    <a
                        class="cl-btn cl-btn--subtle"
                        href=move || format!("/display/{}", campaign.get())
                        target="_blank"
                        rel="noopener"
                    >
                        "Open player display"
                    </a>
                    <ActionIcon title="Zoom in" on_click=Callback::new(move |_| zoom_button(1.25))>"+"</ActionIcon>
                    <ActionIcon title="Zoom out" on_click=Callback::new(move |_| zoom_button(0.8))>"−"</ActionIcon>
                    <ActionIcon title="Fit the map" on_click=Callback::new(move |_| fit())>"⤢"</ActionIcon>
                </div>
            </div>
        }
    };

    let scene_view = move || {
        let Some(m) = map_now() else {
            return view! { <Loading label="Loading the map…" /> }.into_any();
        };
        let (w, h, g) = (
            f64::from(m.width_px),
            f64::from(m.height_px),
            m.grid_size_px,
        );
        let grid = grid_path(m.columns, m.rows, g);
        view! {
            <div
                class="mimir-map__scene"
                class:mimir-map__scene--placing=move || place.with(Option::is_some)
                node_ref=scene
                on:pointerdown=on_down
                on:pointermove=on_move
                on:pointerup=on_up
                on:pointercancel=on_up
                on:wheel=on_wheel
            >
                <svg class="mimir-map__svg" aria-label=format!("Map: {}", m.name) role="img">
                    <defs>
                        <clipPath id="mimir-map-clip">
                            <rect x="0" y="0" width=w height=h />
                        </clipPath>
                        <mask id="mimir-fog-mask" maskUnits="userSpaceOnUse" x="0" y="0" width=w height=h>
                            <rect x="0" y="0" width=w height=h fill="white" />
                            {move || {
                                fog_shapes
                                    .get()
                                    .map(|(_, _, holes)| {
                                        holes
                                            .into_iter()
                                            .map(|d| view! { <path d=d fill="black" /> })
                                            .collect_view()
                                    })
                            }}
                        </mask>
                    </defs>
                    <g transform=move || view.get().transform()>
                        {move || {
                            image
                                .get()
                                .and_then(Result::ok)
                                .flatten()
                                .map(|href| view! { <image href=href x="0" y="0" width=w height=h /> })
                        }}
                        <path class="mimir-map__grid" d=grid />
                        // Lights: placed lights (the DM sees unlit ones faint).
                        {move || {
                            light_list()
                                .into_iter()
                                .map(|l| {
                                    let color = l.color.clone().unwrap_or_else(|| "#ffcc66".into());
                                    let dim = vision::feet_to_px(f64::from(l.dim_radius_ft), g);
                                    let bright = vision::feet_to_px(f64::from(l.bright_radius_ft), g);
                                    let sel = Selected::Light(l.id.clone());
                                    let sel2 = sel.clone();
                                    view! {
                                        <g class="mimir-map__light" class:mimir-map__light--off=!l.active>
                                            <g clip-path="url(#mimir-map-clip)">
                                                <circle cx=l.x cy=l.y r=dim fill=color.clone() opacity="0.10" />
                                                <circle cx=l.x cy=l.y r=bright fill=color.clone() opacity="0.14" />
                                            </g>
                                            <circle
                                                class="mimir-map__handle"
                                                class:mimir-map__handle--selected=move || selected.get() == Some(sel2.clone())
                                                cx=l.x
                                                cy=l.y
                                                r=g * 0.22
                                                fill=color
                                                aria-label=format!("Light {}", l.name.clone().unwrap_or_default())
                                                on:pointerdown=move |ev: web_sys::PointerEvent| {
                                                    ev.stop_propagation();
                                                    selected.set(Some(sel.clone()));
                                                }
                                            />
                                        </g>
                                    }
                                })
                                .collect_view()
                        }}
                        // The shade over what the players do not see.
                        {move || {
                            fog_shapes
                                .get()
                                .map(|(w, h, _)| {
                                    view! {
                                        <rect
                                            class="mimir-map__fog"
                                            x="0"
                                            y="0"
                                            width=w
                                            height=h
                                            mask="url(#mimir-fog-mask)"
                                        />
                                    }
                                })
                        }}
                        // Doors: a tap opens or closes one (this page only).
                        {move || {
                            doors()
                                .into_iter()
                                .enumerate()
                                .map(|(i, d)| {
                                    view! {
                                        <line
                                            class="mimir-map__door"
                                            class:mimir-map__door--open=!d.closed
                                            x1=d.wall.p1.x
                                            y1=d.wall.p1.y
                                            x2=d.wall.p2.x
                                            y2=d.wall.p2.y
                                            on:pointerdown=move |ev: web_sys::PointerEvent| {
                                                ev.stop_propagation();
                                                doors_flipped.update(|f| {
                                                    if !f.remove(&i) {
                                                        f.insert(i);
                                                    }
                                                });
                                            }
                                        />
                                    }
                                })
                                .collect_view()
                        }}
                        // Markers.
                        {move || {
                            let traps = traps.get().and_then(Result::ok).unwrap_or_default();
                            traps
                                .into_iter()
                                .map(|t| {
                                    let (x, y) = ((f64::from(t.grid_x) + 0.5) * g, (f64::from(t.grid_y) + 0.5) * g);
                                    let s = g * 0.3;
                                    let sel = Selected::Trap(t.id.clone());
                                    let sel2 = sel.clone();
                                    view! {
                                        <g
                                            class="mimir-map__marker mimir-map__marker--trap"
                                            class:mimir-map__marker--hidden=!t.visible
                                            class:mimir-map__marker--selected=move || selected.get() == Some(sel2.clone())
                                            on:pointerdown=move |ev: web_sys::PointerEvent| {
                                                ev.stop_propagation();
                                                selected.set(Some(sel.clone()));
                                            }
                                        >
                                            <path d=format!("M{x} {}L{} {}L{} {}Z", y - s, x + s, y + s, x - s, y + s) />
                                            <text x=x y=y + s + g * 0.3 text-anchor="middle">{t.name}</text>
                                        </g>
                                    }
                                })
                                .collect_view()
                        }}
                        {move || {
                            let pois = pois.get().and_then(Result::ok).unwrap_or_default();
                            pois
                                .into_iter()
                                .map(|p| {
                                    let (x, y) = ((f64::from(p.grid_x) + 0.5) * g, (f64::from(p.grid_y) + 0.5) * g);
                                    let sel = Selected::Poi(p.id.clone());
                                    let sel2 = sel.clone();
                                    let color = p.color.clone().unwrap_or_else(|| token::ICE.to_string());
                                    view! {
                                        <g
                                            class="mimir-map__marker mimir-map__marker--poi"
                                            class:mimir-map__marker--hidden=!p.visible
                                            class:mimir-map__marker--selected=move || selected.get() == Some(sel2.clone())
                                            on:pointerdown=move |ev: web_sys::PointerEvent| {
                                                ev.stop_propagation();
                                                selected.set(Some(sel.clone()));
                                            }
                                        >
                                            <circle cx=x cy=y r=g * 0.25 style=format!("fill:{color}") />
                                            <text x=x y=y + g * 0.6 text-anchor="middle">{p.name}</text>
                                        </g>
                                    }
                                })
                                .collect_view()
                        }}
                        // Tokens.
                        {move || {
                            let dragging = drag.get();
                            token_list()
                                .into_iter()
                                .map(|t| {
                                    let cells = cells_for_size(&t.size);
                                    let side = cells * g;
                                    let (mut x, mut y) = (f64::from(t.grid_x) * g, f64::from(t.grid_y) * g);
                                    if let Some(d) = dragging.as_ref().filter(|d| d.token_id == t.id) {
                                        x = d.x - side / 2.0;
                                        y = d.y - side / 2.0;
                                    }
                                    let art = token_art.with(|m| m.get(&t.id).cloned().flatten());
                                    let is_dead = dead.with(|d| d.contains(&t.id));
                                    let tid = t.id.clone();
                                    let sel = Selected::Token(t.id.clone());
                                    let sel2 = sel.clone();
                                    let color = t.color.clone().unwrap_or_else(|| match t.token_type.as_str() {
                                        "pc" => token::OK.to_string(),
                                        "npc" => token::ICE.to_string(),
                                        _ => token::BAD.to_string(),
                                    });
                                    let initial: String = t.name.chars().next().map(String::from).unwrap_or_default();
                                    let clip = format!("mimir-tok-{}", t.id);
                                    view! {
                                        <g
                                            class="mimir-map__token"
                                            class:mimir-map__token--hidden=!t.visible_to_players
                                            class:mimir-map__token--dead=is_dead
                                            class:mimir-map__token--selected=move || selected.get() == Some(sel2.clone())
                                            aria-label=t.name.clone()
                                            on:pointerdown=move |ev: web_sys::PointerEvent| {
                                                ev.stop_propagation();
                                                if let Some(el) = scene.get_untracked() {
                                                    let _ = el.set_pointer_capture(ev.pointer_id());
                                                }
                                                selected.set(Some(sel.clone()));
                                                let (mx, my) = to_map(f64::from(ev.client_x()), f64::from(ev.client_y()));
                                                drag.set(Some(Drag { token_id: tid.clone(), x: mx, y: my, moved: false }));
                                            }
                                        >
                                            <clipPath id=clip.clone()>
                                                <circle cx=x + side / 2.0 cy=y + side / 2.0 r=side * 0.45 />
                                            </clipPath>
                                            <circle class="mimir-map__token-ring" cx=x + side / 2.0 cy=y + side / 2.0 r=side * 0.47 style=format!("stroke:{color}") />
                                            {match art {
                                                Some(href) => view! {
                                                    <image href=href x=x y=y width=side height=side clip-path=format!("url(#{clip})") />
                                                }.into_any(),
                                                None => view! {
                                                    <text class="mimir-map__token-initial" x=x + side / 2.0 y=y + side / 2.0 text-anchor="middle" dominant-baseline="central" font-size=side * 0.45>{initial}</text>
                                                }.into_any(),
                                            }}
                                            <text class="mimir-map__token-name" x=x + side / 2.0 y=y + side + g * 0.28 text-anchor="middle">{t.name.clone()}</text>
                                        </g>
                                    }
                                })
                                .collect_view()
                        }}
                    </g>
                </svg>
            </div>
        }
            .into_any()
    };

    // ---- The side panel of the selected thing ------------------------------
    let panel = move || {
        let sel = selected.get()?;
        let close = Callback::new(move |_| selected.set(None));
        let body = match sel {
            Selected::Token(tid) => {
                let t = token_list().into_iter().find(|t| t.id == tid)?;
                token_panel(t, dead, run).into_any()
            }
            Selected::Light(lid) => {
                let l = light_list().into_iter().find(|l| l.id == lid)?;
                light_panel(l, run).into_any()
            }
            Selected::Trap(tid) => {
                let t = traps
                    .get()
                    .and_then(Result::ok)?
                    .into_iter()
                    .find(|t| t.id == tid)?;
                trap_panel(t, run).into_any()
            }
            Selected::Poi(pid) => {
                let p = pois
                    .get()
                    .and_then(Result::ok)?
                    .into_iter()
                    .find(|p| p.id == pid)?;
                poi_panel(p, run).into_any()
            }
        };
        Some(view! {
            <aside class="mimir-map__panel" aria-label="Selected">
                <Card>
                    <Stack gap="sm">
                        {body}
                        <Button variant="subtle" on_click=close>"Close"</Button>
                    </Stack>
                </Card>
            </aside>
        })
    };

    view! {
        <div class="mimir-map">
            {header}
            {move || {
                place
                    .get()
                    .map(|_| {
                        view! {
                            <div class="mimir-map__hint" role="status">
                                <Text size="sm">"Tap the map to place it."</Text>
                                <Button variant="subtle" on_click=Callback::new(move |_| {
                                    place.set(None);
                                    place_menu.set(String::new());
                                })>"Done"</Button>
                            </div>
                        }
                    })
            }}
            {move || {
                let e = error.get();
                (!e.is_empty())
                    .then(|| {
                        view! {
                            <div class="mimir-map__error">
                                <Alert color=token::BAD>{e}</Alert>
                            </div>
                        }
                    })
            }}
            {move || match map.get() {
                Some(Err(e)) => view! { <ErrorState error=e /> }.into_any(),
                _ => scene_view(),
            }}
            {panel}
        </div>
    }
}

fn patch<T: serde::de::DeserializeOwned + 'static>(path: String, body: serde_json::Value) -> Job {
    Box::pin(async move { api::send::<_, T>("PATCH", &path, &body).await.map(|_| ()) })
}

fn remove(path: String) -> Job {
    Box::pin(async move { api::delete(&path).await })
}

fn confirm(question: &str) -> bool {
    web_sys::window()
        .and_then(|w| w.confirm_with_message(question).ok())
        .unwrap_or(false)
}

/// A number field that may be empty (unlimited).
fn feet(v: Option<i32>) -> String {
    v.map(|f| f.to_string()).unwrap_or_default()
}

fn token_panel(
    t: Token,
    dead: RwSignal<HashSet<String>>,
    run: impl Fn(Job) + Copy + Send + Sync + 'static,
) -> impl IntoView {
    let path = format!("/tokens/{}", t.id);
    let visible = RwSignal::new(t.visible_to_players);
    let is_dead = RwSignal::new(dead.with_untracked(|d| d.contains(&t.id)));
    let bright = RwSignal::new(feet(t.vision_bright_ft));
    let dim = RwSignal::new(feet(t.vision_dim_ft));
    let dark = RwSignal::new(t.vision_dark_ft.to_string());
    let light = RwSignal::new(t.light_radius_ft.to_string());
    let tid = t.id.clone();
    let (p1, p2, p3) = (path.clone(), path.clone(), path);
    view! {
        <Text bold=true>{t.name.clone()}</Text>
        <Text size="xs" dimmed=true>{format!("{} · {}", t.token_type, t.size)}</Text>
        <Switch
            label="Players see it"
            checked=visible
            on_change=Callback::new(move |on: bool| run(patch::<Token>(p1.clone(), json!({ "hidden": !on }))))
        />
        <Switch
            label="Dead"
            checked=is_dead
            on_change=Callback::new(move |on: bool| {
                dead.update(|d| {
                    if on {
                        d.insert(tid.clone());
                    } else {
                        d.remove(&tid);
                    }
                })
            })
        />
        <SectionLabel label="Vision (feet; empty: no limit)" />
        <TextInput label="In bright light" value=bright input_type="number" />
        <TextInput label="In dim light" value=dim input_type="number" />
        <TextInput label="Darkvision" value=dark input_type="number" />
        <TextInput label="Carried light" value=light input_type="number" />
        <Button on_click=Callback::new(move |_| {
            let num = |s: String| s.trim().parse::<i32>().ok();
            let body = json!({
                "vision_bright_ft": num(bright.get_untracked()),
                "vision_dim_ft": num(dim.get_untracked()),
                "vision_dark_ft": num(dark.get_untracked()).unwrap_or(0),
                "light_radius_ft": num(light.get_untracked()).unwrap_or(0),
            });
            run(patch::<Token>(p2.clone(), body))
        })>"Save vision"</Button>
        <Button bad=true variant="light" on_click=Callback::new(move |_| {
            if confirm("Remove this token from the map?") {
                run(remove(p3.clone()))
            }
        })>"Remove token"</Button>
    }
}

fn light_panel(l: Light, run: impl Fn(Job) + Copy + Send + Sync + 'static) -> impl IntoView {
    let path = format!("/lights/{}", l.id);
    let lit = RwSignal::new(l.active);
    let (p1, p2) = (path.clone(), path);
    view! {
        <Text bold=true>{l.name.clone().unwrap_or_else(|| "Light".into())}</Text>
        <Text size="xs" dimmed=true>{format!("bright {} ft · dim {} ft", l.bright_radius_ft, l.dim_radius_ft)}</Text>
        <Switch
            label="Lit"
            checked=lit
            on_change=Callback::new(move |on: bool| run(patch::<Light>(p1.clone(), json!({ "active": on }))))
        />
        <Button bad=true variant="light" on_click=Callback::new(move |_| run(remove(p2.clone())))>
            "Remove light"
        </Button>
    }
}

fn trap_panel(t: Trap, run: impl Fn(Job) + Copy + Send + Sync + 'static) -> impl IntoView {
    let path = format!("/traps/{}", t.id);
    let visible = RwSignal::new(t.visible);
    let triggered = RwSignal::new(t.triggered);
    let (p1, p2, p3) = (path.clone(), path.clone(), path);
    let details: Vec<(String, String)> = [
        ("DC", t.dc.map(|d| d.to_string())),
        ("Trigger", t.trigger_description.clone()),
        ("Effect", t.effect_description.clone()),
        ("Notes", t.description.clone()),
    ]
    .into_iter()
    .filter_map(|(k, v)| v.map(|v| (k.to_string(), v)))
    .collect();
    view! {
        <Text bold=true>{t.name.clone()}</Text>
        {details
            .into_iter()
            .map(|(k, v)| view! { <Text size="sm"><b>{k}": "</b>{v}</Text> })
            .collect_view()}
        <Switch
            label="Players see it"
            checked=visible
            on_change=Callback::new(move |on: bool| run(patch::<Trap>(p1.clone(), json!({ "visible": on }))))
        />
        <Switch
            label="Triggered"
            checked=triggered
            on_change=Callback::new(move |on: bool| run(patch::<Trap>(p2.clone(), json!({ "triggered": on }))))
        />
        <Button bad=true variant="light" on_click=Callback::new(move |_| {
            if confirm("Remove this trap?") {
                run(remove(p3.clone()))
            }
        })>"Remove trap"</Button>
    }
}

fn poi_panel(p: Poi, run: impl Fn(Job) + Copy + Send + Sync + 'static) -> impl IntoView {
    let path = format!("/pois/{}", p.id);
    let visible = RwSignal::new(p.visible);
    let (p1, p2) = (path.clone(), path);
    view! {
        <Text bold=true>{p.name.clone()}</Text>
        {p.description.clone().map(|d| view! { <Text size="sm">{d}</Text> })}
        <Switch
            label="Players see it"
            checked=visible
            on_change=Callback::new(move |on: bool| run(patch::<Poi>(p1.clone(), json!({ "visible": on }))))
        />
        <Button bad=true variant="light" on_click=Callback::new(move |_| {
            if confirm("Remove this point of interest?") {
                run(remove(p2.clone()))
            }
        })>"Remove"</Button>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cells_and_sizes() {
        assert_eq!(cell_of(69.9, 70.0), 0);
        assert_eq!(cell_of(70.0, 70.0), 1);
        assert_eq!(cell_of(-1.0, 70.0), -1);
        assert_eq!(cell_of(10.0, 0.0), 0);
        assert_eq!(cells_for_size("large"), 2.0);
        assert_eq!(cells_for_size("medium"), 1.0);
        assert_eq!(cells_for_size("tiny"), 0.5);
    }

    #[test]
    fn grid_lines() {
        let d = grid_path(2.0, 1.0, 10.0);
        assert_eq!(
            d,
            "M0.0 0V10.0M10.0 0V10.0M20.0 0V10.0M0 0.0H20.0M0 10.0H20.0"
        );
    }

    #[test]
    fn place_menu_values() {
        assert_eq!(Place::parse("pc"), Some(Place::Pc));
        assert_eq!(
            Place::parse("monster:m1"),
            Some(Place::Monster("m1".into()))
        );
        assert_eq!(Place::parse("npc:n1"), Some(Place::Npc("n1".into())));
        assert_eq!(Place::parse("torch"), Some(Place::Torch));
        assert_eq!(Place::parse(""), None);
    }
}
