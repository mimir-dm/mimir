//! Catalog query tools for LLM interactions
//!
//! These tools allow LLMs to search the D&D 5e catalog (monsters, items, spells)

use async_trait::async_trait;
use mimir_dm_core::models::catalog::item::ItemFilters;
use mimir_dm_core::models::catalog::monster::MonsterFilters;
use mimir_dm_core::models::catalog::SpellFilters;
use mimir_dm_core::services::{ItemService, MonsterService, SpellService};
use mimir_dm_core::DatabaseService;
use mimir_dm_llm::ToolTrait;
use serde_json::{json, Value};
use std::error::Error;
use std::sync::Arc;
use tracing::debug;

/// Tool for searching the monster catalog
pub struct SearchMonstersTool {
    db_service: Arc<DatabaseService>,
}

impl SearchMonstersTool {
    pub fn new(db_service: Arc<DatabaseService>) -> Self {
        Self { db_service }
    }
}

#[async_trait]
impl ToolTrait for SearchMonstersTool {
    fn name(&self) -> &str {
        "search_monsters"
    }

    fn description(&self) -> &str {
        "Search the monster catalog by name, CR, size, type, or alignment.

Usage:
- Provide any combination of filters
- Results limited to 50 monsters
- Returns summary info (name, CR, size, type, HP)

When to use:
- Looking up monsters for encounters
- Finding creatures by challenge rating
- Searching for specific monster types
- Building encounter lists

Output:
- List of matching monsters with basic stats
- Use get_monster for full stat blocks"
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "name": {
                    "type": ["string", "null"],
                    "description": "Search by name (partial match)"
                },
                "min_cr": {
                    "type": ["number", "null"],
                    "description": "Minimum challenge rating (0-30)"
                },
                "max_cr": {
                    "type": ["number", "null"],
                    "description": "Maximum challenge rating (0-30)"
                },
                "sizes": {
                    "type": ["array", "null"],
                    "items": { "type": "string" },
                    "description": "Filter by sizes (T, S, M, L, H, G)"
                },
                "creature_types": {
                    "type": ["array", "null"],
                    "items": { "type": "string" },
                    "description": "Filter by types (aberration, beast, celestial, construct, dragon, elemental, fey, fiend, giant, humanoid, monstrosity, ooze, plant, undead)"
                },
                "alignments": {
                    "type": ["array", "null"],
                    "items": { "type": "string" },
                    "description": "Filter by alignments"
                }
            }
        })
    }

    fn requires_confirmation(&self) -> bool {
        false
    }

    async fn execute(&self, arguments: Value) -> Result<String, Box<dyn Error + Send + Sync>> {
        let filters = MonsterFilters {
            name: arguments.get("name").and_then(|v| v.as_str()).map(String::from),
            min_cr: arguments.get("min_cr").and_then(|v| v.as_f64()),
            max_cr: arguments.get("max_cr").and_then(|v| v.as_f64()),
            sizes: arguments.get("sizes").and_then(|v| {
                v.as_array().map(|arr| {
                    arr.iter()
                        .filter_map(|s| s.as_str().map(String::from))
                        .collect()
                })
            }),
            creature_types: arguments.get("creature_types").and_then(|v| {
                v.as_array().map(|arr| {
                    arr.iter()
                        .filter_map(|s| s.as_str().map(String::from))
                        .collect()
                })
            }),
            alignments: arguments.get("alignments").and_then(|v| {
                v.as_array().map(|arr| {
                    arr.iter()
                        .filter_map(|s| s.as_str().map(String::from))
                        .collect()
                })
            }),
            sources: None,
            min_hp: None,
            max_hp: None,
            environment: None,
        };

        let mut conn = self
            .db_service
            .get_connection()
            .map_err(|e| format!("Database error: {}", e))?;

        let mut monster_service = MonsterService::new(&mut conn);
        let monsters = monster_service
            .search_monsters(filters)
            .map_err(|e| format!("Search failed: {}", e))?;

        // Limit results
        let limited: Vec<_> = monsters.into_iter().take(50).collect();

        let result = json!({
            "count": limited.len(),
            "monsters": limited.iter().map(|m| json!({
                "name": m.name,
                "cr": m.cr,
                "size": m.size,
                "creature_type": m.creature_type,
                "hp": m.hp,
                "ac": m.ac,
                "source": m.source
            })).collect::<Vec<_>>()
        });

        debug!("Found {} monsters", limited.len());
        Ok(serde_json::to_string_pretty(&result)?)
    }
}

/// Tool for searching the item catalog
pub struct SearchItemsTool {
    db_service: Arc<DatabaseService>,
}

impl SearchItemsTool {
    pub fn new(db_service: Arc<DatabaseService>) -> Self {
        Self { db_service }
    }
}

#[async_trait]
impl ToolTrait for SearchItemsTool {
    fn name(&self) -> &str {
        "search_items"
    }

