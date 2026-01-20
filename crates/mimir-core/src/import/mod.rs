//! 5etools Import Module
//!
//! This module provides functionality for importing D&D 5e content directly from
//! 5etools data directories. It replaces the previous two-step process (splitter + upload)
//! with direct import to the database.
//!
//! # Architecture
//!
//! - `discovery` - Discovers available books/sources from 5etools data
//! - `filter` - Filters entities by source using multi-pattern matching
//! - `srd` - Identifies and transforms SRD (System Reference Document) content
//!
//! # Usage
//!
//! ```ignore
//! use mimir_core::import::{discover_sources, SourceFilter};
//!
//! // Discover available sources
//! let sources = discover_sources(&fivetools_path)?;
//!
//! // Load and filter content by source
//! let data = load_json_file(&spell_file)?;
//! let phb_spells = data.filter_by_source("PHB");
//! ```

mod discovery;
mod filter;
mod srd;

pub use discovery::*;
pub use filter::*;
pub use srd::*;
