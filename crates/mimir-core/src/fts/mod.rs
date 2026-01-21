//! Full-Text Search Module
//!
//! Provides full-text search capabilities for catalog entities using SQLite FTS5.

mod entry_flattener;
mod search;

pub use entry_flattener::{flatten_entries, flatten_entry, strip_5etools_tags};
pub use search::*;
