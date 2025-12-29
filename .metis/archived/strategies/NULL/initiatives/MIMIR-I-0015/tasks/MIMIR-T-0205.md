---
id: create-tokens-database-schema-and
level: task
title: "Create tokens database schema and model"
short_code: "MIMIR-T-0205"
created_at: 2025-12-21T22:15:20.906606+00:00
updated_at: 2025-12-21T23:02:22.207653+00:00
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

# Create tokens database schema and model

## Parent Initiative
[[MIMIR-I-0015]] - Visual Display System

## Objective
Create the database schema and Rust models for map tokens, enabling persistent storage of token positions, types, sizes, and links to catalog entities (monsters, characters).

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] Migration creates `tokens` table with all required fields
- [ ] Token model supports all token types (Monster, PC, NPC, Trap, Marker)
- [ ] Token model supports all D&D sizes (Tiny, Small, Medium, Large, Huge, Gargantuan)
- [ ] Optional foreign keys link to monsters and characters tables
- [ ] NewToken and UpdateToken structs for insert/update operations
- [ ] Schema compiles and migration runs successfully

## Implementation Notes

### Database Schema

```sql
CREATE TABLE tokens (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    map_id INTEGER NOT NULL REFERENCES maps(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    token_type TEXT NOT NULL,  -- 'monster', 'pc', 'npc', 'trap', 'marker'
    size TEXT NOT NULL DEFAULT 'medium',  -- 'tiny', 'small', 'medium', 'large', 'huge', 'gargantuan'
    x REAL NOT NULL,  -- Grid position (float for sub-grid positioning)
    y REAL NOT NULL,
    visible_to_players INTEGER NOT NULL DEFAULT 1,
    color TEXT,  -- Fallback color if no image
    image_path TEXT,  -- Custom token image
    monster_id INTEGER REFERENCES monsters(id),  -- Link to catalog monster
    character_id INTEGER REFERENCES characters(id),  -- Link to PC/NPC
    notes TEXT,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_tokens_map_id ON tokens(map_id);
```

### Rust Models

```rust
pub enum TokenType {
    Monster,
    PC,
    NPC,
    Trap,
    Marker,
}

pub enum TokenSize {
    Tiny,      // 0.5 grid squares
    Small,     // 1 grid square
    Medium,    // 1 grid square
    Large,     // 2x2 grid squares
    Huge,      // 3x3 grid squares
    Gargantuan, // 4x4 grid squares
}

pub struct Token { ... }
pub struct NewToken { ... }
pub struct UpdateToken { ... }
```

### Files to Create/Modify
- `crates/mimir-dm-core/migrations/037_create_tokens/up.sql`
- `crates/mimir-dm-core/migrations/037_create_tokens/down.sql`
- `crates/mimir-dm-core/src/models/campaign/tokens.rs`
- `crates/mimir-dm-core/src/models/campaign/mod.rs` (add export)
- `crates/mimir-dm-core/src/schema.rs` (diesel will update)

### Dependencies
- Maps table must exist (done in T-0195)
- Monsters and characters tables for optional FK links