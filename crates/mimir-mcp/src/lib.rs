//! Mimir MCP Server
//!
//! Model Context Protocol server for D&D 5e campaign management.
//! Enables Claude Code and other MCP clients to interact with Mimir campaigns.

pub mod context;
pub mod error;
pub mod handler;
pub mod response;
pub mod tools;

pub use context::McpContext;
pub use error::McpError;
pub use handler::MimirHandler;
pub use response::McpResponse;
