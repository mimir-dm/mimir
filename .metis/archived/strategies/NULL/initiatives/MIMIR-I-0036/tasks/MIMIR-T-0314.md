---
id: campaign-import-service
level: task
title: "Campaign Import Service"
short_code: "MIMIR-T-0314"
created_at: 2026-01-04T19:13:28.360441+00:00
updated_at: 2026-01-04T19:30:06.653970+00:00
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

# Campaign Import Service

## Parent Initiative

[[MIMIR-I-0036]] Campaign Import/Export System

## Objective

Add import functionality to `CampaignArchiveService` that extracts a `.mimir-campaign.tar.gz` archive, validates it, and creates a new campaign.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `preview_archive(archive_path)` function returns archive metadata without importing
- [ ] `import_campaign(conn, archive_path, target_name)` function creates campaign from archive
- [ ] Validates manifest.json version and format before import
- [ ] Creates campaign with user-specified name (generates new slug)
- [ ] Copies content files to new campaign directory
- [ ] Copies asset files preserving structure
- [ ] Returns created Campaign record

## Implementation Notes

### Files to Modify
- `crates/mimir-dm-core/src/services/campaign_archive_service.rs`

### Technical Approach
1. Extract archive to temp directory
2. Parse and validate manifest.json (check version, format)
3. Parse campaign.json for default metadata
4. For preview: return ArchivePreview struct with file counts, name, catalog refs
5. For import:
   - Create campaign record with new name/slug via CampaignService
   - Copy content/ directory to campaign path
   - Copy assets/ directory preserving structure
   - Clean up temp directory
6. Handle errors gracefully (corrupted archive, invalid manifest)

### Dependencies
- Depends on T-0313 (export service creates the service file)

### ArchivePreview Struct
```rust
pub struct ArchivePreview {
    pub campaign_name: String,
    pub file_count: usize,
    pub asset_count: usize,
    pub catalog_references: Vec<CatalogReference>,
    pub mimir_version: String,
}
```

## Status Updates

*To be added during implementation*