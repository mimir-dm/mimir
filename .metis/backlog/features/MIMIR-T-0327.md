---
id: hard-delete-campaign-support
level: task
title: "Hard Delete Campaign Support"
short_code: "MIMIR-T-0327"
created_at: 2026-01-07T09:32:29.857152+00:00
updated_at: 2026-01-07T09:32:29.857152+00:00
parent: 
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/backlog"
  - "#feature"


exit_criteria_met: false
strategy_id: NULL
initiative_id: NULL
---

# Hard Delete Campaign Support

## Objective

Add the ability to permanently delete campaigns from the database, as opposed to only archiving them. Currently users can only archive campaigns, but sometimes need to fully remove test campaigns or campaigns with incorrect paths.

## Backlog Item Details

### Type
- [x] Feature - New functionality or enhancement  

### Priority
- [ ] P1 - High (important for user experience)

### Business Justification
- **User Value**: Users need to clean up test campaigns, campaigns with broken paths, or campaigns they no longer want cluttering their list
- **Business Value**: Better data hygiene and user control over their workspace
- **Effort Estimate**: S

## Acceptance Criteria

- [ ] Add "Delete Campaign" option in campaign context menu (distinct from Archive)
- [ ] Show confirmation dialog warning that this is permanent and will delete all associated data
- [ ] Delete all related records: documents, modules, sessions, characters, maps, tokens, etc.
- [ ] Optionally delete campaign folder from filesystem (with separate confirmation)
- [ ] ~~Add corresponding MCP tool for programmatic deletion~~ (removed - too risky for accidental deletion)

## Implementation Notes

### Technical Approach
1. Add `delete_campaign` method to CampaignService that cascades deletes to all child tables
2. Add Tauri command `delete_campaign` 
3. Add UI confirmation dialog with checkbox for "also delete files from disk"
4. No MCP tool - UI only to prevent accidental deletion

### Dependencies
- Existing CampaignArchiveService can be referenced for table relationships

### Risk Considerations
- Destructive operation - must have clear confirmation UI
- Filesystem deletion should be opt-in and clearly explained

## Status Updates

*To be added during implementation*