---
id: modules-tab-detail-pane
level: task
title: "Modules Tab - Detail Pane"
short_code: "MIMIR-T-0307"
created_at: 2026-01-03T21:05:44.612498+00:00
updated_at: 2026-01-03T21:05:44.612498+00:00
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

# Modules Tab - Detail Pane

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative

[[MIMIR-I-0035]] - Unified Campaign Dashboard

## Objective

Create the full-width module detail pane that shows when a module is selected, providing complete module management within the dashboard context.

## Acceptance Criteria

## Acceptance Criteria

- [ ] `ModuleDetailPane.vue` component created in `components/dashboard/`
- [ ] Full-width takeover when route matches `/campaigns/:id/dashboard/modules/:moduleId`
- [ ] Back button returns to module list (navigates to `/campaigns/:id/dashboard/modules`)
- [ ] Reuses ModuleStageLandingView functionality (stage header, documents, encounters)
- [ ] Module document sidebar and editor work within pane
- [ ] Stage transitions work within pane context
- [ ] "Play Module" button navigates to Session tab

## Implementation Notes

### File to Create
```
frontend/src/features/campaigns/components/dashboard/ModuleDetailPane.vue
```

### Layout Structure
```
┌─────────────────────────────────────────────────────────────┐
│  [← Back to Modules]                    Module: "Chapter 1" │
├─────────────────────────────────────────────────────────────┤
│  ┌─────────────┬───────────────────────────────────────────┐│
│  │  Document   │                                           ││
│  │  Sidebar    │     Module Stage Landing                  ││
│  │             │     (encounters, documents, maps)         ││
│  │             │                                           ││
│  │             │     OR Document Editor (when selected)    ││
│  │             │                                           ││
│  └─────────────┴───────────────────────────────────────────┘│
└─────────────────────────────────────────────────────────────┘
```

### Components to Reuse
- `ModuleStageLandingView.vue` - core module management UI
- `ModuleDocumentSidebar.vue` - document tree for module
- `DocumentEditor.vue` - TipTap editor
- `StageHeader.vue` - module stage display

### Behavior
- Renders as nested route child of ModulesTab
- CSS: Expand to full width of content area (tabs remain visible)
- Module context loaded from route param `:moduleId`
- Back button: `router.push(`/campaigns/${campaignId}/dashboard/modules`)`

### Complexity
**HIGH** - Most complex task in initiative. Requires careful integration of existing module views into dashboard context.

### Risk Considerations
- Module context management must work correctly
- State isolation between module detail and other tabs
- Document editing state preservation on navigation

### Dependencies
- Depends on: T-0303 (Foundation), T-0306 (Modules List)
- Reference: `ModuleBoardView.vue` for module context setup

## Status Updates

*To be added during implementation*