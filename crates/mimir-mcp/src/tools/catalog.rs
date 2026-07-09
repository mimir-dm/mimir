//! Catalog Tools
//!
//! Single `search_catalog` tool for searching the D&D 5e catalog across all
//! categories. Registry-based: one typed argument struct, one handler, one
//! registration.

use mimir_core::dal::campaign as campaign_dal;
use mimir_core::dal::catalog as catalog_dal;
use mimir_core::models::catalog::{
    BackgroundFilter, ClassFilter, ConditionFilter, FeatFilter, ItemFilter, MonsterFilter,
    RaceFilter, SpellFilter,
};
use serde_json::{json, Value};
use std::sync::Arc;

use crate::context::McpContext;
use crate::registry::RegisteredTool;
use crate::response::McpResponse;
use crate::{tool, tool_args, McpError};

// =============================================================================
// Registration
// =============================================================================

/// All catalog-family tools.
pub fn registered_tools() -> Vec<RegisteredTool> {
    vec![tool!(
        "search_catalog",
        "Search the D&D 5e catalog by category. Supports monsters, items, spells, races, classes, backgrounds, feats, and conditions. Category-specific filters are available for monsters (cr_min, cr_max, monster_type), items (rarity, item_type), and spells (level, school, class_name). Monster searches also include homebrew monsters from the active campaign by default.",
        SearchCatalogArgs,
        search_catalog
    )]
}

// =============================================================================
// Arguments
// =============================================================================

const VALID_CATEGORIES: &[&str] = &[
    "monster",
    "item",
    "spell",
    "race",
    "class",
    "background",
    "feat",
    "condition",
];

tool_args! {
    pub struct SearchCatalogArgs {
        /// Category to search: monster, item, spell, race, class, background, feat, condition
        pub category: String,
        /// Search by name (partial match)
        pub name: Option<String>,
        /// Maximum results to return (default: 20)
        pub limit: Option<i64>,
        /// Minimum challenge rating (monsters only)
        pub cr_min: Option<f64>,
        /// Maximum challenge rating (monsters only)
        pub cr_max: Option<f64>,
        /// Filter by creature type (monsters only, e.g. undead, dragon)
        pub monster_type: Option<String>,
        /// Include homebrew monsters from active campaign (monsters only, default: true)
        pub include_homebrew: Option<bool>,
        /// Filter by rarity (items only): common, uncommon, rare, very rare, legendary, artifact
        pub rarity: Option<String>,
        /// Filter by item type (items only, e.g. weapon, armor, wondrous item)
        pub item_type: Option<String>,
        /// Filter by spell level (spells only, 0 for cantrips)
        pub level: Option<i64>,
        /// Filter by school of magic (spells only, e.g. evocation, necromancy)
        pub school: Option<String>,
        /// Filter by class spell list (spells only)
        pub class_name: Option<String>,
    }
}

// =============================================================================
// Handler
// =============================================================================

pub async fn search_catalog(
    ctx: &Arc<McpContext>,
    args: SearchCatalogArgs,
) -> Result<Value, McpError> {
    if !VALID_CATEGORIES.contains(&args.category.as_str()) {
        return Err(McpError::InvalidArguments(format!(
            "Invalid category '{}'. Must be one of: {}",
            args.category,
            VALID_CATEGORIES.join(", ")
        )));
    }

    let limit = args.limit.unwrap_or(20);

    let mut db = ctx.connect()?;

    // Get campaign sources for filtering (shared across all categories)
    let campaign_sources = if let Some(campaign_id) = ctx.get_active_campaign_id() {
        let sources = campaign_dal::list_campaign_source_codes(&mut db, &campaign_id)
            .map_err(|e| McpError::Internal(e.to_string()))?;
        if sources.is_empty() {
            None
        } else {
            Some(sources)
        }
    } else {
        None
    };

    match args.category.as_str() {
        "monster" => search_monsters(ctx, &args, &mut db, limit, campaign_sources).await,
        "item" => search_items(&args, &mut db, limit, campaign_sources),
        "spell" => search_spells(&args, &mut db, limit, campaign_sources),
        "race" => search_races(&args, &mut db, limit, campaign_sources),
        "class" => search_classes(&args, &mut db, limit, campaign_sources),
        "background" => search_backgrounds(&args, &mut db, limit, campaign_sources),
        "feat" => search_feats(&args, &mut db, limit, campaign_sources),
        "condition" => search_conditions(&args, &mut db, limit, campaign_sources),
        _ => unreachable!(),
    }
}

