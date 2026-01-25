---
id: add-missing-top-level-routes
level: task
title: "Add missing top-level routes"
short_code: "MIMIR-T-0407"
created_at: 2026-01-21T16:34:58.469794+00:00
updated_at: 2026-01-25T01:03:01.681183+00:00
parent: MIMIR-I-0045
blocked_by: []
archived: true

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: MIMIR-I-0045
---

# Add missing top-level routes

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[MIMIR-I-0045]]

## Objective

Add missing top-level routes per v0.5 design (T-0363). Currently documents and maps are only accessible within module context.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] Add `/documents` route with DocumentList view
- [ ] Add `/maps` route with MapList view  
- [ ] Rename `/sources` to `/catalog` for consistency
- [ ] All routes accessible from sidebar navigation
- [ ] Routes work with or without active campaign context

## Implementation Notes

### Routes to Add

```typescript
// New routes
{ path: '/documents', component: DocumentList },
{ path: '/documents/:id', component: DocumentEditor },
{ path: '/maps', component: MapList },
{ path: '/maps/:id', component: MapEditor },

// Rename
{ path: '/catalog', component: CatalogSearch },  // was /sources
```

### Current Routes (keep)
- `/campaigns`, `/campaigns/:id`
- `/modules`, `/modules/:id`
- `/characters`, `/characters/:id`
- `/settings`

### Views to Create/Adapt
- `DocumentList.vue` - List all campaign documents (filter by module optional)
- `MapList.vue` - List all campaign maps (filter by module optional)

### Dependencies
- Blocked by: [[MIMIR-T-0406]] (sidebar navigation)

## Status Updates

*To be added during implementation*