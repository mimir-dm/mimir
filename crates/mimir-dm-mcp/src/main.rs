//! Mimir MCP Server
//!
//! An MCP server for managing D&D 5e campaigns with Claude Code.
//!
//! # Usage
//!
//! ```bash
//! # Auto-discover database from Mimir app data directory
//! mimir-mcp
//!
//! # Specify database path explicitly
//! mimir-mcp --database /path/to/mimir.db
//!
//! # Or via environment variable
//! MIMIR_DATABASE_PATH=/path/to/mimir.db mimir-mcp
//! ```

use clap::Parser;
use mimir_dm_core::AppPaths;
use mimir_dm_mcp::{context::McpContext, handler::MimirHandler};
use rust_mcp_sdk::mcp_server::{server_runtime, McpServerOptions};
use rust_mcp_sdk::schema::{Implementation, InitializeResult, ServerCapabilities};
use rust_mcp_sdk::{McpServer, StdioTransport, ToMcpServerHandler, TransportOptions};
use std::env;
use tracing::{error, info};
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

/// Mimir MCP Server - Campaign management for Claude Code
#[derive(Parser, Debug)]
#[command(name = "mimir-mcp")]
#[command(about = "MCP server for managing D&D 5e campaigns")]
#[command(version)]
struct Args {
    /// Path to the SQLite database (auto-discovered if not specified)
    #[arg(short, long)]
    database: Option<String>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging to stderr (since stdout is used for MCP transport)
    tracing_subscriber::registry()
        .with(fmt::layer().with_writer(std::io::stderr))
        .with(EnvFilter::from_default_env().add_directive("mimir_dm_mcp=info".parse()?))
        .init();

    let args = Args::parse();

    // Determine database path with priority:
    // 1. CLI argument
    // 2. Environment variable
    // 3. Auto-discovery from Mimir app data directory
    let database_path = if let Some(path) = args.database {
        path
    } else if let Ok(path) = env::var("MIMIR_DATABASE_PATH") {
        path
    } else if let Some(paths) = AppPaths::resolve() {
        info!("Auto-discovered Mimir data directory");
        paths.database_url()
    } else {
        return Err(anyhow::anyhow!(
            "Could not determine database path. Please specify with --database or MIMIR_DATABASE_PATH"
        ));
    };

    info!(database = %database_path, "Starting Mimir MCP server");

    // Verify database exists
    if !std::path::Path::new(&database_path).exists() {
        error!(database = %database_path, "Database file not found");
        return Err(anyhow::anyhow!(
            "Database file not found: {}. Please run the Mimir application first to create the database.",
            database_path
        ));
    }

    // Create context
    let context = McpContext::new(database_path);

    // Create handler
    let handler = MimirHandler::new(context);

    // Server info for MCP protocol
    let server_details = InitializeResult {
        protocol_version: "2024-11-05".to_string(),
        capabilities: ServerCapabilities {
            tools: Some(rust_mcp_sdk::schema::ServerCapabilitiesTools {
                list_changed: None,
            }),
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
             Use set_active_campaign first, then use other tools to manage \
             documents, characters, and NPCs."
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
