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
    pub(crate) fn get_tools() -> Vec<Tool> {
        vec![
            // Campaign tools
            tools::campaign::list_campaigns_tool(),
            tools::campaign::set_active_campaign_tool(),
            tools::campaign::get_campaign_details_tool(),
            tools::campaign::get_campaign_sources_tool(),
            tools::campaign::create_campaign_tool(),
            tools::campaign::update_campaign_tool(),
            tools::campaign::delete_campaign_tool(),
            tools::campaign::export_campaign_tool(),
            tools::campaign::import_campaign_tool(),
            tools::campaign::preview_archive_tool(),
            // Module tools
            tools::module::create_module_tool(),
            tools::module::list_modules_tool(),
            tools::module::get_module_details_tool(),
            tools::module::update_module_tool(),
            tools::module::delete_module_tool(),
            tools::module::add_monster_to_module_tool(),
            tools::module::add_item_to_module_tool(),
            // Document tools
            tools::document::list_documents_tool(),
            tools::document::read_document_tool(),
            tools::document::create_document_tool(),
            tools::document::edit_document_tool(),
            tools::document::delete_document_tool(),
            // Character tools
            tools::character::list_characters_tool(),
            tools::character::get_character_tool(),
            tools::character::create_character_tool(),
            tools::character::edit_character_tool(),
            tools::character::add_item_to_character_tool(),
            tools::character::delete_character_tool(),
            tools::character::level_up_character_tool(),
            tools::character::remove_item_from_character_tool(),
            tools::character::update_character_inventory_tool(),
            tools::character::get_character_inventory_tool(),
            // Map tools
            tools::map::create_map_tool(),
            tools::map::list_maps_tool(),
            tools::map::get_map_tool(),
            tools::map::update_map_tool(),
            tools::map::delete_map_tool(),
            tools::map::add_token_to_map_tool(),
            tools::map::list_tokens_on_map_tool(),
            tools::map::remove_token_tool(),
            // Homebrew tools
            tools::homebrew::list_homebrew_items_tool(),
            tools::homebrew::get_homebrew_item_tool(),
            tools::homebrew::create_homebrew_item_tool(),
            tools::homebrew::update_homebrew_item_tool(),
            tools::homebrew::delete_homebrew_item_tool(),
            // Homebrew monster tools
            tools::homebrew_monster::list_homebrew_monsters_tool(),
            tools::homebrew_monster::get_homebrew_monster_tool(),
            tools::homebrew_monster::create_homebrew_monster_tool(),
            tools::homebrew_monster::update_homebrew_monster_tool(),
            tools::homebrew_monster::delete_homebrew_monster_tool(),
            // Homebrew spell tools
            tools::homebrew_spell::list_homebrew_spells_tool(),
            tools::homebrew_spell::get_homebrew_spell_tool(),
            tools::homebrew_spell::create_homebrew_spell_tool(),
            tools::homebrew_spell::update_homebrew_spell_tool(),
            tools::homebrew_spell::delete_homebrew_spell_tool(),
            // Catalog tools
            tools::catalog::search_monsters_tool(),
            tools::catalog::search_items_tool(),
            tools::catalog::search_spells_tool(),
            tools::catalog::search_races_tool(),
            tools::catalog::search_classes_tool(),
            tools::catalog::search_backgrounds_tool(),
            tools::catalog::search_feats_tool(),
            tools::catalog::search_conditions_tool(),
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
            "create_campaign" => tools::campaign::create_campaign(&self.context, args).await,
            "update_campaign" => tools::campaign::update_campaign(&self.context, args).await,
            "delete_campaign" => tools::campaign::delete_campaign(&self.context, args).await,

            // Module tools
            "create_module" => tools::module::create_module(&self.context, args).await,
            "list_modules" => tools::module::list_modules(&self.context, args).await,
            "get_module_details" => tools::module::get_module_details(&self.context, args).await,
            "update_module" => tools::module::update_module(&self.context, args).await,
            "delete_module" => tools::module::delete_module(&self.context, args).await,
            "add_monster_to_module" => {
                tools::module::add_monster_to_module(&self.context, args).await
            }
            "add_item_to_module" => tools::module::add_item_to_module(&self.context, args).await,

            // Document tools
            "list_documents" => tools::document::list_documents(&self.context, args).await,
            "read_document" => tools::document::read_document(&self.context, args).await,
            "create_document" => tools::document::create_document(&self.context, args).await,
            "edit_document" => tools::document::edit_document(&self.context, args).await,
            "delete_document" => tools::document::delete_document(&self.context, args).await,

            // Character tools
            "list_characters" => tools::character::list_characters(&self.context, args).await,
            "get_character" => tools::character::get_character(&self.context, args).await,
            "create_character" => tools::character::create_character(&self.context, args).await,
            "edit_character" => tools::character::edit_character(&self.context, args).await,
            "add_item_to_character" => {
                tools::character::add_item_to_character(&self.context, args).await
            }
            "delete_character" => tools::character::delete_character(&self.context, args).await,
            "level_up_character" => {
                tools::character::level_up_character(&self.context, args).await
            }
            "remove_item_from_character" => {
                tools::character::remove_item_from_character(&self.context, args).await
            }
            "update_character_inventory" => {
                tools::character::update_character_inventory(&self.context, args).await
            }
            "get_character_inventory" => {
                tools::character::get_character_inventory(&self.context, args).await
            }

            // Map tools
            "create_map" => tools::map::create_map(&self.context, args).await,
            "list_maps" => tools::map::list_maps(&self.context, args).await,
            "get_map" => tools::map::get_map(&self.context, args).await,
            "update_map" => tools::map::update_map(&self.context, args).await,
            "delete_map" => tools::map::delete_map(&self.context, args).await,
            "add_token_to_map" => tools::map::add_token_to_map(&self.context, args).await,
            "list_tokens_on_map" => tools::map::list_tokens_on_map(&self.context, args).await,
            "remove_token" => tools::map::remove_token(&self.context, args).await,

            // Homebrew tools
            "list_homebrew_items" => tools::homebrew::list_homebrew_items(&self.context, args).await,
            "get_homebrew_item" => tools::homebrew::get_homebrew_item(&self.context, args).await,
            "create_homebrew_item" => {
                tools::homebrew::create_homebrew_item(&self.context, args).await
            }
            "update_homebrew_item" => {
                tools::homebrew::update_homebrew_item(&self.context, args).await
            }
            "delete_homebrew_item" => {
                tools::homebrew::delete_homebrew_item(&self.context, args).await
            }

            // Homebrew monster tools
            "list_homebrew_monsters" => tools::homebrew_monster::list_homebrew_monsters(&self.context, args).await,
            "get_homebrew_monster" => tools::homebrew_monster::get_homebrew_monster(&self.context, args).await,
            "create_homebrew_monster" => {
                tools::homebrew_monster::create_homebrew_monster(&self.context, args).await
            }
            "update_homebrew_monster" => {
                tools::homebrew_monster::update_homebrew_monster(&self.context, args).await
            }
            "delete_homebrew_monster" => {
                tools::homebrew_monster::delete_homebrew_monster(&self.context, args).await
            }

            // Homebrew spell tools
            "list_homebrew_spells" => tools::homebrew_spell::list_homebrew_spells(&self.context, args).await,
            "get_homebrew_spell" => tools::homebrew_spell::get_homebrew_spell(&self.context, args).await,
            "create_homebrew_spell" => {
                tools::homebrew_spell::create_homebrew_spell(&self.context, args).await
            }
            "update_homebrew_spell" => {
                tools::homebrew_spell::update_homebrew_spell(&self.context, args).await
            }
            "delete_homebrew_spell" => {
                tools::homebrew_spell::delete_homebrew_spell(&self.context, args).await
            }

            // Catalog tools
            "search_monsters" => tools::catalog::search_monsters(&self.context, args).await,
            "search_items" => tools::catalog::search_items(&self.context, args).await,
            "search_spells" => tools::catalog::search_spells(&self.context, args).await,
            "search_races" => tools::catalog::search_races(&self.context, args).await,
            "search_classes" => tools::catalog::search_classes(&self.context, args).await,
            "search_backgrounds" => tools::catalog::search_backgrounds(&self.context, args).await,
            "search_feats" => tools::catalog::search_feats(&self.context, args).await,
            "search_conditions" => tools::catalog::search_conditions(&self.context, args).await,

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::McpContext;
    use diesel::prelude::*;
    use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
    use std::path::PathBuf;
    use std::sync::Mutex;

    const MIGRATIONS: EmbeddedMigrations =
        embed_migrations!("../mimir-core/migrations");

    /// Expected tool names — every MCP tool the server should publish.
    const EXPECTED_TOOLS: &[&str] = &[
        // Campaign
        "list_campaigns",
        "set_active_campaign",
        "get_campaign_details",
        "get_campaign_sources",
        "create_campaign",
        "update_campaign",
        "delete_campaign",
        "export_campaign",
        "import_campaign",
        "preview_archive",
        // Module
        "create_module",
        "list_modules",
        "get_module_details",
        "update_module",
        "delete_module",
        "add_monster_to_module",
        "add_item_to_module",
        // Document
        "list_documents",
        "read_document",
        "create_document",
        "edit_document",
        "delete_document",
        // Character
        "list_characters",
        "get_character",
        "create_character",
        "edit_character",
        "add_item_to_character",
        "delete_character",
        "level_up_character",
        "remove_item_from_character",
        "update_character_inventory",
        "get_character_inventory",
        // Map
        "create_map",
        "list_maps",
        "get_map",
        "update_map",
        "delete_map",
        "add_token_to_map",
        "list_tokens_on_map",
        "remove_token",
        // Homebrew
        "list_homebrew_items",
        "get_homebrew_item",
        "create_homebrew_item",
        "update_homebrew_item",
        "delete_homebrew_item",
        // Homebrew Monster
        "list_homebrew_monsters",
        "get_homebrew_monster",
        "create_homebrew_monster",
        "update_homebrew_monster",
        "delete_homebrew_monster",
        // Homebrew Spell
        "list_homebrew_spells",
        "get_homebrew_spell",
        "create_homebrew_spell",
        "update_homebrew_spell",
        "delete_homebrew_spell",
        // Catalog
        "search_monsters",
        "search_items",
        "search_spells",
        "search_races",
        "search_classes",
        "search_backgrounds",
        "search_feats",
        "search_conditions",
    ];

    fn test_ctx() -> Arc<McpContext> {
        let mut db = SqliteConnection::establish(":memory:")
            .expect("Failed to create in-memory database");
        db.run_pending_migrations(MIGRATIONS)
            .expect("Failed to run migrations");
        Arc::new(McpContext {
            db: Mutex::new(db),
            assets_dir: PathBuf::from("/tmp/mimir-test-assets"),
            active_campaign_id: Mutex::new(None),
        })
    }

    #[test]
    fn all_expected_tools_are_published() {
        let tools = MimirHandler::get_tools();
        let published: Vec<&str> = tools.iter().map(|t| t.name.as_str()).collect();

        for expected in EXPECTED_TOOLS {
            assert!(
                published.contains(expected),
                "Tool '{}' is missing from get_tools(). Published: {:?}",
                expected,
                published
            );
        }
    }

    #[test]
    fn no_duplicate_tool_names() {
        let tools = MimirHandler::get_tools();
        let mut names: Vec<&str> = tools.iter().map(|t| t.name.as_str()).collect();
        names.sort();
        for window in names.windows(2) {
            assert_ne!(
                window[0], window[1],
                "Duplicate tool name: '{}'",
                window[0]
            );
        }
    }

    #[test]
    fn published_tools_match_expected_count() {
        let tools = MimirHandler::get_tools();
        assert_eq!(
            tools.len(),
            EXPECTED_TOOLS.len(),
            "Tool count mismatch. Published {} tools but expected {}. \
             Published: {:?}",
            tools.len(),
            EXPECTED_TOOLS.len(),
            tools.iter().map(|t| t.name.as_str()).collect::<Vec<_>>()
        );
    }

    #[tokio::test]
    async fn every_published_tool_has_a_route() {
        let ctx = test_ctx();
        let handler = MimirHandler::with_context(ctx);
        let tools = MimirHandler::get_tools();

        for tool in &tools {
            let result = handler
                .execute_tool(&tool.name, serde_json::json!({}))
                .await;
            // We expect errors (missing args, no campaign, etc.) but NOT ToolNotFound
            if let Err(ref e) = result {
                assert!(
                    !matches!(e, McpError::ToolNotFound(_)),
                    "Tool '{}' is published but has no route in execute_tool",
                    tool.name
                );
            }
        }
    }

    #[test]
    fn all_tools_have_descriptions() {
        let tools = MimirHandler::get_tools();
        for tool in &tools {
            assert!(
                tool.description.is_some() && !tool.description.as_ref().unwrap().is_empty(),
                "Tool '{}' is missing a description",
                tool.name
            );
        }
    }

    // =========================================================================
    // Functional Tests
    // =========================================================================

    /// Helper: call a tool by name and assert success, returning the result JSON.
    async fn call_ok(handler: &MimirHandler, name: &str, args: Value) -> Value {
        handler
            .execute_tool(name, args)
            .await
            .unwrap_or_else(|e| panic!("Tool '{}' failed: {}", name, e))
    }

    /// Helper: call a tool by name and assert it returns an error.
    async fn call_err(handler: &MimirHandler, name: &str, args: Value) -> McpError {
        handler
            .execute_tool(name, args)
            .await
            .expect_err(&format!("Tool '{}' should have failed", name))
    }

    // -- Campaign CRUD --------------------------------------------------------

    #[tokio::test]
    async fn campaign_crud_lifecycle() {
        let handler = MimirHandler::with_context(test_ctx());

        // List — starts empty
        let res = call_ok(&handler, "list_campaigns", serde_json::json!({})).await;
        assert_eq!(res["campaigns"].as_array().unwrap().len(), 0);

        // Create
        let res = call_ok(
            &handler,
            "create_campaign",
            serde_json::json!({"name": "Test Campaign", "description": "A test"}),
        )
        .await;
        assert_eq!(res["status"], "created");
        let campaign_id = res["campaign"]["id"].as_str().unwrap().to_string();

        // List — now has one
        let res = call_ok(&handler, "list_campaigns", serde_json::json!({})).await;
        assert_eq!(res["campaigns"].as_array().unwrap().len(), 1);

        // Get details
        let res = call_ok(
            &handler,
            "get_campaign_details",
            serde_json::json!({"campaign_id": campaign_id}),
        )
        .await;
        assert_eq!(res["campaign"]["name"], "Test Campaign");

        // Update
        let res = call_ok(
            &handler,
            "update_campaign",
            serde_json::json!({"campaign_id": campaign_id, "name": "Renamed"}),
        )
        .await;
        assert_eq!(res["status"], "updated");
        assert_eq!(res["campaign"]["name"], "Renamed");

        // Delete
        let res = call_ok(
            &handler,
            "delete_campaign",
            serde_json::json!({"campaign_id": campaign_id}),
        )
        .await;
        assert_eq!(res["status"], "deleted");

        // List — empty again
        let res = call_ok(&handler, "list_campaigns", serde_json::json!({})).await;
        assert_eq!(res["campaigns"].as_array().unwrap().len(), 0);
    }

    // -- Module CRUD ----------------------------------------------------------

    /// Helper: create a campaign and set it active, return the campaign id.
    async fn setup_campaign(handler: &MimirHandler) -> String {
        let res = call_ok(
            handler,
            "create_campaign",
            serde_json::json!({"name": "Test Campaign"}),
        )
        .await;
        let id = res["campaign"]["id"].as_str().unwrap().to_string();
        call_ok(
            handler,
            "set_active_campaign",
            serde_json::json!({"campaign_id": id}),
        )
        .await;
        id
    }

    #[tokio::test]
    async fn module_crud_lifecycle() {
        let handler = MimirHandler::with_context(test_ctx());
        setup_campaign(&handler).await;

        // List — empty
        let res = call_ok(&handler, "list_modules", serde_json::json!({})).await;
        assert_eq!(res["modules"].as_array().unwrap().len(), 0);

        // Create
        let res = call_ok(
            &handler,
            "create_module",
            serde_json::json!({"name": "Dungeon of Doom", "description": "Spooky"}),
        )
        .await;
        let module_id = res["module"]["id"].as_str().unwrap().to_string();

        // List — has one
        let res = call_ok(&handler, "list_modules", serde_json::json!({})).await;
        assert_eq!(res["modules"].as_array().unwrap().len(), 1);

        // Get details
        let res = call_ok(
            &handler,
            "get_module_details",
            serde_json::json!({"module_id": module_id}),
        )
        .await;
        assert_eq!(res["module"]["name"], "Dungeon of Doom");

        // Update
        let res = call_ok(
            &handler,
            "update_module",
            serde_json::json!({"module_id": module_id, "name": "Dungeon of Dread"}),
        )
        .await;
        assert_eq!(res["status"], "updated");
        assert_eq!(res["module"]["name"], "Dungeon of Dread");

        // Delete
        let res = call_ok(
            &handler,
            "delete_module",
            serde_json::json!({"module_id": module_id}),
        )
        .await;
        assert_eq!(res["status"], "deleted");

        // List — empty again
        let res = call_ok(&handler, "list_modules", serde_json::json!({})).await;
        assert_eq!(res["modules"].as_array().unwrap().len(), 0);
    }

    // -- Document CRUD --------------------------------------------------------

    #[tokio::test]
    async fn document_crud_lifecycle() {
        let handler = MimirHandler::with_context(test_ctx());
        setup_campaign(&handler).await;

        // Create a module to hold the document
        let res = call_ok(
            &handler,
            "create_module",
            serde_json::json!({"name": "Test Module"}),
        )
        .await;
        let module_id = res["module"]["id"].as_str().unwrap().to_string();

        // Create document
        let res = call_ok(
            &handler,
            "create_document",
            serde_json::json!({
                "module_id": module_id,
                "title": "Room 1",
                "document_type": "description",
                "content": "A dark room."
            }),
        )
        .await;
        let doc_id = res["document"]["id"].as_str().unwrap().to_string();

        // List documents in module — should include our doc
        let res = call_ok(
            &handler,
            "list_documents",
            serde_json::json!({"module_id": module_id}),
        )
        .await;
        let docs = res["documents"].as_array().unwrap();
        assert!(
            docs.iter().any(|d| d["id"].as_str().unwrap() == doc_id),
            "Created document should appear in list"
        );

        // Read
        let res = call_ok(
            &handler,
            "read_document",
            serde_json::json!({"document_id": doc_id}),
        )
        .await;
        assert_eq!(res["title"], "Room 1");
        assert_eq!(res["content"], "A dark room.");

        // Edit
        let res = call_ok(
            &handler,
            "edit_document",
            serde_json::json!({
                "document_id": doc_id,
                "search": "dark room",
                "replace": "bright chamber"
            }),
        )
        .await;
        assert_eq!(res["status"], "updated");
        assert!(res["document"]["content"]
            .as_str()
            .unwrap()
            .contains("bright chamber"));

        // Delete
        let res = call_ok(
            &handler,
            "delete_document",
            serde_json::json!({"document_id": doc_id}),
        )
        .await;
        assert_eq!(res["status"], "deleted");

        // Verify deleted — reading should fail
        let _err = call_err(
            &handler,
            "read_document",
            serde_json::json!({"document_id": doc_id}),
        )
        .await;
    }

    // -- Campaign-level documents ---------------------------------------------

    #[tokio::test]
    async fn campaign_level_document() {
        let handler = MimirHandler::with_context(test_ctx());
        setup_campaign(&handler).await;

        // Create campaign-level document (no module_id)
        let res = call_ok(
            &handler,
            "create_document",
            serde_json::json!({
                "title": "Session Notes",
                "document_type": "dm_notes",
                "content": "Session 1 notes."
            }),
        )
        .await;
        let doc_id = res["document"]["id"].as_str().unwrap().to_string();

        // List campaign-level docs
        let res = call_ok(&handler, "list_documents", serde_json::json!({})).await;
        let docs = res["documents"].as_array().unwrap();
        assert!(docs.iter().any(|d| d["id"].as_str().unwrap() == doc_id));

        // Read it back
        let res = call_ok(
            &handler,
            "read_document",
            serde_json::json!({"document_id": doc_id}),
        )
        .await;
        assert_eq!(res["title"], "Session Notes");
    }

    // -- Character CRUD -------------------------------------------------------

    #[tokio::test]
    async fn character_crud_lifecycle() {
        let handler = MimirHandler::with_context(test_ctx());
        setup_campaign(&handler).await;

        // List — empty
        let res = call_ok(&handler, "list_characters", serde_json::json!({})).await;
        assert_eq!(res["characters"].as_array().unwrap().len(), 0);

        // Create NPC
        let res = call_ok(
            &handler,
            "create_character",
            serde_json::json!({
                "name": "Gandalf",
                "character_type": "npc"
            }),
        )
        .await;
        let char_id = res["character"]["id"].as_str().unwrap().to_string();

        // List — has one
        let res = call_ok(&handler, "list_characters", serde_json::json!({})).await;
        assert_eq!(res["characters"].as_array().unwrap().len(), 1);

        // Get
        let res = call_ok(
            &handler,
            "get_character",
            serde_json::json!({"character_id": char_id}),
        )
        .await;
        assert_eq!(res["character"]["name"], "Gandalf");

        // Edit — rename
        let res = call_ok(
            &handler,
            "edit_character",
            serde_json::json!({"character_id": char_id, "name": "Gandalf the Grey"}),
        )
        .await;
        assert_eq!(res["status"], "updated");
        assert_eq!(res["character"]["name"], "Gandalf the Grey");

        // Delete
        let res = call_ok(
            &handler,
            "delete_character",
            serde_json::json!({"character_id": char_id}),
        )
        .await;
        assert_eq!(res["status"], "deleted");

        // List — empty again
        let res = call_ok(&handler, "list_characters", serde_json::json!({})).await;
        assert_eq!(res["characters"].as_array().unwrap().len(), 0);
    }

    #[tokio::test]
    async fn character_filter_by_type() {
        let handler = MimirHandler::with_context(test_ctx());
        setup_campaign(&handler).await;

        // Create one NPC and one PC
        call_ok(
            &handler,
            "create_character",
            serde_json::json!({"name": "Villager", "character_type": "npc"}),
        )
        .await;
        call_ok(
            &handler,
            "create_character",
            serde_json::json!({"name": "Hero", "character_type": "pc"}),
        )
        .await;

        // Filter NPCs
        let res = call_ok(
            &handler,
            "list_characters",
            serde_json::json!({"character_type": "npc"}),
        )
        .await;
        let chars = res["characters"].as_array().unwrap();
        assert_eq!(chars.len(), 1);
        assert_eq!(chars[0]["name"], "Villager");

        // Filter PCs
        let res = call_ok(
            &handler,
            "list_characters",
            serde_json::json!({"character_type": "pc"}),
        )
        .await;
        let chars = res["characters"].as_array().unwrap();
        assert_eq!(chars.len(), 1);
        assert_eq!(chars[0]["name"], "Hero");
    }

    // -- Catalog searches (empty DB, should return 0 results) -----------------

    #[tokio::test]
    async fn catalog_searches_return_empty_on_fresh_db() {
        let handler = MimirHandler::with_context(test_ctx());

        let catalog_tools = [
            ("search_monsters", "monsters"),
            ("search_items", "items"),
            ("search_spells", "spells"),
            ("search_races", "races"),
            ("search_classes", "classes"),
            ("search_backgrounds", "backgrounds"),
            ("search_feats", "feats"),
            ("search_conditions", "conditions"),
        ];

        for (tool, key) in catalog_tools {
            let res = call_ok(&handler, tool, serde_json::json!({"name": "nonexistent"})).await;
            assert_eq!(
                res["count"].as_u64().unwrap(),
                0,
                "{} should return 0 results on empty DB",
                tool
            );
            assert!(
                res[key].as_array().unwrap().is_empty(),
                "{} array should be empty",
                tool
            );
        }
    }

    // -- Error cases ----------------------------------------------------------

    #[tokio::test]
    async fn tool_not_found_for_unknown_name() {
        let handler = MimirHandler::with_context(test_ctx());
        let err = call_err(&handler, "nonexistent_tool", serde_json::json!({})).await;
        assert!(matches!(err, McpError::ToolNotFound(_)));
    }

    #[tokio::test]
    async fn campaign_required_tools_fail_without_active_campaign() {
        let handler = MimirHandler::with_context(test_ctx());

        // These tools require an active campaign
        let tools_needing_campaign = [
            "list_modules",
            "list_characters",
            "list_documents",
        ];

        for tool in tools_needing_campaign {
            let err = call_err(&handler, tool, serde_json::json!({})).await;
            assert!(
                matches!(err, McpError::NoActiveCampaign),
                "Tool '{}' should fail with NoActiveCampaign, got: {:?}",
                tool,
                err
            );
        }
    }

    #[tokio::test]
    async fn create_character_requires_name() {
        let handler = MimirHandler::with_context(test_ctx());
        setup_campaign(&handler).await;

        let err = call_err(
            &handler,
            "create_character",
            serde_json::json!({"character_type": "npc"}),
        )
        .await;
        assert!(
            matches!(err, McpError::InvalidArguments(_)),
            "Expected InvalidArguments, got: {:?}",
            err
        );
    }

    #[tokio::test]
    async fn delete_campaign_requires_id() {
        let handler = MimirHandler::with_context(test_ctx());

        let err = call_err(&handler, "delete_campaign", serde_json::json!({})).await;
        assert!(matches!(err, McpError::InvalidArguments(_)));
    }
}
