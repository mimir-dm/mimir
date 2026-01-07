---
id: unified-campaign-dashboard
level: task
title: "Unified Campaign Dashboard Architecture"
short_code: "MIMIR-T-0287"
created_at: 2026-01-03T13:57:42.696438+00:00
updated_at: 2026-01-04T14:18:03.809994+00:00
parent: MIMIR-I-0034
blocked_by: []
archived: true

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: MIMIR-I-0034
---

# Unified Campaign Dashboard Architecture

## Overview

Flatten navigation by creating a unified tabbed dashboard that replaces the current multi-route campaign/module architecture.

**Current Flow:** Home → Campaigns → Campaign Board → Modules → Module Board → Play
**Target Flow:** Home → Campaign Dashboard (with tabs: Overview, Modules, Characters, World, Session)

## User Decisions
- **Approach**: Full redesign (build new, then swap routes)
- **Module detail**: Full-width takeover (replaces tab content area)
- **MVP scope**: Full tab structure (all 5 tabs)

---

## File Structure

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

---

## Implementation Phases

### Phase 1: Foundation (2-3 days)
Create dashboard shell with tab navigation.

**Files:**
- `composables/useDashboardState.ts` - State management
- `components/dashboard/DashboardTabs.vue` - Tab bar component
- `views/CampaignDashboardView.vue` - Main view with router-view
- `app/router/index.ts` - Add new routes

**Routes to add:**
```
/campaigns/:id/dashboard           -> CampaignDashboardView
/campaigns/:id/dashboard/overview  -> OverviewTab
/campaigns/:id/dashboard/modules   -> ModulesTab
/campaigns/:id/dashboard/modules/:moduleId -> ModuleDetailPane
/campaigns/:id/dashboard/characters -> CharactersTab
/campaigns/:id/dashboard/world     -> WorldTab
/campaigns/:id/dashboard/session   -> SessionTab
/campaigns/:id/dashboard/session/:moduleId/play -> ModulePlayView (full-screen)
```

### Phase 2: Overview Tab (1-2 days)
**Reuses:** StageHeader, StageTransitionCard
**Shows:** Current stage, quick stats, recent activity, campaign summary

### Phase 3: World Tab (1 day)
**Reuses:** DocumentSidebar, DocumentEditor
**Layout:** Two-panel (sidebar + editor)

### Phase 4: Modules Tab - List (1-2 days)
**Reuses:** ModulesTable, CreateModuleModal
**Behavior:** Click module → show ModuleDetailPane

### Phase 5: Modules Tab - Detail (2-3 days) ⚠️ Highest complexity
**Reuses:** ModuleStageLandingView, ModuleDocumentSidebar, DocumentEditor
**Behavior:** Full-width takeover, back button returns to list

### Phase 6: Characters Tab (1-2 days)
**Reuses:** Character components from features/characters/
**Shows:** PCs/NPCs filtered by campaign, inline creation

### Phase 7: Session Launcher (1 day)
**Shows:** Ready/active modules, "Start Session" button

### Phase 8: Play Mode Integration (2 days)
**Reuses:** ModulePlayView
**Behavior:** Hides tabs, "End Session" returns to dashboard

### Phase 9: Route Migration (1 day)
```
/campaigns/:id/board → /campaigns/:id/dashboard/world
/campaigns/:id/modules → /campaigns/:id/dashboard/modules
/modules/:id/board → /campaigns/:campaignId/dashboard/modules/:id
```

### Phase 10: Cleanup (1 day)
Remove: CampaignBoardView.vue, ModuleBoardView.vue, ModuleListView.vue

---

## Key Architecture Decisions

1. **Nested routes** for URL state and browser history
2. **Full-width takeover** via CSS (tabs remain visible, content expands)
3. **Component reuse** - wrap existing components, don't copy
4. **localStorage** for tab state per campaign
5. **Phased rollout** - new routes first, redirects second, cleanup last

---

## Estimated Timeline: 13-18 days

