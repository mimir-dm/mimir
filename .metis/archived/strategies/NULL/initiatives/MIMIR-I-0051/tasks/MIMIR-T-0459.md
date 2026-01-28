---
id: frontend-export-import-dialogs
level: task
title: "Frontend Export/Import Dialogs"
short_code: "MIMIR-T-0459"
created_at: 2026-01-28T04:02:49.048810+00:00
updated_at: 2026-01-28T14:09:44.274345+00:00
parent: MIMIR-I-0051
blocked_by: []
archived: true

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: MIMIR-I-0051
---

# Frontend Export/Import Dialogs

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[MIMIR-I-0051]]

## Objective

Create Vue dialogs for exporting and importing campaigns in the frontend.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `CampaignExportDialog.vue` - export options and file save
- [ ] `CampaignImportDialog.vue` - file selection and preview
- [ ] Use Tauri file dialog APIs for native file picker
- [ ] Show export progress indicator
- [ ] Display archive preview before import (name, counts, version)
- [ ] Allow renaming campaign on import
- [ ] Add Export/Import buttons to campaign list or dashboard

## Export Dialog

- Campaign name display
- Content summary (modules, characters, documents, maps)
- Estimated size display
- Export button triggers file save dialog

## Import Dialog

- File picker for .mimir-campaign.tar.gz
- Preview shows: campaign name, content counts, Mimir version, created date
- Editable campaign name field
- Import button with confirmation

## Dependencies

- MIMIR-T-0458 (Tauri Commands)

## Status Updates

### Completed
- Updated `CampaignArchiveExportDialog.vue` to use correct response type (`archive_path`, `size_bytes`)
- Updated `CampaignArchiveImportDialog.vue`:
  - Fixed `ArchivePreview` type to match backend (`counts` object with all entity counts)
  - Updated preview stats to show modules, documents, characters, maps
  - Simplified import function call (removed unused `campaignsDirectory`)
  - Removed unused imports
- Updated `campaigns.ts` store:
  - Implemented real `exportCampaign` calling `export_campaign` command
  - Implemented real `importCampaign` calling `import_campaign` command
  - Implemented real `previewArchive` calling `preview_archive` command
  - Added proper type imports (`ArchiveCounts`, `ArchivePreview`)
- Added archive types to `types/api.ts`:
  - `ArchiveCounts` interface
  - `CatalogReference` interface
  - `ArchivePreview` interface
- All type checks pass