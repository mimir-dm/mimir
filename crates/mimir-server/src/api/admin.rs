//! Settings and admin (MIMIR-T-0724), DM only: campaign archives (export
//! as a download; preview and import from an upload), catalog imports
//! (5etools data and images) as background jobs, removing a source, and
//! the source books of a campaign or a character. Uploads stream to a
//! temporary file under the data folder; downloads stream from one.

use std::collections::HashMap;
use std::path::Path as FsPath;

use axum::body::Body;
use axum::extract::{Multipart, Path, State};
use axum::http::{header, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::Json;
use futures::StreamExt;
use mimir_core::dal::catalog as catalog_dal;
use mimir_core::import::{import_images_from_tarball, CatalogImportService};
use mimir_core::services::{ArchiveService, CampaignService, ServiceError, SourceService};
use mimir_wire as wire;
use serde_json::Value;
use tokio::io::AsyncWriteExt;

use crate::error::{ApiError, JsonBody};
use crate::state::AppState;

type ApiResult<T> = Result<Json<T>, ApiError>;

/// The largest campaign archive upload (it holds the map images).
pub const ARCHIVE_LIMIT: usize = 4 * 1024 * 1024 * 1024;
/// The largest catalog upload (the 5etools image archive is large).
pub const CATALOG_LIMIT: usize = 16 * 1024 * 1024 * 1024;

/// An upload: the file on disk (deleted when dropped) and the text fields.
pub struct Upload {
    pub file: Option<(String, tempfile::TempPath)>,
    pub fields: HashMap<String, String>,
}

fn tmp_dir(state: &AppState) -> std::path::PathBuf {
    state.config.data_dir.join("tmp")
}

/// Read a multipart form, writing the `file` field to disk as it comes.
pub async fn save_upload(mut form: Multipart, dir: &FsPath) -> Result<Upload, ApiError> {
    let bad = |e: axum::extract::multipart::MultipartError| {
        ApiError::from(ServiceError::validation(format!("bad upload: {e}")))
    };
    tokio::fs::create_dir_all(dir)
        .await
        .map_err(ServiceError::Io)?;
    let mut up = Upload {
        file: None,
        fields: HashMap::new(),
    };
    while let Some(mut field) = form.next_field().await.map_err(bad)? {
        let name = field.name().unwrap_or("").to_string();
        if name == "file" {
            let filename = field.file_name().unwrap_or("upload").to_string();
            let tmp = tempfile::NamedTempFile::new_in(dir).map_err(ServiceError::Io)?;
            let (std_file, path) = tmp.into_parts();
            let mut out = tokio::fs::File::from_std(std_file);
            while let Some(chunk) = field.chunk().await.map_err(bad)? {
                out.write_all(&chunk).await.map_err(ServiceError::Io)?;
            }
            out.flush().await.map_err(ServiceError::Io)?;
            up.file = Some((filename, path));
        } else {
            up.fields.insert(name, field.text().await.map_err(bad)?);
        }
    }
    Ok(up)
}

fn the_file(up: &mut Upload) -> Result<(String, tempfile::TempPath), ApiError> {
    up.file
        .take()
        .ok_or_else(|| ServiceError::validation("no file: send it as the field `file`").into())
}

fn counts(c: &impl serde::Serialize) -> wire::ArchiveCounts {
    serde_json::to_value(c)
        .ok()
        .and_then(|v| serde_json::from_value(v).ok())
        .unwrap_or_default()
}

// ------------------------------------------------------------------ archives

/// `GET /campaigns/{id}/export`: the campaign as a `.mimir-campaign.tar.gz`
/// download (documents, characters, maps with their images, homebrew).
/// The DM's choices for an exported adventure (query of
/// `/campaigns/{id}/adventure-export`).
#[derive(Debug, serde::Deserialize)]
pub struct AdventureExportQuery {
    pub source: String,
    pub name: String,
    /// Comma-separated.
    #[serde(default)]
    pub authors: String,
    #[serde(default)]
    pub version: Option<String>,
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default)]
    pub storyline: Option<String>,
    #[serde(default)]
    pub level_start: Option<i64>,
    #[serde(default)]
    pub level_end: Option<i64>,
}

