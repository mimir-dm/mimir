//! 5etools Catalog Types
//!
//! Hand-written extraction types for deserializing 5etools JSON data.
//! These types focus on the fields we need for indexing, searching, and display.
//!
//! # Design Principles
//!
//! 1. **Extraction-focused**: Only define fields we actually use
//! 2. **Permissive deserialization**: Use `Option<T>` liberally since 5etools data varies
//! 3. **JSON blobs for complex content**: Store `entries` as `serde_json::Value`
//! 4. **Polymorphic with `#[serde(untagged)]`**: Handle variant formats gracefully
//!
//! # Schema Reference
//!
//! These types are based on the 5etools JSON schemas in `schema/5etools/`.
//! See <https://github.com/TheGiddyLimit/5etools-utils/tree/master/schema/brew-fast>

mod action;
mod background;
mod class;
mod condition;
mod cult;
mod deity;
mod feat;
mod item;
mod language;
mod monster;
mod object;
mod optionalfeature;
mod psionic;
mod race;
mod reward;
mod spell;
mod table;
mod trap;
mod types;
mod variantrule;
mod vehicle;

pub use action::*;
pub use background::*;
pub use class::*;
pub use condition::*;
pub use cult::*;
pub use deity::*;
pub use feat::*;
pub use item::*;
pub use language::*;
pub use monster::*;
pub use object::*;
pub use optionalfeature::*;
pub use psionic::*;
pub use race::*;
pub use reward::*;
pub use spell::*;
pub use table::*;
pub use trap::*;
pub use types::*;
pub use variantrule::*;
pub use vehicle::*;
