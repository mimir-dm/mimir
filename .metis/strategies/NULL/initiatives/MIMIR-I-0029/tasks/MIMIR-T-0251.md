---
id: migrate-complex-modals-to-appmodal
level: task
title: "Migrate complex modals to AppModal (Batch 4)"
short_code: "MIMIR-T-0251"
created_at: 2025-12-29T15:13:21.292904+00:00
updated_at: 2025-12-29T15:53:20.389919+00:00
parent: MIMIR-I-0029
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: MIMIR-I-0029
---

# Migrate complex modals to AppModal (Batch 4)

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[MIMIR-I-0029]]

## Objective

Migrate complex modals (Batch 4) to use the AppModal component.

## Acceptance Criteria

## Acceptance Criteria

- [x] PdfPreviewModal.vue migrated to AppModal
- [x] CharacterCreationWizard.vue migrated to AppModal
- [x] All components type-check successfully

## Components Migrated

1. **PdfPreviewModal.vue** - PDF preview with custom header (size badge) and action buttons
2. **CharacterCreationWizard.vue** - Multi-step character creation wizard with progress indicator

## Status Updates

- 2025-12-29: Completed all 2 component migrations, removed ~300 lines of duplicate CSS