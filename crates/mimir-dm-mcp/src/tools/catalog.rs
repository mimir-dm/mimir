//! Catalog search tools for MCP
//!
//! Provides tools for searching the D&D 5e catalog (monsters, items, etc.)

use crate::context::McpContext;
use crate::error::McpError;
use mimir_dm_core::models::catalog::item::ItemFilters;
use mimir_dm_core::models::catalog::monster::MonsterFilters;
use mimir_dm_core::models::catalog::trap::TrapFilters;
use mimir_dm_core::services::{ItemService, MonsterService, TrapService};
use rust_mcp_sdk::schema::{Tool, ToolInputSchema};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

/// Create a simple properties map for tool input schema
fn create_properties(
    props: Vec<(&str, &str, &str)>,
) -> Option<HashMap<String, serde_json::Map<String, serde_json::Value>>> {
    let mut map = HashMap::new();
    for (name, prop_type, description) in props {
        let mut inner = serde_json::Map::new();
        inner.insert(
            "type".to_string(),
            serde_json::Value::String(prop_type.to_string()),
        );
        inner.insert(
            "description".to_string(),
            serde_json::Value::String(description.to_string()),
        );
        map.insert(name.to_string(), inner);
    }
    if map.is_empty() {
        None
    } else {
        Some(map)
    }
}

/// Input for search_monsters tool
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SearchMonstersInput {
    /// Search by name (partial match)
    #[serde(default)]
    pub name: Option<String>,

    /// Filter by creature type (e.g., "beast", "dragon", "undead")
    #[serde(default)]
    pub creature_type: Option<String>,

    /// Minimum challenge rating
    #[serde(default)]
    pub min_cr: Option<f64>,

    /// Maximum challenge rating
    #[serde(default)]
    pub max_cr: Option<f64>,

    /// Filter by source book (e.g., "MM", "PHB")
    #[serde(default)]
    pub source: Option<String>,

    /// Maximum number of results (default: 20)
    #[serde(default = "default_limit")]
    pub limit: usize,
}

fn default_limit() -> usize {
    20
}

impl SearchMonstersInput {
    /// Get the tool definition
    pub fn tool() -> Tool {
        Tool {
            name: "search_monsters".to_string(),
            description: Some(
                "Search the monster catalog. Use this to find monsters by name, type, or CR before adding them to a module."
                    .to_string(),
            ),
            input_schema: ToolInputSchema::new(
                vec![],
                create_properties(vec![
                    ("name", "string", "Search by name (partial match)"),
                    (
                        "creature_type",
                        "string",
                        "Filter by creature type (e.g., beast, dragon, undead)",
                    ),
                    ("min_cr", "number", "Minimum challenge rating"),
                    ("max_cr", "number", "Maximum challenge rating"),
                    ("source", "string", "Filter by source book (e.g., MM, PHB)"),
                    ("limit", "integer", "Maximum results (default: 20)"),
                ]),
                None,
            ),
            title: None,
            annotations: None,
            icons: vec![],
            execution: None,
            output_schema: None,
            meta: None,
        }
    }

    /// Execute the search_monsters tool
    pub async fn execute(
        &self,
        context: Arc<McpContext>,
    ) -> Result<Vec<MonsterSearchResult>, McpError> {
        let mut conn = context.get_connection()?;
        let mut service = MonsterService::new(&mut conn);

        let filters = MonsterFilters {
            name: self.name.clone(),
            creature_types: self.creature_type.clone().map(|t| vec![t]),
            min_cr: self.min_cr,
            max_cr: self.max_cr,
            sources: self.source.clone().map(|s| vec![s]),
            sizes: None,
            alignments: None,
            min_hp: None,
            max_hp: None,
            environment: None,
        };

        let monsters = service
            .search_monsters(filters)
            .map_err(|e| McpError::Service(e.to_string()))?;

        let results: Vec<MonsterSearchResult> = monsters
            .into_iter()
            .take(self.limit)
            .map(|m| MonsterSearchResult {
                name: m.name,
                source: m.source,
                creature_type: m.creature_type,
                size: m.size,
                cr: m.cr,
                hp: m.hp,
            })
            .collect();

        Ok(results)
    }
}

/// Monster search result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonsterSearchResult {
    pub name: String,
    pub source: String,
    pub creature_type: String,
    pub size: String,
    pub cr: String,
    pub hp: u32,
}

/// Input for search_items tool
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SearchItemsInput {
    /// Search by name (partial match)
    #[serde(default)]
    pub name: Option<String>,

    /// Filter by item type (e.g., "weapon", "armor", "potion", "wondrous item")
    #[serde(default)]
    pub item_type: Option<String>,

    /// Filter by rarity (e.g., "common", "uncommon", "rare", "very rare", "legendary")
    #[serde(default)]
    pub rarity: Option<String>,

    /// Filter by source book (e.g., "PHB", "DMG")
    #[serde(default)]
    pub source: Option<String>,

    /// Maximum number of results (default: 20)
    #[serde(default = "default_limit")]
    pub limit: usize,
}

