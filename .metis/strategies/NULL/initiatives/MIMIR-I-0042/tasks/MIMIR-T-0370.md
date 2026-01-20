---
id: migration-001-catalogsource-table
level: task
title: "Migration 001: CatalogSource table and types"
short_code: "MIMIR-T-0370"
created_at: 2026-01-20T02:43:35.694614+00:00
updated_at: 2026-01-20T19:55:32.722321+00:00
parent: MIMIR-I-0042
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: MIMIR-I-0042
---

# Migration 001: CatalogSource table and types

## Parent Initiative

[[MIMIR-I-0042]] - v0.5 Catalog Implementation

## Objective

Create the first Diesel migration with the `catalog_sources` table. This table tracks which source books are imported and enabled in the catalog. All other catalog entities reference this table via `source` column.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [x] Migration file: `migrations/001_catalog_sources/up.sql` and `down.sql`
- [x] `catalog_sources` table created with columns: `code`, `name`, `enabled`, `imported_at`
- [x] Diesel schema generated (`diesel print-schema`)
- [x] Rust types: `CatalogSource`, `NewCatalogSource`, `UpdateCatalogSource` in `models/catalog/source.rs`
- [x] DAL functions: `insert_source`, `get_source`, `list_sources`, `set_enabled` (plus 7 more)
- [x] Unit tests for DAL operations (11 tests)
- [x] Migration runs successfully

## Implementation Notes

### SQL Schema
```sql
CREATE TABLE catalog_sources (
    code TEXT PRIMARY KEY NOT NULL,     -- e.g., "PHB", "MM", "XGE"
    name TEXT NOT NULL,                  -- e.g., "Player's Handbook"
    enabled INTEGER NOT NULL DEFAULT 1,  -- SQLite boolean
    imported_at TEXT NOT NULL            -- ISO 8601 timestamp
);

CREATE INDEX idx_catalog_sources_enabled ON catalog_sources(enabled);
```

### Rust Types
```rust
#[derive(Queryable, Identifiable)]
#[diesel(table_name = catalog_sources)]
#[diesel(primary_key(code))]
pub struct CatalogSource {
    pub code: String,
    pub name: String,
    pub enabled: bool,
    pub imported_at: String,
}

#[derive(Insertable)]
#[diesel(table_name = catalog_sources)]
pub struct NewCatalogSource<'a> {
    pub code: &'a str,
    pub name: &'a str,
    pub enabled: bool,
    pub imported_at: &'a str,
}
```

### Dependencies
- Requires [[MIMIR-T-0368]] (mimir-core crate structure)

## Status Updates

**2026-01-20**: Completed implementation:
- Migration files created: `up.sql` and `down.sql`
- Schema generated: `src/schema.rs` with `catalog_sources` table
- Models: `CatalogSource`, `NewCatalogSource`, `UpdateCatalogSource` in `models/catalog/source.rs`
- DAL functions in `dal/catalog/source.rs`:
  - `insert_source`, `insert_sources`
  - `get_source`, `get_source_optional`
  - `list_sources`, `list_enabled_sources`
  - `set_enabled`, `delete_source`
  - `source_exists`, `count_sources`, `count_enabled_sources`
- 11 unit tests for DAL operations (all passing)

159 total tests passing for mimir-core.