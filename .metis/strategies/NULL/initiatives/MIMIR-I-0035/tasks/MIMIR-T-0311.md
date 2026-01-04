---
id: route-migration-and-redirects
level: task
title: "Route Migration and Redirects"
short_code: "MIMIR-T-0311"
created_at: 2026-01-03T21:05:59.790248+00:00
updated_at: 2026-01-03T21:05:59.790248+00:00
parent: MIMIR-I-0035
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/todo"


exit_criteria_met: false
strategy_id: NULL
initiative_id: MIMIR-I-0035
---

# Route Migration and Redirects

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative

[[MIMIR-I-0035]] - Unified Campaign Dashboard

## Objective

Add redirects from old routes to new dashboard routes, ensuring existing bookmarks and links continue to work.

## Acceptance Criteria

- [ ] Old `/campaigns/:id/board` redirects to `/campaigns/:id/dashboard/world`
- [ ] Old `/campaigns/:id/modules` redirects to `/campaigns/:id/dashboard/modules`
- [ ] Old `/modules/:id/board` redirects to `/campaigns/:campaignId/dashboard/modules/:id`
- [ ] All internal links updated to use new routes
- [ ] No broken navigation in app

## Implementation Notes

### Redirects to Add (router/index.ts)
```typescript
// Legacy route redirects
{
  path: '/campaigns/:id/board',
  redirect: to => `/campaigns/${to.params.id}/dashboard/world`
},
{
  path: '/campaigns/:id/modules',
  redirect: to => `/campaigns/${to.params.id}/dashboard/modules`
},
{
  path: '/modules/:id/board',
  redirect: to => {
    // Need to look up campaign ID from module
    // Or change to campaign-scoped route
    return `/campaigns/${campaignId}/dashboard/modules/${to.params.id}`
  }
}
```

### Files to Update
- `app/router/index.ts` - Add redirect routes
- `HomeView.vue` - Update campaign links
- `AppSidebar.vue` - Update navigation links
- Any component using `router.push()` to old routes

### Testing Checklist
- [ ] Click campaign from home → lands on dashboard
- [ ] Old bookmarked URLs still work
- [ ] Back/forward browser navigation works
- [ ] Deep links work (e.g., direct to module detail)

### Complexity
**Low** - Straightforward route configuration

### Dependencies
- Depends on: All previous tasks (T-0303 through T-0310)

## Status Updates

*To be added during implementation*