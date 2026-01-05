//! Consolidated application state for Tauri
//!
//! This module defines the `AppState` struct that consolidates all shared
//! application state into a single managed resource. This improves code
//! organization and makes state dependencies explicit.

use crate::app_init::AppPaths;
use crate::commands::chat_sessions::SessionManager;
use crate::services::context_service::ContextState;
use crate::services::llm::{CancellationTokens, ConfirmationReceivers, LlmService};
use crate::services::mcp_server_manager::McpServerManager;
use diesel::SqliteConnection;
use mimir_dm_core::DatabaseService;
use std::ops::DerefMut;
use std::sync::Arc;
use tracing::error;

/// Consolidated application state managed by Tauri.
///
/// All command handlers receive this via `State<'_, AppState>` and can
/// access the specific services they need.
///
/// # Example
///
/// ```ignore
/// #[tauri::command]
/// pub async fn some_command(
///     state: State<'_, AppState>,
/// ) -> Result<SomeResponse, ApiError> {
///     let conn = state.db.get_connection()?;
///     // Use connection...
/// }
/// ```
pub struct AppState {
    /// Database service for all database operations
    pub db: Arc<DatabaseService>,

    /// Application paths (config dir, data dir, etc.)
    pub paths: Arc<AppPaths>,

    /// Context service for managing conversation context
    pub context: ContextState,

    /// Session manager for chat session persistence
    pub sessions: SessionManager,

    /// Receivers for LLM tool confirmations
    pub confirmations: ConfirmationReceivers,

    /// Cancellation tokens for LLM operations
    pub cancellations: CancellationTokens,

    /// LLM service (initialized asynchronously)
    pub llm: Arc<tokio::sync::Mutex<Option<LlmService>>>,

    /// MCP server process manager
    pub mcp: Arc<McpServerManager>,
}

impl AppState {
    /// Create a new AppState with all required services
    pub fn new(
        db: Arc<DatabaseService>,
        paths: Arc<AppPaths>,
        context: ContextState,
        sessions: SessionManager,
        confirmations: ConfirmationReceivers,
        cancellations: CancellationTokens,
        llm: Arc<tokio::sync::Mutex<Option<LlmService>>>,
        mcp: Arc<McpServerManager>,
    ) -> Self {
        Self {
            db,
            paths,
            context,
            sessions,
            confirmations,
            cancellations,
            llm,
            mcp,
        }
    }

    /// Execute a database operation with connection management and error handling.
    ///
    /// This reduces boilerplate in Tauri commands by:
    /// - Acquiring a connection from the pool
    /// - Logging errors consistently
    /// - Providing consistent error formatting
    ///
    /// # Arguments
    /// * `operation_name` - Description of the operation for error messages
    /// * `f` - Closure that receives a mutable connection and returns the operation result
    ///
    /// # Example
    /// ```ignore
    /// state.with_connection("action search", |conn| {
    ///     ActionService::new(conn).search_actions(filters)
    /// })
    /// ```
    pub fn with_connection<T, E, F>(&self, operation_name: &str, f: F) -> Result<T, String>
    where
        E: std::fmt::Display,
        F: FnOnce(&mut SqliteConnection) -> Result<T, E>,
    {
        let mut conn = self.db.get_connection().map_err(|e| {
            error!("Database connection error during {}: {}", operation_name, e);
            format!("Database connection failed: {}", e)
        })?;

        f(conn.deref_mut()).map_err(|e| {
            error!("{} failed: {}", operation_name, e);
            format!("{} failed: {}", operation_name, e)
        })
    }
}
