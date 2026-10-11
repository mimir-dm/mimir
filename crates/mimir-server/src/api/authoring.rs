//! Campaign content, write side (MIMIR-T-0720): campaigns, documents,
//! modules, module monsters, maps. DM only (the route table). A map is a
//! multipart upload of a UVTT file or an image, checked here: the core
//! stores the bytes as they come. An image becomes a UVTT file with no
//! walls (the desktop sent raw image bytes, which the core cannot read).

use axum::extract::{Multipart, Path, State};
use axum::http::StatusCode;
use axum::Json;
use diesel::SqliteConnection;
use mimir_core::models::campaign::{Document, LightingMode};
use mimir_core::services::{
    AddMonsterInput, CampaignService, CreateCampaignInput, CreateDocumentInput, CreateMapInput,
    CreateModuleInput, DocumentService, HomebrewService, MapService, ModuleService, ModuleType,
    MonsterRef, ServiceError, UpdateCampaignInput, UpdateDocumentInput, UpdateMapInput,
    UpdateModuleInput,
};
use mimir_wire::{self as wire, MapPart};

use super::convert;
use crate::error::{ApiError, JsonBody};
use crate::live::Change;
use crate::state::AppState;

type ApiResult<T> = Result<Json<T>, ApiError>;
type Created<T> = Result<(StatusCode, Json<T>), ApiError>;

/// The largest map upload (a UVTT file carries its image), as the desktop.
pub const MAP_UPLOAD_LIMIT: usize = 500 * 1024 * 1024;

fn not_empty(field: &str, value: &str) -> Result<String, ApiError> {
    let v = value.trim();
    if v.is_empty() {
        return Err(ServiceError::validation(format!("{field} must not be empty")).into());
    }
    Ok(v.to_string())
}

/// An empty text is no text.
fn text(v: Option<String>) -> Option<String> {
    v.map(|s| s.trim().to_string()).filter(|s| !s.is_empty())
}

// ---------------------------------------------------------------- campaigns

/// `POST /campaigns`
pub async fn create_campaign(
    State(state): State<AppState>,
    JsonBody(body): JsonBody<wire::NewCampaign>,
) -> Created<wire::CampaignSummary> {
    let name = not_empty("name", &body.name)?;
    state
        .with_db(move |conn| {
            let mut input = CreateCampaignInput::new(name);
            if let Some(d) = text(body.description) {
                input = input.with_description(d);
            }
            let c = CampaignService::new(conn).create(input)?;
            Ok((StatusCode::CREATED, Json(convert::campaign(c))))
        })
        .await
}

/// `PATCH /campaigns/{id}`
pub async fn patch_campaign(
    State(state): State<AppState>,
    Path(id): Path<String>,
    JsonBody(p): JsonBody<wire::CampaignPatch>,
) -> ApiResult<wire::CampaignSummary> {
    let name = p
        .name
        .as_deref()
        .map(|n| not_empty("name", n))
        .transpose()?;
    state
        .with_db(move |conn| {
            let input = UpdateCampaignInput {
                name,
                description: p.description.map(text),
            };
            Ok(convert::campaign(
                CampaignService::new(conn).update(&id, input)?,
            ))
        })
        .await
        .map(Json)
}