// =============================================================================
// Category-specific implementations
// =============================================================================

async fn search_monsters(
    ctx: &Arc<McpContext>,
    args: &SearchCatalogArgs,
    db: &mut diesel::SqliteConnection,
    limit: i64,
    campaign_sources: Option<Vec<String>>,
) -> Result<Value, McpError> {
    let mut filter = MonsterFilter::new();

    if let Some(name) = args.name.as_deref() {
        filter = filter.with_name_contains(name);
    }
    if let Some(monster_type) = args.monster_type.as_deref() {
        filter = filter.with_creature_type(monster_type);
    }
    if let Some(sources) = campaign_sources {
        filter = filter.with_sources(sources);
    }

    let include_homebrew = args.include_homebrew.unwrap_or(true);

    let monsters = catalog_dal::search_monsters_paginated(db, &filter, limit, 0)
        .map_err(|e| McpError::Internal(e.to_string()))?;

    let mut monster_data: Vec<Value> = monsters
        .iter()
        .map(|m| {
            json!({
                "name": m.name,
                "source": m.source,
                "cr": m.cr,
                "creature_type": m.creature_type,
                "size": m.size_name(),
                "is_homebrew": false
            })
        })
        .collect();

    if include_homebrew {
        if let Some(campaign_id) = ctx.get_active_campaign_id() {
            use mimir_core::services::HomebrewService;
            if let Ok(hb_monsters) = HomebrewService::new(db).list_monsters(&campaign_id) {
                let name_lower = args.name.as_deref().map(|n| n.to_lowercase());
                let type_lower = args.monster_type.as_deref().map(|t| t.to_lowercase());

                for hb in &hb_monsters {
                    if let Some(ref query) = name_lower {
                        if !hb.name.to_lowercase().contains(query) {
                            continue;
                        }
                    }
                    if let Some(ref query) = type_lower {
                        if let Some(ref ct) = hb.creature_type {
                            if !ct.to_lowercase().contains(query) {
                                continue;
                            }
                        } else {
                            continue;
                        }
                    }

                    monster_data.push(json!({
                        "name": hb.name,
                        "source": "Homebrew",
                        "homebrew_id": hb.id,
                        "cr": hb.cr,
                        "creature_type": hb.creature_type,
                        "size": hb.size,
                        "is_homebrew": true
                    }));
                }
            }
        }
    }

    McpResponse::list("monsters", monster_data)
}

fn search_items(
    args: &SearchCatalogArgs,
    db: &mut diesel::SqliteConnection,
    limit: i64,
    campaign_sources: Option<Vec<String>>,
) -> Result<Value, McpError> {
    let mut filter = ItemFilter::new();

    if let Some(name) = args.name.as_deref() {
        filter = filter.with_name_contains(name);
    }
    if let Some(rarity) = args.rarity.as_deref() {
        filter = filter.with_rarity(rarity);
    }
    if let Some(item_type) = args.item_type.as_deref() {
        filter = filter.with_type(item_type);
    }
    if let Some(sources) = campaign_sources {
        filter = filter.with_sources(sources);
    }

    let items = catalog_dal::search_items_paginated(db, &filter, limit, 0)
        .map_err(|e| McpError::Internal(e.to_string()))?;

    let data: Vec<Value> = items
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

    McpResponse::list("items", data)
}

fn search_spells(
    args: &SearchCatalogArgs,
    db: &mut diesel::SqliteConnection,
    limit: i64,
    campaign_sources: Option<Vec<String>>,
) -> Result<Value, McpError> {
    let mut filter = SpellFilter::new();

    if let Some(name) = args.name.as_deref() {
        filter = filter.with_name_contains(name);
    }
    if let Some(level) = args.level {
        filter = filter.with_level(level as i32);
    }
    if let Some(school) = args.school.as_deref() {
        filter = filter.with_school(school);
    }
    if let Some(sources) = campaign_sources {
        filter = filter.with_sources(sources);
    }

    let spells = catalog_dal::search_spells_paginated(db, &filter, limit, 0)
        .map_err(|e| McpError::Internal(e.to_string()))?;

    let data: Vec<Value> = spells
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

    McpResponse::list("spells", data)
}

