---
id: dashboard-foundation-routes-tabs
level: task
title: "Dashboard Foundation - Routes, Tabs, State"
short_code: "MIMIR-T-0303"
created_at: 2026-01-03T21:05:32.730831+00:00
updated_at: 2026-01-03T21:18:24.045333+00:00
parent: MIMIR-I-0035
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: MIMIR-I-0035
---

# Dashboard Foundation - Routes, Tabs, State

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative

[[MIMIR-I-0035]] - Unified Campaign Dashboard

## Objective

Create the foundation for the unified campaign dashboard: routing structure, tab navigation component, and state management composable.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] New routes added for `/campaigns/:id/dashboard` with nested tab routes
- [ ] `useDashboardState.ts` composable manages active tab, selected module, localStorage persistence
- [ ] `DashboardTabs.vue` component renders tab navigation with proper styling
- [ ] `CampaignDashboardView.vue` shell view with tabs and `<router-view>` for tab content
- [ ] Tab navigation updates URL and persists across page refresh
- [ ] Clicking campaign from home navigates to dashboard instead of old board view

## Implementation Notes

### Files to Create
```
frontend/src/features/campaigns/
├── views/
│   └── CampaignDashboardView.vue
├── components/dashboard/
│   └── DashboardTabs.vue
└── composables/
    └── useDashboardState.ts
```

### Routes to Add (router/index.ts)
```typescript
/campaigns/:id/dashboard           -> CampaignDashboardView
/campaigns/:id/dashboard/overview  -> (placeholder for Phase 2)
/campaigns/:id/dashboard/modules   -> (placeholder for Phase 4)
/campaigns/:id/dashboard/characters -> (placeholder for Phase 6)
/campaigns/:id/dashboard/world     -> (placeholder for Phase 3)
/campaigns/:id/dashboard/session   -> (placeholder for Phase 7)
```

### useDashboardState Composable
- `activeTab` - current tab name (persisted to localStorage per campaign)
- `selectedModuleId` - for module detail pane
- `isModuleDetailOpen` - controls full-width takeover
- Functions: `setTab()`, `selectModule()`, `closeModuleDetail()`

### DashboardTabs Component
- Tab items: Overview, Modules, Characters, World, Session
- Active state from route or composable
- Consistent with existing design system (reference AppSidebar styling)

### Dependencies
- Reference `app/router/index.ts` for existing route patterns
- Reference `stores/sharedContext.ts` for state patterns
- Reference `CampaignBoardView.vue` for campaign data loading

## Status Updates

*To be added during implementation*