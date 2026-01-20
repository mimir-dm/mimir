---
id: add-tauri-commands-and-frontend
level: task
title: "Add Tauri commands and frontend integration for frontmatter sync"
short_code: "MIMIR-T-0293"
created_at: 2026-01-03T14:17:34.737340+00:00
updated_at: 2026-01-03T14:55:32.103918+00:00
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

# Add Tauri commands and frontend integration for frontmatter sync

## Objective

Wire up the ModuleFrontmatterService to Tauri commands and update the frontend to use the document-first sync flow.

## Context

Part of Phase 5 (UI Integration) of the Campaign Authoring Framework. This connects the backend sync service to the UI, enabling:
1. Monsters/NPCs/items loaded from document frontmatter on module open
2. UI changes synced back to document frontmatter

## Blocked By

- [[MIMIR-T-0290]] - Database migrations
- [[MIMIR-T-0291]] - Models, DAL, and services
- [[MIMIR-T-0292]] - ModuleFrontmatterService

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `sync_module_from_document` Tauri command
- [ ] `sync_module_to_document` Tauri command
- [ ] Commands registered in `main.rs`
- [ ] Frontend `ModuleService.ts` updated with sync methods
- [ ] `ModuleMonsters.vue` calls sync on add/remove/update
- [ ] Module overview document triggers sync on load
- [ ] Remove old `sync_monsters_to_file` calls (deprecated)
- [ ] `npm run type-check` passes
- [ ] Manual testing: add monster in UI → appears in document frontmatter

## Implementation Notes

### Backend: Tauri Commands

**File:** `crates/mimir-dm/src/commands/campaign/module_frontmatter.rs` (NEW)

```rust
#[tauri::command]
pub async fn sync_module_from_document(
    state: tauri::State<'_, AppState>,
    module_id: i32,
) -> Result<SyncResult, String> {
    let service = ModuleFrontmatterService::new(&state.db);
    service.sync_from_document(module_id)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn sync_module_to_document(
    state: tauri::State<'_, AppState>,
    module_id: i32,
) -> Result<(), String> {
    let service = ModuleFrontmatterService::new(&state.db);
    service.sync_to_document(module_id)
        .map_err(|e| e.to_string())
}
```

**File:** `crates/mimir-dm/src/main.rs`
- Add to `.invoke_handler(tauri::generate_handler![...])`

### Frontend: ModuleService.ts

**File:** `frontend/src/services/ModuleService.ts`

```typescript
export const ModuleService = {
  // ... existing methods
  
  async syncFromDocument(moduleId: number): Promise<SyncResult> {
    return invoke('sync_module_from_document', { moduleId });
  },
  
  async syncToDocument(moduleId: number): Promise<void> {
    return invoke('sync_module_to_document', { moduleId });
  },
};

interface SyncResult {
  monsters_synced: number;
  npcs_synced: number;
  items_synced: number;
}
```

### Frontend: ModuleMonsters.vue Changes

**File:** `frontend/src/features/modules/components/ModuleMonsters.vue`

```typescript
// On add monster
async function addMonster(monster: Monster) {
  await invoke('add_module_monster', { ... });
  await ModuleService.syncToDocument(props.moduleId); // NEW
  await loadMonsters();
}

// On remove monster
async function removeMonster(id: number) {
  await invoke('remove_module_monster', { id });
  await ModuleService.syncToDocument(props.moduleId); // NEW
  await loadMonsters();
}

// On update monster (quantity, encounter tag)
async function updateMonster(id: number, update: UpdateMonster) {
  await invoke('update_module_monster', { id, ...update });
  await ModuleService.syncToDocument(props.moduleId); // NEW
  await loadMonsters();
}

// REMOVE: sync_module_monsters_to_file calls (no longer needed)
```

### Frontend: Module Load Trigger

**File:** `frontend/src/features/modules/views/ModuleBoardView.vue` (or appropriate view)

```typescript
// On mount or when module changes
watch(() => props.moduleId, async (moduleId) => {
  if (moduleId) {
    // Sync from document to populate DB cache
    await ModuleService.syncFromDocument(moduleId);
  }
}, { immediate: true });
```

### Files to Create

- `crates/mimir-dm/src/commands/campaign/module_frontmatter.rs`

### Files to Modify

- `crates/mimir-dm/src/commands/campaign/mod.rs` - Add export
- `crates/mimir-dm/src/main.rs` - Register commands
- `frontend/src/services/ModuleService.ts` - Add sync methods
- `frontend/src/features/modules/components/ModuleMonsters.vue` - Use sync
- `frontend/src/features/modules/views/ModuleBoardView.vue` - Trigger sync on load

## Status Updates

*To be added during implementation*