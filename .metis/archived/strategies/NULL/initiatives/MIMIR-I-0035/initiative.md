---
id: unified-campaign-dashboard
level: initiative
title: "Unified Campaign Dashboard"
short_code: "MIMIR-I-0035"
created_at: 2026-01-03T21:03:54.020132+00:00
updated_at: 2026-01-04T03:54:26.919532+00:00
parent: MIMIR-V-0001
blocked_by: []
archived: true

tags:
  - "#initiative"
  - "#phase/completed"


exit_criteria_met: false
estimated_complexity: XL
strategy_id: NULL
initiative_id: unified-campaign-dashboard
---

# Unified Campaign Dashboard Initiative

*This template includes sections for various types of initiatives. Delete sections that don't apply to your specific use case.*

## Context

The current navigation architecture requires excessive drilling to manage campaigns:
- **Current Flow:** Home → Campaigns → Campaign Board → Modules → Module Board → Play
- **Target Flow:** Home → Campaign Dashboard (with tabs: Overview, Modules, Characters, World, Session)

This fragmented navigation creates cognitive overhead and makes campaign management feel disjointed.

## Goals & Non-Goals

**Goals:**
- Flatten campaign navigation with unified tabbed dashboard
- Replace multi-route campaign/module architecture with single cohesive view
- Module detail as full-width takeover (replaces tab content area)
- Full tab structure: Overview, Modules, Characters, World, Session

**Non-Goals:**
- Changing the underlying data models
- Modifying the play mode functionality (only integrating it)
- Mobile-first responsive design (desktop primary)

## Architecture

### File Structure
```
frontend/src/features/campaigns/
├── views/
│   └── CampaignDashboardView.vue       # New main dashboard
├── components/dashboard/
│   ├── DashboardTabs.vue               # Tab navigation
│   ├── OverviewTab.vue                 # Stage info, quick stats
│   ├── ModulesTab.vue                  # Module list
│   ├── ModuleDetailPane.vue            # Full-width module takeover
│   ├── CharactersTab.vue               # Campaign characters
│   ├── WorldTab.vue                    # Documents (reuses DocumentSidebar)
│   └── SessionTab.vue                  # Play mode launcher
└── composables/
    └── useDashboardState.ts            # Tab/selection state
```

### Routes
```typescript
/campaigns/:id/dashboard           -> CampaignDashboardView
/campaigns/:id/dashboard/overview  -> OverviewTab
/campaigns/:id/dashboard/modules   -> ModulesTab
/campaigns/:id/dashboard/modules/:moduleId -> ModuleDetailPane
/campaigns/:id/dashboard/characters -> CharactersTab
/campaigns/:id/dashboard/world     -> WorldTab
/campaigns/:id/dashboard/session   -> SessionTab
/campaigns/:id/dashboard/session/:moduleId/play -> ModulePlayView
```

## Detailed Design

### Key Architecture Decisions
1. **Nested routes** for URL state and browser history
2. **Full-width takeover** via CSS (tabs remain visible, content expands)
3. **Component reuse** - wrap existing components, don't copy
4. **localStorage** for tab state per campaign
5. **Phased rollout** - new routes first, redirects second, cleanup last

### Critical Files to Reuse
| File | Purpose |
|------|---------|
| `app/router/index.ts` | New routes + redirects |
| `views/CampaignBoardView.vue` | Reference for stage/document logic |
| `views/ModuleBoardView.vue` | Reference for module context |
| `views/ModulePlayView.vue` | Reuse for Session play mode |
| `stores/sharedContext.ts` | Existing state patterns |

## Alternatives Considered

1. **Incremental migration** - Rejected: Would require maintaining two navigation systems simultaneously
2. **Modal for module detail** - Rejected: Too cramped for full module management
3. **Slide-over panel** - Rejected: Module management needs full attention, not partial view

## Implementation Plan

See child tasks for detailed phase breakdown:
- Phase 1: Foundation (routing, tabs, state)
- Phase 2: Overview Tab
- Phase 3: World Tab
- Phase 4: Modules Tab - List
- Phase 5: Modules Tab - Detail (highest complexity)
- Phase 6: Characters Tab
- Phase 7: Session Launcher
- Phase 8: Play Mode Integration
- Phase 9: Route Migration
- Phase 10: Cleanup