//! Mimir D&D Campaign Manager
//!
//! Desktop application for managing D&D 5e campaigns, built with Tauri v2.
//!
//! This crate provides the Tauri command handlers that bridge the Vue.js frontend
//! with the mimir-core service layer.

pub mod commands;
pub mod error;
pub mod state;

pub use error::{CommandError, CommandResult};
pub use state::{is_dev_mode, AppPaths, AppState};
