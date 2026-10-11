//! PDF export routes (MIMIR-T-0724), DM only: a character, a campaign, a
//! module, a map, a document, a module's monster cards. The options are
//! the query (`?include_battle_card=true&…`, the desktop's option names);
//! the answer is `application/pdf`.

use axum::body::Body;
use axum::extract::{Path, Query, State};
use axum::http::header;
use axum::response::{IntoResponse, Response};
use base64::Engine;
use mimir_core::services::ServiceError;
use mimir_print::PrintState;

use crate::error::ApiError;
use crate::print::{
    self, ApiResponse, CampaignExportOptions, CharacterExportOptions, MapPrintOptions,
    ModuleExportOptions, PrintCtx, PrintResult,
};
use crate::state::AppState;

fn ctx(state: &AppState) -> (PrintCtx, PrintState) {
    let url = state.config.database_path().to_string_lossy().into_owned();
    let data = state.config.data_dir.clone();
    let ps = PrintState {
        templates_dir: data.join("templates"),
        assets_dir: data.join("assets"),
    };
    (PrintCtx::new(url, data), ps)
}

/// Run a print function off the async runtime; its result as a PDF
/// download named `file`.
async fn pdf(
    state: &AppState,
    file: String,
    work: impl FnOnce(&PrintCtx, &PrintState) -> ApiResponse<PrintResult> + Send + 'static,
) -> Result<Response, ApiError> {
    let (c, ps) = ctx(state);
    let r = tokio::task::spawn_blocking(move || work(&c, &ps))
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?;
    let Some(result) = r.data else {
        let msg = r
            .error
            .unwrap_or_else(|| "the PDF could not be made".into());
        return Err(if msg.to_lowercase().contains("not found") {
            ServiceError::not_found("Print", msg).into()
        } else {
            ApiError::internal(msg)
        });
    };
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(result.pdf_base64)
        .map_err(|e| ApiError::internal(e.to_string()))?;
    let safe: String = file
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '.' {
                c
            } else {
                '-'
            }
        })
        .collect();
    Ok((
        [
            (header::CONTENT_TYPE, "application/pdf".to_string()),
            (
                header::CONTENT_DISPOSITION,
                format!("attachment; filename=\"{safe}\""),
            ),
        ],
        Body::from(bytes),
    )
        .into_response())
}

/// `GET /characters/{id}/pdf`
pub async fn character(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Query(o): Query<CharacterExportOptions>,
) -> Result<Response, ApiError> {
    pdf(&state, format!("character-{id}.pdf"), move |c, p| {
        print::export_character(c, p, id, Some(o))
    })
    .await
}

/// `GET /campaigns/{id}/pdf`
pub async fn campaign(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Query(o): Query<CampaignExportOptions>,
) -> Result<Response, ApiError> {
    pdf(&state, format!("campaign-{id}.pdf"), move |c, p| {
        print::export_campaign_documents(c, p, id, Some(o))
    })
    .await
}

/// `GET /modules/{id}/pdf`
pub async fn module(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Query(o): Query<ModuleExportOptions>,
) -> Result<Response, ApiError> {
    pdf(&state, format!("module-{id}.pdf"), move |c, p| {
        print::export_module_documents(c, p, id, Some(o))
    })
    .await
}

/// `GET /maps/{id}/pdf`
pub async fn map(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Query(o): Query<MapPrintOptions>,
) -> Result<Response, ApiError> {
    pdf(&state, format!("map-{id}.pdf"), move |c, p| {
        print::print_map(c, p, id, Some(o))
    })
    .await
}

/// `GET /documents/{id}/pdf`
pub async fn document(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Response, ApiError> {
    pdf(&state, format!("document-{id}.pdf"), move |c, p| {
        print::export_campaign_document(c, p, id)
    })
    .await
}

/// `GET /modules/{id}/monster-cards`
pub async fn monster_cards(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Query(o): Query<print::MonsterExportOptions>,
) -> Result<Response, ApiError> {
    pdf(&state, format!("monsters-{id}.pdf"), move |c, p| {
        print::export_module_monsters(c, p, id, Some(o))
    })
    .await
}
