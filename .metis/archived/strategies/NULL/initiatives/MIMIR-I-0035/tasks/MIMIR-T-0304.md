---
id: overview-tab-campaign-summary
level: task
title: "Overview Tab - Campaign Summary"
short_code: "MIMIR-T-0304"
created_at: 2026-01-03T21:05:32.912250+00:00
updated_at: 2026-01-03T21:05:32.912250+00:00
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

# Overview Tab - Campaign Summary

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative

[[MIMIR-I-0035]] - Unified Campaign Dashboard

## Objective

Create the Overview tab showing campaign summary, current stage info, quick stats, and recent activity.

## Acceptance Criteria

## Acceptance Criteria

- [ ] `OverviewTab.vue` component created in `components/dashboard/`
- [ ] Displays current campaign stage with StageHeader component
- [ ] Shows campaign quick stats (modules count, documents count, characters count)
- [ ] Displays recent activity or campaign summary/description
- [ ] Reuses StageTransitionCard for stage progression
- [ ] Route `/campaigns/:id/dashboard/overview` renders this tab

## Implementation Notes

### File to Create
```
frontend/src/features/campaigns/components/dashboard/OverviewTab.vue
```

### Layout Structure
```
┌─────────────────────────────────────────┐
│  Campaign Name / Stage Header           │
├─────────────────────────────────────────┤
│  Quick Stats Cards (3 columns)          │
│  [Modules: X] [Documents: X] [PCs: X]   │
├─────────────────────────────────────────┤
│  Campaign Description / Summary         │
├─────────────────────────────────────────┤
│  Stage Transition Card (if applicable)  │
└─────────────────────────────────────────┘
```

### Components to Reuse
- `StageHeader` - from CampaignBoardView
- `StageTransitionCard` - for stage progression UI
- Quick stat cards pattern from existing dashboard

### Dependencies
- Depends on: T-0303 (Foundation) for routing and dashboard shell
- Reference: `CampaignBoardView.vue` for data loading patterns

## Status Updates

*To be added during implementation*