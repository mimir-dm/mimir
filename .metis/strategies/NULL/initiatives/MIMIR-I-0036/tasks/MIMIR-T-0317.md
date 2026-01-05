---
id: campaign-import-dialog-component
level: task
title: "Campaign Import Dialog Component"
short_code: "MIMIR-T-0317"
created_at: 2026-01-04T19:13:28.941343+00:00
updated_at: 2026-01-04T19:41:07.881640+00:00
parent: MIMIR-I-0036
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: MIMIR-I-0036
---

# Campaign Import Dialog Component

## Parent Initiative

[[MIMIR-I-0036]] Campaign Import/Export System

## Objective

Create Vue dialog component for importing campaigns from archive files, with preview and name editing.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] New `CampaignImportDialog.vue` in `frontend/src/components/campaigns/`
- [ ] File picker to select `.mimir-campaign.tar.gz` file
- [ ] Calls preview API and displays archive contents
- [ ] Shows: campaign name, file count, asset count, catalog references
- [ ] Editable campaign name field (defaults to archive's name)
- [ ] Warning if catalog references not in user's database
- [ ] Import button triggers import with user-specified name
- [ ] Shows progress during import
- [ ] Success: navigates to new campaign
- [ ] Error: displays error with option to retry

## Implementation Notes

### Files to Create
- `frontend/src/components/campaigns/CampaignImportDialog.vue`

### User Flow
1. User clicks "Import Campaign" from campaigns list
2. Tauri file open dialog for `.mimir-campaign.tar.gz`
3. Dialog shows preview: name, stats, catalog refs
4. User can edit campaign name
5. User clicks "Import"
6. Progress indicator during import
7. Success: redirect to new campaign dashboard
8. Error: show message, allow retry

### Preview Display
```
Campaign: "Frost Architect"
Files: 47 markdown documents
Assets: 12 files (maps, images)
References: 23 monsters, 15 spells, 8 items
[Warning icon] 3 referenced items not in your catalog
```

### Design
- Use AppModal component
- Two-step flow: file selection → preview/confirm
- Match existing dialog patterns

### Dependencies
- Depends on T-0315 (Tauri commands)

## Status Updates

*To be added during implementation*