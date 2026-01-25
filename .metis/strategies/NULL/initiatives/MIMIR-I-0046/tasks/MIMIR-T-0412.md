---
id: token-table-schema-migration
level: task
title: "Token table schema & migration"
short_code: "MIMIR-T-0412"
created_at: 2026-01-25T02:44:09.127352+00:00
updated_at: 2026-01-25T16:04:28.577312+00:00
parent: MIMIR-I-0046
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: MIMIR-I-0046
---

# Token table schema & migration

## Parent Initiative

[[MIMIR-I-0046]] - Map & Token VTT System

## Objective

Create the database schema and sqlx migration for the tokens table that stores map token starting positions.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] Migration file created in `crates/mimir-core/migrations/`
- [ ] Tokens table created with all required fields
- [ ] Foreign keys to maps, monsters, and characters tables
- [ ] Migration runs successfully on fresh and existing databases

## Implementation Notes

### Schema

```sql
CREATE TABLE tokens (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    map_id INTEGER NOT NULL REFERENCES maps(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    token_type TEXT NOT NULL,  -- 'monster', 'pc', 'npc', 'trap', 'marker'
    size TEXT NOT NULL DEFAULT 'medium',  -- 'tiny', 'small', 'medium', 'large', 'huge', 'gargantuan'
    x REAL NOT NULL,  -- pixel coordinates
    y REAL NOT NULL,
    visible_to_players INTEGER NOT NULL DEFAULT 1,  -- boolean
    color TEXT,  -- hex color fallback
    monster_id INTEGER REFERENCES monsters(id),
    character_id INTEGER REFERENCES characters(id),
    vision_type TEXT NOT NULL DEFAULT 'normal',  -- 'normal', 'darkvision', 'blindsight', 'tremorsense', 'truesight'
    vision_range_ft INTEGER,
    notes TEXT,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_tokens_map_id ON tokens(map_id);
```

### Files to Create/Modify

- `crates/mimir-core/migrations/XXX_tokens/up.sql`
- `crates/mimir-core/migrations/XXX_tokens/down.sql`

### Dependencies

- Maps table must exist (already does)
- Monsters table must exist (already does)
- Characters table must exist (already does)

## Status Updates

*To be added during implementation*