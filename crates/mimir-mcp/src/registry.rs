//! Tool Registry
//!
//! A tool is defined in ONE place: a typed argument struct (which is both the
//! JSON-schema source and the deserialization target), a handler taking that
//! struct, and a single registration. Dispatch, argument decoding, and schema
//! publication all consume the registry, so the published contract and the
//! enforced contract cannot drift.
//!
//! Families migrate incrementally: `MimirHandler` consults the registry first
//! and falls back to the legacy match for families not yet moved.

use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, OnceLock};

use rust_mcp_sdk::schema::{Tool, ToolInputSchema};
use serde_json::Value;

use crate::context::McpContext;
use crate::McpError;

/// Maps a Rust argument field type to its JSON-schema type string and
/// requiredness. `Option<T>` is optional; everything else is required.
pub trait JsonArgType {
    /// JSON schema `type` value for this field.
    const JSON_TYPE: &'static str;
    /// Whether the field appears in the schema's `required` list.
    const REQUIRED: bool = true;
}

impl JsonArgType for String {
    const JSON_TYPE: &'static str = "string";
}
impl JsonArgType for i64 {
    const JSON_TYPE: &'static str = "integer";
}
impl JsonArgType for i32 {
    const JSON_TYPE: &'static str = "integer";
}
impl JsonArgType for f64 {
    const JSON_TYPE: &'static str = "number";
}
impl JsonArgType for bool {
    const JSON_TYPE: &'static str = "boolean";
}
impl<T: JsonArgType> JsonArgType for Option<T> {
    const JSON_TYPE: &'static str = T::JSON_TYPE;
    const REQUIRED: bool = false;
}
impl<T> JsonArgType for Vec<T> {
    const JSON_TYPE: &'static str = "array";
}

/// Build a `ToolInputSchema` from collected field metadata.
///
/// Mirrors the wire shape of the legacy `create_properties` helper: no
/// `properties` key at all when a tool takes no arguments.
pub fn build_input_schema(
    required: Vec<String>,
    fields: Vec<(&'static str, &'static str, String)>,
) -> ToolInputSchema {
    let properties = if fields.is_empty() {
        None
    } else {
        let mut map = HashMap::new();
        for (name, json_type, description) in fields {
            let mut inner = serde_json::Map::new();
            inner.insert("type".to_string(), Value::String(json_type.to_string()));
            inner.insert("description".to_string(), Value::String(description));
            map.insert(name.to_string(), inner);
        }
        Some(map)
    };
    ToolInputSchema::new(required, properties, None)
}

/// Declare a tool's argument struct once; get `Deserialize` and a
/// schema generator from the same field list.
///
/// Field doc comments become the JSON-schema `description`; `Option<T>`
/// fields are optional, all others required.
///
/// ```ignore
/// tool_args! {
///     pub struct AddMonsterArgs {
///         /// The ID of the module
///         pub module_id: String,
///         /// Number of this monster (default: 1)
///         pub count: Option<i64>,
///     }
/// }
/// ```
#[macro_export]
macro_rules! tool_args {
    (
        $(#[$smeta:meta])*
        pub struct $name:ident {
            $(
                $(#[doc = $fdoc:expr])*
                pub $fname:ident : $fty:ty
            ),* $(,)?
        }
    ) => {
        $(#[$smeta])*
        #[derive(Debug, serde::Deserialize)]
        pub struct $name {
            $( pub $fname: $fty, )*
        }

        impl $name {
            /// Generate the JSON input schema from this struct's fields.
            pub fn input_schema() -> rust_mcp_sdk::schema::ToolInputSchema {
                #[allow(unused_mut)]
                let mut required: Vec<String> = Vec::new();
                #[allow(unused_mut)]
                let mut fields: Vec<(&'static str, &'static str, String)> = Vec::new();
                $(
                    if <$fty as $crate::registry::JsonArgType>::REQUIRED {
                        required.push(stringify!($fname).to_string());
                    }
                    let docs: Vec<&str> = vec![$($fdoc),*];
                    fields.push((
                        stringify!($fname),
                        <$fty as $crate::registry::JsonArgType>::JSON_TYPE,
                        docs.iter().map(|s| s.trim()).collect::<Vec<_>>().join(" "),
                    ));
                )*
                $crate::registry::build_input_schema(required, fields)
            }
        }
    };
}

/// Future type returned by tool handlers.
pub type HandlerFuture<'a> =
    Pin<Box<dyn Future<Output = Result<Value, McpError>> + Send + 'a>>;

/// A registered tool handler: decodes raw args and runs the tool.
pub type ToolHandler = for<'a> fn(&'a Arc<McpContext>, Value) -> HandlerFuture<'a>;

/// One tool: name, description, schema source, and handler — all from a
/// single registration site.
#[derive(Clone, Copy)]
pub struct RegisteredTool {
    /// Published tool name.
    pub name: &'static str,
    /// Published tool description.
    pub description: &'static str,
    /// Generates the input schema (from the arg struct).
    pub input_schema: fn() -> ToolInputSchema,
    /// Decodes args and executes.
    pub handler: ToolHandler,
}

impl RegisteredTool {
    /// Render as an MCP `Tool` for `list_tools`.
    pub fn to_tool(&self) -> Tool {
        Tool {
            name: self.name.to_string(),
            description: Some(self.description.to_string()),
            input_schema: (self.input_schema)(),
            title: None,
            annotations: None,
            icons: vec![],
            execution: None,
            output_schema: None,
            meta: None,
        }
    }
}

/// Register one tool: name, description, arg struct, and handler path.
///
/// The generated glue deserializes the incoming `Value` into the arg struct
/// (mapping failures to `InvalidArguments`) and calls the handler.
#[macro_export]
macro_rules! tool {
    ($name:literal, $desc:literal, $args:ty, $handler:path) => {{
        fn __glue<'a>(
            ctx: &'a std::sync::Arc<$crate::McpContext>,
            value: serde_json::Value,
        ) -> $crate::registry::HandlerFuture<'a> {
            Box::pin(async move {
                let args: $args = serde_json::from_value(value).map_err(|e| {
                    $crate::McpError::InvalidArguments(format!("Invalid arguments: {}", e))
                })?;
                $handler(ctx, args).await
            })
        }
        $crate::registry::RegisteredTool {
            name: $name,
            description: $desc,
            input_schema: <$args>::input_schema,
            handler: __glue,
        }
    }};
}

