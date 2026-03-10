---
id: module-crud-tests-create-update
level: task
title: "Module CRUD tests — create, update, delete, reorder modules and documents"
short_code: "MIMIR-T-0546"
created_at: 2026-03-10T01:31:31.347504+00:00
updated_at: 2026-03-10T01:31:31.347504+00:00
parent: MIMIR-I-0056
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/todo"


exit_criteria_met: false
initiative_id: MIMIR-I-0056
---

# Module CRUD tests — create, update, delete, reorder modules and documents

**Phase 3** — Campaign & Module Coverage

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[MIMIR-I-0056]]

## Objective

Write Vitest component tests for module and document CRUD operations — creating, updating, deleting, and reordering modules and their documents. Verify that form submissions call the correct invoke commands with proper payloads, and that the UI updates after mutations.

## Acceptance Criteria

- [ ] Create module form submits correct data via `create_module`
- [ ] Update module form populates existing data and saves changes via `update_module`
- [ ] Delete module shows confirmation and calls `delete_module`
- [ ] Create document form submits via `create_document` with correct module/campaign ID
- [ ] Update document saves content changes
- [ ] Delete document shows confirmation and calls `delete_document`
- [ ] Reorder documents calls `reorder_document` with correct position data
- [ ] UI refreshes list after create/update/delete operations
- [ ] Validation errors display for required fields
- [ ] All tests pass in CI

## Key Components

- Module create/edit forms
- Document create/edit forms
- Delete confirmation dialogs
- Reorder drag-and-drop or move controls

## Implementation Notes

Focus on the mutation flow: user fills form → submits → invoke is called with correct args → list refreshes. Use the invoke mock's one-shot and sequence modes to verify the refresh call happens after the mutation.

## Status Updates

*To be added during implementation*