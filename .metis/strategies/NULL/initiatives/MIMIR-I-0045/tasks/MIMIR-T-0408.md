---
id: consolidate-shared-component
level: task
title: "Consolidate shared component library"
short_code: "MIMIR-T-0408"
created_at: 2026-01-21T16:34:58.690978+00:00
updated_at: 2026-01-21T16:34:58.690978+00:00
parent: MIMIR-I-0045
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/todo"


exit_criteria_met: false
strategy_id: NULL
initiative_id: MIMIR-I-0045
---

# Consolidate shared component library

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[MIMIR-I-0045]]

## Objective

Consolidate scattered shared components into a centralized library structure per v0.5 design (T-0363). Currently components are spread across `src/components/`, `src/shared/components/`, and feature directories.

## Acceptance Criteria

- [ ] Create `src/components/common/` for reusable UI components
- [ ] Create `src/components/layout/` for layout components
- [ ] Create `src/components/domain/` for domain-specific cards
- [ ] Move existing shared components to appropriate directories
- [ ] Update all imports throughout codebase
- [ ] Remove duplicate components

## Implementation Notes

### Target Structure

```
src/components/
├── common/           # Reusable UI primitives
│   ├── Button.vue
│   ├── Input.vue
│   ├── Modal.vue (from AppModal)
│   ├── Loading.vue
│   └── Badge.vue
├── layout/           # Layout components
│   ├── PageHeader.vue
│   ├── SplitPane.vue
│   └── EmptyState.vue
├── domain/           # Domain-specific cards
│   ├── CampaignCard.vue
│   ├── ModuleCard.vue
│   ├── CharacterCard.vue
│   └── MonsterCard.vue
└── editor/           # Editor components (keep specialized)
    ├── MarkdownEditor.vue
    └── MapCanvas.vue (DmMapViewer)
```

### Components to Move
- `src/shared/components/ui/*` → `src/components/common/`
- `src/shared/components/layout/*` → `src/components/layout/`
- `src/components/shared/AppModal.vue` → `src/components/common/Modal.vue`

### Components to Keep In Place
- `DmMapViewer.vue` - complex, keep as specialized
- Token/lighting/LOS components - specialized map functionality
- Print dialogs - feature-specific

### Dependencies
- Blocked by: [[MIMIR-T-0399]] (frontend migration)

## Status Updates

*To be added during implementation*