impl SearchItemsInput {
    /// Get the tool definition
    pub fn tool() -> Tool {
        Tool {
            name: "search_items".to_string(),
            description: Some(
                "Search the item catalog. Use this to find items by name, type, or rarity before adding them to a module or character."
                    .to_string(),
            ),
            input_schema: ToolInputSchema::new(
                vec![],
                create_properties(vec![
                    ("name", "string", "Search by name (partial match)"),
                    (
                        "item_type",
                        "string",
                        "Filter by type (e.g., weapon, armor, potion, wondrous item)",
                    ),
                    (
                        "rarity",
                        "string",
                        "Filter by rarity (common, uncommon, rare, very rare, legendary)",
                    ),
                    ("source", "string", "Filter by source book (e.g., PHB, DMG)"),
                    ("limit", "integer", "Maximum results (default: 20)"),
                ]),
                None,
            ),
            title: None,
            annotations: None,
            icons: vec![],
            execution: None,
            output_schema: None,
            meta: None,
        }
    }

    /// Execute the search_items tool
    pub async fn execute(
        &self,
        context: Arc<McpContext>,
    ) -> Result<Vec<ItemSearchResult>, McpError> {
        let mut conn = context.get_connection()?;
        let mut service = ItemService::new(&mut conn);

        let filters = ItemFilters {
            name: self.name.clone(),
            item_types: self.item_type.clone().map(|t| vec![t]),
            rarities: self.rarity.clone().map(|r| vec![r]),
            sources: self.source.clone().map(|s| vec![s]),
            min_value: None,
            max_value: None,
        };

        let items = service
            .search_items(filters)
            .map_err(|e| McpError::Service(e.to_string()))?;

        let results: Vec<ItemSearchResult> = items
            .into_iter()
            .take(self.limit)
            .map(|i| ItemSearchResult {
                name: i.name,
                source: i.source,
                item_type: i.item_type,
                rarity: Some(i.rarity),
                value: i.value.map(|v| format!("{} gp", v)),
            })
            .collect();

        Ok(results)
    }
}

/// Item search result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemSearchResult {
    pub name: String,
    pub source: String,
    pub item_type: String,
    pub rarity: Option<String>,
    pub value: Option<String>,
}

/// Input for search_traps tool
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SearchTrapsInput {
    /// Search by name (partial match)
    #[serde(default)]
    pub name: Option<String>,

    /// Filter by category ("Trap" or "Hazard")
    #[serde(default)]
    pub category: Option<String>,

    /// Filter by source book (e.g., "DMG", "XGE")
    #[serde(default)]
    pub source: Option<String>,

    /// Maximum number of results (default: 20)
    #[serde(default = "default_limit")]
    pub limit: usize,
}

impl SearchTrapsInput {
    /// Get the tool definition
    pub fn tool() -> Tool {
        Tool {
            name: "search_traps".to_string(),
            description: Some(
                "Search the trap and hazard catalog. Use this to find traps by name or category (Trap/Hazard)."
                    .to_string(),
            ),
            input_schema: ToolInputSchema::new(
                vec![],
                create_properties(vec![
                    ("name", "string", "Search by name (partial match)"),
                    (
                        "category",
                        "string",
                        "Filter by category: Trap or Hazard",
                    ),
                    ("source", "string", "Filter by source book (e.g., DMG, XGE)"),
                    ("limit", "integer", "Maximum results (default: 20)"),
                ]),
                None,
            ),
            title: None,
            annotations: None,
            icons: vec![],
            execution: None,
            output_schema: None,
            meta: None,
        }
    }

    /// Execute the search_traps tool
    pub async fn execute(
        &self,
        context: Arc<McpContext>,
    ) -> Result<Vec<TrapSearchResult>, McpError> {
        let mut conn = context.get_connection()?;
        let service = TrapService;

        let filters = TrapFilters {
            search: self.name.clone(),
            categories: self.category.clone().map(|c| vec![c]),
            trap_types: None,
            sources: self.source.clone().map(|s| vec![s]),
        };

        let traps = service
            .search_traps(&mut conn, filters)
            .map_err(|e| McpError::Service(e.to_string()))?;

        let results: Vec<TrapSearchResult> = traps
            .into_iter()
            .take(self.limit)
            .map(|t| TrapSearchResult {
                name: t.name,
                source: t.source,
                category: t.category,
                trap_type: t.trap_type,
            })
            .collect();

        Ok(results)
    }
}

/// Trap search result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrapSearchResult {
    pub name: String,
    pub source: String,
    pub category: String,
    pub trap_type: String,
}
