---
id: migration-map-table-with-lighting
level: task
title: "Migration: Map table with lighting mode"
short_code: "MIMIR-T-0387"
created_at: 2026-01-20T21:49:43.592614+00:00
updated_at: 2026-01-21T01:11:16.210612+00:00
parent: MIMIR-I-0043
blocked_by: []
archived: true

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: MIMIR-I-0043
---

# Migration: Map table with lighting mode

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[MIMIR-I-0043]]

## Objective

Create the Map table for storing map metadata and initial play state configuration. UVTT files remain the source of truth for grid, walls, and lighting geometry. The Map table stores:
- Reference to the UVTT asset
- Lighting mode (bright/dim/dark) for initial play state
- Display name and notes

## Schema

```sql
CREATE TABLE maps (
    id TEXT PRIMARY KEY NOT NULL,
    module_id TEXT NOT NULL REFERENCES modules(id) ON DELETE CASCADE,
    
    -- Display info
    name TEXT NOT NULL,
    description TEXT,
    sort_order INTEGER NOT NULL DEFAULT 0,
    
    -- UVTT asset reference (blob storage)
    uvtt_asset_id TEXT NOT NULL REFERENCES campaign_assets(id),
    
    -- Initial play state (not live state)
    lighting_mode TEXT NOT NULL DEFAULT 'bright' CHECK (lighting_mode IN ('bright', 'dim', 'dark')),
    
    -- Timestamps
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_maps_module ON maps(module_id);
```

## Design Notes

- **UVTT as source of truth**: Grid size, wall geometry, and door positions come from the UVTT file at runtime
- **No FogArea table**: `lighting_mode` controls initial visibility (bright = full visibility, dim = partial, dark = no visibility)
- **Initial state only**: Maps define starting conditions; live play state (revealed areas, token positions) is managed in-memory during sessions
- **Sort order**: Allows DM to sequence maps within a module

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] Migration creates maps table with all columns
- [ ] Foreign key to modules with CASCADE delete works
- [ ] Foreign key to campaign_assets for UVTT reference
- [ ] lighting_mode CHECK constraint enforces valid values
- [ ] Diesel models generated with proper derives
- [ ] DAL operations: create, read, update, delete, list_by_module
- [ ] Unit tests pass for all DAL operations

## Implementation Notes

### Dependencies
- T-0381 (Module table must exist)
- T-0383 (CampaignAsset table must exist for UVTT storage)

## Status Updates **[REQUIRED]**

### Session 2026-01-20
- Created migration 015_maps with up.sql and down.sql
- Maps can belong to campaigns (world maps) OR modules (dungeon maps) per user feedback
- campaign_id always required, module_id optional (same pattern as documents)
- Added LightingMode enum (Bright, Dim, Dark) for initial play state
- Created Map, NewMap, UpdateMap models with builder patterns
- Created DAL functions including:
  - list_campaign_maps, list_campaign_level_maps, list_module_maps
  - get_next_campaign_sort_order, get_next_module_sort_order
- All 486 tests passing

Files created:
- migrations/015_maps/up.sql
- migrations/015_maps/down.sql
- src/models/campaign/map.rs
- src/dal/campaign/map.rs