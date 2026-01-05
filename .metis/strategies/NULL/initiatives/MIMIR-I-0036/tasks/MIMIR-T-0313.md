---
id: campaign-export-service
level: task
title: "Campaign Export Service"
short_code: "MIMIR-T-0313"
created_at: 2026-01-04T19:13:28.184459+00:00
updated_at: 2026-01-04T19:29:57.491963+00:00
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

# Campaign Export Service

## Parent Initiative

[[MIMIR-I-0036]] Campaign Import/Export System

## Objective

Create `CampaignArchiveService` in mimir-dm-core with export functionality that packages a campaign into a portable `.mimir-campaign.tar.gz` archive.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] New `campaign_archive_service.rs` in `mimir-dm-core/src/services/`
- [ ] `export_campaign(conn, campaign_id, output_path)` function implemented
- [ ] Archive contains `manifest.json` with version, metadata, catalog references
- [ ] Archive contains `campaign.json` with database record
- [ ] Archive contains `content/` with all markdown files preserving structure
- [ ] Archive contains `assets/` with all binary files (maps, images, handouts)
- [ ] Catalog references extracted from content (monster, spell, item links)
- [ ] Uses `tar` and `flate2` crates (already in workspace)

## Implementation Notes

### Files to Create/Modify
- `crates/mimir-dm-core/src/services/campaign_archive_service.rs` (NEW)
- `crates/mimir-dm-core/src/services/mod.rs` (add module)

### Technical Approach
1. Query campaign from database using `CampaignService`
2. Get campaign directory path from `paths::campaigns_dir()`
3. Walk directory tree collecting files
4. Parse markdown files for catalog reference patterns (e.g., `{@monster Adult White Dragon|MM}`)
5. Build manifest.json with metadata
6. Create tar.gz using streaming API
7. Return path to created archive

### Archive Structure
```
{slug}.mimir-campaign.tar.gz
├── manifest.json
├── campaign.json
├── content/
│   └── {all .md files}
└── assets/
    └── {all binary files}
```

## Status Updates

*To be added during implementation*