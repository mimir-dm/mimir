//! Catalog Tools
//!
//! MCP tools for searching the D&D 5e catalog.

use mimir_core::dal::campaign as campaign_dal;
use mimir_core::dal::catalog as catalog_dal;
use mimir_core::models::catalog::{
    BackgroundFilter, ClassFilter, ConditionFilter, FeatFilter, ItemFilter, MonsterFilter,
    RaceFilter, SpellFilter,
};
use rust_mcp_sdk::schema::{Tool, ToolInputSchema};
use serde_json::{json, Value};
use std::sync::Arc;

use super::create_properties;
use crate::context::McpContext;
use crate::McpError;

// =============================================================================
// Tool Definitions
// =============================================================================

pub fn search_monsters_tool() -> Tool {
    Tool {
        name: "search_monsters".to_string(),
        description: Some("Search the monster catalog".to_string()),
        input_schema: ToolInputSchema::new(
            vec![],
            create_properties(vec![
                ("name", "string", "Search by name (partial match)"),
                ("cr_min", "number", "Minimum challenge rating"),
                ("cr_max", "number", "Maximum challenge rating"),
                ("monster_type", "string", "Filter by type (e.g., undead, dragon)"),
                ("limit", "integer", "Maximum results to return (default: 20)"),
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

pub fn search_items_tool() -> Tool {
    Tool {
        name: "search_items".to_string(),
        description: Some("Search the item catalog".to_string()),
        input_schema: ToolInputSchema::new(
            vec![],
            create_properties(vec![
                ("name", "string", "Search by name (partial match)"),
                (
                    "rarity",
                    "string",
                    "Filter by rarity: common, uncommon, rare, very rare, legendary, artifact",
                ),
                (
                    "item_type",
                    "string",
                    "Filter by type (e.g., weapon, armor, wondrous item)",
                ),
                ("limit", "integer", "Maximum results to return (default: 20)"),
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

pub fn search_spells_tool() -> Tool {
    Tool {
        name: "search_spells".to_string(),
        description: Some("Search the spell catalog".to_string()),
        input_schema: ToolInputSchema::new(
            vec![],
            create_properties(vec![
                ("name", "string", "Search by name (partial match)"),
                ("level", "integer", "Filter by spell level (0 for cantrips)"),
                (
                    "school",
                    "string",
                    "Filter by school (e.g., evocation, necromancy)",
                ),
                ("class_name", "string", "Filter by class spell list"),
                ("limit", "integer", "Maximum results to return (default: 20)"),
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

pub fn search_races_tool() -> Tool {
    Tool {
        name: "search_races".to_string(),
        description: Some("Search the race catalog".to_string()),
        input_schema: ToolInputSchema::new(
            vec![],
            create_properties(vec![
                ("name", "string", "Search by name (partial match)"),
                ("limit", "integer", "Maximum results to return (default: 20)"),
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

pub fn search_classes_tool() -> Tool {
    Tool {
        name: "search_classes".to_string(),
        description: Some("Search the class catalog".to_string()),
        input_schema: ToolInputSchema::new(
            vec![],
            create_properties(vec![
                ("name", "string", "Search by name (partial match)"),
                ("limit", "integer", "Maximum results to return (default: 20)"),
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

pub fn search_backgrounds_tool() -> Tool {
    Tool {
        name: "search_backgrounds".to_string(),
        description: Some("Search the background catalog".to_string()),
        input_schema: ToolInputSchema::new(
            vec![],
            create_properties(vec![
                ("name", "string", "Search by name (partial match)"),
                ("limit", "integer", "Maximum results to return (default: 20)"),
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

pub fn search_feats_tool() -> Tool {
    Tool {
        name: "search_feats".to_string(),
        description: Some("Search the feat catalog".to_string()),
        input_schema: ToolInputSchema::new(
            vec![],
            create_properties(vec![
                ("name", "string", "Search by name (partial match)"),
                ("limit", "integer", "Maximum results to return (default: 20)"),
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

pub fn search_conditions_tool() -> Tool {
    Tool {
        name: "search_conditions".to_string(),
        description: Some("Search the condition catalog".to_string()),
        input_schema: ToolInputSchema::new(
            vec![],
            create_properties(vec![
                ("name", "string", "Search by name (partial match)"),
                ("limit", "integer", "Maximum results to return (default: 20)"),
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

// =============================================================================
// Tool Implementations
// =============================================================================

pub async fn search_monsters(ctx: &Arc<McpContext>, args: Value) -> Result<Value, McpError> {
    let mut db = ctx.db()?;

    // Build filter from args
    let mut filter = MonsterFilter::new();

    if let Some(name) = args.get("name").and_then(|v| v.as_str()) {
        filter = filter.with_name_contains(name);
    }

    if let Some(monster_type) = args.get("monster_type").and_then(|v| v.as_str()) {
        filter = filter.with_creature_type(monster_type);
    }

    // Apply campaign source filtering if active campaign exists
    if let Some(campaign_id) = ctx.get_active_campaign_id() {
        let sources = campaign_dal::list_campaign_source_codes(&mut db, &campaign_id)
            .map_err(|e| McpError::Internal(e.to_string()))?;
        if !sources.is_empty() {
            filter = filter.with_sources(sources);
        }
    }

    let limit = args
        .get("limit")
        .and_then(|v| v.as_i64())
        .unwrap_or(20) as i64;

    let monsters = catalog_dal::search_monsters_paginated(&mut db, &filter, limit, 0)
        .map_err(|e| McpError::Internal(e.to_string()))?;

    let monster_data: Vec<Value> = monsters
        .iter()
        .map(|m| {
            json!({
                "name": m.name,
                "source": m.source,
                "cr": m.cr,
                "creature_type": m.creature_type,
                "size": m.size_name()
            })
        })
        .collect();

    Ok(json!({
        "monsters": monster_data,
        "count": monster_data.len()
    }))
}

pub async fn search_items(ctx: &Arc<McpContext>, args: Value) -> Result<Value, McpError> {
    let mut db = ctx.db()?;

    // Build filter from args
    let mut filter = ItemFilter::new();

    if let Some(name) = args.get("name").and_then(|v| v.as_str()) {
        filter = filter.with_name_contains(name);
    }

    if let Some(rarity) = args.get("rarity").and_then(|v| v.as_str()) {
        filter = filter.with_rarity(rarity);
    }

    if let Some(item_type) = args.get("item_type").and_then(|v| v.as_str()) {
        filter = filter.with_type(item_type);
    }

    // Apply campaign source filtering if active campaign exists
    if let Some(campaign_id) = ctx.get_active_campaign_id() {
        let sources = campaign_dal::list_campaign_source_codes(&mut db, &campaign_id)
            .map_err(|e| McpError::Internal(e.to_string()))?;
        if !sources.is_empty() {
            filter = filter.with_sources(sources);
        }
    }

    let limit = args
        .get("limit")
        .and_then(|v| v.as_i64())
        .unwrap_or(20) as i64;

    let items = catalog_dal::search_items_paginated(&mut db, &filter, limit, 0)
        .map_err(|e| McpError::Internal(e.to_string()))?;

    let item_data: Vec<Value> = items
        .iter()
        .map(|i| {
            json!({
                "name": i.name,
                "source": i.source,
                "rarity": i.rarity,
                "item_type": i.item_type
            })
        })
        .collect();

    Ok(json!({
        "items": item_data,
        "count": item_data.len()
    }))
}

pub async fn search_spells(ctx: &Arc<McpContext>, args: Value) -> Result<Value, McpError> {
    let mut db = ctx.db()?;

    // Build filter from args
    let mut filter = SpellFilter::new();

    if let Some(name) = args.get("name").and_then(|v| v.as_str()) {
        filter = filter.with_name_contains(name);
    }

    if let Some(level) = args.get("level").and_then(|v| v.as_i64()) {
        filter = filter.with_level(level as i32);
    }

    if let Some(school) = args.get("school").and_then(|v| v.as_str()) {
        filter = filter.with_school(school);
    }

    // Apply campaign source filtering if active campaign exists
    if let Some(campaign_id) = ctx.get_active_campaign_id() {
        let sources = campaign_dal::list_campaign_source_codes(&mut db, &campaign_id)
            .map_err(|e| McpError::Internal(e.to_string()))?;
        if !sources.is_empty() {
            filter = filter.with_sources(sources);
        }
    }

    let limit = args
        .get("limit")
        .and_then(|v| v.as_i64())
        .unwrap_or(20) as i64;

    let spells = catalog_dal::search_spells_paginated(&mut db, &filter, limit, 0)
        .map_err(|e| McpError::Internal(e.to_string()))?;

    let spell_data: Vec<Value> = spells
        .iter()
        .map(|s| {
            json!({
                "name": s.name,
                "source": s.source,
                "level": s.level,
                "school": s.school
            })
        })
        .collect();

    Ok(json!({
        "spells": spell_data,
        "count": spell_data.len()
    }))
}

pub async fn search_races(ctx: &Arc<McpContext>, args: Value) -> Result<Value, McpError> {
    let mut db = ctx.db()?;
    let mut filter = RaceFilter::new();

    if let Some(name) = args.get("name").and_then(|v| v.as_str()) {
        filter = filter.with_name_contains(name);
    }
    if let Some(campaign_id) = ctx.get_active_campaign_id() {
        let sources = campaign_dal::list_campaign_source_codes(&mut db, &campaign_id)
            .map_err(|e| McpError::Internal(e.to_string()))?;
        if !sources.is_empty() {
            filter = filter.with_sources(sources);
        }
    }

    let limit = args.get("limit").and_then(|v| v.as_i64()).unwrap_or(20) as i64;
    let races = catalog_dal::search_races_paginated(&mut db, &filter, limit, 0)
        .map_err(|e| McpError::Internal(e.to_string()))?;

    let data: Vec<Value> = races
        .iter()
        .map(|r| json!({"name": r.name, "source": r.source}))
        .collect();

    Ok(json!({"races": data, "count": data.len()}))
}

pub async fn search_classes(ctx: &Arc<McpContext>, args: Value) -> Result<Value, McpError> {
    let mut db = ctx.db()?;
    let mut filter = ClassFilter::new();

    if let Some(name) = args.get("name").and_then(|v| v.as_str()) {
        filter = filter.with_name_contains(name);
    }
    if let Some(campaign_id) = ctx.get_active_campaign_id() {
        let sources = campaign_dal::list_campaign_source_codes(&mut db, &campaign_id)
            .map_err(|e| McpError::Internal(e.to_string()))?;
        if !sources.is_empty() {
            filter = filter.with_sources(sources);
        }
    }

    let limit = args.get("limit").and_then(|v| v.as_i64()).unwrap_or(20) as i64;
    let classes = catalog_dal::search_classes_paginated(&mut db, &filter, limit, 0)
        .map_err(|e| McpError::Internal(e.to_string()))?;

    let data: Vec<Value> = classes
        .iter()
        .map(|c| json!({"name": c.name, "source": c.source}))
        .collect();

    Ok(json!({"classes": data, "count": data.len()}))
}

pub async fn search_backgrounds(ctx: &Arc<McpContext>, args: Value) -> Result<Value, McpError> {
    let mut db = ctx.db()?;
    let mut filter = BackgroundFilter::new();

    if let Some(name) = args.get("name").and_then(|v| v.as_str()) {
        filter = filter.with_name_contains(name);
    }
    if let Some(campaign_id) = ctx.get_active_campaign_id() {
        let sources = campaign_dal::list_campaign_source_codes(&mut db, &campaign_id)
            .map_err(|e| McpError::Internal(e.to_string()))?;
        if !sources.is_empty() {
            filter = filter.with_sources(sources);
        }
    }

    let limit = args.get("limit").and_then(|v| v.as_i64()).unwrap_or(20) as i64;
    let backgrounds = catalog_dal::search_backgrounds_paginated(&mut db, &filter, limit, 0)
        .map_err(|e| McpError::Internal(e.to_string()))?;

    let data: Vec<Value> = backgrounds
        .iter()
        .map(|b| json!({"name": b.name, "source": b.source}))
        .collect();

    Ok(json!({"backgrounds": data, "count": data.len()}))
}

pub async fn search_feats(ctx: &Arc<McpContext>, args: Value) -> Result<Value, McpError> {
    let mut db = ctx.db()?;
    let mut filter = FeatFilter::new();

    if let Some(name) = args.get("name").and_then(|v| v.as_str()) {
        filter = filter.with_name_contains(name);
    }
    if let Some(campaign_id) = ctx.get_active_campaign_id() {
        let sources = campaign_dal::list_campaign_source_codes(&mut db, &campaign_id)
            .map_err(|e| McpError::Internal(e.to_string()))?;
        if !sources.is_empty() {
            filter = filter.with_sources(sources);
        }
    }

    let limit = args.get("limit").and_then(|v| v.as_i64()).unwrap_or(20) as i64;
    let feats = catalog_dal::search_feats_paginated(&mut db, &filter, limit, 0)
        .map_err(|e| McpError::Internal(e.to_string()))?;

    let data: Vec<Value> = feats
        .iter()
        .map(|f| json!({"name": f.name, "source": f.source}))
        .collect();

    Ok(json!({"feats": data, "count": data.len()}))
}

pub async fn search_conditions(ctx: &Arc<McpContext>, args: Value) -> Result<Value, McpError> {
    let mut db = ctx.db()?;
    let mut filter = ConditionFilter::new();

    if let Some(name) = args.get("name").and_then(|v| v.as_str()) {
        filter = filter.with_name_contains(name);
    }
    if let Some(campaign_id) = ctx.get_active_campaign_id() {
        let sources = campaign_dal::list_campaign_source_codes(&mut db, &campaign_id)
            .map_err(|e| McpError::Internal(e.to_string()))?;
        if !sources.is_empty() {
            filter = filter.with_sources(sources);
        }
    }

    let limit = args.get("limit").and_then(|v| v.as_i64()).unwrap_or(20) as i64;
    let conditions = catalog_dal::search_conditions_paginated(&mut db, &filter, limit, 0)
        .map_err(|e| McpError::Internal(e.to_string()))?;

    let data: Vec<Value> = conditions
        .iter()
        .map(|c| json!({"name": c.name, "source": c.source}))
        .collect();

    Ok(json!({"conditions": data, "count": data.len()}))
}