| Phase | Days | Complexity |
|-------|------|------------|
| 1. Foundation | 2-3 | Medium |
| 2. Overview | 1-2 | Low-Medium |
| 3. World | 1 | Low |
| 4. Modules List | 1-2 | Medium |
| 5. Module Detail | 2-3 | **High** |
| 6. Characters | 1-2 | Medium |
| 7. Session Launcher | 1 | Low |
| 8. Play Mode | 2 | Medium-High |
| 9. Migration | 1 | Low |
| 10. Cleanup | 1 | Low |

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[Parent Initiative]]

## Objective **[REQUIRED]**

{Clear statement of what this task accomplishes}

## Backlog Item Details **[CONDITIONAL: Backlog Item]**

{Delete this section when task is assigned to an initiative}

### Type
- [ ] Bug - Production issue that needs fixing
- [ ] Feature - New functionality or enhancement  
- [ ] Tech Debt - Code improvement or refactoring
- [ ] Chore - Maintenance or setup work

### Priority
- [ ] P0 - Critical (blocks users/revenue)
- [ ] P1 - High (important for user experience)
- [ ] P2 - Medium (nice to have)
- [ ] P3 - Low (when time permits)

### Impact Assessment **[CONDITIONAL: Bug]**
- **Affected Users**: {Number/percentage of users affected}
- **Reproduction Steps**: 
  1. {Step 1}
  2. {Step 2}
  3. {Step 3}
- **Expected vs Actual**: {What should happen vs what happens}

### Business Justification **[CONDITIONAL: Feature]**
- **User Value**: {Why users need this}
- **Business Value**: {Impact on metrics/revenue}
- **Effort Estimate**: {Rough size - S/M/L/XL}

### Technical Debt Impact **[CONDITIONAL: Tech Debt]**
- **Current Problems**: {What's difficult/slow/buggy now}
- **Benefits of Fixing**: {What improves after refactoring}
- **Risk Assessment**: {Risks of not addressing this}

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria **[REQUIRED]**

- [ ] {Specific, testable requirement 1}
- [ ] {Specific, testable requirement 2}
- [ ] {Specific, testable requirement 3}

## Test Cases **[CONDITIONAL: Testing Task]**

{Delete unless this is a testing task}

### Test Case 1: {Test Case Name}
- **Test ID**: TC-001
- **Preconditions**: {What must be true before testing}
- **Steps**: 
  1. {Step 1}
  2. {Step 2}
  3. {Step 3}
- **Expected Results**: {What should happen}
- **Actual Results**: {To be filled during execution}
- **Status**: {Pass/Fail/Blocked}

### Test Case 2: {Test Case Name}
- **Test ID**: TC-002
- **Preconditions**: {What must be true before testing}
- **Steps**: 
  1. {Step 1}
  2. {Step 2}
- **Expected Results**: {What should happen}
- **Actual Results**: {To be filled during execution}
- **Status**: {Pass/Fail/Blocked}

## Documentation Sections **[CONDITIONAL: Documentation Task]**

{Delete unless this is a documentation task}

### User Guide Content
- **Feature Description**: {What this feature does and why it's useful}
- **Prerequisites**: {What users need before using this feature}
- **Step-by-Step Instructions**:
  1. {Step 1 with screenshots/examples}
  2. {Step 2 with screenshots/examples}
  3. {Step 3 with screenshots/examples}

### Troubleshooting Guide
- **Common Issue 1**: {Problem description and solution}
- **Common Issue 2**: {Problem description and solution}
- **Error Messages**: {List of error messages and what they mean}

### API Documentation **[CONDITIONAL: API Documentation]**
- **Endpoint**: {API endpoint description}
- **Parameters**: {Required and optional parameters}
- **Example Request**: {Code example}
- **Example Response**: {Expected response format}

## Implementation Notes **[CONDITIONAL: Technical Task]**

{Keep for technical tasks, delete for non-technical. Technical details, approach, or important considerations}

### Technical Approach
{How this will be implemented}

### Dependencies
{Other tasks or systems this depends on}

### Risk Considerations
{Technical risks and mitigation strategies}

## Status Updates **[REQUIRED]**

*To be added during implementation*