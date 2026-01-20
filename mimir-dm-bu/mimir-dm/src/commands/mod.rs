//! Tauri command handlers
//!
//! Commands are organized into subdirectories by responsibility:
//! - `campaign/` - Campaign, module, and session management
//! - `catalog/` - 5e catalog content (spells, monsters, items, etc.)
//! - `character/` - Character and player management
//! - `chat/` - LLM chat sessions and todos
//! - `content/` - Documents, boards, and book imports
//! - `print/` - PDF generation from Typst templates
//! - `system/` - App info, logging, and window management

pub mod campaign;
pub mod catalog;
pub mod character;
pub mod chat;
pub mod content;
pub mod print;
pub mod system;

// Re-export all commands for backward compatibility
pub use campaign::*;
pub use catalog::*;
pub use character::*;
pub use chat::*;
pub use content::*;
pub use print::*;
pub use system::*;