/// `PUT /campaigns/{id}/archive` archives; `DELETE` brings it back.
pub async fn archive_campaign(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> ApiResult<wire::CampaignSummary> {
    set_archived(state, id, true).await
}

pub async fn unarchive_campaign(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> ApiResult<wire::CampaignSummary> {
    set_archived(state, id, false).await
}

async fn set_archived(state: AppState, id: String, on: bool) -> ApiResult<wire::CampaignSummary> {
    state
        .with_db(move |conn| {
            let mut s = CampaignService::new(conn);
            if on {
                s.archive(&id)?;
            } else {
                s.unarchive(&id)?;
            }
            let c = s
                .get(&id)?
                .ok_or_else(|| ServiceError::not_found("Campaign", &id))?;
            Ok(convert::campaign(c))
        })
        .await
        .map(Json)
}

/// `DELETE /campaigns/{id}`: only an archived campaign (archive first, as a
/// guard against a slip). The map files go too.
pub async fn delete_campaign(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, ApiError> {
    let app_dir = state.config.data_dir.clone();
    state
        .with_db(move |conn| {
            let mut s = CampaignService::new(conn);
            let c = s
                .get(&id)?
                .ok_or_else(|| ServiceError::not_found("Campaign", &id))?;
            if c.archived_at.is_none() {
                return Err(
                    ServiceError::validation("archive the campaign before deleting it").into(),
                );
            }
            for map in MapService::new(conn, &app_dir).list_for_campaign(&id)? {
                MapService::new(conn, &app_dir).delete(&map.id)?;
            }
            CampaignService::new(conn).delete(&id)?;
            Ok(StatusCode::NO_CONTENT)
        })
        .await
}

// ---------------------------------------------------------------- documents

fn new_document(
    conn: &mut SqliteConnection,
    campaign_id: String,
    module_id: Option<String>,
    body: wire::NewDocument,
) -> Result<wire::Document, ApiError> {
    let input = CreateDocumentInput {
        campaign_id,
        module_id,
        title: not_empty("title", &body.title)?,
        doc_type: text(body.doc_type),
        content: body.content,
    };
    Ok(convert::document(DocumentService::new(conn).create(input)?))
}

/// `POST /campaigns/{id}/documents`
pub async fn create_campaign_document(
    State(state): State<AppState>,
    Path(id): Path<String>,
    JsonBody(body): JsonBody<wire::NewDocument>,
) -> Created<wire::Document> {
    state
        .with_db(move |conn| {
            CampaignService::new(conn)
                .get(&id)?
                .ok_or_else(|| ServiceError::not_found("Campaign", &id))?;
            Ok((
                StatusCode::CREATED,
                Json(new_document(conn, id, None, body)?),
            ))
        })
        .await
}

/// `POST /modules/{id}/documents`
pub async fn create_module_document(
    State(state): State<AppState>,
    Path(id): Path<String>,
    JsonBody(body): JsonBody<wire::NewDocument>,
) -> Created<wire::Document> {
    state
        .with_db(move |conn| {
            let m = ModuleService::new(conn)
                .get(&id)?
                .ok_or_else(|| ServiceError::not_found("Module", &id))?;
            Ok((
                StatusCode::CREATED,
                Json(new_document(conn, m.campaign_id, Some(id), body)?),
            ))
        })
        .await
}

/// `PATCH /documents/{id}`
pub async fn patch_document(
    State(state): State<AppState>,
    Path(id): Path<String>,
    JsonBody(p): JsonBody<wire::DocumentPatch>,
) -> ApiResult<wire::Document> {
    let title = p
        .title
        .as_deref()
        .map(|t| not_empty("title", t))
        .transpose()?;
    state
        .with_db(move |conn| {
            let input = UpdateDocumentInput {
                title,
                content: p.content,
                doc_type: None,
            };
            Ok(convert::document(
                DocumentService::new(conn).update(&id, input)?,
            ))
        })
        .await
        .map(Json)
}

/// `DELETE /documents/{id}`
pub async fn delete_document(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, ApiError> {
    state
        .with_db(move |conn| {
            DocumentService::new(conn).delete(&id)?;
            Ok(StatusCode::NO_CONTENT)
        })
        .await
}

/// The neighbour of an item in an ordered list, one place up or down.
pub fn neighbour<T>(list: &[T], is_it: impl Fn(&T) -> bool, d: wire::Direction) -> Option<&T> {
    let i = list.iter().position(is_it)?;
    match d {
        wire::Direction::Up => i.checked_sub(1).and_then(|j| list.get(j)),
        wire::Direction::Down => list.get(i + 1),
    }
}

/// `POST /documents/{id}/move`: swap with the neighbour in its list (the
/// campaign's or the module's). Answers the list in its new order; at
/// either end the order does not change.
pub async fn move_document(
    State(state): State<AppState>,
    Path(id): Path<String>,
    JsonBody(m): JsonBody<wire::Move>,
) -> ApiResult<Vec<wire::DocumentSummary>> {
    state
        .with_db(move |conn| {
            let mut s = DocumentService::new(conn);
            let doc = s
                .get(&id)?
                .ok_or_else(|| ServiceError::not_found("Document", &id))?;
            let list_of = |s: &mut DocumentService, d: &Document| match &d.module_id {
                Some(m) => s.list_for_module(m),
                None => s.list_for_campaign(&d.campaign_id),
            };
            let list = list_of(&mut s, &doc)?;
            if let Some(other) = neighbour(&list, |d| d.id == id, m.direction) {
                s.swap_order(&id, &other.id)?;
            }
            let list = list_of(&mut s, &doc)?;
            Ok(list.into_iter().map(convert::document_summary).collect())
        })
        .await
        .map(Json)
}

// ---------------------------------------------------------------- modules

/// The module type of a key; unknown keys are refused.
fn module_type(key: Option<&str>) -> Result<ModuleType, ApiError> {
    Ok(match key.unwrap_or("general") {
        "general" => ModuleType::General,
        "mystery" => ModuleType::Mystery,
        "dungeon" => ModuleType::Dungeon,
        "heist" => ModuleType::Heist,
        "horror" => ModuleType::Horror,
        "political" => ModuleType::Political,
        other => {
            return Err(ServiceError::validation(format!("unknown module type {other:?}")).into())
        }
    })
}

/// `POST /campaigns/{id}/modules`: with the starting documents of its type.
pub async fn create_module(
    State(state): State<AppState>,
    Path(id): Path<String>,
    JsonBody(body): JsonBody<wire::NewModule>,
) -> Created<wire::ModuleSummary> {
    let name = not_empty("name", &body.name)?;
    let kind = module_type(body.module_type.as_deref())?;
    state
        .with_db(move |conn| {
            let mut input = CreateModuleInput::new(id, name).with_type(kind);
            if let Some(d) = text(body.description) {
                input = input.with_description(d);
            }
            let m = ModuleService::new(conn).create(input)?;
            Ok((StatusCode::CREATED, Json(convert::module(m))))
        })
        .await
}

/// `PATCH /modules/{id}`
pub async fn patch_module(
    State(state): State<AppState>,
    Path(id): Path<String>,
    JsonBody(p): JsonBody<wire::ModulePatch>,
) -> ApiResult<wire::ModuleSummary> {
    let name = p
        .name
        .as_deref()
        .map(|n| not_empty("name", n))
        .transpose()?;
    state
        .with_db(move |conn| {
            let input = UpdateModuleInput {
                name,
                description: p.description.map(text),
            };
            Ok(convert::module(
                ModuleService::new(conn).update(&id, input)?,
            ))
        })
        .await
        .map(Json)
}

/// `DELETE /modules/{id}`: with its documents, monsters and maps.
pub async fn delete_module(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, ApiError> {
    let app_dir = state.config.data_dir.clone();
    state
        .with_db(move |conn| {
            // The map files first: the module's rows go with the module.
            for map in MapService::new(conn, &app_dir).list_for_module(&id)? {
                MapService::new(conn, &app_dir).delete(&map.id)?;
            }
            ModuleService::new(conn).delete(&id)?;
            Ok(StatusCode::NO_CONTENT)
        })
        .await
}

/// `POST /modules/{id}/move`: one place up or down. Answers the modules
/// in their new order.
pub async fn move_module(
    State(state): State<AppState>,
    Path(id): Path<String>,
    JsonBody(m): JsonBody<wire::Move>,
) -> ApiResult<Vec<wire::ModuleSummary>> {
    state
        .with_db(move |conn| {
            let mut s = ModuleService::new(conn);
            let module = s
                .get(&id)?
                .ok_or_else(|| ServiceError::not_found("Module", &id))?;
            let list = s.list_for_campaign(&module.campaign_id)?;
            let list = match neighbour(&list, |x| x.id == id, m.direction) {
                Some(other) => s.reorder(&id, other.module_number)?,
                None => list,
            };
            Ok(list.into_iter().map(convert::module).collect())
        })
        .await
        .map(Json)
}

// ---------------------------------------------------------------- module monsters

fn homebrew_name(conn: &mut SqliteConnection, id: Option<&str>) -> Option<String> {
    let id = id?;
    HomebrewService::new(conn)
        .get_monster(id)
        .ok()
        .map(|m| m.name)
}

/// `POST /modules/{id}/monsters`: the same monster again adds to its
/// quantity.
pub async fn add_module_monster(
    State(state): State<AppState>,
    Path(id): Path<String>,
    JsonBody(body): JsonBody<wire::NewModuleMonster>,
) -> Created<wire::ModuleMonsterSummary> {
    let monster = match (
        text(body.homebrew_monster_id),
        text(body.monster_name),
        text(body.monster_source),
    ) {
        (Some(hid), _, _) => MonsterRef::Homebrew { id: hid },
        (None, Some(name), Some(source)) => MonsterRef::Catalog { name, source },
        _ => {
            return Err(ServiceError::validation(
                "give monster_name and monster_source, or homebrew_monster_id",
            )
            .into())
        }
    };
    let quantity = body.quantity.unwrap_or(1);
    if quantity < 1 {
        return Err(ServiceError::validation("quantity must be 1 or more").into());
    }
    state
        .with_db(move |conn| {
            // "HB": the campaign's homebrew monster of that name (MIMIR-T-0722).
            let monster = match monster {
                MonsterRef::Catalog { name, source } if source.eq_ignore_ascii_case("HB") => {
                    let module = ModuleService::new(conn)
                        .get(&id)?
                        .ok_or_else(|| ServiceError::not_found("Module", &id))?;
                    let hb = HomebrewService::new(conn)
                        .get_monster_by_name(&module.campaign_id, &name)?
                        .ok_or_else(|| ServiceError::not_found("Homebrew monster", &name))?;
                    MonsterRef::Homebrew { id: hb.id }
                }
                other => other,
            };
            let mut input = AddMonsterInput::new(id, monster).with_quantity(quantity);
            if let Some(n) = text(body.display_name) {
                input = input.with_display_name(n);
            }
            if let Some(n) = text(body.notes) {
                input = input.with_notes(n);
            }
            let m = ModuleService::new(conn).add_monster(input)?;
            let hb = homebrew_name(conn, m.homebrew_monster_id.as_deref());
            Ok((StatusCode::CREATED, Json(convert::module_monster(m, hb))))
        })
        .await
}

/// `PATCH /module-monsters/{id}`
pub async fn patch_module_monster(
    State(state): State<AppState>,
    Path(id): Path<String>,
    JsonBody(p): JsonBody<wire::ModuleMonsterPatch>,
) -> ApiResult<wire::ModuleMonsterSummary> {
    if p.quantity.is_some_and(|q| q < 1) {
        return Err(ServiceError::validation("quantity must be 1 or more").into());
    }
    state
        .with_db(move |conn| {
            let m = ModuleService::new(conn).update_monster(
                &id,
                p.display_name.as_deref(),
                p.notes.as_deref(),
                p.quantity,
            )?;
            let hb = homebrew_name(conn, m.homebrew_monster_id.as_deref());
            Ok(convert::module_monster(m, hb))
        })
        .await
        .map(Json)
}

/// `DELETE /module-monsters/{id}`
pub async fn delete_module_monster(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, ApiError> {
    state
        .with_db(move |conn| {
            ModuleService::new(conn).remove_monster(&id)?;
            Ok(StatusCode::NO_CONTENT)
        })
        .await
}

// ---------------------------------------------------------------- maps

fn lighting(key: Option<&str>) -> Result<Option<LightingMode>, ApiError> {
    Ok(match key {
        None => None,
        Some("bright") => Some(LightingMode::Bright),
        Some("dim") => Some(LightingMode::Dim),
        Some("dark") => Some(LightingMode::Dark),
        Some(other) => {
            return Err(ServiceError::validation(format!("unknown lighting mode {other:?}")).into())
        }
    })
}

/// A UVTT file (`.dd2vtt`, `.uvtt`): JSON with the map image and its grid.
pub fn check_uvtt(bytes: &[u8]) -> Result<(), String> {
    let v: serde_json::Value =
        serde_json::from_slice(bytes).map_err(|_| "the file is not a UVTT map (not JSON)")?;
    let image = v.get("image").and_then(|i| i.as_str()).unwrap_or("");
    if image.is_empty() {
        return Err("the UVTT file has no image".into());
    }
    if v.get("resolution").is_none() {
        return Err("the UVTT file has no resolution (grid)".into());
    }
    mimir_core::services::parse_geometry(bytes, None).map(|_| ())
}

#[derive(Default)]
struct Upload {
    name: Option<String>,
    description: Option<String>,
    module_id: Option<String>,
    lighting_mode: Option<String>,
    grid_px: Option<String>,
    file: Option<(String, Vec<u8>)>,
}

async fn read_upload(mut form: Multipart) -> Result<Upload, ApiError> {
    let bad = |e: axum::extract::multipart::MultipartError| {
        ApiError::from(ServiceError::validation(format!("bad upload: {e}")))
    };
    let mut u = Upload::default();
    while let Some(field) = form.next_field().await.map_err(bad)? {
        let name = field.name().unwrap_or("").to_string();
        if name == "file" {
            let filename = field.file_name().unwrap_or("map.dd2vtt").to_string();
            let bytes = field.bytes().await.map_err(bad)?;
            u.file = Some((filename, bytes.to_vec()));
            continue;
        }
        let value = field.text().await.map_err(bad)?;
        match name.as_str() {
            "name" => u.name = text(Some(value)),
            "description" => u.description = text(Some(value)),
            "module_id" => u.module_id = text(Some(value)),
            "lighting_mode" => u.lighting_mode = text(Some(value)),
            "grid_px" => u.grid_px = text(Some(value)),
            _ => {}
        }
    }
    Ok(u)
}

/// The bytes to store for an upload: a UVTT file as it is, an image
/// wrapped in a UVTT file with a grid of `grid_px` pixels a square.
pub fn map_bytes(filename: &str, bytes: Vec<u8>, grid_px: Option<&str>) -> Result<Vec<u8>, String> {
    let ext = filename.rsplit('.').next().unwrap_or("").to_lowercase();
    match ext.as_str() {
        "dd2vtt" | "uvtt" | "json" => check_uvtt(&bytes).map(|()| bytes),
        "png" | "jpg" | "jpeg" | "webp" => {
            let px = match grid_px {
                None => 70.0,
                Some(v) => v
                    .trim()
                    .parse::<f64>()
                    .map_err(|_| format!("grid_px {v:?} is not a number"))?,
            };
            mimir_core::services::uvtt_from_image(&bytes, px)
        }
        _ => Err("a map is a UVTT file (.dd2vtt, .uvtt) or an image (PNG, JPEG, WebP)".into()),
    }
}

/// `POST /campaigns/{id}/maps` (multipart): `file` (UVTT or image),
/// `name` (default: the file name), `description`, `module_id`,
/// `lighting_mode`, `grid_px` (an image's pixels a square, default 70).
pub async fn create_map(
    State(state): State<AppState>,
    Path(id): Path<String>,
    form: Multipart,
) -> Created<wire::MapSummary> {
    let u = read_upload(form).await?;
    let (filename, bytes) = u
        .file
        .ok_or_else(|| ServiceError::validation("no file: send the map as the field `file`"))?;
    let bytes = map_bytes(&filename, bytes, u.grid_px.as_deref())
        .map_err(|e| ServiceError::validation(format!("{filename}: {e}")))?;
    let name = u.name.unwrap_or_else(|| {
        filename
            .rsplit_once('.')
            .map(|(stem, _)| stem.to_string())
            .unwrap_or_else(|| filename.clone())
    });
    let mode = lighting(u.lighting_mode.as_deref())?;
    let app_dir = state.config.data_dir.clone();
    state
        .with_db(move |conn| {
            if let Some(m) = &u.module_id {
                let module = ModuleService::new(conn)
                    .get(m)?
                    .ok_or_else(|| ServiceError::not_found("Module", m))?;
                if module.campaign_id != id {
                    return Err(
                        ServiceError::validation("the module is not in this campaign").into(),
                    );
                }
            }
            // The stored file is a UVTT file, whatever came in.
            let stored = match filename.rsplit_once('.') {
                Some((stem, ext)) if !matches!(ext.to_lowercase().as_str(), "dd2vtt" | "uvtt") => {
                    format!("{stem}.uvtt")
                }
                _ => filename,
            };
            let mut input = match u.module_id {
                Some(m) => CreateMapInput::for_module(&id, m, name, stored, bytes),
                None => CreateMapInput::for_campaign(&id, name, stored, bytes),
            };
            if let Some(d) = u.description {
                input = input.with_description(d);
            }
            if let Some(mode) = mode {
                input = input.with_lighting_mode(mode);
            }
            let map = MapService::new(conn, &app_dir).create(input)?;
            Ok((StatusCode::CREATED, Json(convert::map(map))))
        })
        .await
}

/// `PATCH /maps/{id}`: name, description, lighting, or move it to another
/// module of the campaign (or the campaign level).
pub async fn patch_map(
    State(state): State<AppState>,
    Path(id): Path<String>,
    JsonBody(p): JsonBody<wire::MapPatch>,
) -> ApiResult<wire::MapSummary> {
    let name = p
        .name
        .as_deref()
        .map(|n| not_empty("name", n))
        .transpose()?;
    let mode = lighting(p.lighting_mode.as_deref())?;
    let app_dir = state.config.data_dir.clone();
    let live = state.live.clone();
    state
        .with_db(move |conn| {
            let map = MapService::new(conn, &app_dir)
                .get(&id)?
                .ok_or_else(|| ServiceError::not_found("Map", &id))?;
            if let Some(Some(m)) = &p.module_id {
                let module = ModuleService::new(conn)
                    .get(m)?
                    .ok_or_else(|| ServiceError::not_found("Module", m))?;
                if module.campaign_id != map.campaign_id {
                    return Err(ServiceError::validation(
                        "the module is not in this map's campaign",
                    )
                    .into());
                }
            }
            let input = UpdateMapInput {
                name,
                description: p.description.map(text),
                lighting_mode: mode,
                module_id: p.module_id,
                reveal_all: p.reveal_all,
                line_of_sight: p.line_of_sight,
                ..UpdateMapInput::default()
            };
            let map = MapService::new(conn, &app_dir).update(&id, input)?;
            live.publish(Change::Map {
                campaign_id: map.campaign_id.clone(),
                map_id: map.id.clone(),
                part: MapPart::Map,
            });
            Ok(convert::map(map))
        })
        .await
        .map(Json)
}

/// `DELETE /maps/{id}`: with its tokens, fog, lights, markers and file.
pub async fn delete_map(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, ApiError> {
    let app_dir = state.config.data_dir.clone();
    state
        .with_db(move |conn| {
            MapService::new(conn, &app_dir).delete(&id)?;
            Ok(StatusCode::NO_CONTENT)
        })
        .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use wire::Direction::{Down, Up};

    #[test]
    fn neighbours() {
        let l = [1, 2, 3];
        assert_eq!(neighbour(&l, |x| *x == 2, Up), Some(&1));
        assert_eq!(neighbour(&l, |x| *x == 2, Down), Some(&3));
        assert_eq!(neighbour(&l, |x| *x == 1, Up), None);
        assert_eq!(neighbour(&l, |x| *x == 3, Down), None);
        assert_eq!(neighbour(&l, |x| *x == 9, Down), None);
    }

    #[test]
    fn uvtt_checks() {
        assert!(check_uvtt(b"not json").is_err());
        assert!(check_uvtt(br#"{"resolution":{}}"#).is_err(), "no image");
        assert!(check_uvtt(br#"{"image":"aGk="}"#).is_err(), "no grid");
        let ok = br#"{"format":1.0,"resolution":{"map_origin":{"x":0,"y":0},"map_size":{"x":2,"y":2},"pixels_per_grid":70},"line_of_sight":[],"portals":[],"lights":[],"image":"aGk="}"#;
        assert_eq!(check_uvtt(ok), Ok(()));
    }

    #[test]
    fn map_files() {
        assert!(map_bytes("cave.txt", b"x".to_vec(), None).is_err());
        assert!(map_bytes("cave.png", b"not a png".to_vec(), None).is_err());
        let mut png = Vec::new();
        image::RgbImage::new(70, 70)
            .write_to(&mut std::io::Cursor::new(&mut png), image::ImageFormat::Png)
            .unwrap();
        let out = map_bytes("cave.png", png.clone(), Some("35")).unwrap();
        assert_eq!(check_uvtt(&out), Ok(()));
        assert!(map_bytes("cave.png", png, Some("big")).is_err());
    }

    #[test]
    fn module_types_and_lighting() {
        for (key, _) in wire::MODULE_TYPES {
            assert!(module_type(Some(key)).is_ok(), "{key}");
        }
        assert!(module_type(Some("space opera")).is_err());
        for (key, _) in wire::LIGHTING_MODES {
            assert!(lighting(Some(key)).is_ok(), "{key}");
        }
        assert!(lighting(Some("strobe")).is_err());
    }
}
