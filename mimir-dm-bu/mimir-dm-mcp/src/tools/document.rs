//! Document management tools for MCP
//!
//! Provides tools for listing, reading, writing, and creating documents
//! from templates within the active campaign.

use crate::context::McpContext;
use crate::error::McpError;
use mimir_dm_core::services::DocumentService;
use rust_mcp_sdk::schema::{Tool, ToolInputSchema};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

/// Create a simple properties map for tool input schema
fn create_properties(
    props: Vec<(&str, &str, &str)>,
) -> Option<HashMap<String, serde_json::Map<String, serde_json::Value>>> {
    let mut map = HashMap::new();
    for (name, prop_type, description) in props {
        let mut inner = serde_json::Map::new();
        inner.insert(
            "type".to_string(),
            serde_json::Value::String(prop_type.to_string()),
        );
        inner.insert(
            "description".to_string(),
            serde_json::Value::String(description.to_string()),
        );
        map.insert(name.to_string(), inner);
    }
    if map.is_empty() {
        None
    } else {
        Some(map)
    }
}

/// Document item for list responses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentListItem {
    pub id: i32,
    pub title: String,
    pub document_type: String,
    pub template_id: String,
    pub file_path: String,
    pub level: String,
    pub module_id: Option<i32>,
    pub session_id: Option<i32>,
    pub created_at: String,
    pub updated_at: String,
}

/// Input for list_documents tool
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ListDocumentsInput {
    /// Filter by level: "campaign", "module", "session", or "handout". If not specified, returns all.
    #[serde(default)]
    pub level: Option<String>,

    /// Filter by module ID (required when level is "module")
    #[serde(default)]
    pub module_id: Option<i32>,

    /// Filter by session ID (required when level is "session")
    #[serde(default)]
    pub session_id: Option<i32>,
}

