//! MCP Response Helpers
//!
//! Standardized response wrappers for MCP tools, providing a consistent API contract
//! for AI consumers. Analogous to Tauri's `ApiResponse<T>`.

use serde_json::{json, Value};

use crate::McpError;

/// Helper for constructing standardized MCP tool responses.
///
/// All methods return `Result<Value, McpError>` for consistency with tool handlers.
///
/// # Response Patterns
///
/// - **List**: `{ "entity_plural": [...], "count": N }`
/// - **Get**: `{ "entity": {...} }`
/// - **Created**: `{ "status": "created", "entity": {...} }`
/// - **Updated**: `{ "status": "updated", "entity": {...} }`
/// - **Deleted**: `{ "status": "deleted", "id": "..." }`
/// - **Success**: `{ "status": "success", "data": {...} }`
/// - **Ok**: Raw data as-is
pub struct McpResponse;

impl McpResponse {
    /// Response for listing multiple entities.
    ///
    /// # Arguments
    /// * `key` - Plural entity key (e.g., "campaigns", "modules", "items")
    /// * `items` - Vector of JSON values representing the entities
    ///
    /// # Returns
    /// `{ "key": [...], "count": N }`
    ///
    /// # Example
    /// ```ignore
    /// McpResponse::list("campaigns", campaign_data)
    /// // => { "campaigns": [...], "count": 5 }
    /// ```
    pub fn list(key: &str, items: Vec<Value>) -> Result<Value, McpError> {
        let count = items.len();
        Ok(json!({
            key: items,
            "count": count
        }))
    }

    /// Response for getting a single entity.
    ///
    /// # Arguments
    /// * `key` - Singular entity key (e.g., "campaign", "module", "item")
    /// * `data` - JSON value representing the entity
    ///
    /// # Returns
    /// `{ "key": {...} }`
    ///
    /// # Example
    /// ```ignore
    /// McpResponse::get("campaign", campaign_json)
    /// // => { "campaign": {...} }
    /// ```
    pub fn get(key: &str, data: Value) -> Result<Value, McpError> {
        Ok(json!({ key: data }))
    }

    /// Response for creating a new entity.
    ///
    /// # Arguments
    /// * `key` - Singular entity key (e.g., "campaign", "module", "item")
    /// * `data` - JSON value representing the created entity
    ///
    /// # Returns
    /// `{ "status": "created", "key": {...} }`
    ///
    /// # Example
    /// ```ignore
    /// McpResponse::created("campaign", campaign_json)
    /// // => { "status": "created", "campaign": {...} }
    /// ```
    pub fn created(key: &str, data: Value) -> Result<Value, McpError> {
        Ok(json!({
            "status": "created",
            key: data
        }))
    }

    /// Response for updating an existing entity.
    ///
    /// # Arguments
    /// * `key` - Singular entity key (e.g., "campaign", "module", "item")
    /// * `data` - JSON value representing the updated entity
    ///
    /// # Returns
    /// `{ "status": "updated", "key": {...} }`
    ///
    /// # Example
    /// ```ignore
    /// McpResponse::updated("campaign", campaign_json)
    /// // => { "status": "updated", "campaign": {...} }
    /// ```
    pub fn updated(key: &str, data: Value) -> Result<Value, McpError> {
        Ok(json!({
            "status": "updated",
            key: data
        }))
    }

    /// Response for deleting an entity.
    ///
    /// # Arguments
    /// * `id` - The ID of the deleted entity
    ///
    /// # Returns
    /// `{ "status": "deleted", "id": "..." }`
    ///
    /// # Example
    /// ```ignore
    /// McpResponse::deleted("abc-123")
    /// // => { "status": "deleted", "id": "abc-123" }
    /// ```
    pub fn deleted(id: &str) -> Result<Value, McpError> {
        Ok(json!({
            "status": "deleted",
            "id": id
        }))
    }

