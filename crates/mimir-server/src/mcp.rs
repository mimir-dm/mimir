//! MCP at `/mcp` (MIMIR-T-0725): streamable HTTP with rmcp (as Kairos),
//! serving the tools of the mimir-mcp registry over the same core
//! services. One handler per MCP session, so the "active campaign" is the
//! session's. The DM only: the route is behind the API's authentication
//! and `dm_only` (a player link is refused; open mode lets every caller
//! in, as the API).

use std::sync::Arc;

use axum::middleware;
use axum::Router;
use mimir_mcp::{registry, McpContext};
use rmcp::model::{
    CallToolRequestParams, CallToolResult, ContentBlock, Implementation, ListToolsResult,
    PaginatedRequestParams, ServerCapabilities, ServerInfo, Tool,
};
use rmcp::service::RequestContext;
use rmcp::transport::streamable_http_server::session::local::LocalSessionManager;
use rmcp::transport::{StreamableHttpServerConfig, StreamableHttpService};
use rmcp::{ErrorData, RoleServer, ServerHandler};
use serde_json::Value;

use crate::auth;
use crate::state::AppState;

/// The tools for one MCP session.
#[derive(Clone)]
pub struct MimirMcp {
    ctx: Arc<McpContext>,
}

impl MimirMcp {
    pub fn new(state: &AppState) -> Self {
        let url = state.config.database_path().to_string_lossy().into_owned();
        MimirMcp {
            ctx: Arc::new(McpContext::from_parts(url, state.config.data_dir.clone())),
        }
    }
}

/// The registry's tools as MCP tools.
pub fn tools() -> Vec<Tool> {
    registry::all_tools()
        .iter()
        .chain(crate::mcp_tools::server_tools())
        .map(|t| {
            let schema = serde_json::to_value((t.input_schema)())
                .ok()
                .and_then(|v| v.as_object().cloned())
                .unwrap_or_default();
            Tool::new(t.name, t.description, Arc::new(schema))
        })
        .collect()
}

impl ServerHandler for MimirMcp {
    async fn list_tools(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListToolsResult, ErrorData> {
        Ok(ListToolsResult::with_all_items(tools()))
    }

    async fn call_tool(
        &self,
        request: CallToolRequestParams,
        _context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, ErrorData> {
        let Some(tool) = crate::mcp_tools::find(&request.name) else {
            return Err(ErrorData::invalid_params(
                format!("unknown tool: {}", request.name),
                None,
            ));
        };
        let args = Value::Object(request.arguments.unwrap_or_default());
        // As the stdio server: the result as pretty JSON text; an error as
        // a tool error ("Error: …").
        Ok(match (tool.handler)(&self.ctx, args).await {
            Ok(v) => CallToolResult::success(vec![ContentBlock::text(
                serde_json::to_string_pretty(&v).unwrap_or_else(|_| v.to_string()),
            )]),
            Err(e) => CallToolResult::error(vec![ContentBlock::text(format!("Error: {e}"))]),
        })
    }

    fn get_info(&self) -> ServerInfo {
        ServerInfo::new(ServerCapabilities::builder().enable_tools().build())
            .with_server_info(
                Implementation::new("mimir", env!("CARGO_PKG_VERSION")).with_title("Mimir"),
            )
            .with_instructions(mimir_mcp::INSTRUCTIONS)
    }
}

/// `/mcp`, behind the API's authentication, the DM only.
pub fn router(state: AppState) -> Router<AppState> {
    let factory = {
        let state = state.clone();
        move || Ok(MimirMcp::new(&state))
    };
    // The server answers on its tailnet name: no Host allowlist (as Kairos).
    let service = StreamableHttpService::new(
        factory,
        Arc::new(LocalSessionManager::default()),
        StreamableHttpServerConfig::default().disable_allowed_hosts(),
    );
    Router::new()
        .route_service("/mcp", service)
        .route_layer(middleware::from_fn(auth::dm_only))
        .route_layer(middleware::from_fn_with_state(state, auth::authenticate))
}
