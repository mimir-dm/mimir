---
id: migrate-management-modals-to
level: task
title: "Migrate management modals to AppModal (Batch 3)"
short_code: "MIMIR-T-0250"
created_at: 2025-12-29T15:13:21.145335+00:00
updated_at: 2025-12-29T15:43:52.767525+00:00
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

# Migrate management modals to AppModal (Batch 3)

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[MIMIR-I-0029]]

## Objective

Migrate management modals (Batch 3) to use the AppModal component.

## Acceptance Criteria

## Acceptance Criteria

- [x] BookManagementModal.vue migrated to AppModal
- [x] CampaignManagementModal.vue migrated to AppModal
- [x] PlayerManager.vue migrated to AppModal
- [x] All components type-check successfully

## Components Migrated

1. **BookManagementModal.vue** - Book import modal with nested delete confirmation
2. **CampaignManagementModal.vue** - Campaign list with tabs and nested delete confirmation
3. **PlayerManager.vue** - Player CRUD with add/edit and delete dialogs (3 modals total)

## Status Updates

- 2025-12-29: Completed all 3 component migrations with 8 total modals migrated