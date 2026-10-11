//! `/maps/:id`: the DM map. One SVG scene in map pixels under a pan/zoom
//! transform: the map image, the grid, lights, the fog the players have,
//! markers, doors and tokens. Pointer events drive pan, pinch zoom and
//! token drag, so a mouse and a tablet both work. Each change goes to the
//! API; the live socket tells this page (and the other clients) what
//! changed, and the page reads that part again.

use std::collections::HashMap;

use aurora_leptos::components::*;
use aurora_leptos::frame::Card;
use aurora_leptos::tokens::{token, ApiError};
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::hooks::{use_navigate, use_params_map};
use mimir_wire::{
    self as wire, DisplayState, DisplayUpdate, DoorState, Fog, Light, MapDetail, MapGeometry,
    MapPart, ModuleMonsterSummary, ModuleNpcSummary, Poi, ServerMsg, Token, Trap,
};
use serde_json::json;

use super::display::sight_for;
use super::setup::{poi_emoji, Place, PlaceForm, PlaceOptions, PlacedLists, PoiDialog, PoiEdit};
use super::tracker::{self, CombatTracker};
use super::viewport::Viewport;
use super::vision::{self, LightLevel};
use crate::api;
use crate::live::Live;
use crate::reference::detail::{Reference, Target};

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

/// Vision presets (the desktop's): label, sight in bright light, in dim
/// light (`None`: no limit), in darkness, and the carried light, in feet.
pub type VisionPreset = (&'static str, Option<i32>, Option<i32>, i32, i32);

pub const VISION_PRESETS: &[VisionPreset] = &[
    ("Human", None, None, 0, 0),
    ("Human + Torch", None, None, 0, 40),
    ("Human + Lantern", None, None, 0, 60),
    ("Darkvision 60 ft", None, None, 60, 0),
    ("Darkvision 120 ft", None, None, 120, 0),
    ("Blindsight 30 ft", Some(30), Some(30), 30, 0),
    ("Blindsight 60 ft", Some(60), Some(60), 60, 0),
    ("Devil's Sight 120 ft", None, None, 120, 0),
];

/// The preset a token's vision matches, if any (the first one).
pub fn preset_of(t: &Token) -> Option<&'static str> {
    VISION_PRESETS
        .iter()
        .find(|p| {
            (p.1, p.2, p.3, p.4)
                == (
                    t.vision_bright_ft,
                    t.vision_dim_ft,
                    t.vision_dark_ft,
                    t.light_radius_ft,
                )
        })
        .map(|p| p.0)
}

/// A key on the DM map (the desktop's keys).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MapKey {
    ZoomIn,
    ZoomOut,
    Fit,
    ToggleHidden,
    ToggleDead,
    Deselect,
}

pub fn map_key(key: &str) -> Option<MapKey> {
    match key {
        "+" | "=" => Some(MapKey::ZoomIn),
        "-" => Some(MapKey::ZoomOut),
        "0" => Some(MapKey::Fit),
        "h" | "H" => Some(MapKey::ToggleHidden),
        "d" | "D" => Some(MapKey::ToggleDead),
        "Escape" => Some(MapKey::Deselect),
        _ => None,
    }
}

/// A key for the page, not for a field: no modifier, and the focus is not
/// in a text field, a select or an editor.
pub fn page_key(ev: &web_sys::KeyboardEvent) -> bool {
    if ev.ctrl_key() || ev.meta_key() || ev.alt_key() {
        return false;
    }
    let Some(el) = ev
        .target()
        .and_then(|t| wasm_bindgen::JsCast::dyn_into::<web_sys::HtmlElement>(t).ok())
    else {
        return true;
    };
    let tag = el.tag_name().to_ascii_uppercase();
    !(matches!(tag.as_str(), "INPUT" | "TEXTAREA" | "SELECT") || el.is_content_editable())
}

/// The PCs of the campaign that have no token on the map (by name).
pub fn missing_pcs(pcs: &[String], tokens: &[Token]) -> Vec<String> {
    pcs.iter()
        .filter(|name| {
            !tokens
                .iter()
                .any(|t| t.token_type == "pc" && t.name.eq_ignore_ascii_case(name))
        })
        .cloned()
        .collect()
}

/// `n` cells in a row from `start`, wrapping inside the map.
pub fn row_of_cells(start: (i32, i32), n: usize, columns: f64, rows: f64) -> Vec<(i32, i32)> {
    let cols = (columns.floor() as i32).max(1);
    let last_row = (rows.floor() as i32 - 1).max(0);
    let (sx, sy) = (start.0.clamp(0, cols - 1), start.1.clamp(0, last_row));
    (0..n as i32)
        .map(|i| {
            let x = sx + i;
            ((x % cols), (sy + x / cols).min(last_row))
        })
        .collect()
}

/// A second tap on the same thing within 400 ms.
pub fn is_double_tap(last: &str, last_at: f64, key: &str, now: f64) -> bool {
    last == key && now - last_at <= 400.0
}

