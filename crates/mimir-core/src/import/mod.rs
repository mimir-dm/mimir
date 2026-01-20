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
//! - `collector` - Generic entity collection patterns
//!
//! # Usage
//!
//! ```ignore
//! use mimir_core::import::{collect_source_entities, discover_available_sources};
//!
//! // Discover available sources
//! let sources = discover_available_sources(&fivetools_path)?;
//!
//! // Collect all entities from a specific book
//! let phb_content = collect_source_entities(&fivetools_path, "PHB")?;
//! println!("Found {} monsters", phb_content.count("monster"));
//!
//! // Collect SRD content only
//! let srd_content = collect_srd_content(&fivetools_path)?;
//! ```

mod collector;
mod discovery;
mod filter;
mod srd;

pub use collector::*;
pub use discovery::*;
pub use filter::*;
pub use srd::*;
