---
id: campaign-export-dialog-component
level: task
title: "Campaign Export Dialog Component"
short_code: "MIMIR-T-0316"
created_at: 2026-01-04T19:13:28.740801+00:00
updated_at: 2026-01-04T19:41:07.714419+00:00
parent: MIMIR-I-0036
blocked_by: []
archived: true

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: MIMIR-I-0036
---

# Campaign Export Dialog Component

## Parent Initiative

[[MIMIR-I-0036]] Campaign Import/Export System

## Objective

Create Vue dialog component for exporting campaigns to archive files.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] New `CampaignExportDialog.vue` in `frontend/src/components/campaigns/`
- [ ] Uses AppModal for consistent dialog styling
- [ ] Shows campaign name and export confirmation
- [ ] Triggers Tauri file save dialog for destination
- [ ] Shows progress/loading state during export
- [ ] Displays success message with file path
- [ ] Handles errors gracefully

## Implementation Notes

### Files to Create
- `frontend/src/components/campaigns/CampaignExportDialog.vue`

### Component Props
```typescript
interface Props {
  campaignId: number
  campaignName: string
}
```

### User Flow
1. User clicks "Export" from campaign settings menu
2. Dialog opens showing campaign name
3. User clicks "Export" button
4. Tauri save dialog opens for file location
5. Progress indicator shows during export
6. Success: show path to created file
7. Error: show error message with retry option

### Design
- Use existing AppModal component
- Match styling of other campaign dialogs (e.g., CampaignSettingsDialog)
- Include campaign icon and name in header

### Dependencies
- Depends on T-0315 (Tauri commands)

## Status Updates

*To be added during implementation*