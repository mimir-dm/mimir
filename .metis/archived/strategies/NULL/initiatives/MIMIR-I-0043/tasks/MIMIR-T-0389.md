---
id: migration-modulemonster-modulenpc
level: task
title: "Migration: ModuleMonster, ModuleNpc tables"
short_code: "MIMIR-T-0389"
created_at: 2026-01-20T21:49:44.515408+00:00
updated_at: 2026-01-21T01:16:04.674329+00:00
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

# Migration: ModuleMonster, ModuleNpc tables

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[MIMIR-I-0043]]

## Objective

Create module-scoped entity tables:
- **ModuleMonster**: Links catalog monsters to modules with optional customizations (name override, notes)
- **ModuleNpc**: Custom NPCs created by the DM for the module

These entities can then be placed on maps via TokenPlacement.

## Schema

```sql
-- Module monsters: catalog monster instances with optional customizations
CREATE TABLE module_monsters (
    id TEXT PRIMARY KEY NOT NULL,
    module_id TEXT NOT NULL REFERENCES modules(id) ON DELETE CASCADE,
    
    -- Reference to catalog
    catalog_monster_id TEXT NOT NULL,  -- FK to catalog_monsters
    
    -- Customizations (NULL = use catalog value)
    display_name TEXT,  -- Override name (e.g., "Goblin Chief" instead of "Goblin")
    notes TEXT,         -- DM notes for this instance
    
    -- Quantity for encounters
    quantity INTEGER NOT NULL DEFAULT 1,
    
    -- Timestamps
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_module_monsters_module ON module_monsters(module_id);
CREATE INDEX idx_module_monsters_catalog ON module_monsters(catalog_monster_id);

-- Module NPCs: custom characters created by the DM
CREATE TABLE module_npcs (
    id TEXT PRIMARY KEY NOT NULL,
    module_id TEXT NOT NULL REFERENCES modules(id) ON DELETE CASCADE,
    
    -- Core info
    name TEXT NOT NULL,
    role TEXT,  -- e.g., "Quest Giver", "Merchant", "Villain"
    description TEXT,
    
    -- Appearance
    appearance TEXT,
    
    -- Personality & motivation
    personality TEXT,
    motivation TEXT,
    secrets TEXT,  -- DM-only info
    
    -- Stats (optional - not all NPCs need stat blocks)
    stat_block TEXT,  -- JSON blob if needed, or reference to catalog monster
    
    -- Token image (optional)
    token_asset_id TEXT REFERENCES campaign_assets(id),
    
    -- Timestamps
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_module_npcs_module ON module_npcs(module_id);
```

## Design Notes

- **ModuleMonster vs Catalog**: ModuleMonster is an *instance* referencing catalog data. The catalog_monster_id points to the read-only catalog. Customizations override catalog fields at display time.
- **ModuleNpc**: Fully custom entity - no catalog reference. DM writes all fields.
- **stat_block**: JSON for NPCs that need combat stats. Many NPCs (shopkeepers, quest givers) won't need this.
- **token_asset_id**: Custom token image from CampaignAsset blob storage
- **quantity**: For encounter building - "3 goblins" is one ModuleMonster with quantity=3

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] Migration creates both tables with all columns
- [ ] Foreign keys to modules use CASCADE delete
- [ ] ModuleNpc token_asset_id FK to campaign_assets works
- [ ] Diesel models generated with proper derives
- [ ] DAL operations for ModuleMonster: create, read, update, delete, list_by_module
- [ ] DAL operations for ModuleNpc: create, read, update, delete, list_by_module
- [ ] Unit tests for all DAL operations
- [ ] Cascade delete test: deleting module removes all monsters and NPCs

## Implementation Notes

### Dependencies
- T-0381 (Module table must exist)
- T-0383 (CampaignAsset table for NPC tokens)

### Notes
- These tables must be created before T-0388 since TokenPlacement references them

## Status Updates **[REQUIRED]**

*To be added during implementation*