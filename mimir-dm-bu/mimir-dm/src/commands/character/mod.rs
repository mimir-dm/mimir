//! Character and player management command handlers.
//!
//! Contains commands for managing player characters, NPCs,
//! and player associations with campaigns.

#[allow(clippy::module_inception)]
pub mod character;
pub mod player;
mod types;

pub use character::*;
pub use player::*;
