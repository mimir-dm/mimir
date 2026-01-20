---
id: schema-migration-for-db-only
level: task
title: "Schema Migration for DB-Only Document Storage"
short_code: "MIMIR-T-0351"
created_at: 2026-01-19T21:27:10.096247+00:00
updated_at: 2026-01-19T21:29:30.773008+00:00
parent: MIMIR-I-0040
blocked_by: []
archived: true

tags:
  - "#task"
  - "#phase/active"


exit_criteria_met: false
strategy_id: NULL
initiative_id: MIMIR-I-0040
---

# Schema Migration for DB-Only Document Storage

## Parent Initiative

[[MIMIR-I-0040]] - Database-Only Document Storage

## Objective

Create database migration 046 to add content storage column and remove unnecessary file_path columns.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] Migration 046 created with up.sql and down.sql
- [ ] `documents.content` TEXT column added
- [ ] `documents.file_path` made nullable (still needed for binary files)
- [ ] `character_versions.file_path` column dropped
- [ ] Diesel schema.rs regenerated
- [ ] Models updated to reflect schema changes
- [ ] Migration runs successfully on fresh database
- [ ] Migration runs successfully on existing database

## Implementation Notes

### Schema Changes

```sql
-- up.sql
ALTER TABLE documents ADD COLUMN content TEXT;

-- SQLite doesn't support ALTER COLUMN, so for file_path nullable:
-- Option 1: Leave as-is (TEXT is already nullable in SQLite by default unless NOT NULL specified)
-- Option 2: Recreate table if NOT NULL constraint exists

-- Drop character_versions.file_path
-- SQLite requires table recreation to drop columns
CREATE TABLE character_versions_new (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    character_id INTEGER NOT NULL,
    version_number INTEGER NOT NULL,
    character_data TEXT NOT NULL,
    snapshot_reason TEXT,
    level INTEGER NOT NULL,
    created_at TEXT NOT NULL,
    FOREIGN KEY (character_id) REFERENCES characters(id) ON DELETE CASCADE
);
INSERT INTO character_versions_new SELECT id, character_id, version_number, character_data, snapshot_reason, level, created_at FROM character_versions;
DROP TABLE character_versions;
ALTER TABLE character_versions_new RENAME TO character_versions;
```

### Files to Modify
- `crates/mimir-dm-core/migrations/046_db_only_documents/up.sql`
- `crates/mimir-dm-core/migrations/046_db_only_documents/down.sql`
- `crates/mimir-dm-core/src/schema.rs` (regenerate)
- `crates/mimir-dm-core/src/models/campaign/documents.rs`
- `crates/mimir-dm-core/src/models/character/character_versions.rs` (if exists)

### Dependencies
None - this is the first task in the initiative

## Status Updates

*To be added during implementation*