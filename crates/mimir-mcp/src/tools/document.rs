//! Document Tools
//!
//! MCP tools for managing documents (campaign-level and module-level narrative
//! content). This family is registry-based: typed argument structs, handlers,
//! and one entry each in `registered_tools()`.

use mimir_core::services::{CreateDocumentInput, DocumentService, UpdateDocumentInput};
use serde_json::{json, Value};
use std::sync::Arc;

use crate::context::McpContext;
use crate::registry::RegisteredTool;
use crate::response::McpResponse;
use crate::{tool, tool_args, McpError};

// =============================================================================
// Registration
// =============================================================================

/// All document-family tools.
pub fn registered_tools() -> Vec<RegisteredTool> {
    vec![
        tool!(
            "list_documents",
            "List all documents in a module. If module_id is omitted, lists campaign-level documents (requires active campaign).",
            ListDocumentsArgs,
            list_documents
        ),
        tool!(
            "read_document",
            "Read the full content of a document",
            ReadDocumentArgs,
            read_document
        ),
        tool!(
            "create_document",
            "Create a new document in a module or at the campaign level. Omit module_id for a campaign-level document.",
            CreateDocumentArgs,
            create_document
        ),
        tool!(
            "edit_document",
            "Edit a document using search and replace",
            EditDocumentArgs,
            edit_document
        ),
        tool!(
            "reorder_document",
            "Swap sort order between two documents to reorder them.",
            ReorderDocumentArgs,
            reorder_document
        ),
        tool!(
            "delete_document",
            "Delete a document",
            DeleteDocumentArgs,
            delete_document
        ),
    ]
}

// =============================================================================
// Arguments
// =============================================================================

tool_args! {
    pub struct ListDocumentsArgs {
        /// The ID of the module (optional — omit for campaign-level documents)
        pub module_id: Option<String>,
    }
}

tool_args! {
    pub struct ReadDocumentArgs {
        /// The ID of the document
        pub document_id: String,
    }
}

tool_args! {
    pub struct CreateDocumentArgs {
        /// The ID of the module (optional — omit for campaign-level documents)
        pub module_id: Option<String>,
        /// Title of the document
        pub title: String,
        /// Type: backstory, read_aloud, dm_notes, description, custom
        pub document_type: String,
        /// Initial content of the document
        pub content: Option<String>,
    }
}

tool_args! {
    pub struct EditDocumentArgs {
        /// The ID of the document
        pub document_id: String,
        /// Text to search for
        pub search: String,
        /// Text to replace with
        pub replace: String,
    }
}

tool_args! {
    pub struct ReorderDocumentArgs {
        /// The ID of the document to move
        pub document_id: String,
        /// The ID of the document to swap with
        pub swap_with_id: String,
    }
}

tool_args! {
    pub struct DeleteDocumentArgs {
        /// The ID of the document to delete
        pub document_id: String,
    }
}

// =============================================================================
// Handlers
// =============================================================================

pub async fn list_documents(
    ctx: &Arc<McpContext>,
    args: ListDocumentsArgs,
) -> Result<Value, McpError> {
    let module_id = args.module_id.as_deref();

    let mut db = ctx.connect()?;
    let mut service = DocumentService::new(&mut db);

    let documents = if let Some(mid) = module_id {
        service
            .list_for_module(mid)
            .map_err(|e| McpError::Internal(e.to_string()))?
    } else {
        let campaign_id = ctx
            .get_active_campaign_id()
            .ok_or(McpError::NoActiveCampaign)?;
        service
            .list_for_campaign(&campaign_id)
            .map_err(|e| McpError::Internal(e.to_string()))?
    };

    let doc_data: Vec<Value> = documents
        .iter()
        .map(|d| {
            json!({
                "id": d.id,
                "title": d.title,
                "doc_type": d.doc_type,
                "module_id": d.module_id
            })
        })
        .collect();

    McpResponse::ok(json!({
        "module_id": module_id,
        "documents": doc_data
    }))
}

