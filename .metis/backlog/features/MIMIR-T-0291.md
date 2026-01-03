---
id: implement-modulenpc-and-moduleitem
level: task
title: "Implement ModuleNpc and ModuleItem models, DAL, and services"
short_code: "MIMIR-T-0291"
created_at: 2026-01-03T14:17:34.447315+00:00
updated_at: 2026-01-03T14:55:31.775555+00:00
parent: 
blocked_by: []
archived: false

tags:
  - "#task"
  - "#feature"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: NULL
---

# Implement ModuleNpc and ModuleItem models, DAL, and services

## Objective

Create the Rust models, data access layer (DAL), and services for ModuleNpc and ModuleItem entities, following the established `ModuleMonster` pattern.

## Context

Part of Phase 5 (UI Integration) of the Campaign Authoring Framework. These provide the backend infrastructure to cache and query NPCs/items parsed from module frontmatter.

## Blocked By

- [[MIMIR-T-0290]] - Database migrations must exist first

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `ModuleNpc` model with `NewModuleNpc`, `UpdateModuleNpc` structs
- [ ] `ModuleItem` model with `NewModuleItem`, `UpdateModuleItem` structs
- [ ] `ModuleNpcRepository` with CRUD operations
- [ ] `ModuleItemRepository` with CRUD operations
- [ ] `ModuleNpcService` with add/remove/update/list methods
- [ ] `ModuleItemService` with add/remove/update/list methods
- [ ] Group-by methods (npcs by role, items by location)
- [ ] All exports added to `mod.rs` files
- [ ] `cargo check` passes

## Implementation Notes

### Files to Create

**Models:**
- `crates/mimir-dm-core/src/models/campaign/module_npcs.rs`
- `crates/mimir-dm-core/src/models/campaign/module_items.rs`

**DAL:**
- `crates/mimir-dm-core/src/dal/campaign/module_npcs.rs`
- `crates/mimir-dm-core/src/dal/campaign/module_items.rs`

**Services:**
- `crates/mimir-dm-core/src/services/module_npc_service.rs`
- `crates/mimir-dm-core/src/services/module_item_service.rs`

### Files to Modify

- `crates/mimir-dm-core/src/models/campaign/mod.rs` - Add exports
- `crates/mimir-dm-core/src/dal/campaign/mod.rs` - Add exports
- `crates/mimir-dm-core/src/services/mod.rs` - Add exports

### Reference Pattern

**Key difference from monsters:** NPCs link to the `characters` table (NPCs are characters with `is_npc = true`), not catalog references.

```rust
// Model - links to characters table
pub struct ModuleNpc {
    pub id: i32,
    pub module_id: i32,
    pub character_id: i32,        // FK to characters table
    pub role: Option<String>,     // quest_giver, antagonist, ally, etc.
    pub encounter_tag: Option<String>,
    pub notes: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

// Extended model with character data
pub struct ModuleNpcWithCharacter {
    pub module_npc: ModuleNpc,
    pub character: Character,     // Full character data
}

// Repository methods
impl ModuleNpcRepository {
    pub fn create(conn, new_npc) -> Result<ModuleNpc>
    pub fn find_by_id(conn, id) -> Result<ModuleNpc>
    pub fn list_by_module(conn, module_id) -> Result<Vec<ModuleNpc>>
    pub fn list_by_role(conn, module_id, role) -> Result<Vec<ModuleNpc>>
    pub fn find_by_character(conn, module_id, character_id) -> Result<Option<ModuleNpc>>
    pub fn update(conn, id, update) -> Result<ModuleNpc>
    pub fn delete(conn, id) -> Result<usize>
    pub fn delete_by_module(conn, module_id) -> Result<usize>
}

// Service methods
impl ModuleNpcService {
    pub fn add_npc(&self, module_id, character_id, role, notes) -> Result<ModuleNpc>
    pub fn add_npc_by_name(&self, campaign_id, module_id, name, role, notes) -> Result<ModuleNpc>
    pub fn remove_npc(&self, id) -> Result<()>
    pub fn update_npc(&self, id, update) -> Result<ModuleNpc>
    pub fn get_npcs_for_module(&self, module_id) -> Result<Vec<ModuleNpc>>
    pub fn get_npcs_with_characters(&self, module_id) -> Result<Vec<ModuleNpcWithCharacter>>
    pub fn get_npcs_grouped_by_role(&self, module_id) -> Result<Vec<(Option<String>, Vec<ModuleNpcWithCharacter>)>>
    pub fn clear_module_npcs(&self, module_id) -> Result<()>
}
```

### Frontmatter Sync Note

When syncing from document frontmatter:
1. Parse NPC `name` from frontmatter
2. Look up character by name + `is_npc = true` in campaign
3. If found, create `module_npcs` link with `character_id`
4. If not found, log warning (NPC must exist as character first)

## Status Updates

*To be added during implementation*