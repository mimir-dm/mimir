---
id: migration-002-monster-table-with
level: task
title: "Migration 002: Monster table with token_image_path"
short_code: "MIMIR-T-0371"
created_at: 2026-01-20T02:43:47.936935+00:00
updated_at: 2026-01-20T20:20:11.592071+00:00
parent: MIMIR-I-0042
blocked_by: []
archived: true

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: MIMIR-I-0042
---

# Migration 002: Monster table with token_image_path

## Parent Initiative

[[MIMIR-I-0042]] - v0.5 Catalog Implementation

## Objective

Create the `monsters` table with indexed columns for filtering and a JSON `data` blob for full 5etools monster data. Includes `token_image_path` for referencing token images.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [x] Migration file: `migrations/002_monsters/up.sql` and `down.sql`
- [x] `monsters` table with indexed columns: `name`, `source`, `cr`, `creature_type`, `size`
- [x] `data` column for full JSON blob
- [x] `token_image_path` column (nullable)
- [x] Foreign key to `catalog_sources(code)`
- [x] Rust types: `Monster`, `NewMonster`, `MonsterFilter` in `models/catalog/monster.rs`
- [x] DAL functions: `insert_monster`, `get_monster`, `list_monsters`, `search_monsters` (plus 11 more)
- [x] Migration runs successfully

## Implementation Notes

### SQL Schema
```sql
CREATE TABLE monsters (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    source TEXT NOT NULL REFERENCES catalog_sources(code),
    cr TEXT,                             -- Challenge rating (string: "1/4", "1", "10")
    creature_type TEXT,                  -- Extracted from type.type
    size TEXT,                           -- Extracted from size[0]
    token_image_path TEXT,               -- Path to token image file
    data TEXT NOT NULL,                  -- Full 5etools JSON
    UNIQUE(name, source)
);

CREATE INDEX idx_monsters_name ON monsters(name);
CREATE INDEX idx_monsters_source ON monsters(source);
CREATE INDEX idx_monsters_cr ON monsters(cr);
CREATE INDEX idx_monsters_creature_type ON monsters(creature_type);
CREATE INDEX idx_monsters_size ON monsters(size);
```

### Field Extraction
- `creature_type` ← `data.type.type` (nested object)
- `size` ← `data.size[0]` (first element of array)
- `cr` ← `data.cr` (can be string or number)

### Dependencies
- Requires [[MIMIR-T-0370]] (CatalogSource migration)
- Requires [[MIMIR-T-0367]] (typify-generated Monster type for JSON deserialization)

## Status Updates

**2026-01-20**: Completed implementation:
- Migration files: `up.sql` and `down.sql` for `monsters` table
- Schema: Indexed columns (name, source, cr, creature_type, size), JSON data blob, token_image_path
- Models: `Monster`, `NewMonster`, `MonsterFilter` in `models/catalog/monster.rs`
- DAL functions (15 total):
  - `insert_monster`, `insert_monsters`
  - `get_monster`, `get_monster_optional`, `get_monster_by_name`
  - `list_monsters`, `list_monsters_by_source`
  - `search_monsters`, `search_monsters_paginated`
  - `delete_monster`, `delete_monsters_by_source`
  - `count_monsters`, `count_monsters_by_source`
  - `set_token_image_path`
- 10 unit tests for DAL operations

174 total tests passing for mimir-core.