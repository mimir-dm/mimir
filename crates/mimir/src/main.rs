//! Mimir Application Entry Point
//!
//! Initializes the Tauri application with database connection and runs the event loop.

#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use mimir_core::db::init_database;
use mimir_lib::{AppPaths, AppState};
use mimir_print::PrintState;
use tauri::Manager;
use tracing_subscriber::fmt::writer::MakeWriterExt;
use tracing_subscriber::EnvFilter;

fn main() {
    // Set up file-based logging in the app's logs directory
    #[cfg(target_os = "macos")]
    let log_dir = std::env::var("HOME")
        .map(|h| std::path::PathBuf::from(h).join("Library/Application Support/com.mimir.app/logs"))
        .unwrap_or_else(|_| std::path::PathBuf::from("logs"));
    #[cfg(target_os = "windows")]
    let log_dir = std::env::var("APPDATA")
        .map(|h| std::path::PathBuf::from(h).join("com.mimir.app/logs"))
        .unwrap_or_else(|_| std::path::PathBuf::from("logs"));
    #[cfg(target_os = "linux")]
    let log_dir = std::env::var("HOME")
        .map(|h| std::path::PathBuf::from(h).join(".local/share/com.mimir.app/logs"))
        .unwrap_or_else(|_| std::path::PathBuf::from("logs"));
    std::fs::create_dir_all(&log_dir).ok();

    let file_appender = tracing_appender::rolling::daily(&log_dir, "mimir.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    // Log to both stderr and file
    let writer = std::io::stderr.and(non_blocking);

    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("mimir=info,mimir_core=info"))
        )
        .with_writer(writer)
        .init();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            // Get Tauri's app data directory
            let tauri_app_data_dir = app
                .path()
                .app_data_dir()
                .expect("Failed to get app data directory");

            // Initialize paths (creates directories, handles dev/prod separation)
            let paths = AppPaths::from_tauri_path(tauri_app_data_dir)
                .expect("Failed to initialize application paths");

            // Initialize database with migrations (this creates the DB and runs migrations)
            let _conn = init_database(&paths.database_url())
                .expect("Failed to initialize database");
            // Connection is dropped here - we'll create on-demand connections

            // Create and manage app state (stores DB path for on-demand connections)
            let state = AppState::new(paths.clone());
            app.manage(state);

            // Create and manage print state
            let print_state = PrintState::new(
                paths.app_dir.join("templates"),
                paths.assets_dir.clone(),
            );
            app.manage(print_state);

            Ok(())
        })
        .invoke_handler(mimir_lib::ipc::invoke_handler())
        .run(tauri::generate_context!())
        .expect("Error running Mimir application");
}
