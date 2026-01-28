//! Document Tools
//!
//! MCP tools for managing module documents (narrative content).

use mimir_core::services::{CreateDocumentInput, DocumentService, UpdateDocumentInput};
use rust_mcp_sdk::schema::{Tool, ToolInputSchema};
use serde_json::{json, Value};
use std::sync::Arc;

use super::create_properties;
use crate::context::McpContext;
use crate::McpError;

// =============================================================================
// Tool Definitions
// =============================================================================

pub fn list_documents_tool() -> Tool {
    Tool {
        name: "list_documents".to_string(),
        description: Some("List all documents in a module".to_string()),
        input_schema: ToolInputSchema::new(
            vec!["module_id".to_string()],
            create_properties(vec![("module_id", "string", "The ID of the module")]),
            None,
        ),
        title: None,
        annotations: None,
        icons: vec![],
        execution: None,
        output_schema: None,
        meta: None,
    }
}

pub fn read_document_tool() -> Tool {
    Tool {
        name: "read_document".to_string(),
        description: Some("Read the full content of a document".to_string()),
        input_schema: ToolInputSchema::new(
            vec!["document_id".to_string()],
            create_properties(vec![("document_id", "string", "The ID of the document")]),
            None,
        ),
        title: None,
        annotations: None,
        icons: vec![],
        execution: None,
        output_schema: None,
        meta: None,
    }
}

pub fn create_document_tool() -> Tool {
    Tool {
        name: "create_document".to_string(),
        description: Some("Create a new document in a module".to_string()),
        input_schema: ToolInputSchema::new(
            vec![
                "module_id".to_string(),
                "title".to_string(),
                "document_type".to_string(),
            ],
            create_properties(vec![
                ("module_id", "string", "The ID of the module"),
                ("title", "string", "Title of the document"),
                (
                    "document_type",
                    "string",
                    "Type: backstory, read_aloud, dm_notes, description, custom",
                ),
                ("content", "string", "Initial content of the document"),
            ]),
            None,
        ),
        title: None,
        annotations: None,
        icons: vec![],
        execution: None,
        output_schema: None,
        meta: None,
    }
}

pub fn edit_document_tool() -> Tool {
    Tool {
        name: "edit_document".to_string(),
        description: Some("Edit a document using search and replace".to_string()),
        input_schema: ToolInputSchema::new(
            vec![
                "document_id".to_string(),
                "search".to_string(),
                "replace".to_string(),
            ],
            create_properties(vec![
                ("document_id", "string", "The ID of the document"),
                ("search", "string", "Text to search for"),
                ("replace", "string", "Text to replace with"),
            ]),
            None,
        ),
        title: None,
        annotations: None,
        icons: vec![],
        execution: None,
        output_schema: None,
        meta: None,
    }
}

// =============================================================================
// Tool Implementations
// =============================================================================

pub async fn list_documents(ctx: &Arc<McpContext>, args: Value) -> Result<Value, McpError> {
    let module_id = args
        .get("module_id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| McpError::InvalidArguments("module_id is required".to_string()))?;

    let mut db = ctx.db()?;
    let mut service = DocumentService::new(&mut db);

    let documents = service
        .list_for_module(module_id)
        .map_err(|e| McpError::Internal(e.to_string()))?;

    let doc_data: Vec<Value> = documents
        .iter()
        .map(|d| {
            json!({
                "id": d.id,
                "title": d.title,
                "doc_type": d.doc_type
            })
        })
        .collect();

    Ok(json!({
        "module_id": module_id,
        "documents": doc_data
    }))
}

pub async fn read_document(ctx: &Arc<McpContext>, args: Value) -> Result<Value, McpError> {
    let document_id = args
        .get("document_id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| McpError::InvalidArguments("document_id is required".to_string()))?;

    let mut db = ctx.db()?;
    let mut service = DocumentService::new(&mut db);

    let document = service
        .get(document_id)
        .map_err(|e| McpError::Internal(e.to_string()))?
        .ok_or_else(|| {
            McpError::InvalidArguments(format!("Document '{}' not found", document_id))
        })?;

    Ok(json!({
        "document_id": document.id,
        "title": document.title,
        "doc_type": document.doc_type,
        "content": document.content,
        "module_id": document.module_id
    }))
}

pub async fn create_document(ctx: &Arc<McpContext>, args: Value) -> Result<Value, McpError> {
    let campaign_id = ctx
        .get_active_campaign_id()
        .ok_or(McpError::NoActiveCampaign)?;

    let module_id = args
        .get("module_id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| McpError::InvalidArguments("module_id is required".to_string()))?;

    let title = args
        .get("title")
        .and_then(|v| v.as_str())
        .ok_or_else(|| McpError::InvalidArguments("title is required".to_string()))?;

    let document_type = args
        .get("document_type")
        .and_then(|v| v.as_str())
        .ok_or_else(|| McpError::InvalidArguments("document_type is required".to_string()))?;

    let content = args.get("content").and_then(|v| v.as_str());

    let mut db = ctx.db()?;
    let mut service = DocumentService::new(&mut db);

    let mut input = CreateDocumentInput::for_module(&campaign_id, module_id, title)
        .with_type(document_type);

    if let Some(c) = content {
        input = input.with_content(c);
    }

    let document = service
        .create(input)
        .map_err(|e| McpError::Internal(e.to_string()))?;

    Ok(json!({
        "status": "created",
        "document": {
            "id": document.id,
            "title": document.title,
            "doc_type": document.doc_type,
            "content": document.content
        }
    }))
}

pub async fn edit_document(ctx: &Arc<McpContext>, args: Value) -> Result<Value, McpError> {
    let document_id = args
        .get("document_id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| McpError::InvalidArguments("document_id is required".to_string()))?;

    let search = args
        .get("search")
        .and_then(|v| v.as_str())
        .ok_or_else(|| McpError::InvalidArguments("search is required".to_string()))?;

    let replace = args
        .get("replace")
        .and_then(|v| v.as_str())
        .ok_or_else(|| McpError::InvalidArguments("replace is required".to_string()))?;

    let mut db = ctx.db()?;
    let mut service = DocumentService::new(&mut db);

    // Get the current document
    let document = service
        .get(document_id)
        .map_err(|e| McpError::Internal(e.to_string()))?
        .ok_or_else(|| {
            McpError::InvalidArguments(format!("Document '{}' not found", document_id))
        })?;

    // Perform search and replace on content
    if !document.content.contains(search) {
        return Err(McpError::InvalidArguments(format!(
            "Search string not found in document content"
        )));
    }

    let new_content = document.content.replace(search, replace);

    // Update the document
    let update = UpdateDocumentInput::set_content(new_content);
    let updated = service
        .update(document_id, update)
        .map_err(|e| McpError::Internal(e.to_string()))?;

    Ok(json!({
        "status": "updated",
        "document": {
            "id": updated.id,
            "title": updated.title,
            "doc_type": updated.doc_type,
            "content": updated.content
        }
    }))
}
