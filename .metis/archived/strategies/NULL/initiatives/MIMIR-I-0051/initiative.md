---
id: campaign-export-and-import
level: initiative
title: "Campaign Export and Import"
short_code: "MIMIR-I-0051"
created_at: 2026-01-28T03:58:25.076945+00:00
updated_at: 2026-01-28T14:16:28.951557+00:00
parent: MIMIR-V-0001
blocked_by: []
archived: true

tags:
  - "#initiative"
  - "#phase/completed"


exit_criteria_met: false
estimated_complexity: M
strategy_id: NULL
initiative_id: campaign-export-and-import
---

# Campaign Export and Import Initiative

## Context

Users need to share campaigns with others, back up their work, and move campaigns between devices. Currently there's no way to export a complete campaign or import one from another user.

**Reference Implementation**: `mimir-dm-bu/mimir-dm-core/src/services/campaign_archive_service.rs` contains a complete v0.4 implementation. The v0.5 data model has changed significantly but the approach and structure remain valid.

## Goals & Non-Goals

**Goals:**
- Export complete campaigns as portable archive files
- Import campaigns from archive files with proper ID remapping
- Include all campaign data: modules, characters, documents, maps, tokens
- Preview archives before importing
- Provide both UI and MCP tool interfaces

**Non-Goals:**
- Real-time sync between instances
- Partial/incremental export
- Cloud storage integration
- Campaign merging (import is always additive)

## What Gets Exported

| Entity | Data | Assets |
|--------|------|--------|
| Campaign | name, description, sources | - |
| Modules | name, monsters, items, NPC assignments | - |
| Characters | full character data, classes, inventory, spells, proficiencies, features | token images |
| Documents | title, content, type, module association | - |
| Maps | name, dimensions, grid, POIs, fog state, tokens | background image |
| Tokens | position, type, size, vision, character/monster links | - |

## Why This Approach

From the v0.4 implementation:

1. **Tar.gz archive** - Preserves directory structure, good compression, standard format
2. **manifest.json** - Version info, content counts, catalog references for validation
3. **data.json** - Complete structured data with original IDs for reference mapping
4. **Separate content/assets dirs** - Clean separation of text vs binary files
5. **ID remapping on import** - Avoids conflicts, maintains internal references
6. **Catalog references by name+source** - No catalog data exported, just references

## Key Import Behaviors

1. **UUID Regeneration** - All IDs regenerated; mappings track old→new
2. **Reference Resolution** - Character→Module, Token→Map, Token→Character links remapped
3. **Catalog Lookups** - Monster/item references resolved by name+source at import time
4. **Asset Extraction** - Images copied to appropriate campaign directories

## Implementation Plan

1. **Export Service** - Adapt CampaignArchiveService to v0.5 models
2. **Import Service** - Adapt import with v0.5 ID remapping
3. **Tauri Commands** - export_campaign, import_campaign, preview_archive
4. **Frontend UI** - Export options dialog, Import preview dialog
5. **MCP Tools** - export_campaign, import_campaign for Claude integration