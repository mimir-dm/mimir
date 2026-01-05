//! Common test utilities for Tauri command integration tests.
//!
//! Provides helpers to construct test AppState and other shared resources
//! for verifying Tauri commands work correctly with state injection.

use mimir_dm::app_init::AppPaths;
use mimir_dm::commands::chat::chat_sessions::SessionManager;
use mimir_dm::services::context_service::ContextState;
use mimir_dm::services::llm::{CancellationTokens, ConfirmationReceivers, LlmService};
use mimir_dm::services::mcp_server_manager::McpServerManager;
use mimir_dm::state::AppState;
use mimir_dm_core::DatabaseService;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tempfile::TempDir;
use tokio::sync::Mutex;

/// Test environment containing temporary directories and AppState
pub struct TestEnv {
    /// The constructed AppState for testing
    pub state: AppState,
    /// Temp directory - kept alive to prevent cleanup during test
    #[allow(dead_code)]
    pub temp_dir: TempDir,
    /// Test paths
    pub paths: Arc<AppPaths>,
}

impl TestEnv {
    /// Create a new test environment with in-memory database and temp directories
    pub async fn new() -> anyhow::Result<Self> {
        let temp_dir = TempDir::new()?;
        let base_path = temp_dir.path();

        // Create required subdirectories
        let app_dir = base_path.join("app");
        let config_dir = base_path.join("config");
        let data_dir = base_path.join("data");
        let logs_dir = base_path.join("logs");

        std::fs::create_dir_all(&app_dir)?;
        std::fs::create_dir_all(&config_dir)?;
        std::fs::create_dir_all(&data_dir)?;
        std::fs::create_dir_all(&logs_dir)?;
        std::fs::create_dir_all(logs_dir.join("chat_sessions"))?;
        std::fs::create_dir_all(data_dir.join("chat_sessions"))?;
        std::fs::create_dir_all(data_dir.join("books"))?;

        // Create test AppPaths
        let paths = Arc::new(AppPaths {
            app_dir,
            config_dir,
            data_dir: data_dir.clone(),
            logs_dir,
            database_path: data_dir.join("test.db"),
            is_memory_db: true,
        });

        // Initialize database with in-memory connection
        let db = Arc::new(DatabaseService::new(":memory:", true)?);
        {
            let mut conn = db.get_connection()?;
            mimir_dm_core::run_migrations(&mut conn)?;
        }

        // Create session manager
        let session_manager = SessionManager::new(&paths)?;

        // Create other state components
        let context_state = ContextState::new();
        let confirmations: ConfirmationReceivers = Arc::new(Mutex::new(HashMap::new()));
        let cancellations: CancellationTokens = Arc::new(Mutex::new(HashMap::new()));
        let llm: Arc<Mutex<Option<LlmService>>> = Arc::new(Mutex::new(None));
        let mcp = McpServerManager::new(paths.database_path.to_string_lossy().to_string());

        // Construct AppState
        let state = AppState::new(
            db,
            paths.clone(),
            context_state,
            session_manager,
            confirmations,
            cancellations,
            llm,
            mcp,
        );

        Ok(Self {
            state,
            temp_dir,
            paths,
        })
    }
}

/// Create a standalone test AppPaths pointing to a temp directory
#[allow(dead_code)]
pub fn create_test_paths(temp_dir: &TempDir) -> anyhow::Result<Arc<AppPaths>> {
    let base_path = temp_dir.path();

    let app_dir = base_path.join("app");
    let config_dir = base_path.join("config");
    let data_dir = base_path.join("data");
    let logs_dir = base_path.join("logs");

    std::fs::create_dir_all(&app_dir)?;
    std::fs::create_dir_all(&config_dir)?;
    std::fs::create_dir_all(&data_dir)?;
    std::fs::create_dir_all(&logs_dir)?;
    std::fs::create_dir_all(logs_dir.join("chat_sessions"))?;

    Ok(Arc::new(AppPaths {
        app_dir,
        config_dir,
        data_dir: data_dir.clone(),
        logs_dir,
        database_path: data_dir.join("test.db"),
        is_memory_db: true,
    }))
}

/// Create a test log file and return its path
pub fn create_test_log_file(logs_dir: &Path, name: &str, content: &str) -> anyhow::Result<PathBuf> {
    let log_path = logs_dir.join(name);
    std::fs::write(&log_path, content)?;
    Ok(log_path)
}

/// Create a test book directory with minimal content
pub fn create_test_book(data_dir: &Path, book_id: &str) -> anyhow::Result<PathBuf> {
    let book_dir = data_dir.join("books").join(book_id);
    let book_content_dir = book_dir.join("book");
    std::fs::create_dir_all(&book_content_dir)?;

    // Create a minimal book JSON file
    let book_json = serde_json::json!({
        "name": book_id,
        "id": book_id,
        "data": []
    });

    let book_file = book_content_dir.join(format!("book-{}.json", book_id.to_lowercase()));
    std::fs::write(&book_file, serde_json::to_string_pretty(&book_json)?)?;

    Ok(book_dir)
}
