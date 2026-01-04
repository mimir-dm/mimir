//! Database Models
//!
//! Split into distinct domains:
//! - `catalog`: Static D&D reference data (races, classes, spells, items, monsters, etc.)
//! - `campaign`: Campaign management and story organization
//! - `player`: Player management and campaign associations
//! - `character`: Character data and version tracking
//!
//! # Update Pattern: `Option<Option<T>>`
//!
//! Update structs use `Option<Option<T>>` for nullable fields to distinguish between
//! three states:
//!
//! - `None` — Don't change this field (skip in UPDATE query)
//! - `Some(None)` — Set field to NULL
//! - `Some(Some(value))` — Set field to the given value
//!
//! This pattern is necessary because `Option<T>` alone cannot distinguish between
//! "don't update" and "set to NULL".
//!
//! ## Example
//!
//! ```ignore
//! // Don't change notes (field not included in UPDATE)
//! UpdateToken { notes: None, ..Default::default() }
//!
//! // Clear notes (SET notes = NULL)
//! UpdateToken { notes: Some(None), ..Default::default() }
//!
//! // Set notes to a value (SET notes = 'Hello')
//! UpdateToken { notes: Some(Some("Hello".to_string())), ..Default::default() }
//! ```
//!
//! Non-nullable fields use simple `Option<T>` where `None` means "don't update".

// Data models have many fields - documenting each would be verbose and redundant.
// Field names are chosen to be self-documenting.
#![allow(missing_docs)]

pub mod campaign;
pub mod catalog;
pub mod character;
pub mod player;
