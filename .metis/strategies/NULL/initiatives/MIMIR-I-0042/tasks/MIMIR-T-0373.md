---
id: migration-004-spell-table-and-types
level: task
title: "Migration 004: Spell table and types"
short_code: "MIMIR-T-0373"
created_at: 2026-01-20T02:43:48.783821+00:00
updated_at: 2026-01-20T02:43:48.783821+00:00
parent: MIMIR-I-0042
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/todo"


exit_criteria_met: false
strategy_id: NULL
initiative_id: MIMIR-I-0042
---

# Migration 004: Spell table and types

## Parent Initiative

[[MIMIR-I-0042]] - v0.5 Catalog Implementation

## Objective

Create the `spells` table with indexed columns for level, school, ritual, and concentration filtering.



## Acceptance Criteria

- [ ] Migration file: `migrations/004_spells/up.sql` and `down.sql`
- [ ] `spells` table with indexed columns: `name`, `source`, `level`, `school`, `ritual`, `concentration`
- [ ] `data` column for full JSON blob
- [ ] Rust types in `models/catalog/spell.rs`
- [ ] DAL functions for CRUD operations

## Implementation Notes

### SQL Schema
```sql
CREATE TABLE spells (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    source TEXT NOT NULL REFERENCES catalog_sources(code),
    level INTEGER NOT NULL,              -- 0 = cantrip, 1-9 = spell level
    school TEXT,                         -- Single char: C, A, V, etc.
    ritual INTEGER NOT NULL DEFAULT 0,   -- Boolean
    concentration INTEGER NOT NULL DEFAULT 0, -- Boolean
    data TEXT NOT NULL,
    UNIQUE(name, source)
);

CREATE INDEX idx_spells_name ON spells(name);
CREATE INDEX idx_spells_source ON spells(source);
CREATE INDEX idx_spells_level ON spells(level);
CREATE INDEX idx_spells_school ON spells(school);
CREATE INDEX idx_spells_ritual ON spells(ritual);
CREATE INDEX idx_spells_concentration ON spells(concentration);
```

### Field Extraction
- `concentration` ← `data.duration[].concentration` (boolean in duration object)
- `ritual` ← `data.meta.ritual` (boolean)

### Dependencies
- Requires [[MIMIR-T-0370]] (CatalogSource migration)

## Status Updates

*To be added during implementation*