impl AdventureExportQuery {
    pub fn meta(&self) -> crate::adventure::ExportMeta {
        let start = self.level_start.unwrap_or(1).clamp(1, 20);
        crate::adventure::ExportMeta {
            source: self.source.trim().to_string(),
            name: self.name.trim().to_string(),
            authors: self
                .authors
                .split(',')
                .map(str::trim)
                .filter(|a| !a.is_empty())
                .map(String::from)
                .collect(),
            version: self
                .version
                .clone()
                .filter(|v| !v.trim().is_empty())
                .unwrap_or_else(|| "1.0.0".into()),
            url: self.url.clone(),
            storyline: self
                .storyline
                .clone()
                .filter(|s| !s.trim().is_empty())
                .unwrap_or_else(|| "Homebrew".into()),
            level_start: start,
            level_end: self.level_end.unwrap_or(start).clamp(start, 20),
        }
    }
}

/// `GET /campaigns/{id}/adventure-export?source=…&name=…`: the campaign as
/// a 5etools homebrew adventure file (MIMIR-T-0740), self-contained.
pub async fn export_adventure(
    State(state): State<AppState>,
    Path(id): Path<String>,
    axum::extract::Query(q): axum::extract::Query<AdventureExportQuery>,
) -> Result<Response, ApiError> {
    if q.name.trim().is_empty() {
        return Err(ServiceError::validation("the adventure needs a name").into());
    }
    let meta = q.meta();
    let file_name = format!("{}.json", meta.source);
    let v = state
        .with_db(move |conn| crate::adventure::export_adventure(conn, &id, &meta))
        .await?;
    let body =
        serde_json::to_vec_pretty(&v).map_err(|e| ServiceError::validation(e.to_string()))?;
    Ok((
        [
            (
                axum::http::header::CONTENT_TYPE,
                "application/json".to_string(),
            ),
            (
                axum::http::header::CONTENT_DISPOSITION,
                format!("attachment; filename=\"{file_name}\""),
            ),
        ],
        body,
    )
        .into_response())
}

/// `GET /campaigns/{id}/adventure-pack?source=…&name=…` (the query of the
/// adventure export): the campaign as a Mimir adventure pack (MIMIR-T-0741).
pub async fn export_pack(
    State(state): State<AppState>,
    Path(id): Path<String>,
    axum::extract::Query(q): axum::extract::Query<AdventureExportQuery>,
) -> Result<Response, ApiError> {
    if q.name.trim().is_empty() {
        return Err(ServiceError::validation("the adventure needs a name").into());
    }
    let meta = q.meta();
    let file_name = format!("{}.mimir-adventure.zip", meta.source);
    let data_dir = state.config.data_dir.clone();
    let body = state
        .with_db(move |conn| crate::adventure::export_pack(conn, &data_dir, &id, &meta))
        .await?;
    Ok((
        [
            (
                axum::http::header::CONTENT_TYPE,
                "application/zip".to_string(),
            ),
            (
                axum::http::header::CONTENT_DISPOSITION,
                format!("attachment; filename=\"{file_name}\""),
            ),
        ],
        body,
    )
        .into_response())
}

/// `POST /adventure-packs/preview` (multipart `file`): what a pack holds.
pub async fn preview_pack(
    State(state): State<AppState>,
    form: Multipart,
) -> Result<Json<wire::AdventurePackPreview>, ApiError> {
    let mut up = save_upload(form, &tmp_dir(&state)).await?;
    let (_, path) = the_file(&mut up)?;
    let bytes = tokio::fs::read(&path).await.map_err(ServiceError::Io)?;
    Ok(Json(crate::adventure::preview_pack(&bytes)?))
}

