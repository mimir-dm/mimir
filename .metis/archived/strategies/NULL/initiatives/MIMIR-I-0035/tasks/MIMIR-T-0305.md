---
id: world-tab-document-management
level: task
title: "World Tab - Document Management"
short_code: "MIMIR-T-0305"
created_at: 2026-01-03T21:05:33.079480+00:00
updated_at: 2026-01-03T21:05:33.079480+00:00
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

# World Tab - Document Management

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative

[[MIMIR-I-0035]] - Unified Campaign Dashboard

## Objective

Create the World tab for campaign-level document management, reusing existing DocumentSidebar and DocumentEditor components.

## Acceptance Criteria

## Acceptance Criteria

- [ ] `WorldTab.vue` component created in `components/dashboard/`
- [ ] Two-panel layout: DocumentSidebar on left, DocumentEditor on right
- [ ] Documents scoped to campaign (not module-specific)
- [ ] Create/edit/delete documents works within tab
- [ ] Route `/campaigns/:id/dashboard/world` renders this tab

## Implementation Notes

### File to Create
```
frontend/src/features/campaigns/components/dashboard/WorldTab.vue
```

### Layout Structure
```
┌─────────────────┬───────────────────────────────┐
│  Document       │                               │
│  Sidebar        │     Document Editor           │
│                 │                               │
│  - Folder Tree  │     (Markdown/Rich Text)      │
│  - Quick Create │                               │
│                 │                               │
└─────────────────┴───────────────────────────────┘
```

### Components to Reuse
- `DocumentSidebar.vue` - existing document tree/list
- `DocumentEditor.vue` - existing TipTap editor wrapper
- May need to adapt for campaign-level context vs module-level

### Complexity
**Low** - Mostly composition of existing components

### Dependencies
- Depends on: T-0303 (Foundation) for routing and dashboard shell
- Reference: `CampaignBoardView.vue` for document context setup

## Status Updates

*To be added during implementation*