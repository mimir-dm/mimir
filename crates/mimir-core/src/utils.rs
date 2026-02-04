//! Utility functions for mimir-core.
//!
//! Common helper functions used across services and DAL.

use chrono::Utc;

/// Returns the current UTC timestamp in RFC 3339 format.
///
/// This provides a consistent timestamp format across all database operations.
///
/// # Example
///
/// ```
/// use mimir_core::utils::now_rfc3339;
///
/// let timestamp = now_rfc3339();
/// // Returns something like "2026-02-04T14:30:00.123456789+00:00"
/// ```
pub fn now_rfc3339() -> String {
    Utc::now().to_rfc3339()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_now_rfc3339_format() {
        let timestamp = now_rfc3339();
        // Should be parseable as RFC 3339
        assert!(chrono::DateTime::parse_from_rfc3339(&timestamp).is_ok());
    }

    #[test]
    fn test_now_rfc3339_is_utc() {
        let timestamp = now_rfc3339();
        // UTC timestamps end with +00:00
        assert!(timestamp.ends_with("+00:00"));
    }
}
