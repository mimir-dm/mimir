---
id: frontend-createhomebrewservice
level: task
title: "Frontend createHomebrewService factory tests"
short_code: "MIMIR-T-0529"
created_at: 2026-03-08T22:48:31.777346+00:00
updated_at: 2026-03-09T01:27:25.212158+00:00
parent: MIMIR-I-0055
blocked_by: []
archived: true

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: MIMIR-I-0055
---

# Frontend createHomebrewService factory tests

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[MIMIR-I-0055]]

## Objective
Create a test file for the `createHomebrewService` factory that covers the shared CRUD pattern used by all three homebrew entity types (monsters, spells, items).

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria
- [x] Test file at `crates/mimir/frontend/src/services/__tests__/createHomebrewService.test.ts`
- [x] Tests for list, get, create, update, delete operations
- [x] Verifies correct Tauri command names (e.g. `list_homebrew_monsters`, `create_homebrew_monster`)
- [x] Verifies dataEvents are emitted on create/update/delete
- [x] Verifies error handling when API returns `success: false`
- [x] All 17 tests pass (pre-existing failures in other test files are unrelated)

## Implementation Notes
- Use `vi.mock('@tauri-apps/api/core')` pattern from DocumentService.test.ts
- Test the factory directly with `commandSuffix: 'monster'` — covers all 3 entity types since they share the same code
- Mock invoke to return `{ success: true, data: ... }` or `{ success: false, error: '...' }`
- File: `crates/mimir/frontend/src/services/__tests__/createHomebrewService.test.ts`

## Status Updates
- 2026-03-08: Created test file with 17 tests. All pass.
  - list: success, empty array, error response, default error message
  - get: success, error response
  - create: success, emits created event, error without emitting
  - update: success, emits updated event, error response
  - delete: success, emits deleted event with id payload, error response
  - command naming: verifies item suffix → list_homebrew_items, spell suffix → list_homebrew_spells
  - Note: 6 pre-existing test file failures in DocumentService, boardConfigService, ModuleService — unrelated to this change