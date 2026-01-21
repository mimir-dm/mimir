---
id: migration-003-item-table-and-types
level: task
title: "Migration 003: Item table and types"
short_code: "MIMIR-T-0372"
created_at: 2026-01-20T02:43:48.373443+00:00
updated_at: 2026-01-20T20:24:07.190113+00:00
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

# Migration 003: Item table and types

## Parent Initiative

[[MIMIR-I-0042]] - v0.5 Catalog Implementation

## Objective

Create the `items` table for weapons, armor, magic items, and mundane equipment. Indexed columns for filtering by type and rarity.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] Migration file: `migrations/003_items/up.sql` and `down.sql`
- [ ] `items` table with indexed columns: `name`, `source`, `type`, `rarity`
- [ ] `data` column for full JSON blob
- [ ] Rust types in `models/catalog/item.rs`
- [ ] DAL functions for CRUD operations

## Implementation Notes

### SQL Schema
```sql
CREATE TABLE items (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    source TEXT NOT NULL REFERENCES catalog_sources(code),
    item_type TEXT,                      -- Single char: R, A, M, etc.
    rarity TEXT,                         -- common, uncommon, rare, etc.
    data TEXT NOT NULL,
    UNIQUE(name, source)
);

CREATE INDEX idx_items_name ON items(name);
CREATE INDEX idx_items_source ON items(source);
CREATE INDEX idx_items_type ON items(item_type);
CREATE INDEX idx_items_rarity ON items(rarity);
```

### Dependencies
- Requires [[MIMIR-T-0370]] (CatalogSource migration)

## Status Updates

### Session 2026-01-20
- Created migration files `migrations/003_items/up.sql` and `down.sql`
- Ran diesel migration, schema.rs auto-updated
- Created `models/catalog/item.rs` with Item, NewItem, ItemFilter types
  - Includes helper methods: `type_name()`, `is_magic()`, `parse_data()`
- Created `dal/catalog/item.rs` with full CRUD operations:
  - insert_item, insert_items
  - get_item, get_item_optional, get_item_by_name
  - list_items, list_items_by_source
  - search_items, search_items_paginated
  - delete_item, delete_items_by_source
  - count_items, count_items_by_source
- All 185 tests passing