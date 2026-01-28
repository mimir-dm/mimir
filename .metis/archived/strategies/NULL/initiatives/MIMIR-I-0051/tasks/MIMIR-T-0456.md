---
id: export-service-adapt-to-v0-5-models
level: task
title: "Export Service - Adapt to v0.5 Models"
short_code: "MIMIR-T-0456"
created_at: 2026-01-28T04:02:47.361581+00:00
updated_at: 2026-01-28T13:37:31.486470+00:00
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

# Export Service - Adapt to v0.5 Models

## Parent Initiative

[[MIMIR-I-0051]] - Campaign Export and Import

## Objective

Adapt the v0.4 CampaignArchiveService export functionality to work with v0.5 data models. Creates tar.gz archives containing all campaign data.

## Reference

`mimir-dm-bu/mimir-dm-core/src/services/campaign_archive_service.rs`

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [x] Create `ArchiveService` in mimir-core/src/services/archive.rs
- [x] Uses existing v0.5 models directly (no duplicate structs)
- [x] Implement `export_campaign()` that produces `.mimir-campaign.tar.gz`
- [x] Include manifest.json with version, content counts, catalog references
- [x] Include data.json with complete structured data
- [x] Export all campaign assets (maps, tokens, images) for full portability
- [x] Extract catalog references from document content
- [x] Dependencies already existed (tar, flate2, regex)

## Key v0.5 Model Changes

- UUIDs instead of integer IDs
- Characters have classes, inventory, spells, proficiencies, features as separate tables
- Documents simplified (no versioning)
- Maps have POIs, fog_cells

## Status Updates

### 2026-01-28 - Completed

Created `crates/mimir-core/src/services/archive.rs` with:
- `ArchiveService` with `export_campaign()` and `preview_archive()` methods
- Uses existing models directly via `#[serde(flatten)]` for aggregation (no duplicate structs)
- `ArchiveData` includes: Campaign, sources, modules, documents, characters (with classes/inventory/spells/proficiencies/features/feats), maps (with POIs/traps/lights/fog), tokens, module_monsters, module_npcs, and all campaign assets
- Assets are vendored into archive for full portability (maps, tokens, images)
- Extracts catalog references from document content (5etools-style `{@type name|source}`)
- Produces `.mimir-campaign.tar.gz` with manifest.json, data.json, and assets/ directory

Dependencies `tar`, `flate2`, and `regex` already existed in mimir-core.