    fn description(&self) -> &str {
        "Search the item catalog by name, type, or rarity.

Usage:
- Provide any combination of filters
- Results limited to 50 items
- Returns summary info (name, type, rarity, value)

When to use:
- Looking up equipment for characters
- Finding magic items by rarity
- Searching for specific item types
- Building treasure hoards

Output:
- List of matching items with basic info
- Use get_item for full details"
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "name": {
                    "type": ["string", "null"],
                    "description": "Search by name (partial match)"
                },
                "item_types": {
                    "type": ["array", "null"],
                    "items": { "type": "string" },
                    "description": "Filter by types (weapon, armor, potion, wondrous item, etc.)"
                },
                "rarities": {
                    "type": ["array", "null"],
                    "items": { "type": "string" },
                    "description": "Filter by rarities (common, uncommon, rare, very rare, legendary)"
                },
                "min_value": {
                    "type": ["number", "null"],
                    "description": "Minimum value in gold"
                },
                "max_value": {
                    "type": ["number", "null"],
                    "description": "Maximum value in gold"
                }
            }
        })
    }

    fn requires_confirmation(&self) -> bool {
        false
    }

    async fn execute(&self, arguments: Value) -> Result<String, Box<dyn Error + Send + Sync>> {
        let filters = ItemFilters {
            name: arguments.get("name").and_then(|v| v.as_str()).map(String::from),
            item_types: arguments.get("item_types").and_then(|v| {
                v.as_array().map(|arr| {
                    arr.iter()
                        .filter_map(|s| s.as_str().map(String::from))
                        .collect()
                })
            }),
            rarities: arguments.get("rarities").and_then(|v| {
                v.as_array().map(|arr| {
                    arr.iter()
                        .filter_map(|s| s.as_str().map(String::from))
                        .collect()
                })
            }),
            min_value: arguments.get("min_value").and_then(|v| v.as_f64()),
            max_value: arguments.get("max_value").and_then(|v| v.as_f64()),
            sources: None,
        };

        let mut conn = self
            .db_service
            .get_connection()
            .map_err(|e| format!("Database error: {}", e))?;

        let mut item_service = ItemService::new(&mut conn);
        let items = item_service
            .search_items(filters)
            .map_err(|e| format!("Search failed: {}", e))?;

        // Limit results
        let limited: Vec<_> = items.into_iter().take(50).collect();

        let result = json!({
            "count": limited.len(),
            "items": limited.iter().map(|i| json!({
                "name": i.name,
                "item_type": i.item_type,
                "type_name": i.type_name,
                "rarity": i.rarity,
                "value": i.value,
                "weight": i.weight,
                "source": i.source
            })).collect::<Vec<_>>()
        });

        debug!("Found {} items", limited.len());
        Ok(serde_json::to_string_pretty(&result)?)
    }
}

/// Tool for searching the spell catalog
pub struct SearchSpellsTool {
    db_service: Arc<DatabaseService>,
}

impl SearchSpellsTool {
    pub fn new(db_service: Arc<DatabaseService>) -> Self {
        Self { db_service }
    }
}

#[async_trait]
impl ToolTrait for SearchSpellsTool {
    fn name(&self) -> &str {
        "search_spells"
    }

    fn description(&self) -> &str {
        "Search the spell catalog by name, level, or school.

Usage:
- Provide any combination of filters
- Results limited to 50 spells
- Returns summary info (name, level, school, casting time)

When to use:
- Looking up spells for characters
- Finding spells by level or school
- Building spell lists
- Checking spell availability

Output:
- List of matching spells with basic info
- Use get_spell for full details"
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "query": {
                    "type": ["string", "null"],
                    "description": "Search by name (partial match)"
                },
                "levels": {
                    "type": ["array", "null"],
                    "items": { "type": "integer" },
                    "description": "Filter by spell levels (0-9, where 0 = cantrip)"
                },
                "schools": {
                    "type": ["array", "null"],
                    "items": { "type": "string" },
                    "description": "Filter by schools (Abjuration, Conjuration, Divination, Enchantment, Evocation, Illusion, Necromancy, Transmutation)"
                }
            }
        })
    }

    fn requires_confirmation(&self) -> bool {
        false
    }

    async fn execute(&self, arguments: Value) -> Result<String, Box<dyn Error + Send + Sync>> {
        let filters = SpellFilters {
            query: arguments.get("query").and_then(|v| v.as_str()).map(String::from),
            levels: arguments.get("levels").and_then(|v| {
                v.as_array().map(|arr| {
                    arr.iter()
                        .filter_map(|n| n.as_i64().map(|i| i as i32))
                        .collect()
                })
            }).unwrap_or_default(),
            schools: arguments.get("schools").and_then(|v| {
                v.as_array().map(|arr| {
                    arr.iter()
                        .filter_map(|s| s.as_str().map(String::from))
                        .collect()
                })
            }).unwrap_or_default(),
            classes: arguments.get("classes").and_then(|v| {
                v.as_array().map(|arr| {
                    arr.iter()
                        .filter_map(|s| s.as_str().map(String::from))
                        .collect()
                })
            }).unwrap_or_default(),
            sources: Vec::new(),
            tags: Vec::new(),
            limit: Some(50),
            offset: None,
        };

        let mut conn = self
            .db_service
            .get_connection()
            .map_err(|e| format!("Database error: {}", e))?;

        let spells = SpellService::search_spells(&mut conn, filters)
            .map_err(|e| format!("Search failed: {}", e))?;

        let result = json!({
            "count": spells.len(),
            "spells": spells.iter().map(|s| json!({
                "name": s.name,
                "level": s.level,
                "school": s.school,
                "casting_time": s.casting_time,
                "concentration": s.concentration,
                "ritual": s.ritual,
                "source": s.source
            })).collect::<Vec<_>>()
        });

        debug!("Found {} spells", spells.len());
        Ok(serde_json::to_string_pretty(&result)?)
    }
}