impl ListDocumentsInput {
    /// Get the tool definition
    pub fn tool() -> Tool {
        Tool {
            name: "list_documents".to_string(),
            description: Some(
                "List documents in the active campaign. Can filter by level (campaign, module, session, handout), module_id, or session_id."
                    .to_string(),
            ),
            input_schema: ToolInputSchema::new(
                vec![],
                create_properties(vec![
                    ("level", "string", "Filter by level: campaign, module, session, or handout"),
                    ("module_id", "integer", "Filter by module ID (required when level is 'module')"),
                    ("session_id", "integer", "Filter by session ID (required when level is 'session')"),
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

    /// Execute the list_documents tool
    pub async fn execute(
        &self,
        context: Arc<McpContext>,
    ) -> Result<Vec<DocumentListItem>, McpError> {
        let campaign = context.require_active_campaign().await?;
        let mut conn = context.get_connection()?;
        let mut service = DocumentService::new(&mut conn);

        // Get documents based on filters
        let documents = if let Some(level) = &self.level {
            service
                .get_documents_by_level(
                    campaign.id,
                    level,
                    self.module_id,
                    self.session_id,
                )
                .map_err(|e| McpError::Service(e.to_string()))?
        } else {
            service
                .get_campaign_documents(campaign.id)
                .map_err(|e| McpError::Service(e.to_string()))?
        };

        let items: Vec<DocumentListItem> = documents
            .into_iter()
            .map(|d| {
                let level = d.level().as_str().to_string();
                DocumentListItem {
                    id: d.id,
                    title: d.title,
                    document_type: d.document_type,
                    template_id: d.template_id,
                    file_path: d.file_path,
                    level,
                    module_id: d.module_id,
                    session_id: d.session_id,
                    created_at: d.created_at,
                    updated_at: d.updated_at,
                }
            })
            .collect();

        Ok(items)
    }
}

/// Input for read_document tool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadDocumentInput {
    /// The document ID to read
    pub document_id: i32,
}

impl ReadDocumentInput {
    /// Get the tool definition
    pub fn tool() -> Tool {
        Tool {
            name: "read_document".to_string(),
            description: Some(
                "Read the content of a document by its ID. Returns the markdown content including frontmatter."
                    .to_string(),
            ),
            input_schema: ToolInputSchema::new(
                vec!["document_id".to_string()],
                create_properties(vec![
                    ("document_id", "integer", "The document ID to read"),
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

    /// Execute the read_document tool
    pub async fn execute(
        &self,
        context: Arc<McpContext>,
    ) -> Result<ReadDocumentResponse, McpError> {
        let campaign = context.require_active_campaign().await?;
        let mut conn = context.get_connection()?;
        let mut service = DocumentService::new(&mut conn);

        // Get all documents to find the one we want
        let documents = service
            .get_campaign_documents(campaign.id)
            .map_err(|e| McpError::Service(e.to_string()))?;

        let document = documents
            .into_iter()
            .find(|d| d.id == self.document_id)
            .ok_or_else(|| McpError::DocumentNotFound(format!("ID {}", self.document_id)))?;

        // Read the file content
        let content = service
            .read_document_file(&document.file_path)
            .map_err(|e| McpError::Service(e.to_string()))?;

        Ok(ReadDocumentResponse {
            id: document.id,
            title: document.title,
            document_type: document.document_type,
            file_path: document.file_path,
            content,
        })
    }
}

/// Response from read_document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadDocumentResponse {
    pub id: i32,
    pub title: String,
    pub document_type: String,
    pub file_path: String,
    pub content: String,
}

/// Input for edit_document tool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditDocumentInput {
    /// The document ID to edit
    pub document_id: i32,

    /// Text to search for in the document
    pub search: String,

    /// Text to replace the search text with
    pub replace: String,

    /// Replace all occurrences (default: false, only first match)
    #[serde(default)]
    pub replace_all: bool,
}

impl EditDocumentInput {
    /// Get the tool definition
    pub fn tool() -> Tool {
        Tool {
            name: "edit_document".to_string(),
            description: Some(
                "Edit a document using search and replace. Finds the search text and replaces it with the new text. Use replace_all=true to replace all occurrences."
                    .to_string(),
            ),
            input_schema: ToolInputSchema::new(
                vec!["document_id".to_string(), "search".to_string(), "replace".to_string()],
                create_properties(vec![
                    ("document_id", "integer", "The document ID to edit"),
                    ("search", "string", "Text to search for in the document"),
                    ("replace", "string", "Text to replace the search text with"),
                    ("replace_all", "boolean", "Replace all occurrences (default: false, only first match)"),
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

    /// Execute the edit_document tool
    pub async fn execute(
        &self,
        context: Arc<McpContext>,
    ) -> Result<EditDocumentResponse, McpError> {
        let campaign = context.require_active_campaign().await?;
        let mut conn = context.get_connection()?;
        let mut service = DocumentService::new(&mut conn);

        // Get all documents to find the one we want
        let documents = service
            .get_campaign_documents(campaign.id)
            .map_err(|e| McpError::Service(e.to_string()))?;

        let document = documents
            .into_iter()
            .find(|d| d.id == self.document_id)
            .ok_or_else(|| McpError::DocumentNotFound(format!("ID {}", self.document_id)))?;

        // Read current content
        let content = service
            .read_document_file(&document.file_path)
            .map_err(|e| McpError::Service(e.to_string()))?;

        // Check if search text exists
        if !content.contains(&self.search) {
            return Err(McpError::InvalidParameter(format!(
                "Search text not found in document: '{}'",
                if self.search.len() > 50 {
                    format!("{}...", &self.search[..50])
                } else {
                    self.search.clone()
                }
            )));
        }

        // Perform replacement
        let (new_content, replacements) = if self.replace_all {
            let count = content.matches(&self.search).count();
            (content.replace(&self.search, &self.replace), count)
        } else {
            // Replace only first occurrence
            if let Some(pos) = content.find(&self.search) {
                let mut new_content = String::with_capacity(content.len());
                new_content.push_str(&content[..pos]);
                new_content.push_str(&self.replace);
                new_content.push_str(&content[pos + self.search.len()..]);
                (new_content, 1)
            } else {
                (content, 0)
            }
        };

        // Write the updated content
        service
            .save_document_file(&document.file_path, &new_content)
            .map_err(|e| McpError::Service(e.to_string()))?;

        Ok(EditDocumentResponse {
            success: true,
            document_id: document.id,
            file_path: document.file_path,
            replacements_made: replacements,
        })
    }
}

/// Response from edit_document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditDocumentResponse {
    pub success: bool,
    pub document_id: i32,
    pub file_path: String,
    pub replacements_made: usize,
}

/// Input for create_document_from_template tool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateDocumentFromTemplateInput {
    /// The template ID to use (e.g., "campaign_pitch", "session_notes", "npc_roster")
    pub template_id: String,
}

impl CreateDocumentFromTemplateInput {
    /// Get the tool definition
    pub fn tool() -> Tool {
        Tool {
            name: "create_document_from_template".to_string(),
            description: Some(
                "Create a new document from a template. Common templates: campaign_pitch, session_notes, npc_roster, world_building, encounter_planning, player_handout."
                    .to_string(),
            ),
            input_schema: ToolInputSchema::new(
                vec!["template_id".to_string()],
                create_properties(vec![
                    ("template_id", "string", "The template ID (e.g., campaign_pitch, session_notes, npc_roster)"),
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

    /// Execute the create_document_from_template tool
    pub async fn execute(
        &self,
        context: Arc<McpContext>,
    ) -> Result<CreateDocumentResponse, McpError> {
        let campaign = context.require_active_campaign().await?;
        let mut conn = context.get_connection()?;
        let mut service = DocumentService::new(&mut conn);

        let document = service
            .create_document_from_template(campaign.id, &self.template_id)
            .map_err(|e| McpError::Service(e.to_string()))?;

        Ok(CreateDocumentResponse {
            success: true,
            document_id: document.id,
            title: document.title,
            document_type: document.document_type,
            file_path: document.file_path,
        })
    }
}

/// Response from create_document_from_template
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateDocumentResponse {
    pub success: bool,
    pub document_id: i32,
    pub title: String,
    pub document_type: String,
    pub file_path: String,
}

/// Input for list_templates tool
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ListTemplatesInput {}

impl ListTemplatesInput {
    /// Get the tool definition
    pub fn tool() -> Tool {
        Tool {
            name: "list_templates".to_string(),
            description: Some(
                "List all available document templates that can be used with create_document_from_template."
                    .to_string(),
            ),
            input_schema: ToolInputSchema::new(vec![], None, None),
            title: None,
            annotations: None,
            icons: vec![],
            execution: None,
            output_schema: None,
            meta: None,
        }
    }

    /// Execute the list_templates tool
    pub async fn execute(
        &self,
        context: Arc<McpContext>,
    ) -> Result<Vec<TemplateListItem>, McpError> {
        use mimir_dm_core::dal::campaign::template_documents::TemplateRepository;

        let mut conn = context.get_connection()?;
        let templates = TemplateRepository::get_all_active(&mut conn)
            .map_err(|e| McpError::Service(e.to_string()))?;

        let items: Vec<TemplateListItem> = templates
            .into_iter()
            .map(|t| TemplateListItem {
                id: t.document_id,
                document_type: t.document_type,
                level: t.document_level,
                purpose: t.purpose,
            })
            .collect();

        Ok(items)
    }
}

/// Template list item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateListItem {
    pub id: String,
    pub document_type: Option<String>,
    pub level: Option<String>,
    pub purpose: Option<String>,
}

/// Input for create_user_document tool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateUserDocumentInput {
    /// Title for the new document
    pub title: String,

    /// Optional initial content (markdown)
    #[serde(default)]
    pub content: Option<String>,

    /// Optional module ID to associate document with a module
    #[serde(default)]
    pub module_id: Option<i32>,
}

impl CreateUserDocumentInput {
    /// Get the tool definition
    pub fn tool() -> Tool {
        Tool {
            name: "create_user_document".to_string(),
            description: Some(
                "Create a new user document (markdown file) with a title and optional content. Use module_id to create document within a module."
                    .to_string(),
            ),
            input_schema: ToolInputSchema::new(
                vec!["title".to_string()],
                create_properties(vec![
                    ("title", "string", "Title for the new document"),
                    ("content", "string", "Optional initial markdown content"),
                    ("module_id", "integer", "Optional module ID to associate document with"),
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

    /// Execute the create_user_document tool
    pub async fn execute(
        &self,
        context: Arc<McpContext>,
    ) -> Result<CreateDocumentResponse, McpError> {
        let campaign = context.require_active_campaign().await?;
        let mut conn = context.get_connection()?;
        let mut service = DocumentService::new(&mut conn);

        let document = service
            .create_user_document(
                campaign.id,
                self.module_id,
                &self.title,
                self.content.as_deref(),
            )
            .map_err(|e| McpError::Service(e.to_string()))?;

        Ok(CreateDocumentResponse {
            success: true,
            document_id: document.id,
            title: document.title,
            document_type: document.document_type,
            file_path: document.file_path,
        })
    }
}
