---
id: migration-tokenplacement-maptrap
level: task
title: "Migration: TokenPlacement, MapTrap, LightSource tables"
short_code: "MIMIR-T-0388"
created_at: 2026-01-20T21:49:44.071028+00:00
updated_at: 2026-01-21T01:22:46.383527+00:00
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

# Migration: TokenPlacement, MapTrap, LightSource tables

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[MIMIR-I-0043]]

## Objective

Create map overlay tables for initial play state:
- **TokenPlacement**: Pre-placed monsters/NPCs on maps (PCs placed at runtime via Pinia)
- **MapTrap**: Trap placements with position and trigger info
- **LightSource**: Dynamic light sources beyond UVTT static lights

## Schema

```sql
-- Token placements for monsters/NPCs (not PCs - those are placed at runtime)
CREATE TABLE token_placements (
    id TEXT PRIMARY KEY NOT NULL,
    map_id TEXT NOT NULL REFERENCES maps(id) ON DELETE CASCADE,
    
    -- What this token represents (one must be set)
    module_monster_id TEXT REFERENCES module_monsters(id) ON DELETE CASCADE,
    module_npc_id TEXT REFERENCES module_npcs(id) ON DELETE CASCADE,
    
    -- Position (grid coordinates)
    grid_x INTEGER NOT NULL,
    grid_y INTEGER NOT NULL,
    
    -- Display
    label TEXT,  -- Optional override label
    faction_color TEXT,  -- Hex color for faction ring (e.g., "#FF0000" for enemy)
    hidden INTEGER NOT NULL DEFAULT 0,  -- Hidden from players initially
    
    -- Timestamps
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    
    -- Ensure exactly one entity type is referenced
    CHECK (
        (module_monster_id IS NOT NULL AND module_npc_id IS NULL) OR
        (module_monster_id IS NULL AND module_npc_id IS NOT NULL)
    )
);

CREATE INDEX idx_token_placements_map ON token_placements(map_id);

-- Trap placements on maps
CREATE TABLE map_traps (
    id TEXT PRIMARY KEY NOT NULL,
    map_id TEXT NOT NULL REFERENCES maps(id) ON DELETE CASCADE,
    
    -- Catalog reference (if from catalog) or custom
    catalog_trap_id TEXT,  -- FK to catalog_traps if we have that table
    
    -- Position
    grid_x INTEGER NOT NULL,
    grid_y INTEGER NOT NULL,
    
    -- Trap info (can override catalog data)
    name TEXT NOT NULL,
    description TEXT,
    trigger_description TEXT,
    effect_description TEXT,
    dc INTEGER,  -- Detection/disarm DC
    
    -- State
    triggered INTEGER NOT NULL DEFAULT 0,
    visible INTEGER NOT NULL DEFAULT 0,  -- Visible to players
    
    -- Timestamps
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_map_traps_map ON map_traps(map_id);

-- Dynamic light sources (beyond UVTT static lights)
CREATE TABLE light_sources (
    id TEXT PRIMARY KEY NOT NULL,
    map_id TEXT NOT NULL REFERENCES maps(id) ON DELETE CASCADE,
    
    -- Position
    grid_x INTEGER NOT NULL,
    grid_y INTEGER NOT NULL,
    
    -- Light properties
    name TEXT,
    bright_radius INTEGER NOT NULL,  -- Bright light radius in feet
    dim_radius INTEGER NOT NULL,     -- Dim light radius in feet
    color TEXT,  -- Hex color (e.g., "#FFAA00" for torch)
    
    -- State
    active INTEGER NOT NULL DEFAULT 1,
    
    -- Timestamps
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_light_sources_map ON light_sources(map_id);
```

## Design Notes

- **TokenPlacement scope**: Only monsters and NPCs; PCs are placed at runtime via Pinia store and not persisted
- **faction_color**: Visual ring around token (red=enemy, blue=friendly, etc.) - DM can set per-token
- **hidden flag**: Token exists but not shown to players until revealed
- **Trap visibility**: `visible` controls player awareness; `triggered` tracks if already set off
- **Light sources**: For torches, magical lights, etc. that aren't in the UVTT file

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] Migration creates all three tables with constraints
- [ ] TokenPlacement CHECK constraint ensures exactly one entity type
- [ ] All foreign keys use CASCADE delete
- [ ] Diesel models generated for all tables
- [ ] DAL operations for each table: create, read, update, delete, list_by_map
- [ ] Unit tests for constraint validation (entity type check)
- [ ] Unit tests for cascade deletes

## Implementation Notes

### Dependencies
- T-0387 (Map table must exist)
- T-0389 (ModuleMonster, ModuleNpc tables must exist)

## Status Updates **[REQUIRED]**

*To be added during implementation*