/// `POST /adventure-packs/import` (multipart `file`, optional `name`): the
/// pack as a new campaign, in one transaction.
pub async fn import_pack(
    State(state): State<AppState>,
    form: Multipart,
) -> Result<(StatusCode, Json<wire::ArchiveImported>), ApiError> {
    let mut up = save_upload(form, &tmp_dir(&state)).await?;
    let (_, path) = the_file(&mut up)?;
    let name = up
        .fields
        .get("name")
        .map(|n| n.trim().to_string())
        .filter(|n| !n.is_empty());
    let bytes = tokio::fs::read(&path).await.map_err(ServiceError::Io)?;
    let data_dir = state.config.data_dir.clone();
    state
        .with_db(move |conn| {
            use diesel::Connection;
            let r = conn.transaction::<_, ApiError, _>(|c| {
                crate::adventure::import_pack(c, &data_dir, &bytes, name.as_deref())
            })?;
            Ok((
                StatusCode::CREATED,
                Json(wire::ArchiveImported {
                    campaign_id: r.campaign_id,
                    campaign_name: r.campaign_name,
                    counts: counts(&r.counts),
                }),
            ))
        })
        .await
}

pub async fn export_archive(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Response, ApiError> {
    let dir = tmp_dir(&state);
    tokio::fs::create_dir_all(&dir)
        .await
        .map_err(ServiceError::Io)?;
    // The asset paths are relative to the data folder ("assets/<id>.<ext>").
    let assets = state.config.data_dir.clone();
    let (out, path) = state
        .with_db(move |conn| {
            CampaignService::new(conn)
                .get(&id)?
                .ok_or_else(|| ServiceError::not_found("Campaign", &id))?;
            let out = tempfile::tempdir_in(&dir).map_err(ServiceError::Io)?;
            let path = ArchiveService::new(conn).export_campaign(&id, out.path(), &assets)?;
            Ok((out, path))
        })
        .await?;
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| "campaign.mimir-campaign.tar.gz".into());
    let file = tokio::fs::File::open(&path)
        .await
        .map_err(ServiceError::Io)?;
    let size = file.metadata().await.map(|m| m.len()).unwrap_or(0);
    // The stream holds the temporary folder until the download ends.
    let stream = tokio_util::io::ReaderStream::new(file).map(move |chunk| {
        let _keep = &out;
        chunk
    });
    Ok((
        [
            (header::CONTENT_TYPE, "application/gzip".to_string()),
            (header::CONTENT_LENGTH, size.to_string()),
            (
                header::CONTENT_DISPOSITION,
                format!("attachment; filename=\"{}\"", name.replace('"', "")),
            ),
        ],
        Body::from_stream(stream),
    )
        .into_response())
}

fn preview_of(path: &FsPath) -> Result<wire::ArchivePreview, ApiError> {
    let p = ArchiveService::preview_archive(path)
        .map_err(|e| ServiceError::validation(format!("not a Mimir campaign archive: {e}")))?;
    Ok(wire::ArchivePreview {
        campaign_name: p.campaign_name,
        counts: counts(&p.counts),
        catalog_references: p
            .catalog_references
            .iter()
            .map(|r| format!("{} {} ({})", r.ref_type, r.name, r.source))
            .collect(),
        mimir_version: p.mimir_version,
        created_at: p.created_at.to_rfc3339(),
    })
}

/// `POST /archives/preview` (multipart `file`): what the archive holds.
pub async fn preview_archive(
    State(state): State<AppState>,
    form: Multipart,
) -> ApiResult<wire::ArchivePreview> {
    let mut up = save_upload(form, &tmp_dir(&state)).await?;
    let (_, path) = the_file(&mut up)?;
    tokio::task::spawn_blocking(move || preview_of(&path))
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?
        .map(Json)
}

