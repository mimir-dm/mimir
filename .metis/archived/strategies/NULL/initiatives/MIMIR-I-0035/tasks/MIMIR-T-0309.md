---
id: session-tab-play-launcher
level: task
title: "Session Tab - Play Launcher"
short_code: "MIMIR-T-0309"
created_at: 2026-01-03T21:05:45.014745+00:00
updated_at: 2026-01-03T21:05:45.014745+00:00
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

# Session Tab - Play Launcher

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative

[[MIMIR-I-0035]] - Unified Campaign Dashboard

## Objective

Create the Session tab as a launch pad for selecting a module and starting a play session.

## Acceptance Criteria

## Acceptance Criteria

- [ ] `SessionTab.vue` component created in `components/dashboard/`
- [ ] Shows list of modules that are ready for play (active stage)
- [ ] "Start Session" button for each module
- [ ] Clicking start navigates to play mode route
- [ ] Route `/campaigns/:id/dashboard/session` renders this tab

## Implementation Notes

### File to Create
```
frontend/src/features/campaigns/components/dashboard/SessionTab.vue
```

### Layout Structure
```
┌─────────────────────────────────────────┐
│  Ready to Play                          │
├─────────────────────────────────────────┤
│  ┌─────────────────────────────────┐    │
│  │ Module: "Chapter 1"             │    │
│  │ Stage: Active | Maps: 3         │    │
│  │              [▶ Start Session]  │    │
│  └─────────────────────────────────┘    │
│  ┌─────────────────────────────────┐    │
│  │ Module: "Chapter 2"             │    │
│  │ Stage: Active | Maps: 2         │    │
│  │              [▶ Start Session]  │    │
│  └─────────────────────────────────┘    │
├─────────────────────────────────────────┤
│  In Preparation                         │
│  (modules not yet ready - dimmed)       │
└─────────────────────────────────────────┘
```

### Behavior
- Filter modules by stage (show Active first, then others dimmed)
- "Start Session" click: `router.push(`/campaigns/${campaignId}/dashboard/session/${moduleId}/play`)`
- This triggers T-0310 (Play Mode Integration)

### Complexity
**Low** - Simple module list with action buttons

### Dependencies
- Depends on: T-0303 (Foundation) for routing
- Reference: Module stage data from existing stores

## Status Updates

*To be added during implementation*