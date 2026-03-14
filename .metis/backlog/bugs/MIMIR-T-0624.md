---
id: cannot-delete-maps-from-a-module
level: task
title: "Cannot delete maps from a module after upload"
short_code: "MIMIR-T-0624"
created_at: 2026-03-14T11:29:49.906186+00:00
updated_at: 2026-03-14T12:25:59.278884+00:00
parent: 
blocked_by: []
archived: false

tags:
  - "#task"
  - "#bug"
  - "#phase/active"


exit_criteria_met: false
initiative_id: NULL
---

# Cannot delete maps from a module after upload

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Objective

Once a map is uploaded and associated with a module, there is no way to remove or disassociate it from that module. The `delete_map` MCP tool exists and deletes the map entirely, but users need the ability to remove a map from a module without necessarily deleting it (or at minimum, delete it from the module context).

## Details

- **Type**: Bug
- **Priority**: P2 — Medium
- **Discovered**: 2026-03-14
- **Where**: MCP `delete_map` tool works at campaign level, but no UI or MCP operation allows removing a map from a module after upload
- **Relevant code**: `crates/mimir-mcp/src/tools/map.rs` — `update_map` has a `module_id` param but unclear if setting it to null/empty works to disassociate

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] User can remove a map from a module (either delete or disassociate)
- [ ] MCP tool supports the operation
- [ ] UI reflects the change

## Status Updates

### 2026-03-14

**Root cause**: The `ModuleMapsPanel.vue` component (used on the campaign dashboard's module tab) had no delete button. It only emitted `upload` and `select` events. A separate `ModuleMaps.vue` component existed with full action buttons (place tokens, print, delete) but was never imported anywhere — orphaned code.

**Backend**: Already fully functional — `delete_map` Tauri command and `MapService::delete()` work for both campaign-level and module-associated maps. MCP `update_map` with `module_id: "campaign"` also supports disassociating a map.

**Fix**:
- Added delete button to `ModuleMapsPanel.vue` (shows on hover, trash icon, red on hover)
- Added `delete` emit to the component's emit definitions
- Wired `@delete="confirmDeleteMap"` handler in `ModulesTab.vue`
- Handler shows confirmation dialog, calls `delete_map`, reloads map list
- TypeScript type-checks clean