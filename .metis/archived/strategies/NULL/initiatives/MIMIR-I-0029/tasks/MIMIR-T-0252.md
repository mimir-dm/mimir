---
id: migrate-d-d-content-modals-batch-5
level: task
title: "Migrate D&D content modals (Batch 5)"
short_code: "MIMIR-T-0252"
created_at: 2025-12-29T15:13:21.760594+00:00
updated_at: 2025-12-29T16:04:07.794577+00:00
parent: MIMIR-I-0029
blocked_by: []
archived: true

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: MIMIR-I-0029
---

# Migrate D&D content modals (Batch 5)

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[MIMIR-I-0029]]

## Objective

Migrate remaining D&D content modals to use the AppModal component.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [x] LevelUpDialog.vue migrated to AppModal
- [x] InventoryManager.vue migrated to AppModal (2 modals)
- [x] ModulePlayView.vue inline modal migrated to AppModal
- [x] ReaderView.vue inline modal migrated to AppModal
- [x] ChatSidebar.vue delete modal migrated to AppModal
- [x] SearchView.vue migrated from BaseModal to AppModal
- [x] SourceSearch.vue migrated from BaseModal to AppModal
- [x] BaseModal.vue removed (no longer used)
- [x] All components type-check successfully

## Components Migrated

1. **LevelUpDialog.vue** - Multi-step wizard with progress indicator in custom header
2. **InventoryManager.vue** - Main modal + nested "Add Item" modal with stack-index
3. **ModulePlayView.vue** - Cross-reference modal for D&D content
4. **ReaderView.vue** - Cross-reference modal for D&D content
5. **ChatSidebar.vue** - Delete session confirmation dialog
6. **SearchView.vue** - D&D catalog modal stack with cross-reference handling
7. **SourceSearch.vue** - Content preview modal stack

## Status Updates

- 2025-12-29: Completed all 7 component migrations, removed BaseModal.vue