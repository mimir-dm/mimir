//! Generated types from 5etools JSON Schema
//!
//! This module contains Rust types generated from 5etools JSON schemas using
//! [typify](https://github.com/oxidecomputer/typify). These types serve as the
//! authoritative reference implementation for the 5etools data format.
//!
//! # Purpose
//!
//! Generated types provide:
//! - Authoritative type definitions matching the upstream schema
//! - Reference implementation for hand-maintained types in parent modules
//! - Compile-time validation of schema changes
//!
//! # Usage Pattern
//!
//! Generated types are **not** used directly in the application. Instead:
//!
//! 1. Compare generated types against hand-maintained types in `../monster.rs`, etc.
//! 2. Update hand-maintained types to match any schema changes
//! 3. Hand-maintained types include additional features:
//!    - Database mapping (`From` impls for domain models)
//!    - Summary extraction for efficient display
//!    - Custom serde attributes for edge cases
//!
//! # Schema Complexity
//!
//! The 5etools JSON schemas use extensive cross-file `$ref` references (e.g.,
//! `../util.json#/$defs/alignment`). Tools like typify don't support external
//! references directly, so we use one of these approaches:
//!
//! 1. **Schema bundling** - Use a tool like `json-schema-ref-parser` to resolve
//!    all references into a single self-contained schema
//! 2. **Manual extraction** - Extract specific type definitions from schemas
//!    and adapt them for our needs
//! 3. **Reference comparison** - Use schemas as documentation to verify hand-
//!    maintained types match the upstream format
//!
//! For most entity types, approach #3 (reference comparison) combined with
//! targeted extraction works best, since we need custom serde handling anyway.
//!
//! # Regenerating Types
//!
//! To regenerate types after schema updates:
//!
//! ```bash
//! # 1. Download updated schemas
//! cargo run --features schema-download --example download_schemas
//!
//! # 2. For bundled schemas, regenerate:
//! cargo typify schema/vendored/bundled_bestiary.json --output generated/bestiary.rs
//! ```
//!
//! # Feature Flags
//!
//! This module is only compiled when the `generated-types` feature is enabled.
//! This avoids adding the generated code to normal builds.

// Note: Due to external $ref dependencies in 5etools schemas, automated type
// generation requires schema bundling. The vendored schemas serve as the
// authoritative reference for manual type definitions in the parent modules.
//
// See ../schema/vendored/ for the raw schemas
// See ../types.rs for shared polymorphic type definitions
// See ../monster.rs, ../class.rs, etc. for entity-specific types
