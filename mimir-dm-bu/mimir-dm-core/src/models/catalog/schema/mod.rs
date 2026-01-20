//! 5etools JSON Schema management
//!
//! This module provides utilities for downloading and managing 5etools JSON schemas
//! used for type generation and validation.
//!
//! # Schema Source
//!
//! Schemas are sourced from the 5etools-utils repository:
//! <https://github.com/TheGiddyLimit/5etools-utils/tree/master/schema/brew-fast/>
//!
//! The brew-fast schemas use standard JSON Schema Draft 2020-12 format with all
//! `$$merge` preprocessing resolved, making them suitable for direct use with
//! schema-to-type generators like typify.
//!
//! # Updating Schemas
//!
//! To update vendored schemas:
//!
//! 1. Run the download utility (requires network access):
//!    ```bash
//!    cargo run --features schema-download --example download_schemas
//!    ```
//!
//! 2. Review changes to vendored files in `schema/vendored/`
//!
//! 3. If using typify for code generation, regenerate types:
//!    ```bash
//!    cargo run --features generate-types --example generate_types
//!    ```
//!
//! # Available Schemas
//!
//! - `bestiary.json` - Monster/creature definitions
//! - `class.json` - Character class definitions
//! - `races.json` - Race/lineage definitions
//! - `spells.json` - Spell definitions
//! - `items.json` - Equipment and magic item definitions
//! - `backgrounds.json` - Background definitions
//!
//! # Schema Version
//!
//! Schemas are pinned to a specific commit to ensure reproducible builds.
//! Update `SCHEMA_COMMIT` constant when updating to a new version.

/// Git commit hash for the vendored schema version
/// Use "master" for the default branch, or a specific commit hash for pinning
pub const SCHEMA_COMMIT: &str = "master";

/// Base URL for raw schema files
pub const SCHEMA_BASE_URL: &str =
    "https://raw.githubusercontent.com/TheGiddyLimit/5etools-utils";

/// Schema file paths relative to the base URL
pub mod paths {
    pub const BESTIARY: &str = "schema/brew-fast/bestiary/bestiary.json";
    pub const CLASS: &str = "schema/brew-fast/class/class.json";
    pub const RACES: &str = "schema/brew-fast/races.json";
    pub const SPELLS: &str = "schema/brew-fast/spells/spells.json";
    pub const ITEMS: &str = "schema/brew-fast/items.json";
    pub const BACKGROUNDS: &str = "schema/brew-fast/backgrounds.json";

    /// Entry schema - contains shared entry types used by other schemas
    pub const ENTRY: &str = "schema/brew-fast/entry.json";
    /// Util schema - utility types and definitions
    pub const UTIL: &str = "schema/brew-fast/util.json";
}

/// Get the full URL for a schema file
pub fn schema_url(path: &str) -> String {
    format!("{}/{}/{}", SCHEMA_BASE_URL, SCHEMA_COMMIT, path)
}

#[cfg(feature = "schema-download")]
pub mod download;
