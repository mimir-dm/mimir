//! MCP server handler implementation
//!
//! Implements the ServerHandler trait to handle MCP protocol requests.

use crate::context::McpContext;
use crate::tools::{
    AddItemToCharacterInput, AddItemToModuleInput, AddMonsterToModuleInput, AssignNpcToModuleInput,
    CreateModuleInput, CreateNpcInput, CreateUserDocumentInput, EditDocumentInput,
    GetCharacterInput, GetModuleDetailsInput, ListCampaignsInput, ListCharactersInput,
    ListDocumentsInput, ListModulesInput, ReadDocumentInput, SearchItemsInput, SearchMonstersInput,
    SearchTrapsInput, SetActiveCampaignInput, UpdateCharacterCurrencyInput,
};
use async_trait::async_trait;
use rust_mcp_sdk::mcp_server::ServerHandler;
use rust_mcp_sdk::schema::{
    schema_utils::CallToolError, CallToolRequestParams, CallToolResult, ContentBlock,
    ListToolsResult, PaginatedRequestParams, RpcError, Tool,
};
use rust_mcp_sdk::McpServer;
use std::sync::Arc;
use tracing::{error, info};

/// Mimir MCP server handler
pub struct MimirHandler {
    context: Arc<McpContext>,
}

impl MimirHandler {
    /// Create a new Mimir handler with the given context
    pub fn new(context: Arc<McpContext>) -> Self {
        Self { context }
    }

    /// Get all available tools
    fn get_tools() -> Vec<Tool> {
        vec![
            // Campaign tools
            ListCampaignsInput::tool(),
            SetActiveCampaignInput::tool(),
            // Module tools
            CreateModuleInput::tool(),
            ListModulesInput::tool(),
            GetModuleDetailsInput::tool(),
            AddMonsterToModuleInput::tool(),
            AddItemToModuleInput::tool(),
            // Document tools
            ListDocumentsInput::tool(),
            ReadDocumentInput::tool(),
            EditDocumentInput::tool(),
            CreateUserDocumentInput::tool(),
            // Character tools
            ListCharactersInput::tool(),
            GetCharacterInput::tool(),
            CreateNpcInput::tool(),
            AssignNpcToModuleInput::tool(),
            AddItemToCharacterInput::tool(),
            UpdateCharacterCurrencyInput::tool(),
            // Catalog search tools
            SearchMonstersInput::tool(),
            SearchItemsInput::tool(),
            SearchTrapsInput::tool(),
        ]
    }