fn search_races(
    args: &SearchCatalogArgs,
    db: &mut diesel::SqliteConnection,
    limit: i64,
    campaign_sources: Option<Vec<String>>,
) -> Result<Value, McpError> {
    let mut filter = RaceFilter::new();
    if let Some(name) = args.name.as_deref() {
        filter = filter.with_name_contains(name);
    }
    if let Some(sources) = campaign_sources {
        filter = filter.with_sources(sources);
    }

    let results = catalog_dal::search_races_paginated(db, &filter, limit, 0)
        .map_err(|e| McpError::Internal(e.to_string()))?;
    let data: Vec<Value> = results
        .iter()
        .map(|r| json!({"name": r.name, "source": r.source}))
        .collect();
    McpResponse::list("races", data)
}

fn search_classes(
    args: &SearchCatalogArgs,
    db: &mut diesel::SqliteConnection,
    limit: i64,
    campaign_sources: Option<Vec<String>>,
) -> Result<Value, McpError> {
    let mut filter = ClassFilter::new();
    if let Some(name) = args.name.as_deref() {
        filter = filter.with_name_contains(name);
    }
    if let Some(sources) = campaign_sources {
        filter = filter.with_sources(sources);
    }

    let results = catalog_dal::search_classes_paginated(db, &filter, limit, 0)
        .map_err(|e| McpError::Internal(e.to_string()))?;
    let data: Vec<Value> = results
        .iter()
        .map(|c| json!({"name": c.name, "source": c.source}))
        .collect();
    McpResponse::list("classes", data)
}

fn search_backgrounds(
    args: &SearchCatalogArgs,
    db: &mut diesel::SqliteConnection,
    limit: i64,
    campaign_sources: Option<Vec<String>>,
) -> Result<Value, McpError> {
    let mut filter = BackgroundFilter::new();
    if let Some(name) = args.name.as_deref() {
        filter = filter.with_name_contains(name);
    }
    if let Some(sources) = campaign_sources {
        filter = filter.with_sources(sources);
    }

    let results = catalog_dal::search_backgrounds_paginated(db, &filter, limit, 0)
        .map_err(|e| McpError::Internal(e.to_string()))?;
    let data: Vec<Value> = results
        .iter()
        .map(|b| json!({"name": b.name, "source": b.source}))
        .collect();
    McpResponse::list("backgrounds", data)
}

fn search_feats(
    args: &SearchCatalogArgs,
    db: &mut diesel::SqliteConnection,
    limit: i64,
    campaign_sources: Option<Vec<String>>,
) -> Result<Value, McpError> {
    let mut filter = FeatFilter::new();
    if let Some(name) = args.name.as_deref() {
        filter = filter.with_name_contains(name);
    }
    if let Some(sources) = campaign_sources {
        filter = filter.with_sources(sources);
    }

    let results = catalog_dal::search_feats_paginated(db, &filter, limit, 0)
        .map_err(|e| McpError::Internal(e.to_string()))?;
    let data: Vec<Value> = results
        .iter()
        .map(|f| json!({"name": f.name, "source": f.source}))
        .collect();
    McpResponse::list("feats", data)
}

fn search_conditions(
    args: &SearchCatalogArgs,
    db: &mut diesel::SqliteConnection,
    limit: i64,
    campaign_sources: Option<Vec<String>>,
) -> Result<Value, McpError> {
    let mut filter = ConditionFilter::new();
    if let Some(name) = args.name.as_deref() {
        filter = filter.with_name_contains(name);
    }
    if let Some(sources) = campaign_sources {
        filter = filter.with_sources(sources);
    }

    let results = catalog_dal::search_conditions_paginated(db, &filter, limit, 0)
        .map_err(|e| McpError::Internal(e.to_string()))?;
    let data: Vec<Value> = results
        .iter()
        .map(|c| json!({"name": c.name, "source": c.source}))
        .collect();
    McpResponse::list("conditions", data)
}
