---
id: create-maps-database-schema-and
level: task
title: "Create maps database schema and migration"
short_code: "MIMIR-T-0195"
created_at: 2025-12-20T22:24:17.171510+00:00
updated_at: 2025-12-21T01:42:43.122270+00:00
parent: MIMIR-I-0015
blocked_by: []
archived: true

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: MIMIR-I-0015
---

# Create maps database schema and migration

## Parent Initiative
[[MIMIR-I-0015]] - Visual Display System

## Objective

Create the database schema for storing map data, including campaign-level and module-level maps with grid configuration.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] Migration creates `maps` table with all required columns
- [ ] Foreign keys to campaigns and modules (optional) work correctly
- [ ] Grid configuration fields support square, hex, and no-grid options
- [ ] Migration runs successfully on fresh and existing databases

## Implementation Notes

### Schema Design

```sql
CREATE TABLE maps (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    campaign_id INTEGER NOT NULL REFERENCES campaigns(id) ON DELETE CASCADE,
    module_id INTEGER REFERENCES modules(id) ON DELETE CASCADE,  -- NULL = campaign-level
    name TEXT NOT NULL,
    image_path TEXT NOT NULL,
    width_px INTEGER NOT NULL,
    height_px INTEGER NOT NULL,
    grid_type TEXT NOT NULL DEFAULT 'none',  -- 'square', 'hex', 'none'
    grid_size_px INTEGER,           -- pixels per grid cell
    grid_offset_x INTEGER DEFAULT 0,
    grid_offset_y INTEGER DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_maps_campaign ON maps(campaign_id);
CREATE INDEX idx_maps_module ON maps(module_id);
```

### File Location
`crates/mimir-dm-core/migrations/XXX_create_maps/`

### Dependencies
None - this is the foundation task