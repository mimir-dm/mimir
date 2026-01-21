//! Data Access Layer
//!
//! Low-level database operations using Diesel.
//!
//! This module provides direct database access functions that work with
//! Diesel connections. Higher-level service layers should use these
//! functions to implement business logic.

pub mod campaign;
pub mod catalog;

pub use campaign::*;
pub use catalog::*;
