//! Mimir MCP Server Library
//!
//! Provides MCP (Model Context Protocol) server functionality for managing
//! D&D 5e campaigns through Claude Code.
//!
//! # Overview
//!
//! This crate implements an MCP server that exposes campaign management tools:
//!
//! - **Campaign Context**: Set and manage the active campaign
//! - **Document Authoring**: List, read, and write campaign documents
//! - **Character Management**: Create and manage characters and NPCs
//! - **Module Management**: View and manage campaign modules
//!
//! # Usage
//!
//! This crate is primarily used as a binary (`mimir-mcp`), but the modules
//! are also available as a library for testing and integration.

pub mod context;
pub mod error;
pub mod handler;
pub mod tools;

pub use context::McpContext;
pub use error::McpError;
pub use handler::MimirHandler;
