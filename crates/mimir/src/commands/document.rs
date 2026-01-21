//! Document Commands
//!
//! Tauri commands for document management (campaign and module markdown content).

use mimir_core::dal::campaign::DocumentSearchResult;
use mimir_core::models::campaign::Document;
use mimir_core::services::{CreateDocumentInput, DocumentService, UpdateDocumentInput};
use tauri::State;

use super::{to_api_response, ApiResponse};
use crate::state::AppState;

// =============================================================================
// List Commands
// =============================================================================

/// List campaign-level documents (not in any module).
#[tauri::command]
pub fn list_campaign_documents(
    state: State<'_, AppState>,
    campaign_id: String,
) -> ApiResponse<Vec<Document>> {
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
    };

    let result = DocumentService::new(&mut db).list_for_campaign(&campaign_id);
    to_api_response(result)
}

/// List all documents for a specific module.
#[tauri::command]
pub fn list_module_documents(
    state: State<'_, AppState>,
    module_id: String,
) -> ApiResponse<Vec<Document>> {
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
    };

    let result = DocumentService::new(&mut db).list_for_module(&module_id);
    to_api_response(result)
}

// =============================================================================
// CRUD Commands
// =============================================================================

/// Get a document by ID.
#[tauri::command]
pub fn get_document(state: State<'_, AppState>, id: String) -> ApiResponse<Document> {
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
    };

    let result = DocumentService::new(&mut db).get(&id);
    match result {
        Ok(Some(document)) => ApiResponse::ok(document),
        Ok(None) => ApiResponse::err(format!("Document not found: {}", id)),
        Err(e) => ApiResponse::err(e.to_string()),
    }
}

/// Request for creating a document.
#[derive(Debug, serde::Deserialize)]
pub struct CreateDocumentRequest {
    pub campaign_id: String,
    pub module_id: Option<String>,
    pub title: String,
    pub doc_type: Option<String>,
    pub content: Option<String>,
}

/// Create a new document.
#[tauri::command]
pub fn create_document(
    state: State<'_, AppState>,
    request: CreateDocumentRequest,
) -> ApiResponse<Document> {
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
    };

    let mut input = if let Some(module_id) = request.module_id {
        CreateDocumentInput::for_module(&request.campaign_id, module_id, &request.title)
    } else {
        CreateDocumentInput::for_campaign(&request.campaign_id, &request.title)
    };

    if let Some(doc_type) = request.doc_type {
        input = input.with_type(doc_type);
    }

    if let Some(content) = request.content {
        input = input.with_content(content);
    }

    let result = DocumentService::new(&mut db).create(input);
    to_api_response(result)
}

/// Request for updating a document.
#[derive(Debug, serde::Deserialize)]
pub struct UpdateDocumentRequest {
    pub title: Option<String>,
    pub content: Option<String>,
    pub doc_type: Option<String>,
}

/// Update a document.
#[tauri::command]
pub fn update_document(
    state: State<'_, AppState>,
    id: String,
    request: UpdateDocumentRequest,
) -> ApiResponse<Document> {
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
    };

    let input = UpdateDocumentInput {
        title: request.title,
        content: request.content,
        doc_type: request.doc_type,
    };

    let result = DocumentService::new(&mut db).update(&id, input);
    to_api_response(result)
}

/// Delete a document permanently.
#[tauri::command]
pub fn delete_document(state: State<'_, AppState>, id: String) -> ApiResponse<()> {
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
    };

    let result = DocumentService::new(&mut db).delete(&id);
    to_api_response(result)
}

// =============================================================================
// Search Commands
// =============================================================================

/// Search documents using full-text search.
#[tauri::command]
pub fn search_documents(
    state: State<'_, AppState>,
    campaign_id: String,
    query: String,
) -> ApiResponse<Vec<DocumentSearchResult>> {
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
    };

    let result = DocumentService::new(&mut db).search(&campaign_id, &query);
    to_api_response(result)
}

/// Search documents within a specific module.
#[tauri::command]
pub fn search_module_documents(
    state: State<'_, AppState>,
    module_id: String,
    query: String,
) -> ApiResponse<Vec<DocumentSearchResult>> {
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
    };

    let result = DocumentService::new(&mut db).search_in_module(&module_id, &query);
    to_api_response(result)
}
