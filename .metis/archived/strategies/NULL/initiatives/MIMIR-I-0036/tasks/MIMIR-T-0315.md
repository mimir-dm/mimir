---
id: campaign-archive-tauri-commands
level: task
title: "Campaign Archive Tauri Commands"
short_code: "MIMIR-T-0315"
created_at: 2026-01-04T19:13:28.545757+00:00
updated_at: 2026-01-04T19:40:34.436638+00:00
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

# Campaign Archive Tauri Commands

## Parent Initiative

[[MIMIR-I-0036]] Campaign Import/Export System

## Objective

Create Tauri commands that expose campaign archive functionality to the frontend.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] New `archive.rs` in `mimir-dm/src/commands/campaign/`
- [ ] `export_campaign` command - triggers export with file save dialog
- [ ] `preview_campaign_archive` command - returns archive preview
- [ ] `import_campaign_archive` command - imports with user-specified name
- [ ] Commands registered in `main.rs`
- [ ] Add to `CampaignService.ts` frontend service

## Implementation Notes

### Files to Create/Modify
- `crates/mimir-dm/src/commands/campaign/archive.rs` (NEW)
- `crates/mimir-dm/src/commands/campaign/mod.rs` (add module)
- `crates/mimir-dm/src/main.rs` (register commands)
- `frontend/src/services/CampaignService.ts` (add methods)

### Tauri Commands

```rust
#[tauri::command]
pub async fn export_campaign(
    state: State<'_, AppState>,
    campaign_id: i32,
) -> Result<String, String>

#[tauri::command]
pub async fn preview_campaign_archive(
    archive_path: String,
) -> Result<ArchivePreviewResponse, String>

#[tauri::command]
pub async fn import_campaign_archive(
    state: State<'_, AppState>,
    archive_path: String,
    campaign_name: String,
) -> Result<CampaignResponse, String>
```

### Frontend Service Methods

```typescript
export const CampaignService = {
  // ... existing methods
  exportCampaign: (campaignId: number) => invoke<string>('export_campaign', { campaignId }),
  previewArchive: (archivePath: string) => invoke<ArchivePreview>('preview_campaign_archive', { archivePath }),
  importArchive: (archivePath: string, campaignName: string) => invoke<Campaign>('import_campaign_archive', { archivePath, campaignName }),
}
```

### Dependencies
- Depends on T-0313 and T-0314 (core service)

## Status Updates

*To be added during implementation*