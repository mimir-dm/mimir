//! MCP Server Handler
//!
//! Implements the ServerHandler trait to route tool calls to appropriate handlers.

use async_trait::async_trait;
use rust_mcp_sdk::mcp_server::ServerHandler;
use rust_mcp_sdk::schema::{
    CallToolRequestParams, CallToolResult, ContentBlock, ListToolsResult, PaginatedRequestParams,
    RpcError, Tool,
};
use rust_mcp_sdk::McpServer;
use serde_json::Value;
use std::sync::Arc;
use tracing::{error, info};

use crate::context::McpContext;
use crate::tools;
use crate::McpError;

/// Mimir MCP Server Handler.
///
/// Routes incoming MCP requests to the appropriate tool implementations.
pub struct MimirHandler {
    context: Arc<McpContext>,
}

impl MimirHandler {
    /// Create a new handler with initialized context.
    pub fn new() -> Result<Self, McpError> {
        let context = McpContext::new()?;
        Ok(Self {
            context: Arc::new(context),
        })
    }

    /// Create a handler with an existing context.
    pub fn with_context(context: Arc<McpContext>) -> Self {
        Self { context }
    }

    /// Get the list of available tools.
    fn get_tools() -> Vec<Tool> {
        vec![
            // Campaign tools
            tools::campaign::list_campaigns_tool(),
            tools::campaign::set_active_campaign_tool(),
            tools::campaign::get_campaign_details_tool(),
            tools::campaign::get_campaign_sources_tool(),
            tools::campaign::export_campaign_tool(),
            tools::campaign::import_campaign_tool(),
            tools::campaign::preview_archive_tool(),
            // Module tools
            tools::module::create_module_tool(),
            tools::module::list_modules_tool(),
            tools::module::get_module_details_tool(),
            tools::module::add_monster_to_module_tool(),
            tools::module::add_item_to_module_tool(),
            // Document tools
            tools::document::list_documents_tool(),
            tools::document::read_document_tool(),
            tools::document::create_document_tool(),
            tools::document::edit_document_tool(),
            // Character tools
            tools::character::list_characters_tool(),
            tools::character::get_character_tool(),
            tools::character::create_character_tool(),
            tools::character::edit_character_tool(),
            tools::character::add_item_to_character_tool(),
            // Catalog tools
            tools::catalog::search_monsters_tool(),
            tools::catalog::search_items_tool(),
            tools::catalog::search_spells_tool(),
        ]
    }

    /// Route a tool call to the appropriate handler.
    async fn execute_tool(&self, name: &str, args: Value) -> Result<Value, McpError> {
        match name {
            // Campaign tools
            "list_campaigns" => tools::campaign::list_campaigns(&self.context, args).await,
            "set_active_campaign" => {
                tools::campaign::set_active_campaign(&self.context, args).await
            }
            "get_campaign_details" => {
                tools::campaign::get_campaign_details(&self.context, args).await
            }
            "get_campaign_sources" => {
                tools::campaign::get_campaign_sources(&self.context, args).await
            }
            "export_campaign" => tools::campaign::export_campaign(&self.context, args).await,
            "import_campaign" => tools::campaign::import_campaign(&self.context, args).await,
            "preview_archive" => tools::campaign::preview_archive(&self.context, args).await,

            // Module tools
            "create_module" => tools::module::create_module(&self.context, args).await,
            "list_modules" => tools::module::list_modules(&self.context, args).await,
            "get_module_details" => tools::module::get_module_details(&self.context, args).await,
            "add_monster_to_module" => {
                tools::module::add_monster_to_module(&self.context, args).await
            }
            "add_item_to_module" => tools::module::add_item_to_module(&self.context, args).await,

            // Document tools
            "list_documents" => tools::document::list_documents(&self.context, args).await,
            "read_document" => tools::document::read_document(&self.context, args).await,
            "create_document" => tools::document::create_document(&self.context, args).await,
            "edit_document" => tools::document::edit_document(&self.context, args).await,

            // Character tools
            "list_characters" => tools::character::list_characters(&self.context, args).await,
            "get_character" => tools::character::get_character(&self.context, args).await,
            "create_character" => tools::character::create_character(&self.context, args).await,
            "edit_character" => tools::character::edit_character(&self.context, args).await,
            "add_item_to_character" => {
                tools::character::add_item_to_character(&self.context, args).await
            }

            // Catalog tools
            "search_monsters" => tools::catalog::search_monsters(&self.context, args).await,
            "search_items" => tools::catalog::search_items(&self.context, args).await,
            "search_spells" => tools::catalog::search_spells(&self.context, args).await,

            _ => Err(McpError::ToolNotFound(name.to_string())),
        }
    }
}

#[async_trait]
impl ServerHandler for MimirHandler {
    async fn handle_list_tools_request(
        &self,
        _params: Option<PaginatedRequestParams>,
        _runtime: Arc<dyn McpServer>,
    ) -> Result<ListToolsResult, RpcError> {
        info!("Handling list_tools request");
        Ok(ListToolsResult {
            tools: Self::get_tools(),
            meta: None,
            next_cursor: None,
        })
    }

    async fn handle_call_tool_request(
        &self,
        params: CallToolRequestParams,
        _runtime: Arc<dyn McpServer>,
    ) -> Result<CallToolResult, rust_mcp_sdk::schema::schema_utils::CallToolError> {
        info!(tool = %params.name, "Handling call_tool request");

        let args = params
            .arguments
            .map(|m| Value::Object(m))
            .unwrap_or(Value::Object(Default::default()));

        match self.execute_tool(&params.name, args).await {
            Ok(result) => {
                let content = vec![ContentBlock::text_content(
                    serde_json::to_string_pretty(&result).unwrap_or_default(),
                )];
                Ok(CallToolResult {
                    content,
                    is_error: None,
                    meta: None,
                    structured_content: None,
                })
            }
            Err(e) => {
                error!(tool = %params.name, error = %e, "Tool execution failed");
                let content = vec![ContentBlock::text_content(format!("Error: {}", e))];
                Ok(CallToolResult {
                    content,
                    is_error: Some(true),
                    meta: None,
                    structured_content: None,
                })
            }
        }
    }
}