/// Details to open after a double tap.
#[derive(Debug, Clone, PartialEq)]
enum Detail {
    Monster(Option<String>),
    Trap(String),
    Poi(PoiEdit),
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

/// A drag of a token, a light, a trap or a POI: what, and where it is now
/// (map pixels).
#[derive(Debug, Clone, PartialEq)]
struct Drag {
    what: Selected,
    x: f64,
    y: f64,
    moved: bool,
}

impl Drag {
    /// Where the dragged thing is drawn, if it is this one.
    fn at(drag: &Option<Drag>, what: &Selected) -> Option<(f64, f64)> {
        drag.as_ref()
            .filter(|d| &d.what == what && d.moved)
            .map(|d| (d.x, d.y))
    }
}

/// The path of a selected thing in the API.
fn path_of(what: &Selected) -> String {
    match what {
        Selected::Token(id) => format!("/tokens/{id}"),
        Selected::Light(id) => format!("/lights/{id}"),
        Selected::Trap(id) => format!("/traps/{id}"),
        Selected::Poi(id) => format!("/pois/{id}"),
    }
}

/// An entry of a right-click menu.
#[derive(Debug, Clone, PartialEq)]
enum MenuAction {
    Select(Selected),
    Patch(String, serde_json::Value),
    Remove(String, &'static str),
    EditPoi(Poi),
}

#[component]
pub fn DmMapPage() -> impl IntoView {
    // The map as a PDF (MIMIR-T-0724).
    let printing = RwSignal::new(false);
    let params = use_params_map();
    let id = Memo::new(move |_| params.get().get("id").unwrap_or_default());

    // A version per part: a bump reads that part again.
    let tokens_v = RwSignal::new(0u32);
    let fog_v = RwSignal::new(0u32);
    let lights_v = RwSignal::new(0u32);
    let markers_v = RwSignal::new(0u32);
    let doors_v = RwSignal::new(0u32);
    let map_v = RwSignal::new(0u32);
    let bump = move |part: MapPart| match part {
        MapPart::Tokens => tokens_v.update(|v| *v += 1),
        MapPart::Fog => fog_v.update(|v| *v += 1),
        MapPart::Lights => lights_v.update(|v| *v += 1),
        MapPart::Markers => markers_v.update(|v| *v += 1),
        MapPart::Doors => doors_v.update(|v| *v += 1),
        MapPart::Map => {
            map_v.update(|v| *v += 1);
            doors_v.update(|v| *v += 1);
            tokens_v.update(|v| *v += 1);
            fog_v.update(|v| *v += 1);
            lights_v.update(|v| *v += 1);
            markers_v.update(|v| *v += 1);
        }
    };

    let map: Res<MapDetail> = LocalResource::new(move || {
        map_v.track();
        api::get(format!("/maps/{}", id.get()))
    });
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
    let door_states: Res<Vec<DoorState>> = LocalResource::new(move || {
        doors_v.track();
        api::get(format!("/maps/{}/doors", id.get()))
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
    // The module's combat (the tracker reads and changes it).
    let combat = RwSignal::new(None::<wire::Combat>);
    let combat_v = RwSignal::new(0u32);
    let live = StoredValue::new_local(None::<Live>);
    Effect::new(move |_| {
        let c = campaign.get();
        if c.is_empty() {
            return;
        }
        let here = id.get_untracked();
        live.set_value(Some(Live::watch(c, move |msg| match msg {
            ServerMsg::MapChanged { map_id, part } if map_id == here => bump(part),
            ServerMsg::CombatChanged { .. } => combat_v.update(|v| *v += 1),
            ServerMsg::Display { display: d } => display.set(d),
            _ => {}
        })));
    });

    // DM-side state.
    let vp = Viewport::new();
    vp.keep_fitted();
    let view = vp.view;
    let fitted = RwSignal::new(false);
    // The map's play state (MIMIR-T-0728): saved with the map, so the
    // player display follows it.
    let los = RwSignal::new(true);
    let reveal = RwSignal::new(false);
    let ambient = RwSignal::new(String::from("bright"));
    // Overlays for the DM: the walls and the sight of monsters and NPCs.
    let overlays = RwSignal::new(false);
    Effect::new(move |_| {
        if let Some(m) = map_now() {
            ambient.set(LightLevel::from_mode(&m.lighting_mode).as_str().to_string());
            los.set(m.line_of_sight);
            reveal.set(m.reveal_all);
        }
    });
    let selected = RwSignal::new(None::<Selected>);
    // The token the tracker or the map selects.
    let selected_token = RwSignal::new(None::<String>);
    Effect::new(move |_| {
        if let Some(t) = selected_token.get() {
            selected.set(Some(Selected::Token(t)));
        }
    });
    Effect::new(move |_| {
        if !matches!(selected.get(), Some(Selected::Token(_))) {
            selected_token.set(None);
        }
    });
    let marks = Memo::new(move |_| combat.with(|c| tracker::map_marks(c.as_ref())));
    let place = RwSignal::new(None::<Place>);
    let place_menu = RwSignal::new(String::new());
    // The placement form, the POI dialog, the right-click menu (and where,
    // in the stage), and the lists of what is on the map (MIMIR-T-0730).
    let form = PlaceForm::new();
    let poi_edit = RwSignal::new(None::<PoiEdit>);
    let menu = RwSignal::new(None::<(Selected, f64, f64)>);
    let lists_open = RwSignal::new(false);
    let stage = NodeRef::<leptos::html::Div>::new();
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
    let patch_map = move |body: serde_json::Value| {
        run(patch::<wire::MapSummary>(
            format!("/maps/{}", id.get_untracked()),
            body,
        ))
    };
    // The light level control has no change event: send it when it differs
    // from the map's.
    Effect::new(move |_| {
        let a = LightLevel::from_mode(&ambient.get());
        let Some(m) = map.get_untracked().and_then(Result::ok) else {
            return;
        };
        if a != LightLevel::from_mode(&m.lighting_mode) {
            let mode = match a {
                LightLevel::Bright => "bright",
                LightLevel::Dim => "dim",
                LightLevel::Darkness => "dark",
            };
            patch_map(json!({ "lighting_mode": mode }));
        }
    });

    let grid_px = move || map_now().map(|m| m.grid_size_px).unwrap_or(70.0);

    // Screen point (client) → map point.
    let to_map = move |cx: f64, cy: f64| vp.to_map(cx, cy);

    let place_at = move |kind: Place, mx: f64, my: f64| {
        let g = grid_px();
        let (gx, gy) = (cell_of(mx, g), cell_of(my, g));
        let map_id = id.get_untracked();
        let cell = json!({ "grid_x": gx, "grid_y": gy });
        let with = |extra: serde_json::Value| {
            let mut b = cell.clone();
            if let (Some(b), Some(e)) = (b.as_object_mut(), extra.as_object()) {
                b.extend(e.clone());
            }
            b
        };
        let name = form.name.get_untracked().trim().to_string();
        let post = move |what: &'static str, body: serde_json::Value| {
            let path = format!("/maps/{}/{what}", id.get_untracked());
            run(Box::pin(async move {
                api::send::<_, serde_json::Value>("POST", &path, &body)
                    .await
                    .map(|_| ())
            }));
        };
        match &kind {
            Place::Pc if name.is_empty() => {
                error.set("Type the name of the player character first.".into());
                return;
            }
            Place::Trap if name.is_empty() => {
                error.set("Type the name of the trap first.".into());
                return;
            }
            Place::CatalogMonster if form.monster.get_untracked().is_none() => {
                error.set("Search for a monster and choose it first.".into());
                return;
            }
            _ => {}
        }
        error.set(String::new());
        match &kind {
            Place::Pc => post("tokens", with(form.token_fields())),
            Place::Monster(m) => {
                let mut b = with(form.token_fields());
                b["monster_id"] = json!(m);
                post("tokens", b);
            }
            Place::Npc(n) => {
                let mut b = with(form.token_fields());
                b["npc_id"] = json!(n);
                post("tokens", b);
            }
            Place::CatalogMonster => {
                // The monster joins the module, then its token goes down.
                let (Some(choice), Some(module)) =
                    (form.monster.get_untracked(), module.get_untracked())
                else {
                    return;
                };
                let mut b = with(form.token_fields());
                run(Box::pin(async move {
                    let added: wire::ModuleMonsterSummary = api::send(
                        "POST",
                        &format!("/modules/{module}/monsters"),
                        &json!({ "monster_name": choice.name, "monster_source": choice.source }),
                    )
                    .await?;
                    b["monster_id"] = json!(added.id);
                    api::send::<_, Token>("POST", &format!("/maps/{map_id}/tokens"), &b).await?;
                    Ok(())
                }));
            }
            Place::Torch => post("lights", with(json!({ "preset": "torch" }))),
            Place::Lantern => post("lights", with(json!({ "preset": "lantern" }))),
            Place::Candle => post("lights", with(json!({ "preset": "candle" }))),
            Place::Trap => post(
                "traps",
                with(json!({ "name": name, "visible": form.visible.get_untracked() })),
            ),
            Place::Poi => poi_edit.set(Some(PoiEdit::new(map_id, gx, gy))),
        }
        if !kind.repeats() {
            place.set(None);
            place_menu.set(String::new());
        }
    };

    // A double click or tap shows the details: a monster's stat block, or
    // the catalog trap of that name, in the detail drawer; a point of
    // interest in its dialog. (Tokens hold no character, so a PC or NPC
    // token has none, as in the desktop.) Taken from the
    // pointer events: a token holds the pointer while it may drag, so the
    // browser's dblclick goes to the scene, and a tablet has none.
    // The details open after the tap's pointerup and click: opened on the
    // pointerdown, the drawer's scrim would take that click and close it.
    let last_tap = StoredValue::new((String::new(), 0.0_f64));
    let pending_detail = StoredValue::new(None::<Detail>);
    let double_tap = move |key: &str| {
        let now = js_sys::Date::now();
        let (k, at) = last_tap.get_value();
        last_tap.set_value((key.to_string(), now));
        is_double_tap(&k, at, key, now)
    };
    let reference = Reference::get();
    let open_monster = move |monster_id: Option<String>| {
        let (Some(r), Some(mid)) = (reference, monster_id) else {
            return;
        };
        let Some(m) = monsters
            .get_untracked()
            .and_then(Result::ok)
            .and_then(|l| l.into_iter().find(|m| m.id == mid))
        else {
            return;
        };
        if let (Some(name), Some(source)) = (m.monster_name, m.monster_source) {
            r.open(Target::new("monsters", &source, &name));
        }
    };
    let open_trap = move |name: String| {
        let Some(r) = reference else { return };
        spawn_local(async move {
            let path = format!(
                "/catalog/search/traps?limit=20&search={}",
                String::from(js_sys::encode_uri_component(&name))
            );
            if let Ok(page) = api::get::<wire::CatalogPage>(path).await {
                if let Some(row) = page
                    .rows
                    .iter()
                    .find(|r| r.name.eq_ignore_ascii_case(&name))
                {
                    r.open(Target::new("traps", &row.source, &row.name));
                }
            }
        });
    };

    // ---- Pointer handling --------------------------------------------------
    let on_down = move |ev: web_sys::PointerEvent| {
        menu.set(None);
        vp.down(&ev);
    };
    // Start dragging a thing on the map: the scene holds the pointer.
    let start_drag = move |what: Selected, ev: &web_sys::PointerEvent| {
        // Only the main button drags; the right one opens a menu (a drag
        // would hold the pointer and take its contextmenu event).
        if ev.button() != 0 {
            return;
        }
        if let Some(el) = scene.get_untracked() {
            let _ = el.set_pointer_capture(ev.pointer_id());
        }
        let (mx, my) = to_map(f64::from(ev.client_x()), f64::from(ev.client_y()));
        menu.set(None);
        drag.set(Some(Drag {
            what,
            x: mx,
            y: my,
            moved: false,
        }));
    };
    // A right-click menu at the pointer (in the stage).
    let open_menu = move |what: Selected, ev: &web_sys::MouseEvent| {
        ev.prevent_default();
        ev.stop_propagation();
        let Some(el) = stage.get_untracked() else {
            return;
        };
        let r = el.get_bounding_client_rect();
        menu.set(Some((
            what,
            f64::from(ev.client_x()) - r.left(),
            f64::from(ev.client_y()) - r.top(),
        )));
    };
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
        if let Some(detail) = pending_detail.try_update_value(Option::take).flatten() {
            set_timeout(
                move || match detail {
                    Detail::Monster(m) => open_monster(m),
                    Detail::Trap(name) => open_trap(name),
                    Detail::Poi(edit) => poi_edit.set(Some(edit)),
                },
                std::time::Duration::ZERO,
            );
        }
        if let Some(d) = drag.get_untracked() {
            drag.set(None);
            if d.moved {
                let g = grid_px();
                let (gx, gy) = (cell_of(d.x, g), cell_of(d.y, g));
                run(patch::<serde_json::Value>(
                    path_of(&d.what),
                    json!({"grid_x": gx, "grid_y": gy}),
                ));
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
        if let Some(m) = map.get_untracked().and_then(Result::ok) {
            vp.fit(f64::from(m.width_px), f64::from(m.height_px));
        }
    };

    // Full screen: the whole map page (MIMIR-T-0729).
    let root = NodeRef::<leptos::html::Div>::new();
    let full_screen = move || {
        if let Some(el) = root.get_untracked() {
            let _ = el.request_fullscreen();
        }
    };

    // The keys of the desktop: + = - 0 H D Esc.
    let key_token = move |change: fn(&Token) -> serde_json::Value| {
        let Some(Selected::Token(tid)) = selected.get_untracked() else {
            return;
        };
        let Some(t) = tokens
            .get_untracked()
            .and_then(Result::ok)
            .and_then(|l| l.into_iter().find(|t| t.id == tid))
        else {
            return;
        };
        run(patch::<Token>(format!("/tokens/{tid}"), change(&t)));
    };
    let keys = window_event_listener(leptos::ev::keydown, move |ev| {
        if !page_key(&ev) {
            return;
        }
        let Some(k) = map_key(&ev.key()) else { return };
        ev.prevent_default();
        match k {
            MapKey::ZoomIn => vp.zoom(1.25),
            MapKey::ZoomOut => vp.zoom(0.8),
            MapKey::Fit => fit(),
            MapKey::ToggleHidden => key_token(|t| json!({ "hidden": t.visible_to_players })),
            MapKey::ToggleDead => key_token(|t| json!({ "dead": !t.dead })),
            MapKey::Deselect => {
                menu.set(None);
                poi_edit.set(None);
                selected.set(None);
                place.set(None);
                place_menu.set(String::new());
            }
        }
    });
    on_cleanup(move || keys.remove());

    // Add PCs: a token for each PC of the campaign with none on the map,
    // in a row from the centre of the view.
    let add_pcs = move || {
        let c = campaign.get_untracked();
        let map_id = id.get_untracked();
        let Some(m) = map.get_untracked().and_then(Result::ok) else {
            return;
        };
        let have = tokens
            .get_untracked()
            .and_then(Result::ok)
            .unwrap_or_default();
        let g = m.grid_size_px;
        let start = scene
            .get_untracked()
            .map(|el| {
                let r = el.get_bounding_client_rect();
                let (mx, my) = vp.to_map(r.left() + r.width() / 2.0, r.top() + r.height() / 2.0);
                (cell_of(mx, g), cell_of(my, g))
            })
            .unwrap_or((0, 0));
        run(Box::pin(async move {
            let pcs: Vec<wire::CharacterSummary> = api::get(format!("/campaigns/{c}/pcs")).await?;
            let names: Vec<String> = pcs.into_iter().map(|p| p.name).collect();
            let missing = missing_pcs(&names, &have);
            let cells = row_of_cells(start, missing.len(), m.columns, m.rows);
            for (name, (gx, gy)) in missing.into_iter().zip(cells) {
                api::send::<_, Token>(
                    "POST",
                    &format!("/maps/{map_id}/tokens"),
                    &json!({ "grid_x": gx, "grid_y": gy, "label": name }),
                )
                .await?;
            }
            Ok(())
        }));
    };

    // ---- Derived scene data ------------------------------------------------
    let token_list = move || tokens.get().and_then(Result::ok).unwrap_or_default();
    let light_list = move || lights.get().and_then(Result::ok).unwrap_or_default();
    let door_list = move || door_states.get().and_then(Result::ok).unwrap_or_default();
    // Walls, doors and the file's lights: worked out when they change.
    let walls_and_lights = Memo::new(move |_| {
        let g = geometry.get().and_then(Result::ok)?;
        let m = map_now()?;
        let states = door_list();
        let blocking = vision::blocking_for(Some(&g), &states, m.line_of_sight);
        let mut doors = vision::doors_px(&g);
        vision::apply_door_states(&mut doors, &states);
        let lit = vision::file_lit(&g, &blocking, f64::from(m.width_px), f64::from(m.height_px));
        let walls = vision::walls_px(&g);
        Some((blocking, lit, doors, walls))
    });
    let doors = move || {
        walls_and_lights
            .with(|w| w.as_ref().map(|w| w.2.clone()))
            .unwrap_or_default()
    };

    // What the players do not see (DM view: a shade over it). The same
    // sight as the player display's.
    let fog_shapes = Memo::new(move |_| {
        let m = map_now()?;
        let fog = fog.get().and_then(Result::ok)?;
        let (blocking, lit, _, _) = walls_and_lights.get().unwrap_or_default();
        let toks = token_list();
        let s = sight_for(&m, &toks, &light_list(), &fog, &blocking, &lit);
        if s.cover <= 0.0 {
            return None;
        }
        Some((f64::from(m.width_px), f64::from(m.height_px), s.holes))
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
                    show_initiative: display.get_untracked().show_initiative,
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
            ("candle".into(), "Candle".into()),
            ("trap".into(), "Trap".into()),
            ("poi".into(), "Point of interest".into()),
        ];
        if module.get().is_some() {
            o.push(("catalog-monster".into(), "Monster from the catalog…".into()));
        }
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
    let on_place = Callback::new(move |v: String| {
        let kind = Place::parse(&v);
        form.reset(kind.as_ref());
        place.set(kind);
    });

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
                    <Switch
                        label="Line of sight"
                        checked=los
                        on_change=Callback::new(move |on: bool| patch_map(json!({ "line_of_sight": on })))
                    />
                    <Switch
                        label="Reveal map"
                        checked=reveal
                        on_change=Callback::new(move |on: bool| patch_map(json!({ "reveal_all": on })))
                    />
                    <Switch label="Overlays" checked=overlays />
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
                    <Button variant="light" on_click=Callback::new(move |_| add_pcs())>"Add PCs"</Button>
                    <Button variant="light" on_click=Callback::new(move |_| {
                        selected.set(None);
                        lists_open.update(|o| *o = !*o);
                    })>"On the map"</Button>
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
                    <Button variant="subtle" on_click=Callback::new(move |_| printing.set(true))>"PDF…"</Button>
                    <crate::admin::pdf::PdfDialog
                        open=printing
                        title="PDF of the map"
                        path=Signal::derive(move || format!("/maps/{}/pdf", id.get()))
                        options=crate::admin::pdf::MAP
                        file="map.pdf"
                        squares=Signal::derive(move || map_now().map(|m| (m.columns, m.rows)))
                    />
                    <ActionIcon title="Zoom in" on_click=Callback::new(move |_| zoom_button(1.25))>"+"</ActionIcon>
                    <ActionIcon title="Zoom out" on_click=Callback::new(move |_| zoom_button(0.8))>"−"</ActionIcon>
                    <ActionIcon title="Fit the map" on_click=Callback::new(move |_| fit())>"⤢"</ActionIcon>
                    <ActionIcon title="Full screen" on_click=Callback::new(move |_| full_screen())>"⛶"</ActionIcon>
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
                        // The map file's lights: their glow stops at the walls.
                        {move || {
                            walls_and_lights.with(|w| {
                                w.as_ref().map(|(_, lit, _, _)| {
                                    lit.iter()
                                        .map(|l| view! {
                                            <path class="mimir-map__file-light" d=vision::svg_path(&l.area) style=format!("fill:{}", l.color) opacity=0.12 * l.intensity.clamp(0.2, 2.0) />
                                        })
                                        .collect_view()
                                })
                            })
                        }}
                        // Lights: placed lights (the DM sees unlit ones faint).
                        {move || {
                            let dragging = drag.get();
                            light_list()
                                .into_iter()
                                .map(|mut l| {
                                    if let Some((dx, dy)) = Drag::at(&dragging, &Selected::Light(l.id.clone())) {
                                        (l.x, l.y) = (dx, dy);
                                    }
                                    let color = l.color.clone().unwrap_or_else(|| "#ffcc66".into());
                                    let dim = vision::feet_to_px(f64::from(l.dim_radius_ft), g);
                                    let bright = vision::feet_to_px(f64::from(l.bright_radius_ft), g);
                                    let sel = Selected::Light(l.id.clone());
                                    let sel2 = sel.clone();
                                    let toggle_path = format!("/lights/{}", l.id);
                                    let lit = l.active;
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
                                                    start_drag(sel.clone(), &ev);
                                                }
                                                // Right-click turns the light on or off (the desktop's).
                                                on:contextmenu=move |ev: web_sys::MouseEvent| {
                                                    ev.prevent_default();
                                                    ev.stop_propagation();
                                                    run(patch::<Light>(toggle_path.clone(), json!({ "active": !lit })));
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
                        // Overlays: the sight of monsters and NPCs, and the walls.
                        {move || {
                            if !overlays.get() {
                                return None;
                            }
                            let m = map_now()?;
                            let g = m.grid_size_px;
                            let toks = token_list();
                            let mut zones = vision::light_zones(&light_list(), &toks, g);
                            zones.extend(
                                walls_and_lights
                                    .with(|w| w.as_ref().map(|w| w.1.iter().map(|l| l.zone.clone()).collect::<Vec<_>>()))
                                    .unwrap_or_default(),
                            );
                            let level = LightLevel::from_mode(&ambient.get());
                            Some(
                                toks.iter()
                                    .filter(|t| t.token_type != "pc")
                                    .map(|t| (t.color.clone(), vision::token_vision(t, &zones, level, g)))
                                    .filter(|(_, v)| v.radius_px > 0.0 && v.radius_px < vision::UNLIMITED)
                                    .map(|(c, v)| view! {
                                        <circle class="mimir-map__ring" cx=v.x cy=v.y r=v.radius_px style=c.map(|c| format!("stroke:{c}")) />
                                    })
                                    .collect_view(),
                            )
                        }}
                        {move || {
                            overlays.get().then(|| {
                                let d: String = walls_and_lights
                                    .with(|w| w.as_ref().map(|w| w.3.clone()))
                                    .unwrap_or_default()
                                    .iter()
                                    .map(|w| format!("M{} {}L{} {}", w.p1.x, w.p1.y, w.p2.x, w.p2.y))
                                    .collect();
                                view! { <path class="mimir-map__walls" d=d /> }
                            })
                        }}
                        // Doors: a tap opens or closes one; the players see it.
                        {move || {
                            doors()
                                .into_iter()
                                .enumerate()
                                .map(|(i, d)| {
                                    let closed = d.closed;
                                    view! {
                                        <line
                                            class="mimir-map__door"
                                            class:mimir-map__door--open=!d.closed
                                            x1=d.wall.p1.x
                                            y1=d.wall.p1.y
                                            x2=d.wall.p2.x
                                            y2=d.wall.p2.y
                                            aria-label=if closed { "Closed door" } else { "Open door" }
                                            on:pointerdown=move |ev: web_sys::PointerEvent| {
                                                ev.stop_propagation();
                                                let path = format!("/maps/{}/doors/{i}", id.get_untracked());
                                                run(Box::pin(async move {
                                                    api::send::<_, DoorState>("PUT", &path, &json!({ "closed": !closed }))
                                                        .await
                                                        .map(|_| ())
                                                }));
                                            }
                                        />
                                    }
                                })
                                .collect_view()
                        }}
                        // Markers.
                        {move || {
                            let dragging = drag.get();
                            let traps = traps.get().and_then(Result::ok).unwrap_or_default();
                            traps
                                .into_iter()
                                .map(|t| {
                                    let sel = Selected::Trap(t.id.clone());
                                    let (x, y) = Drag::at(&dragging, &sel)
                                        .unwrap_or(((f64::from(t.grid_x) + 0.5) * g, (f64::from(t.grid_y) + 0.5) * g));
                                    let s = g * 0.3;
                                    let sel2 = sel.clone();
                                    let sel3 = sel.clone();
                                    let trap_name = t.name.clone();
                                    view! {
                                        <g
                                            class="mimir-map__marker mimir-map__marker--trap"
                                            class:mimir-map__marker--hidden=!t.visible
                                            class:mimir-map__marker--selected=move || selected.get() == Some(sel2.clone())
                                            on:pointerdown=move |ev: web_sys::PointerEvent| {
                                                ev.stop_propagation();
                                                if double_tap(&trap_name) {
                                                    pending_detail.set_value(Some(Detail::Trap(trap_name.clone())));
                                                }
                                                selected.set(Some(sel.clone()));
                                                start_drag(sel.clone(), &ev);
                                            }
                                            on:contextmenu=move |ev: web_sys::MouseEvent| open_menu(sel3.clone(), &ev)
                                        >
                                            <path d=format!("M{x} {}L{} {}L{} {}Z", y - s, x + s, y + s, x - s, y + s) />
                                            <text x=x y=y + s + g * 0.3 text-anchor="middle">{t.name}</text>
                                        </g>
                                    }
                                })
                                .collect_view()
                        }}
                        {move || {
                            let dragging = drag.get();
                            let pois = pois.get().and_then(Result::ok).unwrap_or_default();
                            pois
                                .into_iter()
                                .map(|p| {
                                    let sel = Selected::Poi(p.id.clone());
                                    let (x, y) = Drag::at(&dragging, &sel)
                                        .unwrap_or(((f64::from(p.grid_x) + 0.5) * g, (f64::from(p.grid_y) + 0.5) * g));
                                    let sel2 = sel.clone();
                                    let sel3 = sel.clone();
                                    let emoji = poi_emoji(&p.icon);
                                    let poi_label = p.name.clone();
                                    let poi_key = p.id.clone();
                                    let poi_details = PoiEdit::of(&p);
                                    let color = p.color.clone().unwrap_or_else(|| token::ICE.to_string());
                                    view! {
                                        <g
                                            class="mimir-map__marker mimir-map__marker--poi"
                                            class:mimir-map__marker--hidden=!p.visible
                                            class:mimir-map__marker--selected=move || selected.get() == Some(sel2.clone())
                                            aria-label=poi_label
                                            on:pointerdown=move |ev: web_sys::PointerEvent| {
                                                ev.stop_propagation();
                                                if double_tap(&poi_key) {
                                                    pending_detail.set_value(Some(Detail::Poi(poi_details.clone())));
                                                }
                                                selected.set(Some(sel.clone()));
                                                start_drag(sel.clone(), &ev);
                                            }
                                            on:contextmenu=move |ev: web_sys::MouseEvent| open_menu(sel3.clone(), &ev)
                                        >
                                            <circle cx=x cy=y r=g * 0.25 style=format!("fill:{color}") />
                                            <text class="mimir-map__poi-icon" x=x y=y text-anchor="middle" dominant-baseline="central" font-size=g * 0.28>{emoji}</text>
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
                                    if let Some((dx, dy)) = Drag::at(&dragging, &Selected::Token(t.id.clone())) {
                                        x = dx - side / 2.0;
                                        y = dy - side / 2.0;
                                    }
                                    let art = token_art.with(|m| m.get(&t.id).cloned().flatten());
                                    let (turn_token, down) = marks.get();
                                    let is_dead = t.dead || down.contains(&t.id);
                                    let is_turn = turn_token.as_deref() == Some(t.id.as_str());
                                    let tid = t.id.clone();
                                    let monster_id = t.monster_id.clone();
                                    let sel = Selected::Token(t.id.clone());
                                    let sel2 = sel.clone();
                                    let sel3 = sel.clone();
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
                                            class:mimir-map__token--turn=is_turn
                                            class:mimir-map__token--selected=move || selected.get() == Some(sel2.clone())
                                            aria-label=t.name.clone()
                                            on:pointerdown=move |ev: web_sys::PointerEvent| {
                                                ev.stop_propagation();
                                                if double_tap(&tid) {
                                                    pending_detail.set_value(Some(Detail::Monster(monster_id.clone())));
                                                }
                                                selected.set(Some(sel.clone()));
                                                selected_token.set(Some(tid.clone()));
                                                start_drag(sel.clone(), &ev);
                                            }
                                            on:contextmenu=move |ev: web_sys::MouseEvent| open_menu(sel3.clone(), &ev)
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
                token_panel(t, run).into_any()
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
                poi_panel(p, run, poi_edit).into_any()
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

    // The lists of what is on the map, when nothing is selected.
    let lists = move || {
        (lists_open.get() && selected.with(Option::is_none)).then(|| {
            let on_select = Callback::new(move |(kind, id): (String, String)| {
                selected.set(match kind.as_str() {
                    "token" => Some(Selected::Token(id)),
                    "light" => Some(Selected::Light(id)),
                    "trap" => Some(Selected::Trap(id)),
                    _ => Some(Selected::Poi(id)),
                })
            });
            view! {
                <aside class="mimir-map__panel" aria-label="On the map">
                    <Card>
                        <Stack gap="sm">
                            <PlacedLists
                                tokens=Signal::derive(token_list)
                                lights=Signal::derive(light_list)
                                traps=Signal::derive(move || traps.get().and_then(Result::ok).unwrap_or_default())
                                pois=Signal::derive(move || pois.get().and_then(Result::ok).unwrap_or_default())
                                on_select=on_select
                                error=error
                            />
                            <Button variant="subtle" on_click=Callback::new(move |_| lists_open.set(false))>"Close"</Button>
                        </Stack>
                    </Card>
                </aside>
            }
        })
    };

    // The right-click menu of a token, a trap or a POI.
    let context_menu = move || {
        let (what, x, y) = menu.get()?;
        let items: Vec<(&'static str, MenuAction)> = match &what {
            Selected::Token(id) => {
                let t = token_list().into_iter().find(|t| &t.id == id)?;
                vec![
                    ("Edit", MenuAction::Select(what.clone())),
                    (
                        if t.visible_to_players { "Hide" } else { "Show" },
                        MenuAction::Patch(
                            path_of(&what),
                            json!({ "hidden": t.visible_to_players }),
                        ),
                    ),
                    (
                        "Remove",
                        MenuAction::Remove(path_of(&what), "Remove this token from the map?"),
                    ),
                ]
            }
            Selected::Trap(id) => {
                let t = traps
                    .get()
                    .and_then(Result::ok)?
                    .into_iter()
                    .find(|t| &t.id == id)?;
                vec![
                    (
                        if t.visible { "Hide" } else { "Show" },
                        MenuAction::Patch(path_of(&what), json!({ "visible": !t.visible })),
                    ),
                    (
                        if t.triggered { "Reset" } else { "Trigger" },
                        MenuAction::Patch(path_of(&what), json!({ "triggered": !t.triggered })),
                    ),
                    (
                        "Remove",
                        MenuAction::Remove(path_of(&what), "Remove this trap?"),
                    ),
                ]
            }
            Selected::Poi(id) => {
                let p = pois
                    .get()
                    .and_then(Result::ok)?
                    .into_iter()
                    .find(|p| &p.id == id)?;
                vec![
                    ("Edit", MenuAction::EditPoi(p.clone())),
                    (
                        if p.visible { "Hide" } else { "Show" },
                        MenuAction::Patch(path_of(&what), json!({ "visible": !p.visible })),
                    ),
                    (
                        "Remove",
                        MenuAction::Remove(path_of(&what), "Remove this point of interest?"),
                    ),
                ]
            }
            Selected::Light(_) => return None,
        };
        Some(view! {
            <div class="mimir-map__menu" role="menu" style=format!("left:{x}px;top:{y}px")>
                {items.into_iter().map(|(label, action)| view! {
                    <button type="button" role="menuitem" class="mimir-map__menu-item" on:click=move |_| {
                        menu.set(None);
                        match action.clone() {
                            MenuAction::Select(w) => selected.set(Some(w)),
                            MenuAction::Patch(path, body) => run(patch::<serde_json::Value>(path, body)),
                            MenuAction::Remove(path, question) => {
                                if confirm(question) {
                                    run(remove(path));
                                }
                            }
                            MenuAction::EditPoi(p) => poi_edit.set(Some(PoiEdit::of(&p))),
                        }
                    }>{label}</button>
                }).collect_view()}
            </div>
        })
    };

    view! {
        <div class="mimir-map" node_ref=root>
            {header}
            <PoiDialog edit=poi_edit />
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
            <div class="mimir-map__body">
                <div class="mimir-map__stage" node_ref=stage>
                    {move || match map.get() {
                        Some(Err(e)) => view! { <ErrorState error=e /> }.into_any(),
                        _ => scene_view(),
                    }}
                    {move || {
                        place
                            .get()
                            .filter(|k| k.is_token() || matches!(k, Place::Trap | Place::Poi))
                            .map(|_| view! {
                                <aside class="mimir-map__placing" aria-label="Placement">
                                    <Card>
                                        <PlaceOptions kind=Signal::derive(move || place.get()) form=form campaign_id=campaign />
                                    </Card>
                                </aside>
                            })
                    }}
                    {panel}
                    {lists}
                    {context_menu}
                </div>
                <CombatTracker
                    module_id=module
                    campaign_id=campaign
                    map_id=id
                    combat=combat
                    combat_v=combat_v
                    display=display
                    selected_token=selected_token
                />
            </div>
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

fn token_panel(t: Token, run: impl Fn(Job) + Copy + Send + Sync + 'static) -> impl IntoView {
    let path = format!("/tokens/{}", t.id);
    let visible = RwSignal::new(t.visible_to_players);
    let is_dead = RwSignal::new(t.dead);
    let bright = RwSignal::new(feet(t.vision_bright_ft));
    let dim = RwSignal::new(feet(t.vision_dim_ft));
    let dark = RwSignal::new(t.vision_dark_ft.to_string());
    let light = RwSignal::new(t.light_radius_ft.to_string());
    let preset = RwSignal::new(preset_of(&t).unwrap_or_default().to_string());
    let (p1, p2, p3, p4, p5) = (path.clone(), path.clone(), path.clone(), path.clone(), path);
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
            on_change=Callback::new(move |on: bool| run(patch::<Token>(p4.clone(), json!({ "dead": on }))))
        />
        <SectionLabel label="Vision (feet; empty: no limit)" />
        <Select
            aria_label="Vision preset"
            placeholder="Preset…"
            options=VISION_PRESETS.iter().map(|p| p.0.to_string()).collect()
            value=preset
            on_change=Callback::new(move |label: String| {
                let Some(p) = VISION_PRESETS.iter().find(|p| p.0 == label) else { return };
                bright.set(feet(p.1));
                dim.set(feet(p.2));
                dark.set(p.3.to_string());
                light.set(p.4.to_string());
                run(patch::<Token>(p5.clone(), json!({
                    "vision_bright_ft": p.1,
                    "vision_dim_ft": p.2,
                    "vision_dark_ft": p.3,
                    "light_radius_ft": p.4,
                })))
            })
        />
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

fn poi_panel(
    p: Poi,
    run: impl Fn(Job) + Copy + Send + Sync + 'static,
    poi_edit: RwSignal<Option<PoiEdit>>,
) -> impl IntoView {
    let editing = p.clone();
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
        <Button variant="light" on_click=Callback::new(move |_| poi_edit.set(Some(PoiEdit::of(&editing))))>"Edit"</Button>
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

    fn tok(name: &str, kind: &str) -> Token {
        Token {
            id: name.into(),
            map_id: "m".into(),
            name: name.into(),
            token_type: kind.into(),
            size: "medium".into(),
            grid_x: 0,
            grid_y: 0,
            x: 35.0,
            y: 35.0,
            visible_to_players: true,
            color: None,
            monster_id: None,
            npc_id: None,
            vision_bright_ft: None,
            vision_dim_ft: None,
            vision_dark_ft: 0,
            light_radius_ft: 0,
            dead: false,
        }
    }

    #[test]
    fn vision_presets_match_tokens() {
        let mut t = tok("Robin", "pc");
        assert_eq!(preset_of(&t), Some("Human"));
        t.vision_dark_ft = 60;
        assert_eq!(preset_of(&t), Some("Darkvision 60 ft"));
        t.vision_bright_ft = Some(30);
        assert_eq!(preset_of(&t), None, "custom");
        assert_eq!(VISION_PRESETS.len(), 8);
        let blind = VISION_PRESETS
            .iter()
            .find(|p| p.0 == "Blindsight 30 ft")
            .unwrap();
        assert_eq!((blind.1, blind.2, blind.3), (Some(30), Some(30), 30));
    }

    #[test]
    fn the_map_keys() {
        assert_eq!(map_key("+"), Some(MapKey::ZoomIn));
        assert_eq!(map_key("="), Some(MapKey::ZoomIn));
        assert_eq!(map_key("-"), Some(MapKey::ZoomOut));
        assert_eq!(map_key("0"), Some(MapKey::Fit));
        assert_eq!(map_key("H"), Some(MapKey::ToggleHidden));
        assert_eq!(map_key("d"), Some(MapKey::ToggleDead));
        assert_eq!(map_key("Escape"), Some(MapKey::Deselect));
        assert_eq!(map_key("x"), None);
    }

    #[test]
    fn pcs_to_add_and_where() {
        let tokens = vec![tok("robin", "pc"), tok("Sam", "monster")];
        let pcs = vec!["Robin".to_string(), "Sam".to_string(), "Jamie".to_string()];
        assert_eq!(missing_pcs(&pcs, &tokens), vec!["Sam", "Jamie"]);
        assert_eq!(
            row_of_cells((2, 3), 3, 10.0, 10.0),
            vec![(2, 3), (3, 3), (4, 3)]
        );
        assert_eq!(
            row_of_cells((8, 3), 3, 10.0, 10.0),
            vec![(8, 3), (9, 3), (0, 4)]
        );
        assert_eq!(
            row_of_cells((-5, 99), 1, 10.0, 10.0),
            vec![(0, 9)],
            "kept on the map"
        );
    }

    #[test]
    fn double_taps() {
        assert!(is_double_tap("a", 1000.0, "a", 1300.0));
        assert!(!is_double_tap("a", 1000.0, "a", 1500.0), "too slow");
        assert!(!is_double_tap("a", 1000.0, "b", 1100.0), "another thing");
    }
}