    /// Execute a tool by name
    async fn execute_tool(
        &self,
        tool_name: &str,
        arguments: serde_json::Map<String, serde_json::Value>,
    ) -> Result<serde_json::Value, String> {
        let args_value = serde_json::Value::Object(arguments);

        match tool_name {
            // Campaign tools
            "list_campaigns" => {
                let input: ListCampaignsInput =
                    serde_json::from_value(args_value).map_err(|e| e.to_string())?;
                let result = input
                    .execute(self.context.clone())
                    .await
                    .map_err(|e| e.to_string())?;
                serde_json::to_value(result).map_err(|e| e.to_string())
            }
            "set_active_campaign" => {
                let input: SetActiveCampaignInput =
                    serde_json::from_value(args_value).map_err(|e| e.to_string())?;
                let result = input
                    .execute(self.context.clone())
                    .await
                    .map_err(|e| e.to_string())?;
                serde_json::to_value(result).map_err(|e| e.to_string())
            }
            "create_module" => {
                let input: CreateModuleInput =
                    serde_json::from_value(args_value).map_err(|e| e.to_string())?;
                let result = input
                    .execute(self.context.clone())
                    .await
                    .map_err(|e| e.to_string())?;
                serde_json::to_value(result).map_err(|e| e.to_string())
            }
            "list_modules" => {
                let input: ListModulesInput =
                    serde_json::from_value(args_value).map_err(|e| e.to_string())?;
                let result = input
                    .execute(self.context.clone())
                    .await
                    .map_err(|e| e.to_string())?;
                serde_json::to_value(result).map_err(|e| e.to_string())
            }
            "get_module_details" => {
                let input: GetModuleDetailsInput =
                    serde_json::from_value(args_value).map_err(|e| e.to_string())?;
                let result = input
                    .execute(self.context.clone())
                    .await
                    .map_err(|e| e.to_string())?;
                serde_json::to_value(result).map_err(|e| e.to_string())
            }
            "add_monster_to_module" => {
                let input: AddMonsterToModuleInput =
                    serde_json::from_value(args_value).map_err(|e| e.to_string())?;
                let result = input
                    .execute(self.context.clone())
                    .await
                    .map_err(|e| e.to_string())?;
                serde_json::to_value(result).map_err(|e| e.to_string())
            }
            "add_item_to_module" => {
                let input: AddItemToModuleInput =
                    serde_json::from_value(args_value).map_err(|e| e.to_string())?;
                let result = input
                    .execute(self.context.clone())
                    .await
                    .map_err(|e| e.to_string())?;
                serde_json::to_value(result).map_err(|e| e.to_string())
            }
            // Document tools
            "list_documents" => {
                let input: ListDocumentsInput =
                    serde_json::from_value(args_value).map_err(|e| e.to_string())?;
                let result = input
                    .execute(self.context.clone())
                    .await
                    .map_err(|e| e.to_string())?;
                serde_json::to_value(result).map_err(|e| e.to_string())
            }
            "read_document" => {
                let input: ReadDocumentInput =
                    serde_json::from_value(args_value).map_err(|e| e.to_string())?;
                let result = input
                    .execute(self.context.clone())
                    .await
                    .map_err(|e| e.to_string())?;
                serde_json::to_value(result).map_err(|e| e.to_string())
            }
            "edit_document" => {
                let input: EditDocumentInput =
                    serde_json::from_value(args_value).map_err(|e| e.to_string())?;
                let result = input
                    .execute(self.context.clone())
                    .await
                    .map_err(|e| e.to_string())?;
                serde_json::to_value(result).map_err(|e| e.to_string())
            }
            "create_user_document" => {
                let input: CreateUserDocumentInput =
                    serde_json::from_value(args_value).map_err(|e| e.to_string())?;
                let result = input
                    .execute(self.context.clone())
                    .await
                    .map_err(|e| e.to_string())?;
                serde_json::to_value(result).map_err(|e| e.to_string())
            }
            // Character tools
            "list_characters" => {
                let input: ListCharactersInput =
                    serde_json::from_value(args_value).map_err(|e| e.to_string())?;
                let result = input
                    .execute(self.context.clone())
                    .await
                    .map_err(|e| e.to_string())?;
                serde_json::to_value(result).map_err(|e| e.to_string())
            }
            "get_character" => {
                let input: GetCharacterInput =
                    serde_json::from_value(args_value).map_err(|e| e.to_string())?;
                let result = input
                    .execute(self.context.clone())
                    .await
                    .map_err(|e| e.to_string())?;
                serde_json::to_value(result).map_err(|e| e.to_string())
            }
            "create_npc" => {
                let input: CreateNpcInput =
                    serde_json::from_value(args_value).map_err(|e| e.to_string())?;
                let result = input
                    .execute(self.context.clone())
                    .await
                    .map_err(|e| e.to_string())?;
                serde_json::to_value(result).map_err(|e| e.to_string())
            }
            "assign_npc_to_module" => {
                let input: AssignNpcToModuleInput =
                    serde_json::from_value(args_value).map_err(|e| e.to_string())?;
                let result = input
                    .execute(self.context.clone())
                    .await
                    .map_err(|e| e.to_string())?;
                serde_json::to_value(result).map_err(|e| e.to_string())
            }
            "add_item_to_character" => {
                let input: AddItemToCharacterInput =
                    serde_json::from_value(args_value).map_err(|e| e.to_string())?;
                let result = input
                    .execute(self.context.clone())
                    .await
                    .map_err(|e| e.to_string())?;
                serde_json::to_value(result).map_err(|e| e.to_string())
            }
            "update_character_currency" => {
                let input: UpdateCharacterCurrencyInput =
                    serde_json::from_value(args_value).map_err(|e| e.to_string())?;
                let result = input
                    .execute(self.context.clone())
                    .await
                    .map_err(|e| e.to_string())?;
                serde_json::to_value(result).map_err(|e| e.to_string())
            }
            // Catalog search tools
            "search_monsters" => {
                let input: SearchMonstersInput =
                    serde_json::from_value(args_value).map_err(|e| e.to_string())?;
                let result = input
                    .execute(self.context.clone())
                    .await
                    .map_err(|e| e.to_string())?;
                serde_json::to_value(result).map_err(|e| e.to_string())
            }
            "search_items" => {
                let input: SearchItemsInput =
                    serde_json::from_value(args_value).map_err(|e| e.to_string())?;
                let result = input
                    .execute(self.context.clone())
                    .await
                    .map_err(|e| e.to_string())?;
                serde_json::to_value(result).map_err(|e| e.to_string())
            }
            "search_traps" => {
                let input: SearchTrapsInput =
                    serde_json::from_value(args_value).map_err(|e| e.to_string())?;
                let result = input
                    .execute(self.context.clone())
                    .await
                    .map_err(|e| e.to_string())?;
                serde_json::to_value(result).map_err(|e| e.to_string())
            }
            _ => Err(format!("Unknown tool: {}", tool_name)),
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
    ) -> Result<CallToolResult, CallToolError> {
        info!(tool = %params.name, "Handling call_tool request");

        let arguments = params.arguments.unwrap_or_else(serde_json::Map::new);

        match self.execute_tool(&params.name, arguments).await {
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
