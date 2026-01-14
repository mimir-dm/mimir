//! Data Access Layer - Repository pattern for database operations
//!
//! Organized into domains matching the model structure:
//! - `campaign`: Repositories for campaign management
//! - `character`: Repositories for character management
//! - `player`: Repositories for player management
//! - `traits`: Repository trait definitions for testability
//! - `mocks`: Mock implementations for unit testing

pub mod campaign;
pub mod character;
pub mod mocks;
pub mod player;
pub mod traits;
