---
id: integrate-import-export-into
level: task
title: "Integrate Import/Export into Campaigns View"
short_code: "MIMIR-T-0318"
created_at: 2026-01-04T19:13:29.147268+00:00
updated_at: 2026-01-04T19:41:08.056643+00:00
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

# Integrate Import/Export into Campaigns View

## Parent Initiative

[[MIMIR-I-0036]] Campaign Import/Export System

## Objective

Wire up import/export dialogs to the campaigns list view and campaign settings.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] "Import Campaign" button added to CampaignsView header
- [ ] "Export" option added to campaign settings/context menu
- [ ] Clicking "Import Campaign" opens CampaignImportDialog
- [ ] Clicking "Export" opens CampaignExportDialog with campaign context
- [ ] End-to-end flow works: export → import → new campaign visible

## Implementation Notes

### Files to Modify
- `frontend/src/features/campaigns/CampaignsView.vue`
- `frontend/src/features/campaigns/components/CampaignSettingsDialog.vue` (or context menu)

### Import Button Location
Add next to "New Campaign" button in campaigns list header:
```vue
<AppButton @click="showImportDialog = true">
  Import Campaign
</AppButton>
```

### Export Trigger Location
Add to campaign settings menu or context menu:
```vue
<DropdownItem @click="showExportDialog = true">
  Export Campaign
</DropdownItem>
```

### State Management
```typescript
const showImportDialog = ref(false)
const showExportDialog = ref(false)
const exportCampaignId = ref<number | null>(null)
```

### Dependencies
- Depends on T-0316 (export dialog) and T-0317 (import dialog)

## Status Updates

*To be added during implementation*