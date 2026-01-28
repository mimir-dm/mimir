---
id: consolidate-shared-components
level: task
title: "Consolidate Shared Components"
short_code: "MIMIR-T-0451"
created_at: 2026-01-28T03:54:04.736931+00:00
updated_at: 2026-01-28T04:13:28.906774+00:00
parent: MIMIR-I-0049
blocked_by: []
archived: true

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: MIMIR-I-0049
---

# Consolidate Shared Components

## Parent Initiative

[[MIMIR-I-0049]] - Frontend Organizational Debt Cleanup

## Objective

Consolidate scattered modal/dialog components into a consistent location and reorganize shared components to follow clear placement patterns.

## Current State

Modal components are spread across 5+ locations:
- `src/components/` - BookManagementModal, CampaignManagementModal, CreateDocumentModal
- `src/components/print/` - 8 print-related dialogs
- `src/components/campaigns/` - 3 campaign dialogs
- `src/features/campaigns/components/dashboard/` - AddCharacterModal
- `src/features/characters/components/levelup/` - LevelUpDialog

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] All shared modals consolidated into `src/components/dialogs/`
- [ ] Feature-specific modals remain in their feature directories
- [ ] Clear decision documented: shared (2+ features) vs feature-specific
- [ ] All imports updated to new locations
- [ ] No broken imports (app builds successfully)

## Implementation Notes

### Files to Move

**To `src/components/dialogs/`:**
- `src/components/BookManagementModal.vue`
- `src/components/CampaignManagementModal.vue`
- `src/components/CreateDocumentModal.vue`

**Keep in place (feature-specific):**
- `src/features/campaigns/components/dashboard/AddCharacterModal.vue`
- `src/features/characters/components/levelup/LevelUpDialog.vue`

**Evaluate print components:**
- Consider if `src/components/print/` should become `src/components/dialogs/print/`

### Approach

1. Create `src/components/dialogs/` directory
2. Move shared modals one at a time
3. Update all imports after each move
4. Verify build passes after each change

## Status Updates

### Session 1 - Completed
- Created `src/components/dialogs/` directory
- Moved 3 modals from root `src/components/`:
  - BookManagementModal.vue
  - CampaignManagementModal.vue
  - CreateDocumentModal.vue
- Updated imports in 4 files:
  - src/views/SettingsView.vue
  - src/features/modules/components/ModuleUserDocuments.vue
  - src/features/campaigns/components/dashboard/ModulesTab.vue
  - src/features/campaigns/components/DocumentSidebar.vue (also fixed relative imports)
- Fixed broken relative import in BookManagementModal.vue (`../types/book` → `@/types/book`)
- Print components (`src/components/print/`) kept as-is - already well-organized
- Build passes