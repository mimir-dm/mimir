//! Homebrew Tools
//!
//! MCP tools for campaign homebrew CRUD operations (items, monsters, spells).
//! Uses a `content_type` parameter to dispatch to the appropriate service.
//! This family is registry-based: typed argument structs, handlers, and one
//! entry each in `registered_tools()`.

use mimir_core::services::{
    CreateHomebrewItemInput, CreateHomebrewMonsterInput, CreateHomebrewSpellInput, HomebrewService,
    UpdateHomebrewItemInput, UpdateHomebrewMonsterInput, UpdateHomebrewSpellInput,
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

/// All homebrew-family tools.
pub fn registered_tools() -> Vec<RegisteredTool> {
    vec![
        tool!(
            "list_homebrew",
            "List all homebrew content of a given type in the active campaign. Requires an active campaign.",
            ListHomebrewArgs,
            list_homebrew
        ),
        tool!(
            "get_homebrew",
            "Get a homebrew item, monster, or spell by ID",
            HomebrewIdArgs,
            get_homebrew
        ),
        tool!(
            "create_homebrew",
            "Create a new homebrew item, monster, or spell in the active campaign. Requires an active campaign. To clone from a catalog entry, provide cloned_from_name and cloned_from_source — the catalog entry's full data will be used as the base, and any fields in data will override specific properties. When cloning, data is optional. IMPORTANT: Use search_catalog first to find the exact name and source before cloning.",
            CreateHomebrewArgs,
            create_homebrew
        ),
        tool!(
            "update_homebrew",
            "Update a homebrew item, monster, or spell",
            UpdateHomebrewArgs,
            update_homebrew
        ),
        tool!(
            "delete_homebrew",
            "Delete a homebrew item, monster, or spell by ID",
            HomebrewIdArgs,
            delete_homebrew
        ),
    ]
}

// =============================================================================
// Content type validation
// =============================================================================

const VALID_CONTENT_TYPES: &[&str] = &["item", "monster", "spell"];

fn validate_content_type(ct: &str) -> Result<(), McpError> {
    if !VALID_CONTENT_TYPES.contains(&ct) {
        return Err(McpError::InvalidArguments(format!(
            "Invalid content_type '{}'. Must be one of: item, monster, spell",
            ct
        )));
    }
    Ok(())
}

// =============================================================================
// Arguments
// =============================================================================

tool_args! {
    pub struct ListHomebrewArgs {
        /// Type of homebrew content: item, monster, or spell
        pub content_type: String,
    }
}

tool_args! {
    pub struct HomebrewIdArgs {
        /// Type of homebrew content: item, monster, or spell
        pub content_type: String,
        /// The homebrew content ID
        pub id: String,
    }
}

tool_args! {
    pub struct CreateHomebrewArgs {
        /// Type of homebrew content: item, monster, or spell
        pub content_type: String,
        /// Name of the homebrew content
        pub name: String,
        /// JSON string with content data. Required when not cloning. When cloning, fields here override the catalog data.
        pub data: Option<String>,
        /// Item type (items only): weapon, armor, potion, ring, rod, scroll, staff, wand, wondrous item, adventuring gear
        pub item_type: Option<String>,
        /// Rarity (items only): common, uncommon, rare, very rare, legendary, artifact
        pub rarity: Option<String>,
        /// Challenge rating (monsters only, e.g. '1/4', '1', '5', '20')
        pub cr: Option<String>,
        /// Creature type (monsters only, e.g. 'humanoid', 'dragon', 'undead')
        pub creature_type: Option<String>,
        /// Size (monsters only): T, S, M, L, H, G
        pub size: Option<String>,
        /// Spell level (spells only, 0 for cantrip, 1-9)
        pub level: Option<i64>,
        /// School of magic (spells only, e.g. 'evocation', 'necromancy')
        pub school: Option<String>,
        /// Name of the catalog entry to clone from. Must be used with cloned_from_source.
        pub cloned_from_name: Option<String>,
        /// Source book of the catalog entry to clone from (e.g. PHB, DMG, MM). Must be used with cloned_from_name.
        pub cloned_from_source: Option<String>,
    }
}

tool_args! {
    pub struct UpdateHomebrewArgs {
        /// Type of homebrew content: item, monster, or spell
        pub content_type: String,
        /// The homebrew content ID
        pub id: String,
        /// New name
        pub name: Option<String>,
        /// New JSON data string
        pub data: Option<String>,
        /// New item type (items only)
        pub item_type: Option<String>,
        /// New rarity (items only)
        pub rarity: Option<String>,
        /// New challenge rating (monsters only)
        pub cr: Option<String>,
        /// New creature type (monsters only)
        pub creature_type: Option<String>,
        /// New size (monsters only)
        pub size: Option<String>,
        /// New spell level (spells only)
        pub level: Option<i64>,
        /// New school of magic (spells only)
        pub school: Option<String>,
    }
}

// =============================================================================
// JSON serialization helpers
// =============================================================================

fn item_to_json(item: &mimir_core::models::campaign::CampaignHomebrewItem) -> Value {
    json!({
        "id": item.id,
        "campaign_id": item.campaign_id,
        "name": item.name,
        "item_type": item.item_type,
        "rarity": item.rarity,
        "data": item.data,
        "cloned_from_name": item.cloned_from_name,
        "cloned_from_source": item.cloned_from_source,
        "created_at": item.created_at,
        "updated_at": item.updated_at,
    })
}

fn monster_to_json(monster: &mimir_core::models::campaign::CampaignHomebrewMonster) -> Value {
    json!({
        "id": monster.id,
        "campaign_id": monster.campaign_id,
        "name": monster.name,
        "cr": monster.cr,
        "creature_type": monster.creature_type,
        "size": monster.size,
        "data": monster.data,
        "cloned_from_name": monster.cloned_from_name,
        "cloned_from_source": monster.cloned_from_source,
        "created_at": monster.created_at,
        "updated_at": monster.updated_at,
    })
}

fn spell_to_json(spell: &mimir_core::models::campaign::CampaignHomebrewSpell) -> Value {
    json!({
        "id": spell.id,
        "campaign_id": spell.campaign_id,
        "name": spell.name,
        "level": spell.level,
        "school": spell.school,
        "data": spell.data,
        "cloned_from_name": spell.cloned_from_name,
        "cloned_from_source": spell.cloned_from_source,
        "created_at": spell.created_at,
        "updated_at": spell.updated_at,
    })
}

// =============================================================================
// Handlers
// =============================================================================

pub async fn list_homebrew(
    ctx: &Arc<McpContext>,
    args: ListHomebrewArgs,
) -> Result<Value, McpError> {
    validate_content_type(&args.content_type)?;
    let campaign_id = ctx
        .get_active_campaign_id()
        .ok_or(McpError::NoActiveCampaign)?;

    let mut db = ctx.connect()?;
    let mut svc = HomebrewService::new(&mut db);

    match args.content_type.as_str() {
        "item" => {
            let items = svc.list_items(&campaign_id)?;
            McpResponse::list("items", items.iter().map(item_to_json).collect())
        }
        "monster" => {
            let monsters = svc.list_monsters(&campaign_id)?;
            McpResponse::list("monsters", monsters.iter().map(monster_to_json).collect())
        }
        "spell" => {
            let spells = svc.list_spells(&campaign_id)?;
            McpResponse::list("spells", spells.iter().map(spell_to_json).collect())
        }
        _ => unreachable!(),
    }
}

pub async fn get_homebrew(
    ctx: &Arc<McpContext>,
    args: HomebrewIdArgs,
) -> Result<Value, McpError> {
    validate_content_type(&args.content_type)?;

    let mut db = ctx.connect()?;
    let mut svc = HomebrewService::new(&mut db);

    match args.content_type.as_str() {
        "item" => {
            let item = svc.get_item(&args.id)?;
            McpResponse::get("item", item_to_json(&item))
        }
        "monster" => {
            let monster = svc.get_monster(&args.id)?;
            McpResponse::get("monster", monster_to_json(&monster))
        }
        "spell" => {
            let spell = svc.get_spell(&args.id)?;
            McpResponse::get("spell", spell_to_json(&spell))
        }
        _ => unreachable!(),
    }
}

pub async fn create_homebrew(
    ctx: &Arc<McpContext>,
    args: CreateHomebrewArgs,
) -> Result<Value, McpError> {
    validate_content_type(&args.content_type)?;
    let campaign_id = ctx
        .get_active_campaign_id()
        .ok_or(McpError::NoActiveCampaign)?;

    // data is required when not cloning
    if args.data.is_none()
        && (args.cloned_from_name.is_none() || args.cloned_from_source.is_none())
    {
        return Err(McpError::InvalidArguments(
            "data is required when not cloning from catalog (provide cloned_from_name and cloned_from_source to clone)".to_string(),
        ));
    }

    let mut db = ctx.connect()?;
    let mut svc = HomebrewService::new(&mut db);

    match args.content_type.as_str() {
        "item" => {
            let input = CreateHomebrewItemInput {
                campaign_id,
                name: args.name,
                data: args.data,
                item_type: args.item_type,
                rarity: args.rarity,
                cloned_from_name: args.cloned_from_name,
                cloned_from_source: args.cloned_from_source,
            };
            let item = svc.create_item(input)?;
            McpResponse::created("item", item_to_json(&item))
        }
        "monster" => {
            let input = CreateHomebrewMonsterInput {
                campaign_id,
                name: args.name,
                data: args.data,
                cr: args.cr,
                creature_type: args.creature_type,
                size: args.size,
                cloned_from_name: args.cloned_from_name,
                cloned_from_source: args.cloned_from_source,
            };
            let monster = svc.create_monster(input)?;
            McpResponse::created("monster", monster_to_json(&monster))
        }
        "spell" => {
            let input = CreateHomebrewSpellInput {
                campaign_id,
                name: args.name,
                data: args.data,
                level: args.level.map(|l| l as i32),
                school: args.school,
                cloned_from_name: args.cloned_from_name,
                cloned_from_source: args.cloned_from_source,
            };
            let spell = svc.create_spell(input)?;
            McpResponse::created("spell", spell_to_json(&spell))
        }
        _ => unreachable!(),
    }
}

pub async fn update_homebrew(
    ctx: &Arc<McpContext>,
    args: UpdateHomebrewArgs,
) -> Result<Value, McpError> {
    validate_content_type(&args.content_type)?;

    let mut db = ctx.connect()?;
    let mut svc = HomebrewService::new(&mut db);

    match args.content_type.as_str() {
        "item" => {
            let input = UpdateHomebrewItemInput {
                name: args.name,
                data: args.data,
                item_type: args.item_type.map(Some),
                rarity: args.rarity.map(Some),
            };
            let item = svc.update_item(&args.id, input)?;
            McpResponse::updated("item", item_to_json(&item))
        }
        "monster" => {
            let input = UpdateHomebrewMonsterInput {
                name: args.name,
                data: args.data,
                cr: args.cr.map(Some),
                creature_type: args.creature_type.map(Some),
                size: args.size.map(Some),
            };
            let monster = svc.update_monster(&args.id, input)?;
            McpResponse::updated("monster", monster_to_json(&monster))
        }
        "spell" => {
            let input = UpdateHomebrewSpellInput {
                name: args.name,
                data: args.data,
                level: args.level.map(|l| Some(l as i32)),
                school: args.school.map(Some),
            };
            let spell = svc.update_spell(&args.id, input)?;
            McpResponse::updated("spell", spell_to_json(&spell))
        }
        _ => unreachable!(),
    }
}

pub async fn delete_homebrew(
    ctx: &Arc<McpContext>,
    args: HomebrewIdArgs,
) -> Result<Value, McpError> {
    validate_content_type(&args.content_type)?;

    let mut db = ctx.connect()?;
    let mut svc = HomebrewService::new(&mut db);

    match args.content_type.as_str() {
        "item" => svc.delete_item(&args.id)?,
        "monster" => svc.delete_monster(&args.id)?,
        "spell" => svc.delete_spell(&args.id)?,
        _ => unreachable!(),
    }

    McpResponse::deleted(&args.id)
}
