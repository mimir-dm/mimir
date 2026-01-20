//! MCP Server Tauri commands
//!
//! Commands for managing the MCP server process from the frontend.

use crate::state::AppState;
use serde::Serialize;
use tauri::State;

/// Status response for MCP server
#[derive(Debug, Serialize)]
pub struct McpServerStatus {
    /// Whether the MCP server process is running
    pub running: bool,
}

/// Get the current status of the MCP server
#[tauri::command]
pub async fn get_mcp_server_status(state: State<'_, AppState>) -> Result<McpServerStatus, String> {
    let running = state.mcp.is_running().await;
    Ok(McpServerStatus { running })
}

/// Start the MCP server process
#[tauri::command]
pub async fn start_mcp_server(state: State<'_, AppState>) -> Result<McpServerStatus, String> {
    state.mcp.start().await?;
    let running = state.mcp.is_running().await;
    Ok(McpServerStatus { running })
}

/// Stop the MCP server process
#[tauri::command]
pub async fn stop_mcp_server(state: State<'_, AppState>) -> Result<McpServerStatus, String> {
    state.mcp.stop().await?;
    let running = state.mcp.is_running().await;
    Ok(McpServerStatus { running })
}

/// Restart the MCP server process
#[tauri::command]
pub async fn restart_mcp_server(state: State<'_, AppState>) -> Result<McpServerStatus, String> {
    state.mcp.restart().await?;
    let running = state.mcp.is_running().await;
    Ok(McpServerStatus { running })
}
