//! Shared server state (MIMIR-T-0704).

use std::sync::Arc;

use diesel::SqliteConnection;

use crate::config::Config;
use crate::error::ApiError;
use crate::live::Hub;

/// State shared by every handler.
#[derive(Clone)]
pub struct AppState {
    pub config: Arc<Config>,
    /// Live events and the display states (MIMIR-T-0712).
    pub live: Arc<Hub>,
    db_url: Arc<String>,
}

impl AppState {
    pub fn new(config: Config) -> Self {
        let db_url = config.database_path().to_string_lossy().into_owned();
        Self {
            config: Arc::new(config),
            live: Arc::new(Hub::new()),
            db_url: Arc::new(db_url),
        }
    }

    /// A new connection (WAL; connections are opened per request, as the
    /// desktop app does).
    pub fn connect(&self) -> Result<SqliteConnection, ApiError> {
        mimir_core::db::create_connection(&self.db_url)
            .map_err(|e| ApiError::internal(format!("database connection failed: {e}")))
    }

    /// Run blocking database work off the async runtime.
    pub async fn with_db<T, F>(&self, work: F) -> Result<T, ApiError>
    where
        T: Send + 'static,
        F: FnOnce(&mut SqliteConnection) -> Result<T, ApiError> + Send + 'static,
    {
        let state = self.clone();
        tokio::task::spawn_blocking(move || {
            let mut conn = state.connect()?;
            work(&mut conn)
        })
        .await
        .map_err(|e| ApiError::internal(format!("database task failed: {e}")))?
    }
}
