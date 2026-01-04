---
id: modules-tab-list-view
level: task
title: "Modules Tab - List View"
short_code: "MIMIR-T-0306"
created_at: 2026-01-03T21:05:33.260671+00:00
updated_at: 2026-01-03T21:05:33.260671+00:00
parent: MIMIR-I-0035
blocked_by: []
archived: true

tags:
  - "#task"
  - "#phase/todo"


exit_criteria_met: false
strategy_id: NULL
initiative_id: MIMIR-I-0035
---

# Modules Tab - List View

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative

[[MIMIR-I-0035]] - Unified Campaign Dashboard

## Objective

Create the Modules tab list view showing all campaign modules with selection handling that triggers the detail pane.

## Acceptance Criteria

## Acceptance Criteria

- [ ] `ModulesTab.vue` component created in `components/dashboard/`
- [ ] Displays modules using ModulesTable component
- [ ] Create Module button opens CreateModuleModal
- [ ] Clicking a module row triggers navigation to module detail route
- [ ] Shows module stage/status indicators
- [ ] Route `/campaigns/:id/dashboard/modules` renders this tab

## Implementation Notes

### File to Create
```
frontend/src/features/campaigns/components/dashboard/ModulesTab.vue
```

### Layout Structure
```
┌─────────────────────────────────────────┐
│  [+ Create Module]          [Filters]   │
├─────────────────────────────────────────┤
│  Module Table                           │
│  ┌──────────────────────────────────┐   │
│  │ Name    │ Stage   │ Status │ ... │   │
│  ├─────────┼─────────┼────────┼─────│   │
│  │ Mod 1   │ Active  │ ●      │     │   │
│  │ Mod 2   │ Design  │ ○      │     │   │
│  └──────────────────────────────────┘   │
└─────────────────────────────────────────┘
```

### Components to Reuse
- `ModulesTable.vue` - from StageLandingView
- `CreateModuleModal.vue` - existing modal for module creation

### Behavior
- Row click: `router.push(`/campaigns/${campaignId}/dashboard/modules/${moduleId}`)`
- This triggers T-0307 (ModuleDetailPane) to show

### Complexity
**Medium** - Reusing components but wiring up selection behavior

### Dependencies
- Depends on: T-0303 (Foundation) for routing
- Reference: `StageLandingView.vue` for ModulesTable usage

## Status Updates

*To be added during implementation*