---
id: campaign-source-rules-configuration
level: task
title: "Campaign Source/Rules Configuration"
short_code: "MIMIR-T-0449"
created_at: 2026-01-28T02:06:46.058705+00:00
updated_at: 2026-01-28T02:53:52.476983+00:00
parent: 
blocked_by: []
archived: false

tags:
  - "#task"
  - "#feature"
  - "#phase/active"


exit_criteria_met: false
strategy_id: NULL
initiative_id: NULL
---

# Campaign Source/Rules Configuration

Allow DMs to configure which rulebooks/sources are allowed for a campaign. This controls which versions of spells, classes, races, items, etc. are available.

## Objective

Enable campaign-level source book configuration so that catalog lookups (spells, races, classes, items, etc.) can be filtered to only show content from allowed sources.

## Backlog Item Details

### Type
- [x] Feature - New functionality or enhancement  

### Priority
- [x] P1 - High (important for user experience)

### Business Justification
- **User Value**: Campaigns often restrict which books are allowed (e.g., "PHB 2024 only" or "Core + Xanathar's"). Without this, players see duplicate/conflicting content from multiple sources.
- **Business Value**: Essential for proper D&D 5e campaign management
- **Effort Estimate**: M

## Acceptance Criteria

- [x] Campaign settings include a "Sources" or "Allowed Books" configuration
- [x] UI to select/deselect available source books from imported catalog
- [x] Campaign source settings are persisted to database
- [x] Catalog queries can be filtered by campaign's allowed sources
- [x] Monster, item, trap searches respect campaign source filtering
- [x] Default behavior when no sources configured (allow all)

## Implementation Notes

### Technical Approach
- Add `campaign_sources` table (campaign_id, source_code) - **Already exists**
- Add UI in campaign settings to manage allowed sources - **Done**
- Modify catalog service queries to accept optional source filter - **Future work**
- Frontend passes campaign sources when loading spell lists, etc. - **Future work**

### Dependencies
- Existing catalog import system (source codes like PHB, XPHB, XGE, etc.)
- Campaign settings infrastructure

## Status Updates

### 2026-01-27: Core implementation complete
- Database table `campaign_sources` already existed from migration 009
- DAL functions in `campaign_source.rs` already existed with full CRUD
- Added Tauri commands: `list_campaign_sources`, `add_campaign_source`, `remove_campaign_source`, `set_campaign_sources`
- Created `CampaignSourcesModal.vue` with:
  - Checkbox list of all imported sources
  - Quick actions: Select All, Select None, Core Only
  - Persists selections to database
- Added "Sources" button to campaign dashboard header

### 2026-01-27: Catalog filtering by campaign sources
- Added `currentCampaignSources` to campaign store (Pinia)
- Sources are loaded automatically when a campaign is set as current
- Added `refreshCampaignSources()` to update store after editing sources

**Composables updated to filter by campaign sources:**
- `useMonsters.ts` - Monster searches filter by campaign sources
- `useItems.ts` - Item searches filter by campaign sources
- `useTraps.ts` - Trap searches filter by campaign sources
- `useCatalogSearch.ts` - Generic catalog search applies campaign source filtering

**UI components updated:**
- `QuickAddTokenModal.vue` - Monster search uses campaign sources
- `TokenPalette.vue` - Monster search uses campaign sources
- `CampaignDashboardView.vue` - Refreshes store sources after editing

**Behavior:**
- If campaign has sources configured: only show content from those sources
- If no sources configured (empty): show all content (default)
- Sources are automatically applied to all catalog searches in the campaign context