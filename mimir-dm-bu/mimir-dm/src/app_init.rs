//! Application initialization and directory management

use anyhow::{Context, Result};
use directories::ProjectDirs;
use std::fs;
use std::path::PathBuf;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Layer};

pub struct AppPaths {
    pub app_dir: PathBuf,
    pub config_dir: PathBuf,
    pub data_dir: PathBuf,
    pub logs_dir: PathBuf,
    pub database_path: PathBuf,
    pub is_memory_db: bool,
}

impl AppPaths {
    /// Get the campaigns folder name based on dev/release mode
    pub fn campaigns_folder_name() -> &'static str {
        let is_dev = cfg!(debug_assertions) || std::env::var("MIMIR_DEV").is_ok();
        if is_dev {
            "DevMimirCampaigns"
        } else {
            "MimirCampaigns"
        }
    }

    /// Get application directories, creating them if they don't exist (without logging)
    pub fn init_directories() -> Result<Self> {
        // Check if we're in development mode
        let is_dev = cfg!(debug_assertions) || std::env::var("MIMIR_DEV").is_ok();

        // Determine app name based on mode
        let app_name = if is_dev { "mimir-test" } else { "mimir" };
        let project_dirs = ProjectDirs::from("com", "mimir", app_name)
            .context("Failed to determine application directories")?;

        let app_dir = project_dirs.data_dir().to_path_buf();
        let config_dir = project_dirs.config_dir().to_path_buf();
        let data_dir = app_dir.join("data");
        let logs_dir = app_dir.join("logs");
        let database_path = data_dir.join("mimir.db");

        eprintln!(
            "Initializing {} application directories:",
            if is_dev { "DEVELOPMENT" } else { "PRODUCTION" }
        );
        eprintln!("  App dir: {}", app_dir.display());
        eprintln!("  Config dir: {}", config_dir.display());
        eprintln!("  Data dir: {}", data_dir.display());
        eprintln!("  Logs dir: {}", logs_dir.display());
        eprintln!("  Database: {}", database_path.display());

        // Create directories if they don't exist
        fs::create_dir_all(&app_dir)
            .with_context(|| format!("Failed to create app directory: {}", app_dir.display()))?;

        fs::create_dir_all(&config_dir).with_context(|| {
            format!(
                "Failed to create config directory: {}",
                config_dir.display()
            )
        })?;

        fs::create_dir_all(&data_dir)
            .with_context(|| format!("Failed to create data directory: {}", data_dir.display()))?;

        fs::create_dir_all(&logs_dir)
            .with_context(|| format!("Failed to create logs directory: {}", logs_dir.display()))?;

        // Create chat sessions log subdirectory
        let chat_logs_dir = logs_dir.join("chat_sessions");
        fs::create_dir_all(&chat_logs_dir).with_context(|| {
            format!(
                "Failed to create chat logs directory: {}",
                chat_logs_dir.display()
            )
        })?;

        Ok(AppPaths {
            app_dir,
            config_dir,
            data_dir,
            logs_dir,
            database_path,
            is_memory_db: false,
        })
    }

    /// Check if this is a new database (for seeding purposes)
    pub fn is_new_database(&self) -> bool {
        !self.database_path.exists()
    }

    /// Get the database path as a string
    pub fn database_path_str(&self) -> String {
        self.database_path.to_string_lossy().to_string()
    }
}

/// Initialize the application on startup
pub fn initialize_app() -> Result<AppPaths> {
    // First initialize directories (without logging since we need the logs dir)
    let app_paths =
        AppPaths::init_directories().context("Failed to initialize application directories")?;

    // Now set up logging with the logs directory available
    setup_logging(&app_paths.logs_dir).context("Failed to set up logging")?;

    info!("Starting Mimir application initialization...");
    info!("Application directories initialized successfully");

    // Note: Database initialization now happens in main.rs using core's DatabaseService

    Ok(app_paths)
}

/// Set up logging to both console and rotating files (blocking writers for simplicity)
fn setup_logging(logs_dir: &PathBuf) -> Result<()> {
    // Determine log level based on environment and build type
    let default_level = if cfg!(debug_assertions) {
        "debug"
    } else {
        "info"
    };
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| {
        // Default filter: debug for our crates, info for others
        EnvFilter::new(format!(
            "{}={},mimir_dm=debug,mimir_dm_llm=debug,mimir_dm_core=debug",
            default_level, default_level
        ))
    });

    // Create daily rotating file appender (blocking)
    let file_appender = tracing_appender::rolling::daily(logs_dir, "mimir.log");

    // Create separate filters for console and file
    let file_filter = env_filter;
    // Console filter has same levels but excludes file_only target logs
    let console_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| {
        // Same filter as file but with file_only=off
        EnvFilter::new(format!(
            "{}={},mimir_dm=debug,mimir_dm_llm=debug,mimir_dm_core=debug,file_only=off",
            default_level, default_level
        ))
    });

    // Build the subscriber with both console and file outputs
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer()
                .with_writer(file_appender)
                .with_ansi(false) // No color codes in files
                .with_target(true)
                .with_thread_ids(true)
                .with_line_number(true)
                .with_file(true)
                .with_filter(file_filter), // Full logging to file
        )
        .with(
            tracing_subscriber::fmt::layer()
                .with_writer(std::io::stderr)
                .with_ansi(true) // Color codes for console
                .with_target(false) // Less verbose for console
                .with_thread_ids(false)
                .with_line_number(false)
                .with_file(false)
                .with_filter(console_filter), // Truncated logging to console
        )
        .init();

    Ok(())
}
