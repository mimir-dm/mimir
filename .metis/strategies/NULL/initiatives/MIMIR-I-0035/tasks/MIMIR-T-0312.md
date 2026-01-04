---
id: cleanup-deprecated-views
level: task
title: "Cleanup Deprecated Views"
short_code: "MIMIR-T-0312"
created_at: 2026-01-03T21:05:59.950083+00:00
updated_at: 2026-01-04T02:14:33.575684+00:00
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

# Cleanup Deprecated Views

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative

[[MIMIR-I-0035]] - Unified Campaign Dashboard

## Objective

Remove deprecated view files that are no longer used after the dashboard migration is complete.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `CampaignBoardView.vue` removed (replaced by dashboard)
- [ ] `ModuleBoardView.vue` removed (replaced by ModuleDetailPane)
- [ ] `ModuleListView.vue` removed (replaced by ModulesTab)
- [ ] No orphaned imports or references to removed files
- [ ] App builds and runs without errors
- [ ] All routes still function correctly

## Implementation Notes

### Files to Remove
```
frontend/src/features/campaigns/views/
├── CampaignBoardView.vue    # DELETE - replaced by CampaignDashboardView
└── ModuleListView.vue       # DELETE - replaced by ModulesTab

frontend/src/features/modules/views/
└── ModuleBoardView.vue      # DELETE - replaced by ModuleDetailPane
```

### Pre-Deletion Checklist
- [ ] Verify no components import from these files
- [ ] Verify no router references to these files
- [ ] Run full app test to confirm functionality
- [ ] Git commit the dashboard working state first

### Post-Deletion Verification
- [ ] `npm run build` succeeds
- [ ] `npm run type-check` passes
- [ ] Manual testing of all dashboard flows

### Complexity
**Low** - Straightforward file deletion after verification

### Risk Considerations
- Delete only after thorough testing of new dashboard
- Keep old files in git history for reference
- Consider keeping for one release cycle if needed

### Dependencies
- Depends on: All previous tasks (T-0303 through T-0311)
- Must be last task - only after full dashboard is verified working

## Status Updates

*To be added during implementation*