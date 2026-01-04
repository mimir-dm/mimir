---
id: play-mode-integration
level: task
title: "Play Mode Integration"
short_code: "MIMIR-T-0310"
created_at: 2026-01-03T21:05:45.244862+00:00
updated_at: 2026-01-03T21:05:45.244862+00:00
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

# Play Mode Integration

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative

[[MIMIR-I-0035]] - Unified Campaign Dashboard

## Objective

Integrate the full-screen play mode (ModulePlayView) within the dashboard flow, hiding tabs during play and providing "End Session" to return.

## Acceptance Criteria

- [ ] Route `/campaigns/:id/dashboard/session/:moduleId/play` renders ModulePlayView
- [ ] Dashboard tabs hidden when in play mode (full-screen experience)
- [ ] "End Session" button returns to Session tab
- [ ] Play mode state preserved during session
- [ ] Uses existing ModulePlayView composables (useModuleMonsters, useModuleMaps, useSessionNotes)

## Implementation Notes

### Route Configuration
```typescript
{
  path: 'session/:moduleId/play',
  name: 'dashboard-play',
  component: () => import('@/features/modules/views/ModulePlayView.vue'),
  meta: { hideTabBar: true, fullScreen: true }
}
```

### Behavior
- Dashboard detects `meta.hideTabBar` and hides DashboardTabs component
- ModulePlayView renders in full content area
- "End Session" or close button: `router.push(`/campaigns/${campaignId}/dashboard/session`)`
- Module context loaded from route params

### Layout During Play
```
┌─────────────────────────────────────────────────────────────┐
│  [End Session]                    Module: "Chapter 1"       │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│                   Full Screen Play View                     │
│                   (Map, monsters, notes)                    │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

### Components to Reuse
- `ModulePlayView.vue` - existing play mode view (recently refactored with composables)
- `useModuleMonsters`, `useModuleMaps`, `useSessionNotes` composables

### Complexity
**Medium-High** - Needs proper routing and tab hiding logic

### Dependencies
- Depends on: T-0303 (Foundation), T-0309 (Session Launcher)
- Reference: `ModulePlayView.vue` for play mode implementation

## Status Updates

*To be added during implementation*