pub async fn read_document(
    ctx: &Arc<McpContext>,
    args: ReadDocumentArgs,
) -> Result<Value, McpError> {
    let document_id = &args.document_id;

    let mut db = ctx.connect()?;
    let mut service = DocumentService::new(&mut db);

    let document = service
        .get(document_id)
        .map_err(|e| McpError::Internal(e.to_string()))?
        .ok_or_else(|| {
            McpError::InvalidArguments(format!("Document '{}' not found", document_id))
        })?;

    McpResponse::get("document", json!({
        "id": document.id,
        "title": document.title,
        "doc_type": document.doc_type,
        "content": document.content,
        "module_id": document.module_id
    }))
}

pub async fn create_document(
    ctx: &Arc<McpContext>,
    args: CreateDocumentArgs,
) -> Result<Value, McpError> {
    let campaign_id = ctx
        .get_active_campaign_id()
        .ok_or(McpError::NoActiveCampaign)?;

    let mut db = ctx.connect()?;
    let mut service = DocumentService::new(&mut db);

    let mut input = if let Some(mid) = args.module_id.as_deref() {
        CreateDocumentInput::for_module(&campaign_id, mid, &args.title)
    } else {
        CreateDocumentInput::for_campaign(&campaign_id, &args.title)
    }
    .with_type(&args.document_type);

    if let Some(c) = args.content {
        input = input.with_content(c);
    }

    let document = service
        .create(input)
        .map_err(|e| McpError::Internal(e.to_string()))?;

    McpResponse::created("document", json!({
        "id": document.id,
        "title": document.title,
        "doc_type": document.doc_type,
        "content": document.content
    }))
}

pub async fn edit_document(
    ctx: &Arc<McpContext>,
    args: EditDocumentArgs,
) -> Result<Value, McpError> {
    let document_id = &args.document_id;

    let mut db = ctx.connect()?;
    let mut service = DocumentService::new(&mut db);

    // Get the current document
    let document = service
        .get(document_id)
        .map_err(|e| McpError::Internal(e.to_string()))?
        .ok_or_else(|| {
            McpError::InvalidArguments(format!("Document '{}' not found", document_id))
        })?;

    // Perform search and replace on content (MCP-only convenience; the
    // service seam is a plain content update)
    if !document.content.contains(&args.search) {
        return Err(McpError::InvalidArguments(
            "Search string not found in document content".to_string(),
        ));
    }

    let new_content = document.content.replace(&args.search, &args.replace);

    // Update the document
    let update = UpdateDocumentInput::set_content(new_content);
    let updated = service
        .update(document_id, update)
        .map_err(|e| McpError::Internal(e.to_string()))?;

    McpResponse::updated("document", json!({
        "id": updated.id,
        "title": updated.title,
        "doc_type": updated.doc_type,
        "content": updated.content
    }))
}

pub async fn reorder_document(
    ctx: &Arc<McpContext>,
    args: ReorderDocumentArgs,
) -> Result<Value, McpError> {
    let mut db = ctx.connect()?;
    let mut service = DocumentService::new(&mut db);

    let documents = service
        .swap_order(&args.document_id, &args.swap_with_id)
        .map_err(|e| McpError::Internal(e.to_string()))?;

    let doc_data: Vec<Value> = documents
        .iter()
        .map(|d| {
            json!({
                "id": d.id,
                "title": d.title,
                "doc_type": d.doc_type,
                "sort_order": d.sort_order
            })
        })
        .collect();

    McpResponse::ok(json!({
        "documents": doc_data
    }))
}

pub async fn delete_document(
    ctx: &Arc<McpContext>,
    args: DeleteDocumentArgs,
) -> Result<Value, McpError> {
    let mut db = ctx.connect()?;
    let mut service = DocumentService::new(&mut db);

    service
        .delete(&args.document_id)
        .map_err(|e| McpError::Internal(e.to_string()))?;

    McpResponse::deleted(&args.document_id)
}
