---
id: homebrew-monster-vtt-integration
level: initiative
title: "Homebrew Monster VTT Integration"
short_code: "MIMIR-I-0057"
created_at: 2026-03-11T14:40:44.878600+00:00
updated_at: 2026-03-11T21:00:16.897838+00:00
parent: MIMIR-V-0001
blocked_by: []
archived: true

tags:
  - "#initiative"
  - "#phase/completed"


exit_criteria_met: false
estimated_complexity: M
initiative_id: homebrew-monster-vtt-integration
---

# Homebrew Monster VTT Integration Initiative

## Context

Homebrew monsters are currently siloed from the entire VTT pipeline. They live in `campaign_homebrew_monsters` with full JSON stat blocks, but cannot be:
- Added to modules (only catalog monsters via `monster_name`/`monster_source` lookup)
- Placed as tokens on maps (token enrichment only resolves catalog monsters)
- Found via `search_monsters` MCP tool (only searches catalog)

The `module_monsters` table references catalog entries by `(monster_name, monster_source)` text columns. `token_placements` references `module_monsters.id`. The entire chain from module → token → stat block resolution assumes catalog-only monsters.

### Key Files
- `migrations/016_module_entities/up.sql` — `module_monsters` table (catalog-only)
- `migrations/017_map_overlays/up.sql` — `token_placements` table
- `migrations/025_campaign_homebrew_monsters/up.sql` — `campaign_homebrew_monsters` table
- `crates/mimir-core/src/services/token.rs` — token enrichment (catalog-only lookup)
- `crates/mimir-mcp/src/tools/catalog.rs` — `search_monsters` (catalog-only)
- `crates/mimir-mcp/src/tools/module.rs` — `add_monster_to_module` (catalog-only)

## Goals & Non-Goals

**Goals:**
- Enable homebrew monsters to be added to modules alongside catalog monsters
- Allow homebrew monsters to be placed as tokens on maps with full stat resolution
- Include homebrew monsters in MCP `search_monsters` results (or a unified search)
- Maintain backward compatibility with existing catalog monster references

**Non-Goals:**
- Homebrew monster creation/editing UI changes (already functional)
- Homebrew spell or item VTT integration (separate initiative)
- Monster image/token art management

## Architecture

### Overview

Add an optional `homebrew_monster_id` FK to `module_monsters`, making `monster_name`/`monster_source` nullable. A module monster entry is either a catalog reference (name+source) OR a homebrew reference (homebrew_monster_id), never both. This is enforced with a CHECK constraint.

### Schema Change

```sql
ALTER TABLE module_monsters
  ADD COLUMN homebrew_monster_id TEXT
    REFERENCES campaign_homebrew_monsters(id) ON DELETE CASCADE;

-- Make catalog columns nullable (existing data keeps values)
-- Add CHECK: exactly one of (monster_name, homebrew_monster_id) must be non-null
```

### Resolution Chain

```
module_monster
  ├── monster_name + monster_source → catalog lookup (existing)
  └── homebrew_monster_id → campaign_homebrew_monsters lookup (new)

token_placement → module_monster → resolved stat block (unified)
```

The token enrichment service (`services/token.rs`) currently does:
1. Load token placement → get `module_monster_id`
2. Load module monster → get `(name, source)`
3. Search catalog by name+source → get stats

With this change, step 2-3 become: check if homebrew_monster_id is set; if so, load from `campaign_homebrew_monsters` and parse the JSON blob; otherwise, do the existing catalog lookup.

## Detailed Design

### Migration
- New migration adding `homebrew_monster_id` column with FK to `campaign_homebrew_monsters`
- CHECK constraint ensuring exactly one path: `(monster_name IS NOT NULL) != (homebrew_monster_id IS NOT NULL)`
- `monster_name` and `monster_source` become nullable

### Models (`mimir-core/src/models/campaign.rs`)
- `ModuleMonster` gains `homebrew_monster_id: Option<String>`
- `NewModuleMonster` gains builder method `.with_homebrew_monster(id)` as alternative to name+source
- Add validation: cannot set both name and homebrew_monster_id

### DAL (`mimir-core/src/dal/campaign.rs`)
- `insert_module_monster` — handle new column
- `list_module_monsters` — join with `campaign_homebrew_monsters` when homebrew_monster_id is set
- New: `list_module_monsters_with_homebrew_data` — returns enriched data for both types

### Services
- `ModuleService` — accept homebrew monster additions
- `TokenService` — unified stat resolution that checks homebrew first, then catalog
- Consider a `MonsterResolver` trait or helper that abstracts catalog vs homebrew lookup

### Tauri Commands (`mimir-lib/src/commands/module.rs`)
- `add_module_monster` — accept optional `homebrew_monster_id` parameter
- `list_module_monsters_with_data` — return homebrew monster data alongside catalog data

### MCP Tools (`mimir-mcp/src/tools/module.rs`)
- `add_monster_to_module` — accept `homebrew_monster_id` as alternative to `monster_name`
- `search_monsters` — optionally include homebrew results, or add `search_homebrew_monsters`

### Frontend
- Module monster list component — display homebrew monsters with visual indicator
- "Add Monster" dialog — show homebrew tab alongside catalog search
- Token placement — homebrew monsters available in token type picker

## Alternatives Considered

### Option B: Synthetic Source Code
Use a synthetic `monster_source = "HOMEBREW"` and store the homebrew monster's name in `monster_name`. Resolution would check for the magic source string and redirect to the homebrew table.

**Rejected because:**
- Fragile: relies on a magic string convention
- Name collisions: homebrew monster names aren't guaranteed unique across campaigns
- No referential integrity: no FK means orphaned references if homebrew monster is deleted
- Harder to query: can't simply join on an ID

### Option C: Unified Monster Table
Merge catalog and homebrew monsters into one table.

**Rejected because:**
- Massive migration effort for catalog data
- Catalog monsters are read-only imported data; homebrew are user-editable
- Different lifecycle and ownership semantics
- Would complicate catalog import/update pipeline

## Implementation Plan

### Phase 1: Schema & Models
- Write migration (add column, CHECK constraint, nullable changes)
- Update Diesel schema and models
- Add builder methods and validation

### Phase 2: DAL & Services
- Update DAL functions for insert/list/get with homebrew support
- Update `TokenService` for unified stat resolution
- Add integration tests

### Phase 3: Tauri Commands & MCP Tools
- Update `add_module_monster` command and MCP tool
- Update monster list endpoints to return enriched data
- Update `search_monsters` or add homebrew search

### Phase 4: Frontend
- Module monster list shows homebrew monsters
- Add monster dialog includes homebrew tab
- Token placement supports homebrew monsters

### Phase 5: Print/Export
- Monster cards support homebrew stat blocks
- Campaign export/import includes homebrew module monster references