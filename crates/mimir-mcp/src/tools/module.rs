//! Module Tools
//!
//! MCP tools for module management. This family is registry-based: each tool
//! is a typed argument struct (schema + parser from one source), a handler,
//! and one entry in `registered_tools()`.

use mimir_core::dal::campaign as dal;
use mimir_core::services::{
    AddMonsterInput, CreateModuleInput, ModuleService, ModuleType, MonsterRef, ServiceError,
    UpdateModuleInput,
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

/// All module-family tools.
pub fn registered_tools() -> Vec<RegisteredTool> {
    vec![
        tool!(
            "create_module",
            "Create a new module (an adventure chapter) in the active campaign. Requires an active campaign.",
            CreateModuleArgs,
            create_module
        ),
        tool!(
            "list_modules",
            "List all modules in the active campaign. Requires an active campaign.",
            ListModulesArgs,
            list_modules
        ),
        tool!(
            "get_module_details",
            "Get detailed information about a module including documents, monsters, and items",
            GetModuleDetailsArgs,
            get_module_details
        ),
        tool!(
            "update_module",
            "Update a module's name or description",
            UpdateModuleArgs,
            update_module
        ),
        tool!(
            "delete_module",
            "Delete a module and all its contents",
            DeleteModuleArgs,
            delete_module
        ),
        tool!(
            "add_monster_to_module",
            "Add a monster to a module. Use monster_name for catalog monsters or homebrew_monster_id for homebrew monsters. Exactly one path must be provided. If the module already has an entry for the same monster, its quantity is incremented instead of creating a duplicate; use update_module_monster to set an exact quantity.",
            AddMonsterArgs,
            add_monster_to_module
        ),
        tool!(
            "update_module_monster",
            "Update a module monster entry's quantity, display name, or notes. Use the id returned by add_monster_to_module or get_module_details.",
            UpdateModuleMonsterArgs,
            update_module_monster
        ),
        tool!(
            "remove_monster_from_module",
            "Remove a monster from a module",
            RemoveMonsterArgs,
            remove_monster_from_module
        ),
        tool!(
            "add_item_to_module",
            "NOT YET IMPLEMENTED — currently returns an error. Module-level item/loot tracking does not exist yet. To give a character loot, use add_item_to_character instead.",
            AddItemToModuleArgs,
            add_item_to_module
        ),
    ]
}

// =============================================================================
// Arguments
// =============================================================================

tool_args! {
    pub struct CreateModuleArgs {
        /// Name of the module
        pub name: String,
        /// Description of the module
        pub description: Option<String>,
        /// Type of module: adventure, location, organization (default: adventure)
        pub module_type: Option<String>,
    }
}

tool_args! {
    pub struct ListModulesArgs {}
}

tool_args! {
    pub struct GetModuleDetailsArgs {
        /// The ID of the module
        pub module_id: String,
    }
}

tool_args! {
    pub struct UpdateModuleArgs {
        /// The ID of the module
        pub module_id: String,
        /// New module name
        pub name: Option<String>,
        /// New module description
        pub description: Option<String>,
    }
}

tool_args! {
    pub struct DeleteModuleArgs {
        /// The ID of the module to delete
        pub module_id: String,
    }
}

tool_args! {
    pub struct AddMonsterArgs {
        /// The ID of the module
        pub module_id: String,
        /// Name of the monster from the catalog (use with monster_source)
        pub monster_name: Option<String>,
        /// Source book code for catalog monster (e.g. MM, VGM). Defaults to MM if monster_name is provided.
        pub monster_source: Option<String>,
        /// ID of a homebrew monster from the active campaign (alternative to monster_name)
        pub homebrew_monster_id: Option<String>,
        /// Number of this monster (default: 1)
        pub count: Option<i64>,
        /// Optional display name override
        pub display_name: Option<String>,
        /// Optional notes about this monster
        pub notes: Option<String>,
    }
}

tool_args! {
    pub struct UpdateModuleMonsterArgs {
        /// The ID of the module monster entry
        pub module_monster_id: String,
        /// New quantity (sets the exact count)
        pub quantity: Option<i64>,
        /// New display name override
        pub display_name: Option<String>,
        /// New notes
        pub notes: Option<String>,
    }
}

tool_args! {
    pub struct RemoveMonsterArgs {
        /// The ID of the module monster entry to remove
        pub module_monster_id: String,
    }
}

tool_args! {
    pub struct AddItemToModuleArgs {
        /// The ID of the module
        pub module_id: String,
        /// Name of the item from the catalog
        pub item_name: String,
        /// Quantity of this item (default: 1)
        pub quantity: Option<i64>,
        /// Optional notes about this item
        pub notes: Option<String>,
    }
}

// =============================================================================
// Handlers
// =============================================================================

/// Map a service error to the MCP vocabulary: not-found and validation are
/// caller errors; everything else is internal.
fn service_err(e: ServiceError) -> McpError {
    match e {
        ServiceError::NotFound { entity_type, id } => {
            McpError::InvalidArguments(format!("{} '{}' not found", entity_type, id))
        }
        ServiceError::Validation(msg) => McpError::InvalidArguments(msg),
        other => McpError::Internal(other.to_string()),
    }
}

pub async fn create_module(
    ctx: &Arc<McpContext>,
    args: CreateModuleArgs,
) -> Result<Value, McpError> {
    let campaign_id = ctx
        .get_active_campaign_id()
        .ok_or(McpError::NoActiveCampaign)?;

    let module_type = ModuleType::from(args.module_type.as_deref());

    let mut db = ctx.connect()?;
    let mut service = ModuleService::new(&mut db);

    let mut input = CreateModuleInput::new(&campaign_id, &args.name).with_type(module_type);
    if let Some(desc) = args.description {
        input = input.with_description(desc);
    }

    let module = service
        .create(input)
        .map_err(|e| McpError::Internal(e.to_string()))?;

    McpResponse::created("module", json!({
        "id": module.id,
        "name": module.name,
        "description": module.description,
        "module_number": module.module_number
    }))
}

pub async fn list_modules(
    ctx: &Arc<McpContext>,
    _args: ListModulesArgs,
) -> Result<Value, McpError> {
    let campaign_id = ctx
        .get_active_campaign_id()
        .ok_or(McpError::NoActiveCampaign)?;

    let mut db = ctx.connect()?;
    let mut service = ModuleService::new(&mut db);

    let modules = service
        .list_for_campaign(&campaign_id)
        .map_err(|e| McpError::Internal(e.to_string()))?;

    let module_data: Vec<Value> = modules
        .iter()
        .map(|m| {
            json!({
                "id": m.id,
                "name": m.name,
                "description": m.description,
                "module_number": m.module_number
            })
        })
        .collect();

    McpResponse::list("modules", module_data)
}

pub async fn get_module_details(
    ctx: &Arc<McpContext>,
    args: GetModuleDetailsArgs,
) -> Result<Value, McpError> {
    let module_id = &args.module_id;
    let mut db = ctx.connect()?;

    // Get module
    let mut service = ModuleService::new(&mut db);
    let module = service
        .get(module_id)
        .map_err(|e| McpError::Internal(e.to_string()))?
        .ok_or_else(|| McpError::InvalidArguments(format!("Module '{}' not found", module_id)))?;

    // Get documents for this module
    let documents = dal::list_module_documents(&mut db, module_id)
        .map_err(|e| McpError::Internal(e.to_string()))?;

    let doc_data: Vec<Value> = documents
        .iter()
        .map(|d| {
            json!({
                "id": d.id,
                "title": d.title,
                "doc_type": d.doc_type
            })
        })
        .collect();

    // Get monsters for this module
    let monsters = ModuleService::new(&mut db)
        .list_monsters(module_id)
        .map_err(|e| McpError::Internal(e.to_string()))?;

    let monster_data: Vec<Value> = monsters
        .iter()
        .map(|m| {
            json!({
                "id": m.id,
                "monster_name": m.monster_name,
                "monster_source": m.monster_source,
                "homebrew_monster_id": m.homebrew_monster_id,
                "display_name": m.display_name,
                "quantity": m.quantity,
                "notes": m.notes,
                "is_homebrew": m.is_homebrew()
            })
        })
        .collect();

    McpResponse::ok(json!({
        "module": {
            "id": module.id,
            "name": module.name,
            "description": module.description,
            "module_number": module.module_number
        },
        "documents": doc_data,
        "monsters": monster_data
    }))
}

pub async fn update_module(
    ctx: &Arc<McpContext>,
    args: UpdateModuleArgs,
) -> Result<Value, McpError> {
    let mut input = UpdateModuleInput::default();
    if let Some(name) = args.name {
        input.name = Some(name);
    }
    if let Some(desc) = args.description {
        input.description = Some(Some(desc));
    }

    let mut db = ctx.connect()?;
    let mut service = ModuleService::new(&mut db);

    let module = service
        .update(&args.module_id, input)
        .map_err(|e| McpError::Internal(e.to_string()))?;

    McpResponse::updated("module", json!({
        "id": module.id,
        "name": module.name,
        "description": module.description,
        "module_number": module.module_number
    }))
}

pub async fn delete_module(
    ctx: &Arc<McpContext>,
    args: DeleteModuleArgs,
) -> Result<Value, McpError> {
    let mut db = ctx.connect()?;
    let mut service = ModuleService::new(&mut db);

    service
        .delete(&args.module_id)
        .map_err(|e| McpError::Internal(e.to_string()))?;

    McpResponse::deleted(&args.module_id)
}

pub async fn add_monster_to_module(
    ctx: &Arc<McpContext>,
    args: AddMonsterArgs,
) -> Result<Value, McpError> {
    // Validate mutual exclusivity (transport-level check)
    if args.monster_name.is_some() && args.homebrew_monster_id.is_some() {
        return Err(McpError::InvalidArguments(
            "Cannot specify both monster_name and homebrew_monster_id".to_string(),
        ));
    }
    let monster = if let Some(hb_id) = args.homebrew_monster_id {
        MonsterRef::Homebrew { id: hb_id }
    } else if let Some(name) = args.monster_name {
        MonsterRef::Catalog {
            name,
            source: args.monster_source.unwrap_or_else(|| "MM".to_string()),
        }
    } else {
        return Err(McpError::InvalidArguments(
            "Must specify either monster_name (catalog) or homebrew_monster_id (homebrew)"
                .to_string(),
        ));
    };

    let mut db = ctx.connect()?;

    let mut input = AddMonsterInput::new(&args.module_id, monster)
        .with_quantity(args.count.unwrap_or(1) as i32);
    if let Some(n) = args.notes {
        input = input.with_notes(n);
    }
    if let Some(dn) = args.display_name {
        input = input.with_display_name(dn);
    }

    let monster = ModuleService::new(&mut db)
        .add_monster(input)
        .map_err(service_err)?;

    McpResponse::added("module_monster", json!({
        "id": monster.id,
        "monster_name": monster.monster_name,
        "monster_source": monster.monster_source,
        "homebrew_monster_id": monster.homebrew_monster_id,
        "quantity": monster.quantity,
        "display_name": monster.display_name,
        "notes": monster.notes
    }))
}

pub async fn update_module_monster(
    ctx: &Arc<McpContext>,
    args: UpdateModuleMonsterArgs,
) -> Result<Value, McpError> {
    if args.quantity.is_none() && args.display_name.is_none() && args.notes.is_none() {
        return Err(McpError::InvalidArguments(
            "Provide at least one of quantity, display_name, or notes".to_string(),
        ));
    }

    let mut db = ctx.connect()?;
    let monster = ModuleService::new(&mut db)
        .update_monster(
            &args.module_monster_id,
            args.display_name.as_deref(),
            args.notes.as_deref(),
            args.quantity.map(|q| q as i32),
        )
        .map_err(service_err)?;

    McpResponse::updated("module_monster", json!({
        "id": monster.id,
        "monster_name": monster.monster_name,
        "monster_source": monster.monster_source,
        "homebrew_monster_id": monster.homebrew_monster_id,
        "quantity": monster.quantity,
        "display_name": monster.display_name,
        "notes": monster.notes
    }))
}

pub async fn remove_monster_from_module(
    ctx: &Arc<McpContext>,
    args: RemoveMonsterArgs,
) -> Result<Value, McpError> {
    let mut db = ctx.connect()?;

    ModuleService::new(&mut db)
        .remove_monster(&args.module_monster_id)
        .map_err(|e| match e {
            ServiceError::NotFound { .. } => McpError::InvalidArguments(format!(
                "Module monster '{}' not found",
                args.module_monster_id
            )),
            other => McpError::Internal(other.to_string()),
        })?;

    McpResponse::removed(&args.module_monster_id)
}

pub async fn add_item_to_module(
    _ctx: &Arc<McpContext>,
    _args: AddItemToModuleArgs,
) -> Result<Value, McpError> {
    // Module items table doesn't exist yet - this feature is not implemented
    Err(McpError::Internal(
        "Module item tracking is not yet implemented. Items can be added to character inventories instead.".to_string()
    ))
}
