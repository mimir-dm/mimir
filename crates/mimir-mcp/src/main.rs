//! Mimir MCP Server Binary
//!
//! Runs the MCP server over stdio for Claude Code integration.

use mimir_mcp::{McpContext, MimirHandler};
use rust_mcp_sdk::mcp_server::{server_runtime, McpServerOptions};
use rust_mcp_sdk::schema::{
    Implementation, InitializeResult, ServerCapabilities, ServerCapabilitiesTools,
    LATEST_PROTOCOL_VERSION,
};
use rust_mcp_sdk::{McpServer, StdioTransport, ToMcpServerHandler, TransportOptions};
use std::sync::Arc;
use tracing::info;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging to stderr (stdout is used for MCP protocol)
    tracing_subscriber::registry()
        .with(fmt::layer().with_writer(std::io::stderr))
        .with(EnvFilter::from_default_env().add_directive("mimir_mcp=info".parse()?))
        .init();

    info!("Starting Mimir MCP server");

    // Create context
    let context = Arc::new(McpContext::new().map_err(|e| anyhow::anyhow!("{}", e))?);

    // Create handler
    let handler = MimirHandler::with_context(context);

    // Server info for MCP protocol
    let server_details = InitializeResult {
        protocol_version: LATEST_PROTOCOL_VERSION.to_string(),
        capabilities: ServerCapabilities {
            tools: Some(ServerCapabilitiesTools { list_changed: None }),
            ..Default::default()
        },
        server_info: Implementation {
            name: "mimir-mcp".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            description: Some("MCP server for D&D 5e campaign management".to_string()),
            title: Some("Mimir MCP Server".to_string()),
            icons: vec![],
            website_url: Some("https://github.com/colliery-io/mimir".to_string()),
        },
        instructions: Some(
            "Mimir MCP Server for D&D 5e campaign management. \
             Use list_campaigns to see available campaigns, then set_active_campaign \
             to select one before using other tools."
                .to_string(),
        ),
        meta: None,
    };

    // Create stdio transport
    let transport =
        StdioTransport::new(TransportOptions::default()).map_err(|e| anyhow::anyhow!("{}", e))?;

    // Convert handler to MCP server handler
    let mcp_handler = handler.to_mcp_server_handler();

    // Create server options
    let options = McpServerOptions {
        server_details,
        transport,
        handler: mcp_handler,
        task_store: None,
        client_task_store: None,
    };

    // Create and start server
    let server = server_runtime::create_server(options);

    info!("Mimir MCP server ready");
    server.start().await.map_err(|e| anyhow::anyhow!("{:?}", e))?;

    Ok(())
}