/// `POST /archives/import` (multipart `file`, `name` optional): a new
/// campaign from the archive, in one transaction (the desktop could leave
/// half a campaign on an error).
pub async fn import_archive(
    State(state): State<AppState>,
    form: Multipart,
) -> Result<(StatusCode, Json<wire::ArchiveImported>), ApiError> {
    let mut up = save_upload(form, &tmp_dir(&state)).await?;
    let (_, path) = the_file(&mut up)?;
    let name = up
        .fields
        .get("name")
        .map(|n| n.trim().to_string())
        .filter(|n| !n.is_empty());
    // The asset paths are relative to the data folder ("assets/<id>.<ext>").
    let assets = state.config.data_dir.clone();
    state
        .with_db(move |conn| {
            preview_of(&path)?;
            use diesel::Connection;
            let r = conn.transaction::<_, ServiceError, _>(|c| {
                ArchiveService::new(c).import_campaign(&path, &assets, name.as_deref())
            })?;
            Ok((
                StatusCode::CREATED,
                Json(wire::ArchiveImported {
                    campaign_id: r.campaign_id,
                    campaign_name: r.campaign_name,
                    counts: counts(&r.counts),
                }),
            ))
        })
        .await
}

// ------------------------------------------------------------------ catalog

/// `POST /catalog/import` (multipart `file`: a 5etools data `.tar.gz`):
/// imports the core and supplement books as a job.
pub async fn import_catalog(
    State(state): State<AppState>,
    form: Multipart,
) -> Result<(StatusCode, Json<wire::Job>), ApiError> {
    let mut up = save_upload(form, &tmp_dir(&state)).await?;
    let (filename, path) = the_file(&mut up)?;
    let job = state
        .jobs
        .start("catalog", &format!("Importing {filename}"));
    let id = job.id.clone();
    let st = state.clone();
    tokio::task::spawn_blocking(move || {
        let _keep = &path;
        let run = || -> Result<Value, String> {
            let mut conn = st.connect().map_err(|e| format!("{e:?}"))?;
            let r = CatalogImportService::new(&mut conn)
                .import_from_tarball(&path)
                .map_err(|e| format!("{e:#}"))?;
            Ok(serde_json::json!({
                "sources_imported": r.sources_imported,
                "sources_failed": r.sources_failed.iter().map(|(s, e)| format!("{s}: {e}")).collect::<Vec<_>>(),
                "total_entities": r.total_entities,
            }))
        };
        match run() {
            Ok(v) => {
                let n = v["sources_imported"].as_array().map(Vec::len).unwrap_or(0);
                let total = v["total_entities"].as_u64().unwrap_or(0);
                job.done(format!("Imported {n} sources, {total} entries."), v);
            }
            Err(e) => job.failed(e),
        }
    });
    let j = state.jobs.get(&id).expect("the job");
    Ok((StatusCode::ACCEPTED, Json(j)))
}

/// The largest homebrew file.
pub const HOMEBREW_LIMIT: usize = 64 * 1024 * 1024;

/// `POST /catalog/import-homebrew` (multipart `file`: a 5etools homebrew
/// `.json`, MIMIR-T-0736): its sources and entries go to the catalog, its
/// books and adventures to the reader. A source already in the catalog is
/// refused.
pub async fn import_homebrew(
    State(state): State<AppState>,
    form: Multipart,
) -> Result<(StatusCode, Json<wire::HomebrewImport>), ApiError> {
    let mut up = save_upload(form, &tmp_dir(&state)).await?;
    let (_, path) = the_file(&mut up)?;
    let bytes = tokio::fs::read(&path).await.map_err(ServiceError::Io)?;
    let file: Value = serde_json::from_slice(&bytes)
        .map_err(|e| ServiceError::validation(format!("not a JSON file: {e}")))?;
    let r = state
        .with_db(move |conn| {
            CatalogImportService::new(conn)
                .import_homebrew(&file)
                .map_err(|e| ServiceError::validation(format!("{e:#}")).into())
        })
        .await?;
    let count = |k: &str| r.entity_counts.get(k).copied().unwrap_or(0);
    Ok((
        StatusCode::CREATED,
        Json(wire::HomebrewImport {
            entries: r.total_entities,
            adventures: count("adventure"),
            books: count("book"),
            sources: r.sources_imported,
        }),
    ))
}

