//! System and utility command handlers.
//!
//! Contains commands for application info, logging, development tools,
//! and window management.

pub mod app_info;
pub mod dev_tools;
pub mod logs;
pub mod mcp_server;
pub mod window_manager;

pub use app_info::*;
pub use dev_tools::*;
pub use mcp_server::*;
pub use window_manager::*;
