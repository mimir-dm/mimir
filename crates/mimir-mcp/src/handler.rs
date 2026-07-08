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

    /// Get the list of available tools: registry-based families first, then
    /// families not yet migrated to the registry.
    pub(crate) fn get_tools() -> Vec<Tool> {
        let mut all: Vec<Tool> = crate::registry::all_tools()
            .iter()
            .map(|t| t.to_tool())
            .collect();
        all.extend(Self::legacy_tools());
        all
    }

    /// Tool definitions for families still on the legacy dispatch path.
    fn legacy_tools() -> Vec<Tool> {
        vec![
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
            tools::character::add_character_spell_tool(),
            tools::character::remove_character_spell_tool(),
            tools::character::list_character_spells_tool(),
            // Map tools
            tools::map::create_map_tool(),
            tools::map::list_maps_tool(),
            tools::map::get_map_tool(),
            tools::map::update_map_tool(),
            tools::map::delete_map_tool(),
            tools::map::add_token_to_map_tool(),
            tools::map::list_tokens_on_map_tool(),
            tools::map::remove_token_tool(),
            // Homebrew tools (items, monsters, spells — unified by content_type)
            tools::homebrew::list_homebrew_tool(),
            tools::homebrew::get_homebrew_tool(),
            tools::homebrew::create_homebrew_tool(),
            tools::homebrew::update_homebrew_tool(),
            tools::homebrew::delete_homebrew_tool(),
            // Map generation tools
            tools::mapgen::generate_map_tool(),
            tools::mapgen::list_map_presets_tool(),
            tools::mapgen::validate_map_config_tool(),
            // Catalog search (all categories unified by category param)
            tools::catalog::search_catalog_tool(),
        ]
    }

    /// Route a tool call: registry-based families first, then the legacy
    /// match for families not yet migrated.
    async fn execute_tool(&self, name: &str, args: Value) -> Result<Value, McpError> {
        if let Some(tool) = crate::registry::find(name) {
            return (tool.handler)(&self.context, args).await;
        }
        match name {
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
            "add_character_spell" => {
                tools::character::add_character_spell(&self.context, args).await
            }
            "remove_character_spell" => {
                tools::character::remove_character_spell(&self.context, args).await
            }
            "list_character_spells" => {
                tools::character::list_character_spells(&self.context, args).await
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

            // Homebrew tools (items, monsters, spells — dispatched by content_type)
            "list_homebrew" => tools::homebrew::list_homebrew(&self.context, args).await,
            "get_homebrew" => tools::homebrew::get_homebrew(&self.context, args).await,
            "create_homebrew" => tools::homebrew::create_homebrew(&self.context, args).await,
            "update_homebrew" => tools::homebrew::update_homebrew(&self.context, args).await,
            "delete_homebrew" => tools::homebrew::delete_homebrew(&self.context, args).await,

            // Map generation tools (no campaign context needed)
            "generate_map" => tools::mapgen::generate_map(args).await,
            "list_map_presets" => tools::mapgen::list_map_presets(args).await,
            "validate_map_config" => tools::mapgen::validate_map_config(args).await,

            // Catalog search (dispatched by category param)
            "search_catalog" => tools::catalog::search_catalog(&self.context, args).await,

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
    use serde_json::json;

    /// Expected tool names — every MCP tool the server should publish.
    const EXPECTED_TOOLS: &[&str] = &[
        // Campaign
        "list_campaigns",
        "get_active_campaign",
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
        "update_module_monster",
        "remove_monster_from_module",
        "add_item_to_module",
        // Document
        "list_documents",
        "read_document",
        "create_document",
        "edit_document",
        "delete_document",
        "reorder_document",
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
        "add_character_spell",
        "remove_character_spell",
        "list_character_spells",
        // Map
        "create_map",
        "list_maps",
        "get_map",
        "update_map",
        "delete_map",
        "add_token_to_map",
        "list_tokens_on_map",
        "remove_token",
        // Homebrew (items, monsters, spells)
        "list_homebrew",
        "get_homebrew",
        "create_homebrew",
        "update_homebrew",
        "delete_homebrew",
        // Map generation
        "generate_map",
        "list_map_presets",
        "validate_map_config",
        // Catalog
        "search_catalog",
    ];

    fn test_ctx() -> Arc<McpContext> {
        Arc::new(McpContext::for_testing())
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

    #[tokio::test]
    async fn get_active_campaign_reports_and_self_heals() {
        let handler = MimirHandler::with_context(test_ctx());

        // Nothing selected — must not error, reports active=false with guidance.
        let res = call_ok(&handler, "get_active_campaign", serde_json::json!({})).await;
        assert_eq!(res["active"], false);
        assert!(res["campaign"].is_null());
        assert!(res["hint"].as_str().unwrap().contains("list_campaigns"));

        // Creating a campaign activates it — get_active_campaign reflects that.
        let created = call_ok(
            &handler,
            "create_campaign",
            serde_json::json!({"name": "Active One"}),
        )
        .await;
        let campaign_id = created["campaign"]["id"].as_str().unwrap().to_string();

        let res = call_ok(&handler, "get_active_campaign", serde_json::json!({})).await;
        assert_eq!(res["active"], true);
        assert_eq!(res["campaign"]["id"], campaign_id);
        assert_eq!(res["campaign"]["name"], "Active One");

        // After the active campaign is deleted, get_active_campaign reports
        // active=false rather than erroring. (The self-heal branch also covers the
        // case where another process removes the row out from under a live selection.)
        call_ok(
            &handler,
            "delete_campaign",
            serde_json::json!({"campaign_id": campaign_id}),
        )
        .await;
        let res = call_ok(&handler, "get_active_campaign", serde_json::json!({})).await;
        assert_eq!(res["active"], false);
        assert!(res["campaign"].is_null());
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
        assert_eq!(res["document"]["title"], "Room 1");
        assert_eq!(res["document"]["content"], "A dark room.");

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
        assert_eq!(res["document"]["title"], "Session Notes");
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

        let categories = [
            ("monster", "monsters"),
            ("item", "items"),
            ("spell", "spells"),
            ("race", "races"),
            ("class", "classes"),
            ("background", "backgrounds"),
            ("feat", "feats"),
            ("condition", "conditions"),
        ];

        for (category, key) in categories {
            let res = call_ok(
                &handler,
                "search_catalog",
                serde_json::json!({"category": category, "name": "nonexistent"}),
            )
            .await;
            assert_eq!(
                res["count"].as_u64().unwrap(),
                0,
                "search_catalog({}) should return 0 results on empty DB",
                category
            );
            assert!(
                res[key].as_array().unwrap().is_empty(),
                "{} array should be empty",
                category
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

    // -- Module Monsters (catalog + homebrew paths) ----------------------------

    /// Helper: create a module in the active campaign, return its id.
    async fn setup_module(handler: &MimirHandler) -> String {
        let res = call_ok(
            handler,
            "create_module",
            json!({"name": "Test Module"}),
        )
        .await;
        res["module"]["id"].as_str().unwrap().to_string()
    }

    #[tokio::test]
    async fn add_and_remove_catalog_monster() {
        let handler = MimirHandler::with_context(test_ctx());
        setup_campaign(&handler).await;
        let module_id = setup_module(&handler).await;

        // Add via monster_name (catalog path)
        let res = call_ok(
            &handler,
            "add_monster_to_module",
            json!({
                "module_id": module_id,
                "monster_name": "Guard",
                "monster_source": "MM",
                "count": 3,
                "notes": "Gate guards"
            }),
        )
        .await;
        assert_eq!(res["status"], "added");
        assert_eq!(res["module_monster"]["monster_name"], "Guard");
        assert_eq!(res["module_monster"]["quantity"], 3);
        assert!(res["module_monster"]["homebrew_monster_id"].is_null());
        let mm_id = res["module_monster"]["id"].as_str().unwrap().to_string();

        // Appears in module details
        let res = call_ok(
            &handler,
            "get_module_details",
            json!({"module_id": module_id}),
        )
        .await;
        let monsters = res["monsters"].as_array().unwrap();
        assert_eq!(monsters.len(), 1);
        assert_eq!(monsters[0]["monster_name"], "Guard");

        // Remove
        let res = call_ok(
            &handler,
            "remove_monster_from_module",
            json!({"module_monster_id": mm_id}),
        )
        .await;
        assert_eq!(res["status"], "removed");

        // Gone from module details
        let res = call_ok(
            &handler,
            "get_module_details",
            json!({"module_id": module_id}),
        )
        .await;
        assert_eq!(res["monsters"].as_array().unwrap().len(), 0);
    }

    #[tokio::test]
    async fn add_and_remove_homebrew_monster() {
        let handler = MimirHandler::with_context(test_ctx());
        setup_campaign(&handler).await;
        let module_id = setup_module(&handler).await;

        // Create a homebrew monster to reference
        let res = call_ok(
            &handler,
            "create_homebrew",
            json!({
                "content_type": "monster",
                "name": "Frost Architect",
                "data": r#"{"name":"Frost Architect","hp":{"average":150}}"#,
                "cr": "15",
                "creature_type": "construct",
                "size": "H"
            }),
        )
        .await;
        let hb_id = res["monster"]["id"].as_str().unwrap().to_string();

        // Add via homebrew_monster_id (homebrew path) — this is the call that
        // hung the live server on 2026-07-07; guard against regression.
        let res = tokio::time::timeout(
            std::time::Duration::from_secs(10),
            handler.execute_tool(
                "add_monster_to_module",
                json!({
                    "module_id": module_id,
                    "homebrew_monster_id": hb_id,
                    "count": 1,
                    "display_name": "The Architect"
                }),
            ),
        )
        .await
        .expect("add_monster_to_module(homebrew) timed out — handler hang")
        .expect("add_monster_to_module(homebrew) failed");

        assert_eq!(res["status"], "added");
        assert_eq!(res["module_monster"]["homebrew_monster_id"], hb_id.as_str());
        assert!(res["module_monster"]["monster_name"].is_null());
        let mm_id = res["module_monster"]["id"].as_str().unwrap().to_string();

        // Appears in module details flagged as homebrew
        let res = call_ok(
            &handler,
            "get_module_details",
            json!({"module_id": module_id}),
        )
        .await;
        let monsters = res["monsters"].as_array().unwrap();
        assert_eq!(monsters.len(), 1);
        assert_eq!(monsters[0]["homebrew_monster_id"], hb_id.as_str());

        // Remove
        let res = call_ok(
            &handler,
            "remove_monster_from_module",
            json!({"module_monster_id": mm_id}),
        )
        .await;
        assert_eq!(res["status"], "removed");
    }

    #[tokio::test]
    async fn identical_add_increments_quantity_instead_of_duplicating() {
        let handler = MimirHandler::with_context(test_ctx());
        setup_campaign(&handler).await;
        let module_id = setup_module(&handler).await;

        // First add creates the entry
        let res = call_ok(
            &handler,
            "add_monster_to_module",
            json!({"module_id": module_id, "monster_name": "Guard", "monster_source": "MM", "count": 2}),
        )
        .await;
        let first_id = res["module_monster"]["id"].as_str().unwrap().to_string();
        assert_eq!(res["module_monster"]["quantity"], 2);

        // Retry / repeat add increments the same row — no duplicate
        let res = call_ok(
            &handler,
            "add_monster_to_module",
            json!({"module_id": module_id, "monster_name": "Guard", "monster_source": "MM", "count": 2}),
        )
        .await;
        assert_eq!(res["module_monster"]["id"], first_id.as_str());
        assert_eq!(res["module_monster"]["quantity"], 4);

        let res = call_ok(
            &handler,
            "get_module_details",
            json!({"module_id": module_id}),
        )
        .await;
        assert_eq!(res["monsters"].as_array().unwrap().len(), 1, "must not duplicate");

        // update_module_monster sets an exact quantity
        let res = call_ok(
            &handler,
            "update_module_monster",
            json!({"module_monster_id": first_id, "quantity": 3, "notes": "gate detail"}),
        )
        .await;
        assert_eq!(res["status"], "updated");
        assert_eq!(res["module_monster"]["quantity"], 3);
        assert_eq!(res["module_monster"]["notes"], "gate detail");

        // Unknown id errors cleanly
        let err = call_err(
            &handler,
            "update_module_monster",
            json!({"module_monster_id": "no-such-id", "quantity": 1}),
        )
        .await;
        assert!(matches!(err, McpError::InvalidArguments(_)));
    }

    #[tokio::test]
    async fn add_monster_argument_validation() {
        let handler = MimirHandler::with_context(test_ctx());
        setup_campaign(&handler).await;
        let module_id = setup_module(&handler).await;

        // Both paths specified — rejected
        let err = call_err(
            &handler,
            "add_monster_to_module",
            json!({
                "module_id": module_id,
                "monster_name": "Guard",
                "homebrew_monster_id": "some-id"
            }),
        )
        .await;
        assert!(matches!(err, McpError::InvalidArguments(_)));

        // Neither path specified — rejected
        let err = call_err(
            &handler,
            "add_monster_to_module",
            json!({"module_id": module_id}),
        )
        .await;
        assert!(matches!(err, McpError::InvalidArguments(_)));

        // Nonexistent homebrew id — rejected, not hung
        let err = call_err(
            &handler,
            "add_monster_to_module",
            json!({"module_id": module_id, "homebrew_monster_id": "no-such-id"}),
        )
        .await;
        assert!(matches!(err, McpError::InvalidArguments(_)));

        // Nonexistent module — rejected
        let err = call_err(
            &handler,
            "add_monster_to_module",
            json!({"module_id": "no-such-module", "monster_name": "Guard"}),
        )
        .await;
        assert!(matches!(err, McpError::InvalidArguments(_)));
    }

    // -- Homebrew Monster CRUD ------------------------------------------------

    #[tokio::test]
    async fn homebrew_monster_crud_lifecycle() {
        let handler = MimirHandler::with_context(test_ctx());
        setup_campaign(&handler).await;

        // List — empty
        let res = call_ok(&handler, "list_homebrew", json!({"content_type": "monster"})).await;
        assert_eq!(res["monsters"].as_array().unwrap().len(), 0);

        // Create
        let res = call_ok(
            &handler,
            "create_homebrew",
            json!({
                "content_type": "monster",
                "name": "Frost Colossus",
                "data": r#"{"name":"Frost Colossus","hp":{"average":200}}"#,
                "cr": "20",
                "creature_type": "elemental",
                "size": "G"
            }),
        )
        .await;
        assert_eq!(res["status"], "created");
        let monster_id = res["monster"]["id"].as_str().unwrap().to_string();
        assert_eq!(res["monster"]["name"], "Frost Colossus");
        assert_eq!(res["monster"]["cr"], "20");

        // List — has one
        let res = call_ok(&handler, "list_homebrew", json!({"content_type": "monster"})).await;
        assert_eq!(res["monsters"].as_array().unwrap().len(), 1);

        // Get
        let res = call_ok(
            &handler,
            "get_homebrew",
            json!({"content_type": "monster", "id": monster_id}),
        )
        .await;
        assert_eq!(res["monster"]["name"], "Frost Colossus");
        assert_eq!(res["monster"]["creature_type"], "elemental");

        // Update
        let res = call_ok(
            &handler,
            "update_homebrew",
            json!({"content_type": "monster", "id": monster_id, "name": "Ice Titan", "cr": "25"}),
        )
        .await;
        assert_eq!(res["status"], "updated");
        assert_eq!(res["monster"]["name"], "Ice Titan");
        assert_eq!(res["monster"]["cr"], "25");

        // Delete
        let res = call_ok(
            &handler,
            "delete_homebrew",
            json!({"content_type": "monster", "id": monster_id}),
        )
        .await;
        assert_eq!(res["status"], "deleted");

        // List — empty again
        let res = call_ok(&handler, "list_homebrew", json!({"content_type": "monster"})).await;
        assert_eq!(res["monsters"].as_array().unwrap().len(), 0);
    }

    // -- Homebrew Spell CRUD --------------------------------------------------

    #[tokio::test]
    async fn homebrew_spell_crud_lifecycle() {
        let handler = MimirHandler::with_context(test_ctx());
        setup_campaign(&handler).await;

        // Create
        let res = call_ok(
            &handler,
            "create_homebrew",
            json!({
                "content_type": "spell",
                "name": "Arcane Blast",
                "data": r#"{"name":"Arcane Blast","level":3}"#,
                "level": 3,
                "school": "evocation"
            }),
        )
        .await;
        assert_eq!(res["status"], "created");
        let spell_id = res["spell"]["id"].as_str().unwrap().to_string();

        // List
        let res = call_ok(&handler, "list_homebrew", json!({"content_type": "spell"})).await;
        assert_eq!(res["spells"].as_array().unwrap().len(), 1);

        // Get
        let res = call_ok(
            &handler,
            "get_homebrew",
            json!({"content_type": "spell", "id": spell_id}),
        )
        .await;
        assert_eq!(res["spell"]["name"], "Arcane Blast");
        assert_eq!(res["spell"]["school"], "evocation");

        // Update
        let res = call_ok(
            &handler,
            "update_homebrew",
            json!({"content_type": "spell", "id": spell_id, "name": "Eldritch Blast", "level": 0}),
        )
        .await;
        assert_eq!(res["status"], "updated");
        assert_eq!(res["spell"]["name"], "Eldritch Blast");

        // Delete
        let res = call_ok(
            &handler,
            "delete_homebrew",
            json!({"content_type": "spell", "id": spell_id}),
        )
        .await;
        assert_eq!(res["status"], "deleted");

        // List — empty
        let res = call_ok(&handler, "list_homebrew", json!({"content_type": "spell"})).await;
        assert_eq!(res["spells"].as_array().unwrap().len(), 0);
    }

    // -- Homebrew Item CRUD ---------------------------------------------------

    #[tokio::test]
    async fn homebrew_item_crud_lifecycle() {
        let handler = MimirHandler::with_context(test_ctx());
        setup_campaign(&handler).await;

        // Create
        let res = call_ok(
            &handler,
            "create_homebrew",
            json!({
                "content_type": "item",
                "name": "Flame Blade",
                "data": r#"{"name":"Flame Blade"}"#,
                "item_type": "weapon",
                "rarity": "rare"
            }),
        )
        .await;
        assert_eq!(res["status"], "created");
        let item_id = res["item"]["id"].as_str().unwrap().to_string();

        // List
        let res = call_ok(&handler, "list_homebrew", json!({"content_type": "item"})).await;
        assert_eq!(res["items"].as_array().unwrap().len(), 1);

        // Get
        let res = call_ok(
            &handler,
            "get_homebrew",
            json!({"content_type": "item", "id": item_id}),
        )
        .await;
        assert_eq!(res["item"]["name"], "Flame Blade");
        assert_eq!(res["item"]["rarity"], "rare");

        // Update
        let res = call_ok(
            &handler,
            "update_homebrew",
            json!({"content_type": "item", "id": item_id, "name": "Frost Blade", "rarity": "legendary"}),
        )
        .await;
        assert_eq!(res["status"], "updated");
        assert_eq!(res["item"]["name"], "Frost Blade");

        // Delete
        let res = call_ok(
            &handler,
            "delete_homebrew",
            json!({"content_type": "item", "id": item_id}),
        )
        .await;
        assert_eq!(res["status"], "deleted");

        // List — empty
        let res = call_ok(&handler, "list_homebrew", json!({"content_type": "item"})).await;
        assert_eq!(res["items"].as_array().unwrap().len(), 0);
    }

    // -- Homebrew Error Cases -------------------------------------------------

    #[tokio::test]
    async fn homebrew_not_found_errors() {
        let handler = MimirHandler::with_context(test_ctx());
        setup_campaign(&handler).await;

        let fake_id = "00000000-0000-0000-0000-000000000000";

        for ct in ["monster", "spell", "item"] {
            let err = call_err(&handler, "get_homebrew", json!({"content_type": ct, "id": fake_id})).await;
            assert!(!matches!(err, McpError::InvalidArguments(_)), "get_homebrew({}) should be a not-found error", ct);

            let err = call_err(&handler, "update_homebrew", json!({"content_type": ct, "id": fake_id, "name": "x"})).await;
            assert!(!matches!(err, McpError::InvalidArguments(_)));

            let err = call_err(&handler, "delete_homebrew", json!({"content_type": ct, "id": fake_id})).await;
            assert!(!matches!(err, McpError::InvalidArguments(_)));
        }
    }

    #[tokio::test]
    async fn homebrew_create_requires_name_and_data() {
        let handler = MimirHandler::with_context(test_ctx());
        setup_campaign(&handler).await;

        for ct in ["monster", "spell", "item"] {
            // Missing name
            let err = call_err(
                &handler,
                "create_homebrew",
                json!({"content_type": ct, "data": "{}"}),
            )
            .await;
            assert!(matches!(err, McpError::InvalidArguments(_)));

            // Missing data
            let err = call_err(
                &handler,
                "create_homebrew",
                json!({"content_type": ct, "name": "Test"}),
            )
            .await;
            assert!(matches!(err, McpError::InvalidArguments(_)));
        }
    }

    #[tokio::test]
    async fn homebrew_get_update_delete_require_id() {
        let handler = MimirHandler::with_context(test_ctx());
        setup_campaign(&handler).await;

        for ct in ["monster", "spell", "item"] {
            for tool in ["get_homebrew", "update_homebrew", "delete_homebrew"] {
                let err = call_err(&handler, tool, json!({"content_type": ct})).await;
                assert!(
                    matches!(err, McpError::InvalidArguments(_)),
                    "{}(content_type={}) should fail with InvalidArguments when id is missing, got: {:?}",
                    tool, ct, err
                );
            }
        }
    }

    // -- Character inventory ----------------------------------------------------

    /// Helper: create a character in the active campaign, return its id.
    async fn setup_character(handler: &MimirHandler) -> String {
        let res = call_ok(
            handler,
            "create_character",
            json!({"name": "Hero", "character_type": "pc"}),
        )
        .await;
        res["character"]["id"].as_str().unwrap().to_string()
    }

    #[tokio::test]
    async fn character_inventory_lifecycle() {
        let handler = MimirHandler::with_context(test_ctx());
        setup_campaign(&handler).await;
        let char_id = setup_character(&handler).await;

        // Add an item
        let res = call_ok(
            &handler,
            "add_item_to_character",
            json!({
                "character_id": char_id,
                "item_name": "Longsword",
                "item_source": "PHB",
                "quantity": 1,
                "equipped": true
            }),
        )
        .await;
        assert_eq!(res["status"], "added");
        assert_eq!(res["inventory_item"]["item_name"], "Longsword");
        assert_eq!(res["inventory_item"]["equipped"], true);
        let inv_id = res["inventory_item"]["id"].as_str().unwrap().to_string();

        // Full inventory has one item
        let res = call_ok(
            &handler,
            "get_character_inventory",
            json!({"character_id": char_id}),
        )
        .await;
        assert_eq!(res["count"], 1);

        // Equipped filter also finds it
        let res = call_ok(
            &handler,
            "get_character_inventory",
            json!({"character_id": char_id, "filter": "equipped"}),
        )
        .await;
        assert_eq!(res["count"], 1);

        // Update quantity
        let res = call_ok(
            &handler,
            "update_character_inventory",
            json!({"inventory_id": inv_id, "quantity": 3}),
        )
        .await;
        assert_eq!(res["status"], "updated");
        assert_eq!(res["inventory_item"]["quantity"], 3);

        // Remove
        let res = call_ok(
            &handler,
            "remove_item_from_character",
            json!({"inventory_id": inv_id}),
        )
        .await;
        assert_eq!(res["status"], "removed");

        // Inventory empty again
        let res = call_ok(
            &handler,
            "get_character_inventory",
            json!({"character_id": char_id}),
        )
        .await;
        assert_eq!(res["count"], 0);
    }

    // -- Character spells ---------------------------------------------------------

    #[tokio::test]
    async fn character_spells_lifecycle() {
        let handler = MimirHandler::with_context(test_ctx());
        setup_campaign(&handler).await;
        let char_id = setup_character(&handler).await;

        // Add a spell
        let res = call_ok(
            &handler,
            "add_character_spell",
            json!({
                "character_id": char_id,
                "spell_name": "Fireball",
                "spell_source": "PHB",
                "source_class": "Wizard",
                "prepared": true
            }),
        )
        .await;
        assert_eq!(res["status"], "success");
        assert_eq!(res["data"]["spell"]["spell_name"], "Fireball");

        // Duplicate add is rejected
        let err = call_err(
            &handler,
            "add_character_spell",
            json!({
                "character_id": char_id,
                "spell_name": "Fireball",
                "spell_source": "PHB",
                "source_class": "Wizard"
            }),
        )
        .await;
        assert!(matches!(err, McpError::InvalidArguments(_)));

        // Listed
        let res = call_ok(
            &handler,
            "list_character_spells",
            json!({"character_id": char_id}),
        )
        .await;
        assert!(
            serde_json::to_string(&res).unwrap().contains("Fireball"),
            "spell list should contain Fireball: {}",
            res
        );

        // Remove — case-insensitive, matching the desktop UI behavior
        let res = call_ok(
            &handler,
            "remove_character_spell",
            json!({"character_id": char_id, "spell_name": "fireball"}),
        )
        .await;
        assert_eq!(res["status"], "success");

        // Removing again fails
        let err = call_err(
            &handler,
            "remove_character_spell",
            json!({"character_id": char_id, "spell_name": "Fireball"}),
        )
        .await;
        assert!(matches!(err, McpError::InvalidArguments(_)));
    }

    #[tokio::test]
    async fn level_up_character_adds_class_level() {
        let handler = MimirHandler::with_context(test_ctx());
        setup_campaign(&handler).await;
        let char_id = setup_character(&handler).await;

        // Classes are stored by name+source (like monsters), so this works
        // even without catalog data loaded.
        let res = call_ok(
            &handler,
            "level_up_character",
            json!({"character_id": char_id, "class_name": "Fighter"}),
        )
        .await;
        assert_eq!(res["status"], "success");
        assert_eq!(res["data"]["class"]["class_name"], "Fighter");
        assert_eq!(res["data"]["new_total_level"], 1);
    }

    // -- Document reordering ------------------------------------------------------

    #[tokio::test]
    async fn reorder_documents_swaps_sort_order() {
        let handler = MimirHandler::with_context(test_ctx());
        setup_campaign(&handler).await;

        let doc_a = call_ok(
            &handler,
            "create_document",
            json!({"title": "A", "document_type": "dm_notes", "content": "a"}),
        )
        .await["document"]["id"]
            .as_str()
            .unwrap()
            .to_string();
        let doc_b = call_ok(
            &handler,
            "create_document",
            json!({"title": "B", "document_type": "dm_notes", "content": "b"}),
        )
        .await["document"]["id"]
            .as_str()
            .unwrap()
            .to_string();

        let res = call_ok(
            &handler,
            "reorder_document",
            json!({"document_id": doc_a, "swap_with_id": doc_b}),
        )
        .await;
        // Campaign creation seeds template documents, so the response holds
        // more than just our two — find ours by id.
        let docs = res["documents"].as_array().unwrap();
        let order_of = |id: &str| {
            docs.iter()
                .find(|d| d["id"] == id)
                .unwrap()["sort_order"]
                .as_i64()
                .unwrap()
        };
        assert!(order_of(&doc_b) < order_of(&doc_a), "B should now sort before A");
    }

    // -- Campaign sources ---------------------------------------------------------

    #[tokio::test]
    async fn get_campaign_sources_returns_source_list() {
        let handler = MimirHandler::with_context(test_ctx());
        setup_campaign(&handler).await;

        let res = call_ok(&handler, "get_campaign_sources", json!({})).await;
        assert!(res["sources"].is_array());
    }

    // -- Module items (not implemented) --------------------------------------------

    #[tokio::test]
    async fn add_item_to_module_is_unimplemented_error() {
        let handler = MimirHandler::with_context(test_ctx());
        setup_campaign(&handler).await;
        let module_id = setup_module(&handler).await;

        let _err = call_err(
            &handler,
            "add_item_to_module",
            json!({"module_id": module_id, "item_name": "Longsword"}),
        )
        .await;
    }

    // -- Map generation -------------------------------------------------------------

    #[tokio::test]
    async fn mapgen_presets_validate_and_generate() {
        let handler = MimirHandler::with_context(test_ctx());

        // Presets exist
        let res = call_ok(&handler, "list_map_presets", json!({})).await;
        let presets = res["presets"].as_array().unwrap();
        assert!(!presets.is_empty(), "expected built-in map presets");
        let preset_name = presets[0]["name"].as_str().unwrap().to_string();

        // Invalid YAML is reported, not an error
        let res = call_ok(
            &handler,
            "validate_map_config",
            json!({"config_yaml": ": not yaml : ["}),
        )
        .await;
        assert_eq!(res["valid"], false);

        // Generate from a preset into a temp file
        let out = std::env::temp_dir().join(format!(
            "mimir-mcp-test-map-{}.json",
            std::process::id()
        ));
        let res = call_ok(
            &handler,
            "generate_map",
            json!({"preset": preset_name, "output_path": out.to_str().unwrap(), "seed": 42}),
        )
        .await;
        assert_eq!(res["success"], true);
        assert!(out.exists(), "generated map file should exist");
        let _ = std::fs::remove_file(&out);

        // Unknown preset is rejected
        let err = call_err(
            &handler,
            "generate_map",
            json!({"preset": "definitely-not-a-preset", "output_path": "/dev/null"}),
        )
        .await;
        assert!(matches!(err, McpError::InvalidArguments(_)));
    }

    // -- Maps and tokens (error paths — happy path needs a real UVTT fixture) -------

    #[tokio::test]
    async fn map_tools_reject_bad_input() {
        let handler = MimirHandler::with_context(test_ctx());
        setup_campaign(&handler).await;

        // Empty list on fresh campaign
        let res = call_ok(&handler, "list_maps", json!({})).await;
        assert_eq!(res["maps"].as_array().map(|a| a.len()).unwrap_or(0), 0);

        // Missing file
        let err = call_err(
            &handler,
            "create_map",
            json!({"name": "Cave", "file_path": "/nonexistent/map.uvtt"}),
        )
        .await;
        assert!(matches!(err, McpError::InvalidArguments(_)));

        // Unknown ids error rather than hang
        let _err = call_err(&handler, "get_map", json!({"map_id": "nope"})).await;
        let _err = call_err(&handler, "remove_token", json!({"token_id": "nope"})).await;
    }

    // -- Export / preview / import roundtrip -----------------------------------------

    #[tokio::test]
    async fn campaign_archive_roundtrip() {
        let handler = MimirHandler::with_context(test_ctx());
        setup_campaign(&handler).await;

        // Give the campaign some content
        let module_id = setup_module(&handler).await;
        call_ok(
            &handler,
            "create_document",
            json!({"module_id": module_id, "title": "Notes", "document_type": "dm_notes", "content": "hi"}),
        )
        .await;

        let out_dir = std::env::temp_dir().join(format!("mimir-mcp-test-export-{}", std::process::id()));
        std::fs::create_dir_all(&out_dir).unwrap();

        // Export
        let res = call_ok(
            &handler,
            "export_campaign",
            json!({"output_path": out_dir.to_str().unwrap()}),
        )
        .await;
        assert_eq!(res["status"], "success");
        let archive_path = res["data"]["archive_path"].as_str().unwrap().to_string();
        assert!(std::path::Path::new(&archive_path).exists());

        // Preview
        let res = call_ok(
            &handler,
            "preview_archive",
            json!({"archive_path": archive_path}),
        )
        .await;
        assert_eq!(res["campaign_name"], "Test Campaign");
        assert_eq!(res["counts"]["modules"], 1);

        // Import as a copy
        let res = call_ok(
            &handler,
            "import_campaign",
            json!({"archive_path": archive_path, "new_name": "Imported Copy"}),
        )
        .await;
        assert_eq!(res["status"], "success");
        assert_eq!(res["data"]["campaign_name"], "Imported Copy");

        // Two campaigns now exist
        let res = call_ok(&handler, "list_campaigns", json!({})).await;
        assert_eq!(res["campaigns"].as_array().unwrap().len(), 2);

        let _ = std::fs::remove_dir_all(&out_dir);
    }

    #[tokio::test]
    async fn homebrew_list_requires_active_campaign() {
        let handler = MimirHandler::with_context(test_ctx());
        // No campaign set

        for ct in ["monster", "spell", "item"] {
            let err = call_err(&handler, "list_homebrew", json!({"content_type": ct})).await;
            assert!(
                matches!(err, McpError::NoActiveCampaign),
                "list_homebrew(content_type={}) should fail with NoActiveCampaign, got: {:?}",
                ct, err
            );
        }
    }

    #[tokio::test]
    async fn homebrew_create_requires_active_campaign() {
        let handler = MimirHandler::with_context(test_ctx());
        // No campaign set

        for ct in ["monster", "spell", "item"] {
            let err = call_err(
                &handler,
                "create_homebrew",
                json!({"content_type": ct, "name": "Test", "data": "{}"}),
            )
            .await;
            assert!(matches!(err, McpError::NoActiveCampaign));
        }
    }
}