/// All registered tools across migrated families.
pub fn all_tools() -> &'static [RegisteredTool] {
    static TOOLS: OnceLock<Vec<RegisteredTool>> = OnceLock::new();
    TOOLS.get_or_init(|| {
        let mut tools = Vec::new();
        tools.extend(crate::tools::campaign::registered_tools());
        tools.extend(crate::tools::module::registered_tools());
        tools.extend(crate::tools::document::registered_tools());
        tools.extend(crate::tools::character::registered_tools());
        tools
    })
}

/// Look up a registered tool by name.
pub fn find(name: &str) -> Option<&'static RegisteredTool> {
    all_tools().iter().find(|t| t.name == name)
}

#[cfg(test)]
mod tests {
    tool_args! {
        pub struct DemoArgs {
            /// A required id
            pub id: String,
            /// An optional count
            pub count: Option<i64>,
            /// An optional flag
            pub flag: Option<bool>,
        }
    }

    tool_args! {
        pub struct EmptyArgs {}
    }

    #[test]
    fn schema_marks_non_option_fields_required() {
        let schema = DemoArgs::input_schema();
        let json = serde_json::to_value(&schema).unwrap();
        let required: Vec<&str> = json["required"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap())
            .collect();
        assert_eq!(required, vec!["id"]);
    }

    #[test]
    fn schema_types_and_descriptions_come_from_fields() {
        let schema = DemoArgs::input_schema();
        let json = serde_json::to_value(&schema).unwrap();
        let props = &json["properties"];
        assert_eq!(props["id"]["type"], "string");
        assert_eq!(props["id"]["description"], "A required id");
        assert_eq!(props["count"]["type"], "integer");
        assert_eq!(props["flag"]["type"], "boolean");
    }

    #[test]
    fn empty_args_omit_properties() {
        let schema = EmptyArgs::input_schema();
        let json = serde_json::to_value(&schema).unwrap();
        assert!(json.get("properties").is_none() || json["properties"].is_null());
    }

    #[test]
    fn deserialization_enforces_the_same_contract() {
        // Missing required field fails
        let err = serde_json::from_value::<DemoArgs>(serde_json::json!({})).unwrap_err();
        assert!(err.to_string().contains("id"));

        // Optional fields default to None
        let ok: DemoArgs = serde_json::from_value(serde_json::json!({"id": "x"})).unwrap();
        assert_eq!(ok.id, "x");
        assert!(ok.count.is_none());
        assert!(ok.flag.is_none());

        // Unknown fields are ignored (legacy parser behavior)
        let ok: DemoArgs =
            serde_json::from_value(serde_json::json!({"id": "x", "extra": 1})).unwrap();
        assert_eq!(ok.id, "x");
    }
}
