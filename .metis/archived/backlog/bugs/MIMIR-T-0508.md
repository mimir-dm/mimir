---
id: import-campaign-option-not-visible
level: task
title: "Import campaign option not visible when no campaigns exist"
short_code: "MIMIR-T-0508"
created_at: 2026-02-01T03:47:23.314695+00:00
updated_at: 2026-02-02T01:13:44.950283+00:00
parent: 
blocked_by: []
archived: true

tags:
  - "#task"
  - "#bug"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: NULL
---

# Import campaign option not visible when no campaigns exist

## Objective

The "Import Campaign" option is not accessible until at least one campaign already exists. New users or users on a fresh install cannot import a campaign archive, which is a critical onboarding path.

## Backlog Item Details

### Type
- [x] Bug - Production issue that needs fixing

### Priority
- [x] P1 - High (important for user experience)

### Impact Assessment
- **Affected Users**: All new users or users with no campaigns
- **Reproduction Steps**: 
  1. Start the app with no campaigns (fresh install or all deleted)
  2. Look for an "Import Campaign" option
  3. No import option is visible — only "Create Campaign" is shown
- **Expected vs Actual**: The import option should always be available regardless of existing campaign count. Currently it only appears once at least one campaign exists.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] "Import Campaign" button/option is visible when zero campaigns exist
- [ ] Import works correctly from the empty-state view
- [ ] Import still works from the existing campaign list view (no regression)

## Implementation Notes

### Technical Approach
The campaign list view likely has an empty state that only shows a "Create Campaign" button. The import option needs to be added to this empty state as well (or the empty state needs to be restructured so the header actions — which include import — are always visible).

## Status Updates

### Fix Applied
- Root cause: In `CampaignSelector.vue`, the empty state (lines 25-30) only showed a "Create your first campaign" link. The "Import Campaign" button was inside the `v-else` block that only renders when `campaigns.length > 0`.
- Fix: Added an "Import Campaign" button to the empty state div, reusing the same `openImportDialog` handler and SVG icon.
- File modified: `crates/mimir/frontend/src/features/campaigns/components/CampaignSelector.vue`