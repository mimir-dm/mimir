//! Catalog Tools
//!
//! Single `search_catalog` tool for searching the D&D 5e catalog across all
//! categories. Registry-based: one typed argument struct, one handler, one
//! registration.

use mimir_core::services::{
    CampaignContext, CatalogCategory, CatalogHit, CatalogQuery, CatalogSearch,
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
        /// Filter by school of magic (spells only): a name such as evocation or necromancy, or the catalog code such as V
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
    let category = CatalogCategory::parse(&args.category).ok_or_else(|| {
        let valid: Vec<&str> = CatalogCategory::ALL.iter().map(|c| c.as_str()).collect();
        McpError::InvalidArguments(format!(
            "Invalid category '{}'. Must be one of: {}",
            args.category,
            valid.join(", ")
        ))
    })?;

    let query = CatalogQuery {
        name: args.name,
        monster_type: args.monster_type,
        cr_min: args.cr_min,
        cr_max: args.cr_max,
        item_type: args.item_type,
        rarity: args.rarity,
        spell_level: args.level.map(|l| l as i32),
        school: args.school,
        class_name: args.class_name,
        limit: args.limit.unwrap_or(20),
    };
    // The active campaign scopes results to its sources and, for monsters,
    // merges its homebrew unless the caller opts out.
    let campaign = ctx
        .get_active_campaign_id()
        .map(|campaign_id| CampaignContext {
            campaign_id,
            include_homebrew: args.include_homebrew.unwrap_or(true),
        });

    let mut db = ctx.connect()?;
    let hits = CatalogSearch::new(&mut db)
        .search(category, &query, campaign.as_ref())
        .map_err(|e| McpError::Internal(e.to_string()))?;

    McpResponse::list(list_key(category), hits.into_iter().map(hit_json).collect())
}

/// The response key for a category's result list.
fn list_key(category: CatalogCategory) -> &'static str {
    match category {
        CatalogCategory::Monster => "monsters",
        CatalogCategory::Item => "items",
        CatalogCategory::Spell => "spells",
        CatalogCategory::Race => "races",
        CatalogCategory::Class => "classes",
        CatalogCategory::Background => "backgrounds",
        CatalogCategory::Feat => "feats",
        CatalogCategory::Condition => "conditions",
    }
}

/// One result row, in the shape this tool has always returned.
fn hit_json(hit: CatalogHit) -> Value {
    match hit {
        CatalogHit::Monster {
            name,
            source,
            cr,
            creature_type,
            size,
            homebrew_id: None,
        } => json!({
            "name": name,
            "source": source,
            "cr": cr,
            "creature_type": creature_type,
            "size": size,
            "is_homebrew": false
        }),
        CatalogHit::Monster {
            name,
            source,
            cr,
            creature_type,
            size,
            homebrew_id: Some(id),
        } => json!({
            "name": name,
            "source": source,
            "homebrew_id": id,
            "cr": cr,
            "creature_type": creature_type,
            "size": size,
            "is_homebrew": true
        }),
        CatalogHit::Item {
            name,
            source,
            rarity,
            item_type,
        } => json!({
            "name": name,
            "source": source,
            "rarity": rarity,
            "item_type": item_type
        }),
        CatalogHit::Spell {
            name,
            source,
            level,
            school,
        } => json!({
            "name": name,
            "source": source,
            "level": level,
            "school": school
        }),
        CatalogHit::Entry { name, source } => json!({"name": name, "source": source}),
    }
}
