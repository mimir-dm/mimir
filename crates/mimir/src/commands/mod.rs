//! Tauri Command Handlers
//!
//! This module contains all Tauri commands that bridge the Vue.js frontend
//! with the mimir-core service layer.

pub mod archive;
pub mod asset;
pub mod campaign;
pub mod catalog;
pub mod character;
pub mod dev;
pub mod dm_map;
pub mod document;
pub mod map;
pub mod module;
pub mod player_display;
pub mod print;
pub mod source;

use serde::Serialize;
use serde_json::Value;

/// API Response wrapper for frontend compatibility.
///
/// The frontend expects responses in this format with success/error handling.
#[derive(Debug, Serialize)]
pub struct ApiResponse<T: Serialize> {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl<T: Serialize> ApiResponse<T> {
    /// Create a successful response with data.
    pub fn ok(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
        }
    }

    /// Create an error response.
    pub fn err(message: impl Into<String>) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(message.into()),
        }
    }
}

/// Helper to convert service results to API responses.
pub fn to_api_response<T: Serialize, E: std::fmt::Display>(
    result: Result<T, E>,
) -> ApiResponse<T> {
    match result {
        Ok(data) => ApiResponse::ok(data),
        Err(e) => ApiResponse::err(e.to_string()),
    }
}

/// Trait for catalog entities that store JSON data as a string.
/// This allows us to return parsed JSON to the frontend.
pub trait CatalogEntity {
    fn id(&self) -> Option<i32>;
    fn name(&self) -> &str;
    fn source(&self) -> &str;
    fn data(&self) -> &str;
    fn fluff(&self) -> Option<&str> { None }
}

/// Convert a catalog entity to a JSON Value with parsed data.
/// Merges the entity's metadata (id, name, source) with the parsed data blob.
pub fn entity_to_json<E: CatalogEntity>(entity: &E) -> Value {
    // Parse the data field
    let mut json: Value = serde_json::from_str(entity.data()).unwrap_or(Value::Object(Default::default()));

    // Ensure it's an object and add/override metadata fields
    if let Value::Object(ref mut map) = json {
        if let Some(id) = entity.id() {
            map.insert("id".to_string(), Value::Number(id.into()));
        }
        map.insert("name".to_string(), Value::String(entity.name().to_string()));
        map.insert("source".to_string(), Value::String(entity.source().to_string()));

        // Add fluff if present
        if let Some(fluff) = entity.fluff() {
            if let Ok(fluff_json) = serde_json::from_str::<Value>(fluff) {
                map.insert("fluff".to_string(), fluff_json);
            }
        }
    }

    json
}

/// Convert a vector of catalog entities to JSON Values.
pub fn entities_to_json<E: CatalogEntity>(entities: Vec<E>) -> Vec<Value> {
    entities.iter().map(|e| entity_to_json(e)).collect()
}
