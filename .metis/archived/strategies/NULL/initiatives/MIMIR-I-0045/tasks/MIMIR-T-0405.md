---
id: migrate-pinia-stores-to-new-backend
level: task
title: "Migrate Pinia stores to new backend"
short_code: "MIMIR-T-0405"
created_at: 2026-01-21T16:34:58.011037+00:00
updated_at: 2026-01-25T01:03:04.080288+00:00
parent: MIMIR-I-0045
blocked_by: []
archived: true

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: MIMIR-I-0045
---

# Migrate Pinia stores to new backend

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[MIMIR-I-0045]]

## Objective

Update migrated Pinia stores to work with new Tauri commands. Handle ApiResponse wrapper differences and update type definitions to match `mimir-core` models.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [x] Add `unwrapResponse()` utility for ApiResponse handling
- [x] Update useCampaignStore to use new command signatures
- [ ] Update useModuleStore/ModuleService to use new command signatures
- [x] Update useCharacterStore to use new command signatures
- [ ] Update useDocumentStore to use new command signatures
- [x] Update TypeScript types to match mimir-core models
- [ ] Update Vue components to use new field names
- [ ] All stores successfully fetch and mutate data

## Implementation Notes

### ApiResponse Handling

Existing frontend expects `ApiResponse<T>` wrapper:
```typescript
interface ApiResponse<T> {
  success: boolean;
  data?: T;
  error?: string;
}
```

Options:
1. **Adapter approach**: Add utility to unwrap in stores
2. **Backend approach**: Have Tauri commands return ApiResponse format

### Stores to Update
- `useCampaignStore` - 90% reusable, minor signature updates
- `useModuleStore` - align with new module service
- `useCharacterStore` - normalize state shape, update types
- `useDocumentStore` - update for DB-stored content
- `useMapStore` - update for UVTT asset handling
- `useCatalogStore` - update search command signatures

### Dependencies
- Blocked by: [[MIMIR-T-0400]], [[MIMIR-T-0401]], [[MIMIR-T-0402]], [[MIMIR-T-0403]], [[MIMIR-T-0404]] (Tauri commands)

## Status Updates

### Session 2026-01-21 Progress

**Completed:**
- [x] Created `unwrapResponse()` utility in `src/shared/utils/api.ts`
- [x] Updated TypeScript types in `src/types/api.ts`:
  - Changed Campaign, Module, Document IDs from `number` to `string` (UUID)
  - Updated field names to match mimir-core models
  - Removed obsolete fields (status, directory_path, session dates)
  - Added request types matching Tauri commands
- [x] Updated TypeScript types in `src/types/character.ts`:
  - Simplified Character to match mimir-core model
  - Added CharacterInventory type
  - Added request types for PC/NPC creation
  - Added helper functions (isNpc, isPc, abilityModifier)
- [x] Updated `src/shared/utils/dataEvents.ts`:
  - Changed all IDs from `number` to `string`
- [x] Updated `useCampaignStore`:
  - Changed all ID parameters from `number` to `string`
  - Removed obsolete methods (exportCampaign, importCampaign, etc.)
  - Updated command signatures to match backend
- [x] Updated `useCharacterStore`:
  - Simplified to match new Character model
  - Updated command names (list_characters, list_pcs, list_npcs)
  - Added inventory management commands
  - Removed versioning-related methods (not in new backend)

**Remaining Work (discovered type errors):**
1. **ModuleService.ts**: Uses old Module type with numeric IDs and calls commands that don't exist (`list_campaign_modules`, `transition_module_stage`, etc.)
2. **Components using old character fields**: `character_name` → `name`, `current_level` → removed, `race` → `race_name`, `class` → removed
3. **Components using campaign.status**: No longer exists, use `archived_at` instead
4. **Components using campaign.directory_path**: No longer exists
5. **CampaignArchiveExportDialog.vue**: Uses methods that don't exist in store
6. **CampaignArchiveImportDialog.vue**: Uses methods that don't exist in store
7. **Multiple dashboard components**: Use old field names and numeric IDs

**Technical Notes:**
- Backend uses UUID strings for all IDs
- Backend Character model is simpler: no versioning, ability scores directly on Character
- Backend doesn't have: session management, workflow cards, campaign archive export/import
- Frontend components need significant updates to match new data models