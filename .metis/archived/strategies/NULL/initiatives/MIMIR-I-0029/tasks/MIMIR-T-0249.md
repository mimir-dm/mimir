---
id: migrate-standard-modals-to
level: task
title: "Migrate standard modals to AppModal (Batch 2)"
short_code: "MIMIR-T-0249"
created_at: 2025-12-29T15:13:21.005778+00:00
updated_at: 2025-12-29T15:35:59.363786+00:00
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

# Migrate standard modals to AppModal (Batch 2)

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[MIMIR-I-0029]]

## Objective

Migrate standard modals with moderate complexity (Batch 2) to use the AppModal component.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [x] MapUploadModal.vue migrated to AppModal
- [x] MapGridConfigModal.vue migrated to AppModal
- [x] QuickAddTokenModal.vue migrated to AppModal
- [x] MapTokenSetupModal.vue migrated to AppModal
- [x] All components type-check successfully

## Components Migrated

1. **MapUploadModal.vue** - Upload modal with conditional close behavior during upload
2. **MapGridConfigModal.vue** - Dynamic size based on grid type (lg when grid enabled, sm otherwise)
3. **QuickAddTokenModal.vue** - Monster search modal with vision configuration
4. **MapTokenSetupModal.vue** - Large modal (xl) with token placement canvas

## Status Updates

- 2025-12-29: Completed all 4 component migrations, removed duplicate modal CSS from each