---
id: create-module-npcs-and-module
level: task
title: "Create module_npcs and module_items database migrations"
short_code: "MIMIR-T-0290"
created_at: 2026-01-03T14:17:34.310718+00:00
updated_at: 2026-01-03T14:55:22.528739+00:00
parent: 
blocked_by: []
archived: true

tags:
  - "#task"
  - "#feature"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: NULL
---

# Create module_npcs and module_items database migrations

## Objective

Create database tables to cache parsed module frontmatter for NPCs and items, enabling fast UI queries while maintaining the document as source of truth.

## Context

Part of Phase 5 (UI Integration) of the Campaign Authoring Framework. These tables cache data parsed from module document YAML frontmatter, following the existing `module_monsters` pattern.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `module_npcs` migration created with proper schema
- [ ] `module_items` migration created with proper schema
- [ ] Both tables have foreign key to `modules` with CASCADE delete
- [ ] Indices created for common query patterns
- [ ] Migrations run successfully
- [ ] Schema updated in `schema.rs`

## Implementation Notes

### Schema: module_npcs

NPCs are characters (`is_npc = true`), so this table links modules to character records rather than storing name/source like monsters.

```sql
CREATE TABLE module_npcs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    module_id INTEGER NOT NULL REFERENCES modules(id) ON DELETE CASCADE,
    character_id INTEGER NOT NULL REFERENCES characters(id) ON DELETE CASCADE,
    role TEXT,           -- quest_giver, antagonist, ally, informant, etc.
    encounter_tag TEXT,  -- For grouping NPCs by scene/encounter
    notes TEXT,          -- Module-specific notes about this NPC's role
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_module_npcs_module ON module_npcs(module_id);
CREATE INDEX idx_module_npcs_character ON module_npcs(character_id);
CREATE INDEX idx_module_npcs_role ON module_npcs(module_id, role);
```

### Schema: module_items

```sql
CREATE TABLE module_items (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    module_id INTEGER NOT NULL REFERENCES modules(id) ON DELETE CASCADE,
    location TEXT,       -- boss_chamber, hidden_cache, reward, etc.
    name TEXT NOT NULL,
    source TEXT NOT NULL, -- DMG, PHB, campaign, etc.
    quantity INTEGER NOT NULL DEFAULT 1,
    notes TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_module_items_module ON module_items(module_id);
CREATE INDEX idx_module_items_location ON module_items(module_id, location);
```

### Files to Create

- `crates/mimir-dm-core/migrations/034_create_module_npcs/up.sql`
- `crates/mimir-dm-core/migrations/034_create_module_npcs/down.sql`
- `crates/mimir-dm-core/migrations/035_create_module_items/up.sql`
- `crates/mimir-dm-core/migrations/035_create_module_items/down.sql`

### Files to Modify

- `crates/mimir-dm-core/src/schema.rs` - Add table definitions

## Status Updates

*To be added during implementation*