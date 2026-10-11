//! Maps and what is on them (MIMIR-T-0711): the map, its geometry and
//! image, tokens, fog, lights, traps and points of interest, and the player
//! view. Handlers call MapService, MapStateService and TokenService.

use std::path::{Path as FsPath, PathBuf};

use axum::extract::{Path, State};
use axum::http::{header, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::{Extension, Json};
use diesel::{Connection, SqliteConnection};
use mimir_core::models::campaign::{LightSource, Map, MapPoi, MapTrap};
use mimir_core::services::{
    CombatService, CreateLightInput, CreatePoiInput, CreateTokenInput, CreateTrapInput, MapService,
    MapStateService, ServiceError, TokenResponse, TokenService, UpdateLightInput, UpdatePoiInput,
    UpdateTokenInput, UpdateTrapInput,
};
use mimir_wire::{self as wire, MapPart};

use super::scope;
use crate::auth::Caller;
use crate::error::{ApiError, JsonBody};
use crate::live::{Change, Hub};
use crate::state::AppState;

type ApiResult<T> = Result<Json<T>, ApiError>;

/// Pixels per grid when a map has no resolution data.
const DEFAULT_PPG: f64 = 70.0;

fn require_map(conn: &mut SqliteConnection, app_dir: &FsPath, id: &str) -> Result<Map, ApiError> {
    MapService::new(conn, app_dir)
        .get(id)?
        .ok_or_else(|| ServiceError::not_found("Map", id).into())
}

fn detail(conn: &mut SqliteConnection, app_dir: &FsPath, map: Map) -> wire::MapDetail {
    let meta = MapService::new(conn, app_dir).ensure_resolution_meta(&map);
    let (ppg, columns, rows) = meta
        .map(|m| (m.pixels_per_grid, m.map_size_x, m.map_size_y))
        .unwrap_or((DEFAULT_PPG, 25.0, 25.0));
    wire::MapDetail {
        id: map.id,
        campaign_id: map.campaign_id,
        module_id: map.module_id,
        name: map.name,
        description: map.description,
        sort_order: map.sort_order,
        lighting_mode: map.lighting_mode,
        fog_enabled: map.fog_enabled != 0,
        grid_size_px: ppg,
        columns,
        rows,
        width_px: (ppg * columns).round() as i32,
        height_px: (ppg * rows).round() as i32,
    }
}

fn centre(grid: i32, ppg: f64) -> f64 {
    (f64::from(grid) + 0.5) * ppg
}

fn token(t: TokenResponse) -> wire::Token {
    wire::Token {
        id: t.id,
        map_id: t.map_id,
        name: t.name,
        token_type: t.token_type,
        size: t.size,
        grid_x: t.grid_x,
        grid_y: t.grid_y,
        x: t.x,
        y: t.y,
        visible_to_players: t.visible_to_players,
        color: t.color,
        monster_id: t.monster_id,
        npc_id: t.character_id,
        vision_bright_ft: t.vision_bright_ft,
        vision_dim_ft: t.vision_dim_ft,
        vision_dark_ft: t.vision_dark_ft,
        light_radius_ft: t.light_radius_ft,
    }
}

fn light(l: LightSource, ppg: f64) -> wire::Light {
    wire::Light {
        x: centre(l.grid_x, ppg),
        y: centre(l.grid_y, ppg),
        id: l.id,
        map_id: l.map_id,
        name: l.name,
        grid_x: l.grid_x,
        grid_y: l.grid_y,
        bright_radius_ft: l.bright_radius,
        dim_radius_ft: l.dim_radius,
        color: l.color,
        active: l.active != 0,
    }
}

fn trap(t: MapTrap) -> wire::Trap {
    wire::Trap {
        id: t.id,
        map_id: t.map_id,
        grid_x: t.grid_x,
        grid_y: t.grid_y,
        name: t.name,
        description: t.description,
        trigger_description: t.trigger_description,
        effect_description: t.effect_description,
        dc: t.dc,
        triggered: t.triggered != 0,
        visible: t.visible != 0,
    }
}

fn poi(p: MapPoi) -> wire::Poi {
    wire::Poi {
        id: p.id,
        map_id: p.map_id,
        grid_x: p.grid_x,
        grid_y: p.grid_y,
        name: p.name,
        description: p.description,
        icon: p.icon,
        color: p.color,
        visible: p.visible != 0,
    }
}

fn fog(conn: &mut SqliteConnection, map_id: &str) -> Result<wire::Fog, ApiError> {
    let state = MapStateService::new(conn).fog_state(map_id)?;
    Ok(wire::Fog {
        enabled: state.fog_enabled,
        revealed: state
            .revealed_areas
            .into_iter()
            .map(|a| wire::FogArea {
                id: a.id,
                x: a.x,
                y: a.y,
                width: a.width,
                height: a.height,
            })
            .collect(),
    })
}

fn ppg_of(conn: &mut SqliteConnection, app_dir: &FsPath, map_id: &str) -> f64 {
    let mut maps = MapService::new(conn, app_dir);
    match maps.get(map_id) {
        Ok(Some(map)) => maps
            .ensure_resolution_meta(&map)
            .map(|m| m.pixels_per_grid)
            .unwrap_or(DEFAULT_PPG),
        _ => DEFAULT_PPG,
    }
}

/// The campaign of a map.
pub fn campaign_of(
    conn: &mut SqliteConnection,
    app_dir: &FsPath,
    map_id: &str,
) -> Result<String, ApiError> {
    Ok(require_map(conn, app_dir, map_id)?.campaign_id)
}

/// Tell the live sockets that a part of a map changed.
fn notify(live: &Hub, conn: &mut SqliteConnection, app_dir: &FsPath, map_id: &str, part: MapPart) {
    if let Ok(Some(map)) = MapService::new(conn, app_dir).get(map_id) {
        live.publish(Change::Map {
            campaign_id: map.campaign_id,
            map_id: map_id.to_string(),
            part,
        });
    }
}

/// An image file as a response: its bytes, its type, and a cache header
/// (`private`: the API needs a token).
fn image_response(path: PathBuf) -> Result<Response, ApiError> {
    let bytes = std::fs::read(&path).map_err(ServiceError::Io)?;
    let mime = mime_guess::from_path(&path).first_or_octet_stream();
    Ok((
        [
            (header::CONTENT_TYPE, mime.to_string()),
            (header::CACHE_CONTROL, "private, max-age=3600".to_string()),
        ],
        bytes,
    )
        .into_response())
}

// ---- Maps ----------------------------------------------------------------

/// `GET /maps/{id}`
pub async fn get_map(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> ApiResult<wire::MapDetail> {
    let app_dir = state.config.data_dir.clone();
    state
        .with_db(move |conn| {
            let map = require_map(conn, &app_dir, &id)?;
            Ok(detail(conn, &app_dir, map))
        })
        .await
        .map(Json)
}

/// `GET /maps/{id}/geometry`: walls, doors and lights from the map file.
pub async fn get_geometry(
    State(state): State<AppState>,
    Extension(caller): Extension<Caller>,
    Path(id): Path<String>,
) -> ApiResult<wire::MapGeometry> {
    let live = state.live.clone();
    let app_dir = state.config.data_dir.clone();
    state
        .with_db(move |conn| {
            scope::shown_map(&caller, &live, conn, &app_dir, &id)?;
            let map = require_map(conn, &app_dir, &id)?;
            let g = MapService::new(conn, &app_dir).geometry(&map)?;
            let point = |p: mimir_core::services::GridPoint| wire::GridPoint { x: p.x, y: p.y };
            Ok(wire::MapGeometry {
                grid_size_px: g.pixels_per_grid,
                columns: g.columns,
                rows: g.rows,
                walls: g
                    .walls
                    .into_iter()
                    .map(|w| w.into_iter().map(point).collect())
                    .collect(),
                portals: g
                    .portals
                    .into_iter()
                    .map(|p| wire::Portal {
                        position: point(p.position),
                        bounds: [point(p.bounds[0]), point(p.bounds[1])],
                        rotation: p.rotation,
                        closed: p.closed,
                        freestanding: p.freestanding,
                    })
                    .collect(),
                lights: g
                    .lights
                    .into_iter()
                    .map(|l| wire::MapFileLight {
                        position: point(l.position),
                        range: l.range,
                        intensity: l.intensity,
                        color: l.color,
                        shadows: l.shadows,
                    })
                    .collect(),
                ambient_light: g.ambient_light,
                baked_lighting: g.baked_lighting,
            })
        })
        .await
        .map(Json)
}

/// `GET /maps/{id}/image`: the map image (JPEG, or PNG for old maps).
pub async fn get_map_image(
    State(state): State<AppState>,
    Extension(caller): Extension<Caller>,
    Path(id): Path<String>,
) -> Result<Response, ApiError> {
    let live = state.live.clone();
    let app_dir = state.config.data_dir.clone();
    state
        .with_db(move |conn| {
            scope::shown_map(&caller, &live, conn, &app_dir, &id)?;
            let map = require_map(conn, &app_dir, &id)?;
            let path = MapService::new(conn, &app_dir)
                .get_map_image_path(&map)?
                .ok_or_else(|| ApiError::not_found(format!("no image for map {id}")))?;
            image_response(path)
        })
        .await
}

/// The player view of a map: what the player display may show.
/// With `show_initiative`, it holds the turn order of the module's combat.
pub fn build_player_view(
    conn: &mut SqliteConnection,
    app_dir: &FsPath,
    map_id: &str,
    show_initiative: bool,
) -> Result<wire::PlayerView, ApiError> {
    let map = require_map(conn, app_dir, map_id)?;
    let map = detail(conn, app_dir, map);
    let ppg = map.grid_size_px;
    let fog = fog(conn, map_id)?;
    let tokens: Vec<wire::PlayerToken> = TokenService::new(conn, app_dir)
        .list_visible(map_id)?
        .into_iter()
        .map(|t| wire::PlayerToken {
            id: t.id,
            name: t.name,
            token_type: t.token_type,
            size: t.size,
            grid_x: t.grid_x,
            grid_y: t.grid_y,
            x: t.x,
            y: t.y,
            color: t.color,
            vision_bright_ft: t.vision_bright_ft,
            vision_dim_ft: t.vision_dim_ft,
            vision_dark_ft: t.vision_dark_ft,
            light_radius_ft: t.light_radius_ft,
        })
        .collect();
    let mut map_state = MapStateService::new(conn);
    let lights = map_state
        .list_lights(map_id)?
        .into_iter()
        .filter(|l| l.active != 0)
        .map(|l| wire::PlayerLight {
            x: centre(l.grid_x, ppg),
            y: centre(l.grid_y, ppg),
            id: l.id,
            grid_x: l.grid_x,
            grid_y: l.grid_y,
            bright_radius_ft: l.bright_radius,
            dim_radius_ft: l.dim_radius,
            color: l.color,
        })
        .collect();
    let mut markers: Vec<wire::PlayerMarker> = map_state
        .list_visible_traps(map_id)?
        .into_iter()
        .map(|t| wire::PlayerMarker {
            id: t.id,
            kind: "trap".into(),
            grid_x: t.grid_x,
            grid_y: t.grid_y,
            name: t.name,
            icon: "trap".into(),
            color: None,
        })
        .collect();
    markers.extend(
        map_state
            .list_visible_pois(map_id)?
            .into_iter()
            .map(|p| wire::PlayerMarker {
                id: p.id,
                kind: "poi".into(),
                grid_x: p.grid_x,
                grid_y: p.grid_y,
                name: p.name,
                icon: p.icon,
                color: p.color,
            }),
    );
    let initiative = match (&map.module_id, show_initiative) {
        (Some(module), true) => {
            let seen: Vec<&str> = tokens
                .iter()
                .map(|t: &wire::PlayerToken| t.id.as_str())
                .collect();
            CombatService::new(conn)
                .active(module)?
                .map(|state| crate::api::combat::player_initiative(&state, &seen))
        }
        _ => None,
    };
    Ok(wire::PlayerView {
        map,
        fog,
        tokens,
        lights,
        markers,
        initiative,
    })
}

/// `GET /maps/{id}/player-view`
pub async fn player_view(
    State(state): State<AppState>,
    Extension(caller): Extension<Caller>,
    Path(id): Path<String>,
) -> ApiResult<wire::PlayerView> {
    let app_dir = state.config.data_dir.clone();
    let live = state.live.clone();
    state
        .with_db(move |conn| {
            scope::shown_map(&caller, &live, conn, &app_dir, &id)?;
            let campaign = require_map(conn, &app_dir, &id)?.campaign_id;
            let show = live.display(&campaign).show_initiative;
            build_player_view(conn, &app_dir, &id, show)
        })
        .await
        .map(Json)
}

// ---- Tokens --------------------------------------------------------------

/// `GET /maps/{id}/tokens`
pub async fn list_tokens(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> ApiResult<Vec<wire::Token>> {
    let app_dir = state.config.data_dir.clone();
    state
        .with_db(move |conn| {
            require_map(conn, &app_dir, &id)?;
            let list = TokenService::new(conn, &app_dir).list(&id)?;
            Ok(list.into_iter().map(token).collect())
        })
        .await
        .map(Json)
}

/// `POST /maps/{id}/tokens`
pub async fn create_token(
    State(state): State<AppState>,
    Path(id): Path<String>,
    JsonBody(body): JsonBody<wire::NewToken>,
) -> Result<(StatusCode, Json<wire::Token>), ApiError> {
    let live = state.live.clone();
    let app_dir = state.config.data_dir.clone();
    state
        .with_db(move |conn| {
            require_map(conn, &app_dir, &id)?;
            let created = TokenService::new(conn, &app_dir).create(CreateTokenInput {
                map_id: id,
                module_monster_id: body.monster_id,
                module_npc_id: body.npc_id,
                grid_x: body.grid_x,
                grid_y: body.grid_y,
                label: body.label,
                faction_color: body.color,
                hidden: body.hidden,
            })?;
            notify(&live, conn, &app_dir, &created.map_id, MapPart::Tokens);
            Ok((StatusCode::CREATED, Json(token(created))))
        })
        .await
}

/// `PATCH /tokens/{id}`
pub async fn update_token(
    State(state): State<AppState>,
    Path(id): Path<String>,
    JsonBody(patch): JsonBody<wire::TokenPatch>,
) -> ApiResult<wire::Token> {
    let live = state.live.clone();
    let app_dir = state.config.data_dir.clone();
    state
        .with_db(move |conn| {
            let updated = TokenService::new(conn, &app_dir).update(
                &id,
                UpdateTokenInput {
                    grid_x: patch.grid_x,
                    grid_y: patch.grid_y,
                    label: patch.label,
                    faction_color: patch.color,
                    hidden: patch.hidden,
                    vision_bright_ft: patch.vision_bright_ft,
                    vision_dim_ft: patch.vision_dim_ft,
                    vision_dark_ft: patch.vision_dark_ft,
                    light_radius_ft: patch.light_radius_ft,
                },
            )?;
            notify(&live, conn, &app_dir, &updated.map_id, MapPart::Tokens);
            Ok(token(updated))
        })
        .await
        .map(Json)
}

/// `DELETE /tokens/{id}`
pub async fn delete_token(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, ApiError> {
    let live = state.live.clone();
    let app_dir = state.config.data_dir.clone();
    state
        .with_db(move |conn| {
            let mut tokens = TokenService::new(conn, &app_dir);
            let map_id = tokens
                .get(&id)?
                .ok_or_else(|| ServiceError::not_found("Token", &id))?
                .map_id;
            tokens.delete(&id)?;
            notify(&live, conn, &app_dir, &map_id, MapPart::Tokens);
            Ok(StatusCode::NO_CONTENT)
        })
        .await
}

/// `GET /tokens/{id}/image`: a monster token's art; 404 when it has none.
pub async fn get_token_image(
    State(state): State<AppState>,
    Extension(caller): Extension<Caller>,
    Path(id): Path<String>,
) -> Result<Response, ApiError> {
    let app_dir = state.config.data_dir.clone();
    let live = state.live.clone();
    state
        .with_db(move |conn| {
            // A player: a token they see on the shown map.
            if matches!(caller, Caller::Player(_)) {
                let t = TokenService::new(conn, &app_dir)
                    .get(&id)?
                    .ok_or_else(|| ServiceError::not_found("Token", &id))?;
                scope::shown_map(&caller, &live, conn, &app_dir, &t.map_id)?;
                if !t.visible_to_players {
                    return Err(ApiError::forbidden());
                }
            }
            let path = TokenService::new(conn, &app_dir)
                .image_path(&id)?
                .ok_or_else(|| ApiError::not_found(format!("no image for token {id}")))?;
            image_response(path)
        })
        .await
}

// ---- Fog -----------------------------------------------------------------

/// `GET /maps/{id}/fog`
pub async fn get_fog(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> ApiResult<wire::Fog> {
    let app_dir = state.config.data_dir.clone();
    state
        .with_db(move |conn| {
            require_map(conn, &app_dir, &id)?;
            fog(conn, &id)
        })
        .await
        .map(Json)
}

/// `PUT /maps/{id}/fog`: fog on or off.
pub async fn set_fog(
    State(state): State<AppState>,
    Path(id): Path<String>,
    JsonBody(setting): JsonBody<wire::FogSetting>,
) -> ApiResult<wire::Fog> {
    let live = state.live.clone();
    let app_dir = state.config.data_dir.clone();
    state
        .with_db(move |conn| {
            require_map(conn, &app_dir, &id)?;
            let mut service = MapStateService::new(conn);
            if setting.enabled {
                service.enable_fog(&id)?;
            } else {
                service.disable_fog(&id)?;
            }
            notify(&live, conn, &app_dir, &id, MapPart::Fog);
            fog(conn, &id)
        })
        .await
        .map(Json)
}

/// `POST /maps/{id}/fog/reveal`
pub async fn reveal(
    State(state): State<AppState>,
    Path(id): Path<String>,
    JsonBody(shape): JsonBody<wire::Reveal>,
) -> Result<(StatusCode, Json<wire::Fog>), ApiError> {
    let live = state.live.clone();
    let app_dir = state.config.data_dir.clone();
    state
        .with_db(move |conn| {
            let map = require_map(conn, &app_dir, &id)?;
            let size = detail(conn, &app_dir, map);
            let mut service = MapStateService::new(conn);
            match shape {
                wire::Reveal::Rect {
                    x,
                    y,
                    width,
                    height,
                } => service.reveal_rect(&id, x, y, width, height)?,
                wire::Reveal::Circle {
                    center_x,
                    center_y,
                    radius,
                } => service.reveal_circle(&id, center_x, center_y, radius)?,
                wire::Reveal::All => {
                    service.reveal_all(&id, f64::from(size.width_px), f64::from(size.height_px))?
                }
            };
            notify(&live, conn, &app_dir, &id, MapPart::Fog);
            Ok((StatusCode::CREATED, Json(fog(conn, &id)?)))
        })
        .await
}

/// `DELETE /maps/{id}/fog/revealed`: cover the whole map again.
pub async fn reset_fog(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> ApiResult<wire::Fog> {
    let live = state.live.clone();
    let app_dir = state.config.data_dir.clone();
    state
        .with_db(move |conn| {
            require_map(conn, &app_dir, &id)?;
            MapStateService::new(conn).reset_fog(&id)?;
            notify(&live, conn, &app_dir, &id, MapPart::Fog);
            fog(conn, &id)
        })
        .await
        .map(Json)
}

/// `DELETE /maps/{id}/fog/revealed/{area_id}`: cover one revealed area again.
pub async fn delete_fog_area(
    State(state): State<AppState>,
    Path((map_id, area_id)): Path<(String, String)>,
) -> Result<StatusCode, ApiError> {
    let live = state.live.clone();
    let app_dir = state.config.data_dir.clone();
    state
        .with_db(move |conn| {
            require_map(conn, &app_dir, &map_id)?;
            let mut service = MapStateService::new(conn);
            if !service
                .fog_state(&map_id)?
                .revealed_areas
                .iter()
                .any(|a| a.id == area_id)
            {
                return Err(ServiceError::not_found("FogArea", &area_id).into());
            }
            service.delete_revealed_area(&area_id)?;
            notify(&live, conn, &app_dir, &map_id, MapPart::Fog);
            Ok(StatusCode::NO_CONTENT)
        })
        .await
}

// ---- Lights --------------------------------------------------------------

/// `GET /maps/{id}/lights`
pub async fn list_lights(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> ApiResult<Vec<wire::Light>> {
    let app_dir = state.config.data_dir.clone();
    state
        .with_db(move |conn| {
            require_map(conn, &app_dir, &id)?;
            let ppg = ppg_of(conn, &app_dir, &id);
            let list = MapStateService::new(conn).list_lights(&id)?;
            Ok(list.into_iter().map(|l| light(l, ppg)).collect())
        })
        .await
        .map(Json)
}

/// `POST /maps/{id}/lights`
pub async fn create_light(
    State(state): State<AppState>,
    Path(id): Path<String>,
    JsonBody(body): JsonBody<wire::NewLight>,
) -> Result<(StatusCode, Json<wire::Light>), ApiError> {
    let live = state.live.clone();
    let app_dir = state.config.data_dir.clone();
    state
        .with_db(move |conn| {
            require_map(conn, &app_dir, &id)?;
            let ppg = ppg_of(conn, &app_dir, &id);
            let mut service = MapStateService::new(conn);
            let created = match body.preset.as_deref() {
                Some("torch") => service.create_torch(&id, body.grid_x, body.grid_y)?,
                Some("lantern") => service.create_lantern(&id, body.grid_x, body.grid_y)?,
                Some(other) => {
                    return Err(ServiceError::validation(format!(
                        "unknown light preset {other:?} (torch, lantern)"
                    ))
                    .into())
                }
                None => service.create_light(CreateLightInput {
                    map_id: id,
                    grid_x: body.grid_x,
                    grid_y: body.grid_y,
                    bright_radius_ft: body.bright_radius_ft.unwrap_or(20),
                    dim_radius_ft: body.dim_radius_ft.unwrap_or(40),
                    name: body.name.unwrap_or_else(|| "Light".to_string()),
                    color: body.color,
                    is_active: body.active.unwrap_or(true),
                })?,
            };
            notify(&live, conn, &app_dir, &created.map_id, MapPart::Lights);
            Ok((StatusCode::CREATED, Json(light(created, ppg))))
        })
        .await
}

/// `DELETE /maps/{id}/lights`: remove all placed lights.
pub async fn delete_all_lights(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, ApiError> {
    let live = state.live.clone();
    let app_dir = state.config.data_dir.clone();
    state
        .with_db(move |conn| {
            require_map(conn, &app_dir, &id)?;
            MapStateService::new(conn).delete_all_lights(&id)?;
            notify(&live, conn, &app_dir, &id, MapPart::Lights);
            Ok(StatusCode::NO_CONTENT)
        })
        .await
}

/// `PATCH /lights/{id}`
pub async fn update_light(
    State(state): State<AppState>,
    Path(id): Path<String>,
    JsonBody(patch): JsonBody<wire::LightPatch>,
) -> ApiResult<wire::Light> {
    let live = state.live.clone();
    let app_dir = state.config.data_dir.clone();
    state
        .with_db(move |conn| {
            let updated = conn.transaction::<_, ApiError, _>(|conn| {
                let mut service = MapStateService::new(conn);
                let current = service.get_light(&id)?;
                if patch.grid_x.is_some() || patch.grid_y.is_some() {
                    service.move_light(
                        &id,
                        patch.grid_x.unwrap_or(current.grid_x),
                        patch.grid_y.unwrap_or(current.grid_y),
                    )?;
                }
                Ok(service.update_light(
                    &id,
                    UpdateLightInput {
                        name: patch.name,
                        bright_radius_ft: patch.bright_radius_ft,
                        dim_radius_ft: patch.dim_radius_ft,
                        color: patch.color,
                        is_active: patch.active,
                    },
                )?)
            })?;
            let ppg = ppg_of(conn, &app_dir, &updated.map_id);
            notify(&live, conn, &app_dir, &updated.map_id, MapPart::Lights);
            Ok(light(updated, ppg))
        })
        .await
        .map(Json)
}

/// `DELETE /lights/{id}`
pub async fn delete_light(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, ApiError> {
    let app_dir = state.config.data_dir.clone();
    let live = state.live.clone();
    state
        .with_db(move |conn| {
            let mut service = MapStateService::new(conn);
            let map_id = service.get_light(&id)?.map_id;
            service.delete_light(&id)?;
            notify(&live, conn, &app_dir, &map_id, MapPart::Lights);
            Ok(StatusCode::NO_CONTENT)
        })
        .await
}

// ---- Traps ---------------------------------------------------------------

/// `GET /maps/{id}/traps`
pub async fn list_traps(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> ApiResult<Vec<wire::Trap>> {
    let app_dir = state.config.data_dir.clone();
    state
        .with_db(move |conn| {
            require_map(conn, &app_dir, &id)?;
            let list = MapStateService::new(conn).list_traps(&id)?;
            Ok(list.into_iter().map(trap).collect())
        })
        .await
        .map(Json)
}

/// `POST /maps/{id}/traps`
pub async fn create_trap(
    State(state): State<AppState>,
    Path(id): Path<String>,
    JsonBody(body): JsonBody<wire::NewTrap>,
) -> Result<(StatusCode, Json<wire::Trap>), ApiError> {
    let live = state.live.clone();
    let app_dir = state.config.data_dir.clone();
    state
        .with_db(move |conn| {
            require_map(conn, &app_dir, &id)?;
            let created = MapStateService::new(conn).create_trap(CreateTrapInput {
                map_id: id,
                name: body.name,
                grid_x: body.grid_x,
                grid_y: body.grid_y,
                description: body.description,
                trigger_description: body.trigger_description,
                effect_description: body.effect_description,
                dc: body.dc,
                visible: body.visible,
            })?;
            notify(&live, conn, &app_dir, &created.map_id, MapPart::Markers);
            Ok((StatusCode::CREATED, Json(trap(created))))
        })
        .await
}

/// `GET /traps/{id}`
pub async fn get_trap(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> ApiResult<wire::Trap> {
    state
        .with_db(move |conn| Ok(trap(MapStateService::new(conn).get_trap(&id)?)))
        .await
        .map(Json)
}

/// `PATCH /traps/{id}`
pub async fn update_trap(
    State(state): State<AppState>,
    Path(id): Path<String>,
    JsonBody(patch): JsonBody<wire::TrapPatch>,
) -> ApiResult<wire::Trap> {
    let app_dir = state.config.data_dir.clone();
    let live = state.live.clone();
    state
        .with_db(move |conn| {
            conn.transaction::<_, ApiError, _>(|conn| {
                let mut service = MapStateService::new(conn);
                let current = service.get_trap(&id)?;
                let text = UpdateTrapInput {
                    name: patch.name,
                    description: patch.description,
                    trigger_description: patch.trigger_description,
                    effect_description: patch.effect_description,
                    dc: patch.dc,
                };
                if text.name.is_some()
                    || text.description.is_some()
                    || text.trigger_description.is_some()
                    || text.effect_description.is_some()
                    || text.dc.is_some()
                {
                    service.update_trap(&id, text)?;
                }
                if patch.grid_x.is_some() || patch.grid_y.is_some() {
                    service.move_trap(
                        &id,
                        patch.grid_x.unwrap_or(current.grid_x),
                        patch.grid_y.unwrap_or(current.grid_y),
                    )?;
                }
                if patch.visible.is_some_and(|v| v != (current.visible != 0)) {
                    service.toggle_trap_visibility(&id)?;
                }
                match patch.triggered {
                    Some(true) => {
                        service.trigger_trap(&id)?;
                    }
                    Some(false) => {
                        service.reset_trap(&id)?;
                    }
                    None => {}
                }
                Ok(trap(service.get_trap(&id)?))
            })
            .inspect(|t| notify(&live, conn, &app_dir, &t.map_id, MapPart::Markers))
        })
        .await
        .map(Json)
}

/// `DELETE /traps/{id}`
pub async fn delete_trap(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, ApiError> {
    let app_dir = state.config.data_dir.clone();
    let live = state.live.clone();
    state
        .with_db(move |conn| {
            let mut service = MapStateService::new(conn);
            let map_id = service.get_trap(&id)?.map_id;
            service.delete_trap(&id)?;
            notify(&live, conn, &app_dir, &map_id, MapPart::Markers);
            Ok(StatusCode::NO_CONTENT)
        })
        .await
}

// ---- Points of interest --------------------------------------------------

/// `GET /maps/{id}/pois`
pub async fn list_pois(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> ApiResult<Vec<wire::Poi>> {
    let app_dir = state.config.data_dir.clone();
    state
        .with_db(move |conn| {
            require_map(conn, &app_dir, &id)?;
            let list = MapStateService::new(conn).list_pois(&id)?;
            Ok(list.into_iter().map(poi).collect())
        })
        .await
        .map(Json)
}

/// `POST /maps/{id}/pois`
pub async fn create_poi(
    State(state): State<AppState>,
    Path(id): Path<String>,
    JsonBody(body): JsonBody<wire::NewPoi>,
) -> Result<(StatusCode, Json<wire::Poi>), ApiError> {
    let live = state.live.clone();
    let app_dir = state.config.data_dir.clone();
    state
        .with_db(move |conn| {
            require_map(conn, &app_dir, &id)?;
            let created = MapStateService::new(conn).create_poi(CreatePoiInput {
                map_id: id,
                name: body.name,
                grid_x: body.grid_x,
                grid_y: body.grid_y,
                description: body.description,
                icon: body.icon,
                color: body.color,
                visible: body.visible,
            })?;
            notify(&live, conn, &app_dir, &created.map_id, MapPart::Markers);
            Ok((StatusCode::CREATED, Json(poi(created))))
        })
        .await
}

/// `GET /pois/{id}`
pub async fn get_poi(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> ApiResult<wire::Poi> {
    state
        .with_db(move |conn| Ok(poi(MapStateService::new(conn).get_poi(&id)?)))
        .await
        .map(Json)
}

/// `PATCH /pois/{id}`
pub async fn update_poi(
    State(state): State<AppState>,
    Path(id): Path<String>,
    JsonBody(patch): JsonBody<wire::PoiPatch>,
) -> ApiResult<wire::Poi> {
    let app_dir = state.config.data_dir.clone();
    let live = state.live.clone();
    state
        .with_db(move |conn| {
            conn.transaction::<_, ApiError, _>(|conn| {
                let mut service = MapStateService::new(conn);
                let current = service.get_poi(&id)?;
                let text = UpdatePoiInput {
                    name: patch.name,
                    description: patch.description,
                    icon: patch.icon,
                    color: patch.color,
                };
                if text.name.is_some()
                    || text.description.is_some()
                    || text.icon.is_some()
                    || text.color.is_some()
                {
                    service.update_poi(&id, text)?;
                }
                if patch.grid_x.is_some() || patch.grid_y.is_some() {
                    service.move_poi(
                        &id,
                        patch.grid_x.unwrap_or(current.grid_x),
                        patch.grid_y.unwrap_or(current.grid_y),
                    )?;
                }
                if patch.visible.is_some_and(|v| v != (current.visible != 0)) {
                    service.toggle_poi_visibility(&id)?;
                }
                Ok(poi(service.get_poi(&id)?))
            })
            .inspect(|p| notify(&live, conn, &app_dir, &p.map_id, MapPart::Markers))
        })
        .await
        .map(Json)
}

/// `DELETE /pois/{id}`
pub async fn delete_poi(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, ApiError> {
    let app_dir = state.config.data_dir.clone();
    let live = state.live.clone();
    state
        .with_db(move |conn| {
            let mut service = MapStateService::new(conn);
            let map_id = service.get_poi(&id)?.map_id;
            service.delete_poi(&id)?;
            notify(&live, conn, &app_dir, &map_id, MapPart::Markers);
            Ok(StatusCode::NO_CONTENT)
        })
        .await
}
