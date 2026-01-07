---
id: implement-modulefrontmatterservice
level: task
title: "Implement ModuleFrontmatterService for document-first sync"
short_code: "MIMIR-T-0292"
created_at: 2026-01-03T14:17:34.590254+00:00
updated_at: 2026-01-03T14:55:31.930119+00:00
parent: 
blocked_by: []
archived: true

tags:
  - "#task"
  - "#feature"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: NULL
---

# Implement ModuleFrontmatterService for document-first sync

## Objective

Create the service that orchestrates bidirectional sync between module documents (YAML frontmatter) and database tables. This is the core of the "document as source of truth" architecture.

## Context

Part of Phase 5 (UI Integration) of the Campaign Authoring Framework. This service handles:
1. Parsing frontmatter from document → syncing to DB (on document load)
2. Reading from DB → updating frontmatter in document (on UI change)

## Blocked By

- [[MIMIR-T-0290]] - Database migrations
- [[MIMIR-T-0291]] - Models, DAL, and services for NPCs/Items

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `sync_from_document(module_id)` - Parse doc frontmatter → replace DB data
- [ ] `sync_to_document(module_id)` - Read DB → update doc frontmatter
- [ ] `get_module_overview_document(module_id)` - Find the module's overview doc
- [ ] Frontmatter serialization preserves non-catalog fields (title, theme, etc.)
- [ ] REPLACE strategy for sync (clear + insert, no merge)
- [ ] Handles missing/invalid frontmatter gracefully
- [ ] Unit tests for parse/serialize round-trip
- [ ] `cargo check` passes

## Implementation Notes

### File to Create

- `crates/mimir-dm-core/src/services/module_frontmatter_service.rs`

### Key Methods

```rust
pub struct ModuleFrontmatterService {
    conn: DbConnection,
    document_service: DocumentService,
    monster_service: ModuleMonsterService,
    npc_service: ModuleNpcService,
    item_service: ModuleItemService,
}

impl ModuleFrontmatterService {
    /// Find the module overview document (template starts with "module_")
    pub fn get_module_overview_document(&self, module_id: i32) -> Result<Option<Document>>
    
    /// Parse document frontmatter and sync to database tables
    /// Uses REPLACE strategy: clears existing data, inserts parsed data
    pub fn sync_from_document(&self, module_id: i32) -> Result<SyncResult>
    
    /// Read database tables and update document frontmatter
    /// Preserves non-catalog fields in frontmatter
    pub fn sync_to_document(&self, module_id: i32) -> Result<()>
    
    /// Update frontmatter section in markdown document
    fn update_document_frontmatter(&self, file_path: &str, frontmatter: &ModuleFrontmatter) -> Result<()>
}

pub struct SyncResult {
    pub monsters_synced: usize,
    pub npcs_synced: usize,
    pub items_synced: usize,
}
```

### Sync From Document Flow

```
1. get_module_overview_document(module_id)
2. read_document_file(file_path)
3. ModuleFrontmatter::parse_from_markdown(content)
4. If parse succeeds:
   a. monster_service.clear_module_monsters(module_id)
   b. For each monster in frontmatter.monsters:
      - monster_service.add_monster(...)
   c. npc_service.clear_module_npcs(module_id)
   d. For each npc in frontmatter.npcs:
      - npc_service.add_npc(...)
   e. item_service.clear_module_items(module_id)
   f. For each item in frontmatter.items:
      - item_service.add_item(...)
5. Return SyncResult with counts
```

### Sync To Document Flow

```
1. get_module_overview_document(module_id)
2. read_document_file(file_path)
3. Parse existing frontmatter (to preserve non-catalog fields)
4. Load current data from DB:
   - monsters = monster_service.get_monsters_for_module(module_id)
   - npcs = npc_service.get_npcs_for_module(module_id)
   - items = item_service.get_items_for_module(module_id)
5. Convert DB records to frontmatter format (MonsterReference, etc.)
6. Update frontmatter.monsters, frontmatter.npcs, frontmatter.items
7. Serialize frontmatter to YAML
8. Replace frontmatter section in document
9. save_document_file(file_path, updated_content)
```

### Frontmatter Replacement

```rust
fn update_document_frontmatter(content: &str, frontmatter: &ModuleFrontmatter) -> Result<String> {
    // 1. Extract content after frontmatter
    let body = ModuleFrontmatter::extract_content(content);
    
    // 2. Serialize frontmatter to YAML
    let yaml = serde_yaml::to_string(frontmatter)?;
    
    // 3. Reconstruct document
    Ok(format!("---\n{}---\n{}", yaml, body))
}
```

### Edge Cases

- **No overview document**: Return Ok(None) or error
- **No frontmatter in document**: Create minimal frontmatter with just catalog data
- **Parse error**: Log warning, return empty sync result
- **Empty arrays**: Valid state, clear DB tables

## Status Updates

*To be added during implementation*