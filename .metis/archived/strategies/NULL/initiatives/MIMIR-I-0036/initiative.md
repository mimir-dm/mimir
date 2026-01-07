---
id: campaign-import-export-system
level: initiative
title: "Campaign Import/Export System"
short_code: "MIMIR-I-0036"
created_at: 2026-01-04T19:11:39.702097+00:00
updated_at: 2026-01-04T19:41:17.708311+00:00
parent: MIMIR-V-0001
blocked_by: []
archived: true

tags:
  - "#initiative"
  - "#phase/completed"


exit_criteria_met: false
estimated_complexity: M
strategy_id: NULL
initiative_id: campaign-import-export-system
---

# Campaign Import/Export System Initiative

## Context

Users want to share their campaign creative work with others. Currently there's no way to export a complete campaign (files, assets, structure) or import one from another user.

## Goals & Non-Goals

**Goals:**
- Enable export of complete campaigns to portable `.mimir-campaign.tar.gz` archives
- Enable import of campaign archives with user control over campaign naming
- Include all binary assets (maps, handouts, images) in exports
- Store catalog references by name+source (not embedded data)

**Non-Goals:**
- Syncing campaigns between users (this is one-time export/import)
- Merging campaigns or partial imports
- Cloud storage integration

## Use Cases

### Use Case 1: Export Campaign
- **Actor**: Campaign creator/DM
- **Scenario**: User finishes designing campaign → opens campaign settings → clicks "Export" → chooses save location → archive created
- **Expected Outcome**: `.mimir-campaign.tar.gz` file with all content and assets

### Use Case 2: Import Campaign  
- **Actor**: New user receiving campaign
- **Scenario**: User gets archive file → clicks "Import Campaign" → selects file → previews contents → edits campaign name if needed → confirms import
- **Expected Outcome**: New campaign created with all files and assets, ready to add player characters

## Architecture

### Archive Format

```
{campaign-slug}.mimir-campaign.tar.gz
├── manifest.json           # Version, metadata, catalog refs
├── campaign.json           # Campaign database record
├── content/                # Markdown files (structure preserved)
│   ├── README.md
│   ├── session_zero/
│   ├── world/
│   ├── modules/
│   ├── sessions/
│   ├── characters/
│   ├── npcs/
│   └── resources/
└── assets/                 # Binary files
    ├── maps/
    ├── handouts/
    └── images/
```

### Components
- **CampaignArchiveService** (mimir-dm-core): Core export/import logic
- **Tauri Commands** (mimir-dm): Frontend API endpoints  
- **Export Dialog** (frontend): UI for triggering export
- **Import Dialog** (frontend): File picker, preview, name editor

## Detailed Design

### Export Flow
1. Query campaign metadata from database
2. Walk campaign directory, collecting all files
3. Scan content for catalog references (monster, spell, item links)
4. Create manifest.json with metadata and references
5. Stream files into tar.gz archive
6. Return path to created file

### Import Flow
1. Extract archive to temp directory
2. Validate manifest.json version and format
3. Parse campaign.json for default name
4. Show preview dialog with file count, assets, catalog refs
5. User confirms/edits campaign name
6. Create campaign in database with new slug
7. Copy content/ to campaign directory
8. Copy assets/ preserving structure

## Alternatives Considered

| Alternative | Rejected Because |
|-------------|------------------|
| ZIP format | tar.gz better for streaming, matches existing 5etools-splitter |
| Embed catalog data | Bloats archive, data may be outdated vs user's catalog |
| Auto-generate name | User should control naming to avoid conflicts |

## Implementation Plan

Tasks to be created under this initiative:
1. Core export service with tar.gz creation
2. Core import service with validation
3. Tauri commands for frontend API
4. Export dialog component
5. Import dialog with preview
6. Wire up to campaigns view