//! Tauri Command Integration Tests
//!
//! This test module verifies that Tauri commands work correctly with AppState,
//! catching state management issues that would otherwise only appear at runtime.
//!
//! The tests construct a real AppState with:
//! - In-memory SQLite database
//! - Temporary directories for paths
//! - Real service instances (SessionManager, ContextState, etc.)
//!
//! This helps prevent issues like "state not managed" errors that occur when
//! commands use State<'_, T> where T is not registered with .manage().

mod commands;