/// `POST /catalog/import-images` (multipart `file`: a 5etools image
/// `.tar.gz`): unpacks the images under the catalog assets, as a job.
pub async fn import_images(
    State(state): State<AppState>,
    form: Multipart,
) -> Result<(StatusCode, Json<wire::Job>), ApiError> {
    let mut up = save_upload(form, &tmp_dir(&state)).await?;
    let (filename, path) = the_file(&mut up)?;
    let job = state.jobs.start("images", &format!("Unpacking {filename}"));
    let id = job.id.clone();
    let dest = state.config.assets_dir().join("catalog");
    tokio::task::spawn_blocking(move || {
        let progress = job.clone();
        match import_images_from_tarball(&path, &dest, |n| {
            progress.progress(format!("{n} images so far"))
        }) {
            Ok(n) => job.done(
                format!("Imported {n} images."),
                serde_json::json!({ "images": n }),
            ),
            Err(e) => job.failed(e),
        }
    });
    let j = state.jobs.get(&id).expect("the job");
    Ok((StatusCode::ACCEPTED, Json(j)))
}

/// `GET /jobs/{id}`
pub async fn job(State(state): State<AppState>, Path(id): Path<String>) -> ApiResult<wire::Job> {
    state
        .jobs
        .get(&id)
        .map(Json)
        .ok_or_else(|| ServiceError::not_found("Job", &id).into())
}

/// `DELETE /catalog/sources/{code}`: the source and all its entries (its
/// images stay on disk).
pub async fn delete_source(
    State(state): State<AppState>,
    Path(code): Path<String>,
) -> Result<StatusCode, ApiError> {
    state
        .with_db(move |conn| {
            catalog_dal::get_source_optional(conn, &code)
                .map_err(ServiceError::from)?
                .ok_or_else(|| ServiceError::not_found("Source", &code))?;
            catalog_dal::delete_source_cascade(conn, &code).map_err(ServiceError::from)?;
            Ok(StatusCode::NO_CONTENT)
        })
        .await
}

// ------------------------------------------------------------------ sources

/// `GET /campaigns/{id}/sources`
pub async fn campaign_sources(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> ApiResult<wire::Sources> {
    state
        .with_db(move |conn| {
            Ok(wire::Sources {
                codes: SourceService::new(conn).list_campaign_sources(&id)?,
            })
        })
        .await
        .map(Json)
}

/// `PUT /campaigns/{id}/sources`: replaces the list (empty: all).
pub async fn set_campaign_sources(
    State(state): State<AppState>,
    Path(id): Path<String>,
    JsonBody(body): JsonBody<wire::Sources>,
) -> ApiResult<wire::Sources> {
    state
        .with_db(move |conn| {
            CampaignService::new(conn)
                .get(&id)?
                .ok_or_else(|| ServiceError::not_found("Campaign", &id))?;
            check_codes(conn, &body.codes)?;
            Ok(wire::Sources {
                codes: SourceService::new(conn).set_campaign_sources(&id, &body.codes)?,
            })
        })
        .await
        .map(Json)
}

/// `GET /characters/{id}/sources`
pub async fn character_sources(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> ApiResult<wire::Sources> {
    state
        .with_db(move |conn| {
            Ok(wire::Sources {
                codes: SourceService::new(conn).list_character_sources(&id)?,
            })
        })
        .await
        .map(Json)
}

/// `PUT /characters/{id}/sources`
pub async fn set_character_sources(
    State(state): State<AppState>,
    Path(id): Path<String>,
    JsonBody(body): JsonBody<wire::Sources>,
) -> ApiResult<wire::Sources> {
    state
        .with_db(move |conn| {
            check_codes(conn, &body.codes)?;
            Ok(wire::Sources {
                codes: SourceService::new(conn).set_character_sources(&id, &body.codes)?,
            })
        })
        .await
        .map(Json)
}

fn check_codes(conn: &mut diesel::SqliteConnection, codes: &[String]) -> Result<(), ApiError> {
    for c in codes {
        if catalog_dal::get_source_optional(conn, c)
            .map_err(ServiceError::from)?
            .is_none()
        {
            return Err(ServiceError::validation(format!("no source {c:?} in the catalog")).into());
        }
    }
    Ok(())
}
