//! Mimir DM - D&D Campaign Assistant
//!
//! This library crate exposes the application modules for testing.
//! The main entry point is in main.rs.

// Use path attributes to share modules with main.rs
#[path = "app_init.rs"]
pub mod app_init;

#[path = "commands/mod.rs"]
pub mod commands;

#[path = "embedded_test_book.rs"]
pub mod embedded_test_book;

#[path = "seed_templates.rs"]
pub mod seed_templates;

#[path = "services/mod.rs"]
pub mod services;

#[path = "state.rs"]
pub mod state;

#[path = "types.rs"]
pub mod types;
