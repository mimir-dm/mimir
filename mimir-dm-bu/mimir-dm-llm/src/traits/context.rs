//! Tool context for managing state between tool calls
//!
//! This module provides a shared context store that allows tools to communicate
//! state without direct coupling, enabling sophisticated tool coordination while
//! maintaining clean separation of concerns.

use chrono::{DateTime, Utc};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Shared context store for tool communication
///
/// The ToolContext provides a thread-safe way for tools to share state during
/// a conversation session. Tools can store arbitrary data that can be accessed
/// by other tools, enabling coordination without direct coupling.
///
/// Example usage:
/// ```rust
/// use mimir_dm_llm::traits::ToolContext;
/// use serde_json::json;
///
/// let context = ToolContext::new();
/// context.set("document_read:campaign_bible", json!({"timestamp": "2023-01-01T00:00:00Z"}));
/// if context.has_key("document_read:campaign_bible") {
///     // Safe to proceed with write operation
/// }
/// ```
#[derive(Debug, Clone)]
pub struct ToolContext {
    store: Arc<Mutex<HashMap<String, Value>>>,
    created_at: DateTime<Utc>,
    last_cleared: Arc<Mutex<DateTime<Utc>>>,
}

impl ToolContext {
    /// Create a new empty tool context
    pub fn new() -> Self {
        let now = Utc::now();
        Self {
            store: Arc::new(Mutex::new(HashMap::new())),
            created_at: now,
            last_cleared: Arc::new(Mutex::new(now)),
        }
    }

    /// Set a value in the context
    ///
    /// # Arguments
    /// * `key` - The key to store the value under
    /// * `value` - The JSON value to store
    ///
    /// # Returns
    /// Ok(()) on success, Err on lock poisoning
    pub fn set(&self, key: &str, value: Value) -> Result<(), String> {
        let mut store = self
            .store
            .lock()
            .map_err(|e| format!("Failed to acquire context lock: {}", e))?;
        store.insert(key.to_string(), value);
        Ok(())
    }

    /// Get a value from the context
    ///
    /// # Arguments
    /// * `key` - The key to retrieve
    ///
    /// # Returns
    /// Some(value) if the key exists, None otherwise
    pub fn get(&self, key: &str) -> Option<Value> {
        let store = self.store.lock().ok()?;
        store.get(key).cloned()
    }

    /// Check if a key exists in the context
    ///
    /// # Arguments
    /// * `key` - The key to check for
    ///
    /// # Returns
    /// true if the key exists, false otherwise
    pub fn has_key(&self, key: &str) -> bool {
        if let Ok(store) = self.store.lock() {
            store.contains_key(key)
        } else {
            false
        }
    }

    /// Clear all data from the context
    ///
    /// This is typically called between conversation sessions to ensure
    /// tools don't have stale state from previous interactions.
    pub fn clear(&self) -> Result<(), String> {
        let mut store = self
            .store
            .lock()
            .map_err(|e| format!("Failed to acquire context lock: {}", e))?;
        store.clear();

        let mut last_cleared = self
            .last_cleared
            .lock()
            .map_err(|e| format!("Failed to acquire last_cleared lock: {}", e))?;
        *last_cleared = Utc::now();

        Ok(())
    }

    /// Clear entries that match a specific pattern
    ///
    /// This can be used to clear related entries without affecting
    /// the entire context. For example, clearing all document reads
    /// for a specific campaign.
    ///
    /// # Arguments
    /// * `pattern` - The pattern to match keys against (currently supports prefix matching)
    pub fn clear_pattern(&self, pattern: &str) -> Result<(), String> {
        let mut store = self
            .store
            .lock()
            .map_err(|e| format!("Failed to acquire context lock: {}", e))?;
        store.retain(|k, _| !k.starts_with(pattern));
        Ok(())
    }

    /// Get the number of entries in the context
    pub fn len(&self) -> usize {
        if let Ok(store) = self.store.lock() {
            store.len()
        } else {
            0
        }
    }

    /// Check if the context is empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Get when the context was created
    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    /// Get when the context was last cleared
    pub fn last_cleared(&self) -> Option<DateTime<Utc>> {
        self.last_cleared.lock().ok().map(|dt| *dt)
    }

    /// Get all keys in the context (for debugging)
    pub fn keys(&self) -> Vec<String> {
        if let Ok(store) = self.store.lock() {
            store.keys().cloned().collect()
        } else {
            Vec::new()
        }
    }
}

impl Default for ToolContext {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_context_basic_operations() {
        let context = ToolContext::new();

        // Test set and get
        context.set("test_key", json!("test_value")).unwrap();
        assert_eq!(context.get("test_key"), Some(json!("test_value")));

        // Test has_key
        assert!(context.has_key("test_key"));
        assert!(!context.has_key("nonexistent_key"));

        // Test len
        assert_eq!(context.len(), 1);
        assert!(!context.is_empty());
    }

    #[test]
    fn test_context_clear() {
        let context = ToolContext::new();

        context.set("key1", json!("value1")).unwrap();
        context.set("key2", json!("value2")).unwrap();

        assert_eq!(context.len(), 2);

        context.clear().unwrap();

        assert_eq!(context.len(), 0);
        assert!(context.is_empty());
        assert!(!context.has_key("key1"));
    }

    #[test]
    fn test_context_clear_pattern() {
        let context = ToolContext::new();

        context.set("read:doc1", json!("value1")).unwrap();
        context.set("read:doc2", json!("value2")).unwrap();
        context.set("other:doc1", json!("value3")).unwrap();

        assert_eq!(context.len(), 3);

        context.clear_pattern("read:").unwrap();

        assert_eq!(context.len(), 1);
        assert!(!context.has_key("read:doc1"));
        assert!(!context.has_key("read:doc2"));
        assert!(context.has_key("other:doc1"));
    }

    #[test]
    fn test_context_complex_values() {
        let context = ToolContext::new();

        let complex_value = json!({
            "timestamp": "2023-01-01T00:00:00Z",
            "content_hash": "abc123",
            "metadata": {
                "file_path": "/path/to/doc.md",
                "size": 1024
            }
        });

        context
            .set("document:campaign_1:bible", complex_value.clone())
            .unwrap();
        assert_eq!(
            context.get("document:campaign_1:bible"),
            Some(complex_value)
        );
    }

    #[test]
    fn test_context_keys() {
        let context = ToolContext::new();

        context.set("key1", json!("value1")).unwrap();
        context.set("key2", json!("value2")).unwrap();

        let mut keys = context.keys();
        keys.sort();

        assert_eq!(keys, vec!["key1", "key2"]);
    }
}
