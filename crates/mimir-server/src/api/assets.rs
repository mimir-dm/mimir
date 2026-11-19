//! Images of a campaign or a module (MIMIR-T-0731): handouts, portraits,
//! pictures of places. DM only (the route table). An upload is checked by
//! its extension and its first bytes; the file route serves an SVG under a
//! policy that runs no script.

use axum::extract::{Multipart, Path, State};
use axum::http::{header, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::Json;
use mimir_core::models::campaign::CampaignAsset;
use mimir_core::services::{AssetService, ModuleService, ServiceError, UploadAssetInput};
use mimir_wire as wire;

use crate::error::ApiError;
use crate::state::AppState;

type ApiResult<T> = Result<Json<T>, ApiError>;

/// The largest image upload.
pub const IMAGE_UPLOAD_LIMIT: usize = 50 * 1024 * 1024;

fn asset(a: CampaignAsset) -> wire::Asset {
    wire::Asset {
        id: a.id,
        campaign_id: a.campaign_id,
        module_id: a.module_id,
        filename: a.filename,
        title: a.description,
        mime_type: a.mime_type,
        file_size: a.file_size,
        uploaded_at: a.uploaded_at,
    }
}

/// Do the first bytes match the image type? (A wrong extension, or a file
/// that is not an image, is refused.)
pub fn looks_like(mime: &str, bytes: &[u8]) -> bool {
    match mime {
        "image/png" => bytes.starts_with(b"\x89PNG\r\n\x1a\n"),
        "image/jpeg" => bytes.starts_with(&[0xFF, 0xD8, 0xFF]),
        "image/gif" => bytes.starts_with(b"GIF87a") || bytes.starts_with(b"GIF89a"),
        "image/webp" => bytes.len() > 12 && &bytes[..4] == b"RIFF" && &bytes[8..12] == b"WEBP",
        "image/svg+xml" => {
            let head = String::from_utf8_lossy(&bytes[..bytes.len().min(4096)]).to_lowercase();
            head.contains("<svg")
        }
        _ => false,
    }
}

/// `GET /campaigns/{id}/images`: the campaign's own images.
pub async fn campaign_images(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> ApiResult<Vec<wire::Asset>> {
    let app_dir = state.config.data_dir.clone();
    state
        .with_db(move |conn| {
            Ok(AssetService::new(conn, &app_dir)
                .list_images_for_campaign(&id)?
                .into_iter()
                .map(asset)
                .collect())
        })
        .await
        .map(Json)
}

/// `GET /modules/{id}/images`: the module's images.
pub async fn module_images(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> ApiResult<Vec<wire::Asset>> {
    let app_dir = state.config.data_dir.clone();
    state
        .with_db(move |conn| {
            Ok(AssetService::new(conn, &app_dir)
                .list_for_module(&id)?
                .into_iter()
                .filter(CampaignAsset::is_image)
                .map(asset)
                .collect())
        })
        .await
        .map(Json)
}

/// `POST /campaigns/{id}/images` (multipart): `file` (PNG, JPEG, WebP,
/// GIF or SVG), `title`, and `module_id` for a module's image.
pub async fn upload_image(
    State(state): State<AppState>,
    Path(id): Path<String>,
    form: Multipart,
) -> Result<(StatusCode, Json<wire::Asset>), ApiError> {
    store(state, Some(id), None, form).await
}

/// `POST /modules/{id}/images` (multipart): as the campaign's, for this
/// module.
pub async fn upload_module_image(
    State(state): State<AppState>,
    Path(id): Path<String>,
    form: Multipart,
) -> Result<(StatusCode, Json<wire::Asset>), ApiError> {
    store(state, None, Some(id), form).await
}

/// Store an uploaded image of a campaign, or of a module (`module`, or the
/// form's `module_id`).
async fn store(
    state: AppState,
    campaign: Option<String>,
    module: Option<String>,
    mut form: Multipart,
) -> Result<(StatusCode, Json<wire::Asset>), ApiError> {
    let bad = |e: axum::extract::multipart::MultipartError| {
        ApiError::from(ServiceError::validation(format!("bad upload: {e}")))
    };
    let (mut file, mut title, mut module_id) = (None, None, None);
    while let Some(field) = form.next_field().await.map_err(bad)? {
        match field.name().unwrap_or("") {
            "file" => {
                let name = field.file_name().unwrap_or("image").to_string();
                file = Some((name, field.bytes().await.map_err(bad)?.to_vec()));
            }
            "title" => title = Some(field.text().await.map_err(bad)?),
            "module_id" => module_id = Some(field.text().await.map_err(bad)?),
            _ => {}
        }
    }
    let (filename, bytes) = file
        .ok_or_else(|| ServiceError::validation("no file: send the image as the field `file`"))?;
    let mime = wire::image_type(&filename).ok_or_else(|| {
        ServiceError::validation(format!(
            "{filename}: an image is a PNG, JPEG, WebP, GIF or SVG file"
        ))
    })?;
    if !looks_like(mime, &bytes) {
        return Err(ServiceError::validation(format!(
            "{filename}: the file is not a {mime} image"
        ))
        .into());
    }
    let title = title
        .map(|t| t.trim().to_string())
        .filter(|t| !t.is_empty());
    let module_id = module.or_else(|| {
        module_id
            .map(|m| m.trim().to_string())
            .filter(|m| !m.is_empty())
    });
    let app_dir = state.config.data_dir.clone();
    state
        .with_db(move |conn| {
            let mut input = match &module_id {
                Some(m) => {
                    let module = ModuleService::new(conn)
                        .get(m)?
                        .ok_or_else(|| ServiceError::not_found("Module", m))?;
                    if campaign.as_ref().is_some_and(|c| c != &module.campaign_id) {
                        return Err(
                            ServiceError::validation("the module is not in this campaign").into(),
                        );
                    }
                    UploadAssetInput::for_module(m, &filename, mime, bytes)
                }
                None => {
                    let id = campaign.ok_or_else(|| ServiceError::validation("no campaign"))?;
                    UploadAssetInput::for_campaign(&id, &filename, mime, bytes)
                }
            };
            if let Some(t) = title {
                input = input.with_description(t);
            }
            let created = AssetService::new(conn, &app_dir).upload(input)?;
            Ok((StatusCode::CREATED, Json(asset(created))))
        })
        .await
}

fn require_image(service: &mut AssetService<'_>, id: &str) -> Result<CampaignAsset, ApiError> {
    let a = service
        .get(id)?
        .ok_or_else(|| ServiceError::not_found("Asset", id))?;
    if !a.is_image() {
        // Map files are assets too; they go with their map.
        return Err(ServiceError::not_found("Image", id).into());
    }
    Ok(a)
}

/// `GET /assets/{id}/file`: the image. An SVG gets a policy that blocks
/// scripts, and no type sniffing.
pub async fn image_file(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Response, ApiError> {
    let app_dir = state.config.data_dir.clone();
    state
        .with_db(move |conn| {
            let mut service = AssetService::new(conn, &app_dir);
            let a = require_image(&mut service, &id)?;
            let bytes = service.read_file(&a)?;
            Ok((
                [
                    (header::CONTENT_TYPE, a.mime_type),
                    (header::CACHE_CONTROL, "private, max-age=3600".to_string()),
                    (header::X_CONTENT_TYPE_OPTIONS, "nosniff".to_string()),
                    (
                        header::CONTENT_SECURITY_POLICY,
                        "default-src 'none'; img-src data:; style-src 'unsafe-inline'; sandbox"
                            .to_string(),
                    ),
                ],
                bytes,
            )
                .into_response())
        })
        .await
}

/// `DELETE /assets/{id}`: an image and its file.
pub async fn delete_image(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, ApiError> {
    let app_dir = state.config.data_dir.clone();
    state
        .with_db(move |conn| {
            let mut service = AssetService::new(conn, &app_dir);
            require_image(&mut service, &id)?;
            service.delete(&id)?;
            Ok(StatusCode::NO_CONTENT)
        })
        .await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_bytes_must_match() {
        assert!(looks_like("image/png", b"\x89PNG\r\n\x1a\n...."));
        assert!(!looks_like("image/png", b"GIF89a"));
        assert!(looks_like("image/jpeg", &[0xFF, 0xD8, 0xFF, 0xE0]));
        assert!(looks_like("image/gif", b"GIF89a...."));
        assert!(looks_like("image/webp", b"RIFF\x00\x00\x00\x00WEBPVP8 "));
        assert!(looks_like(
            "image/svg+xml",
            b"<?xml version=\"1.0\"?><svg xmlns=\"http://www.w3.org/2000/svg\"/>"
        ));
        assert!(!looks_like("image/svg+xml", b"<html></html>"));
        assert!(!looks_like("text/plain", b"x"));
    }
}
