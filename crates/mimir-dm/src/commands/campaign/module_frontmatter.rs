//! Module frontmatter sync commands.
//!
//! Provides Tauri commands for syncing module document frontmatter with database.
//! Document is the source of truth - sync flows in both directions.

use crate::state::AppState;
use crate::types::{ApiError, ApiResponse};
use mimir_dm_core::services::module_frontmatter_service::SyncResult;
use mimir_dm_core::services::ModuleFrontmatterService;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tauri::State;
use tracing::{error, info, warn};

#[derive(Debug, Serialize, Deserialize)]
pub struct SyncFromDocumentRequest {
    pub module_id: i32,
    pub document_path: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SyncToDocumentRequest {
    pub module_id: i32,
    pub document_path: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SyncResultResponse {
    pub monsters_synced: usize,
    pub npcs_synced: usize,
    pub items_synced: usize,
    pub warnings: Vec<String>,
}

impl From<SyncResult> for SyncResultResponse {
    fn from(result: SyncResult) -> Self {
        Self {
            monsters_synced: result.monsters_synced,
            npcs_synced: result.npcs_synced,
            items_synced: result.items_synced,
            warnings: result.warnings,
        }
    }
}

/// Sync module data from document frontmatter to database.
///
/// Parses the module's overview document, extracts frontmatter,
/// and updates the database tables (module_monsters, module_npcs, module_items).
///
/// Uses REPLACE strategy: clears existing data and inserts from frontmatter.
#[tauri::command]
pub async fn sync_module_from_document(
    request: SyncFromDocumentRequest,
    state: State<'_, AppState>,
) -> Result<ApiResponse<SyncResultResponse>, ApiError> {
    info!(
        "Syncing module {} from document: {}",
        request.module_id, request.document_path
    );

    let document_path = PathBuf::from(&request.document_path);

    if !document_path.exists() {
        return Ok(ApiResponse::error(format!(
            "Document not found: {}",
            request.document_path
        )));
    }

    let mut conn = state.db.get_connection()?;
    let mut service = ModuleFrontmatterService::new(&mut conn);

    match service.sync_from_document(request.module_id, &document_path) {
        Ok(result) => {
            // Log warnings
            for warning in &result.warnings {
                warn!("Sync warning: {}", warning);
            }

            info!(
                "Synced from document: {} monsters, {} NPCs, {} items",
                result.monsters_synced, result.npcs_synced, result.items_synced
            );
            Ok(ApiResponse::success(result.into()))
        }
        Err(e) => {
            error!("Failed to sync from document: {}", e);
            Ok(ApiResponse::error(format!(
                "Failed to sync from document: {}",
                e
            )))
        }
    }
}

/// Sync module data from database to document frontmatter.
///
/// Reads current monsters/NPCs/items from database and updates
/// the frontmatter section of the document.
#[tauri::command]
pub async fn sync_module_to_document(
    request: SyncToDocumentRequest,
    state: State<'_, AppState>,
) -> Result<ApiResponse<()>, ApiError> {
    info!(
        "Syncing module {} to document: {}",
        request.module_id, request.document_path
    );

    let document_path = PathBuf::from(&request.document_path);

    if !document_path.exists() {
        return Ok(ApiResponse::error(format!(
            "Document not found: {}",
            request.document_path
        )));
    }

    let mut conn = state.db.get_connection()?;
    let mut service = ModuleFrontmatterService::new(&mut conn);

    match service.sync_to_document(request.module_id, &document_path) {
        Ok(()) => {
            info!("Successfully synced to document");
            Ok(ApiResponse::success(()))
        }
        Err(e) => {
            error!("Failed to sync to document: {}", e);
            Ok(ApiResponse::error(format!(
                "Failed to sync to document: {}",
                e
            )))
        }
    }
}