    /// Response for generic success with additional data.
    ///
    /// Use this for operations that don't fit the CRUD pattern,
    /// such as setting active campaign, export/import, etc.
    ///
    /// # Arguments
    /// * `data` - JSON value with the success details
    ///
    /// # Returns
    /// `{ "status": "success", "data": {...} }`
    ///
    /// # Example
    /// ```ignore
    /// McpResponse::success(json!({ "archive_path": "/path/to/file.tar.gz" }))
    /// // => { "status": "success", "data": { "archive_path": "/path/to/file.tar.gz" } }
    /// ```
    pub fn success(data: Value) -> Result<Value, McpError> {
        Ok(json!({
            "status": "success",
            "data": data
        }))
    }

    /// Response for raw data (no wrapper).
    ///
    /// Use sparingly - prefer structured responses for consistency.
    /// Useful for complex nested structures that don't fit standard patterns.
    ///
    /// # Arguments
    /// * `data` - JSON value to return as-is
    ///
    /// # Returns
    /// The data value unchanged.
    pub fn ok(data: Value) -> Result<Value, McpError> {
        Ok(data)
    }

    /// Response for operations that add/attach something to a parent entity.
    ///
    /// # Arguments
    /// * `key` - Singular key for what was added (e.g., "monster", "token")
    /// * `data` - JSON value representing the added item
    ///
    /// # Returns
    /// `{ "status": "added", "key": {...} }`
    ///
    /// # Example
    /// ```ignore
    /// McpResponse::added("monster", monster_json)
    /// // => { "status": "added", "monster": {...} }
    /// ```
    pub fn added(key: &str, data: Value) -> Result<Value, McpError> {
        Ok(json!({
            "status": "added",
            key: data
        }))
    }

    /// Response for operations that remove/detach something from a parent entity.
    ///
    /// # Arguments
    /// * `id` - The ID of the removed item
    ///
    /// # Returns
    /// `{ "status": "removed", "id": "..." }`
    ///
    /// # Example
    /// ```ignore
    /// McpResponse::removed("abc-123")
    /// // => { "status": "removed", "id": "abc-123" }
    /// ```
    pub fn removed(id: &str) -> Result<Value, McpError> {
        Ok(json!({
            "status": "removed",
            "id": id
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_response() {
        let items = vec![json!({"id": "1"}), json!({"id": "2"})];
        let result = McpResponse::list("items", items).unwrap();

        assert_eq!(result["count"], 2);
        assert!(result["items"].is_array());
        assert_eq!(result["items"].as_array().unwrap().len(), 2);
    }

    #[test]
    fn test_get_response() {
        let result = McpResponse::get("campaign", json!({"id": "1", "name": "Test"})).unwrap();

        assert_eq!(result["campaign"]["id"], "1");
        assert_eq!(result["campaign"]["name"], "Test");
    }

    #[test]
    fn test_created_response() {
        let result = McpResponse::created("module", json!({"id": "1"})).unwrap();

        assert_eq!(result["status"], "created");
        assert_eq!(result["module"]["id"], "1");
    }

    #[test]
    fn test_updated_response() {
        let result = McpResponse::updated("module", json!({"id": "1"})).unwrap();

        assert_eq!(result["status"], "updated");
        assert_eq!(result["module"]["id"], "1");
    }

    #[test]
    fn test_deleted_response() {
        let result = McpResponse::deleted("abc-123").unwrap();

        assert_eq!(result["status"], "deleted");
        assert_eq!(result["id"], "abc-123");
    }

    #[test]
    fn test_success_response() {
        let result = McpResponse::success(json!({"path": "/tmp/file"})).unwrap();

        assert_eq!(result["status"], "success");
        assert_eq!(result["data"]["path"], "/tmp/file");
    }

    #[test]
    fn test_added_response() {
        let result = McpResponse::added("token", json!({"id": "1"})).unwrap();

        assert_eq!(result["status"], "added");
        assert_eq!(result["token"]["id"], "1");
    }

    #[test]
    fn test_removed_response() {
        let result = McpResponse::removed("abc-123").unwrap();

        assert_eq!(result["status"], "removed");
        assert_eq!(result["id"], "abc-123");
    }
}
