---
id: v0-5-catalog-implementation
level: initiative
title: "v0.5 Catalog Implementation"
short_code: "MIMIR-I-0042"
created_at: 2026-01-20T02:01:06.560364+00:00
updated_at: 2026-01-20T21:03:27.847387+00:00
parent: MIMIR-V-0001
blocked_by: []
archived: true

tags:
  - "#initiative"
  - "#phase/completed"


exit_criteria_met: false
estimated_complexity: L
strategy_id: NULL
initiative_id: v0-5-catalog-implementation
---

# v0.5 Catalog Implementation Initiative

## Context

The Catalog is the read-only reference data layer for Mimir - all 5e content (monsters, spells, items, classes, races, etc.) imported from source books. It is the **basal dependency** for campaign entities:

- Characters reference Classes, Races, Backgrounds, Feats
- Modules reference Monsters, Items, Traps
- Character spells reference Spells
- Token placements reference Monster token images

This initiative implements the catalog data layer for v0.5, establishing the foundation before campaign entities can be built.

**Reference:** Ontology defined in [[MIMIR-T-0357]]

**Data Source:** Raw 5etools 2024 format
- Source: https://github.com/5etools-mirror-3/5etools-src/releases/tag/v2.23.0
- Download: https://github.com/5etools-mirror-3/5etools-src/releases/download/v2.23.0/5etools-v2.23.0.zip
- Using original 5etools structure directly (no splitter transformation)
- Includes fluff files for flavor text and images

## Goals & Non-Goals

**Goals:**
- Implement all 20 catalog entity types with indexed columns + JSON data blob
- Source management (CatalogSource) for enabling/disabling books
- FTS5 full-text search on all 20 catalog entity types
- Token image storage on Monster entities
- Import pipeline for 5etools JSON data
- Rust types for all catalog entities

**Non-Goals:**
- Campaign entities (separate initiative)
- MCP tools (separate initiative, depends on this)
- UI for catalog browsing (future)
- Homebrew content creation UI (future)

## Catalog Entities (20 types)

### Indexed Columns (Search/Filter Only)
Full entity data stored in JSON `data` column. Indexed columns for efficient queries:

| Entity | Indexed Columns | Notes |
|--------|-----------------|-------|
| CatalogSource | `code`, `name`, `enabled` | Filter by active sources |
| Monster | `name`, `source`, `cr`, `creature_type`, `size` | Core filters |
| Item | `name`, `source`, `type`, `rarity` | Type is single char code |
| Spell | `name`, `source`, `level`, `school`, `ritual`, `concentration` | Extract concentration from duration |
| Class | `name`, `source` | Details in JSON |
| Race | `name`, `source`, `size` | |
| Background | `name`, `source` | |
| Feat | `name`, `source` | Prerequisites complex, in JSON |
| Trap | `name`, `source` | |
| Action | `name`, `source` | |
| Condition | `name`, `source` | |
| Language | `name`, `source` | |
| Deity | `name`, `source`, `pantheon` | Filter by pantheon |
| Vehicle | `name`, `source` | |
| Object | `name`, `source` | |
| Reward | `name`, `source` | |
| OptionalFeature | `name`, `source`, `feature_type` | Invocations vs fighting styles |
| Psionic | `name`, `source` | |
| VariantRule | `name`, `source` | |
| Cult | `name`, `source` | |
| Table | `name`, `source` | |

### Join Tables

| Table | Columns | Purpose |
|-------|---------|---------|
| spell_classes | `spell_id`, `class_name`, `source` | Spells available to each class |
| spell_subclasses | `spell_id`, `subclass_name`, `class_name`, `source` | Subclass spell lists (Eldritch Knight, etc.) |
| item_attunement_classes | `item_id`, `class_name` | Item attunement requirements |

Data for join tables extracted from `gendata-spell-source-lookup.json` and item `reqAttuneTags`.

### Field Extraction Rules

Some indexed columns require extraction from nested 5etools structures:

| Column | 5etools Path | Notes |
|--------|--------------|-------|
| Monster.creature_type | `type.type` | Nested object, extract inner type string |
| Monster.size | `size[0]` | Array, take first element |
| Monster.cr | `cr` | Can be string ("1/4") or number |
| Spell.concentration | `duration[].concentration` | Boolean in duration object |
| Spell.ritual | `meta.ritual` | Boolean |
| Item.type | `type` | Single char code (R=ranged, A=ammo, etc.) |
| OptionalFeature.feature_type | `featureType[0]` | Array, e.g. ["EI"] = Eldritch Invocation |

## Schema & Type Generation

**Approach:** Use typify to generate Rust types from 5etools JSON schemas
- Schemas sourced from `5etools-utils/schema/brew-fast/`
- Generated types match 5etools structure exactly (no transformation layer)
- Vendored schemas in `crates/mimir-core/schema/`

**Schema Mapping:**

| Entity | Data File(s) | Schema | Fluff |
|--------|-------------|--------|-------|
| Monster | `bestiary/*.json` | bestiary.json | bestiary/fluff-*.json |
| Item | `items.json` | items.json | fluff-items.json |
| Spell | `spells/*.json` | spells.json | spells/fluff-*.json |
| Class | `class/*.json` | class.json | class/fluff-*.json |
| Race | `races.json` | races.json | fluff-races.json |
| Background | `backgrounds.json` | backgrounds.json | fluff-backgrounds.json |
| Feat | `feats.json` | feats.json | fluff-feats.json |
| Trap | `trapshazards.json` | trapshazards.json | fluff-trapshazards.json |
| Action | `actions.json` | actions.json | - |
| Condition | `conditionsdiseases.json` | conditionsdiseases.json | fluff-conditionsdiseases.json |
| Language | `languages.json` | languages.json | fluff-languages.json |
| Deity | `deities.json` | deities.json | - |
| Vehicle | `vehicles.json` | vehicles.json | fluff-vehicles.json |
| Object | `objects.json` | objects.json | fluff-objects.json |
| Reward | `rewards.json` | rewards.json | fluff-rewards.json |
| OptionalFeature | `optionalfeatures.json` | optionalfeatures.json | fluff-optionalfeatures.json |
| Psionic | `psionics.json` | psionics.json | - |
| VariantRule | `variantrules.json` | variantrules.json | - |
| Cult | `cultsboons.json` | cultsboons.json | - |
| Table | `tables.json` | tables.json | - |

**Shared schemas:** `entry.json` (rich text entries), `util.json` (common types)

**Vendoring Status:** Currently 6 schemas vendored in old codebase. Need to fetch all 20+ from `brew-fast`.

## Architecture

### Crate Structure
```
crates/
├── mimir-core/                # Data model + basic CRUD
│   ├── Cargo.toml
│   ├── src/
│   │   ├── lib.rs
│   │   ├── schema.rs          # Diesel schema (generated)
│   │   ├── models/
│   │   │   ├── mod.rs
│   │   │   └── catalog/
│   │   │       ├── mod.rs
│   │   │       ├── source.rs      # CatalogSource
│   │   │       ├── monster.rs
│   │   │       ├── item.rs
│   │   │       ├── spell.rs
│   │   │       ├── class.rs
│   │   │       ├── race.rs
│   │   │       ├── background.rs
│   │   │       ├── feat.rs
│   │   │       ├── trap.rs
│   │   │       └── ... (10 more)
│   │   └── dal/
│   │       ├── mod.rs
│   │       └── catalog.rs     # CRUD operations
│   └── migrations/
│       └── 001_catalog_schema/
│           ├── up.sql
│           └── down.sql
│
└── mimir-services/            # Business logic, import pipeline
    ├── Cargo.toml
    ├── src/
    │   ├── lib.rs
    │   └── catalog/
    │       ├── mod.rs
    │       ├── import.rs      # 5etools import logic
    │       └── search.rs      # FTS query building
    └── (depends on mimir-core)
```

### Data Pattern
Each catalog entity follows the same pattern:
- **Indexed columns** for search/filter (extracted from JSON)
- **data** column with full JSON blob
- **source** references CatalogSource.code

### FTS Strategy

**Unified FTS table** with content type separation:

```sql
CREATE VIRTUAL TABLE catalog_fts USING fts5(
  entity_type,    -- 'monster', 'spell', 'item', etc.
  entity_id,      -- References the entity's primary key
  content_type,   -- 'rules' | 'fluff'
  name,           -- Entity name (always indexed)
  text_content    -- Flattened entries text
);
```

**Content types:**
- `rules` - name + flattened `entries` (mechanics, stat blocks, descriptions)
- `fluff` - name + flattened fluff `entries` (flavor text, lore, images descriptions)

**Query examples:**
- Search rules only: `WHERE content_type = 'rules' AND catalog_fts MATCH 'fire damage'`
- Search fluff only: `WHERE content_type = 'fluff' AND catalog_fts MATCH 'ancient dragon'`
- Search everything: `WHERE catalog_fts MATCH 'Tiamat'`

**Entry flattening:** Recursive extraction of text from 5etools nested entry format (lists, tables, insets, etc.)

### Token Image Storage
Monster token images stored in app data directory with path pointer in DB:
- `Monster.token_image_path` → `{app_data}/tokens/{source}/{monster_id}.png`
- Images imported from 5etools during catalog import
- ~2000 tokens, 200-600MB total for full catalog

## Design Decisions

**typify Integration:** Standalone script, commit generated code
- Faster builds, explicit schema updates
- Can move to build.rs later if needed

**App Data Directory:** Tauri injects via `app_data_dir()` at runtime
- Token images: `{app_data_dir}/tokens/{source}/{monster_id}.png`
- Passed to import service as parameter

**Import Error Handling:** Fail per-book, continue to next
- Transaction per source book
- If book fails, rollback that book, log error, continue
- Report summary at end (X books imported, Y failed)

## Detailed Design

### Migration 001: Catalog Schema
- CatalogSource table
- All 20 catalog entity tables
- FTS5 virtual tables for all catalog entities
- Indexes on commonly filtered columns

### Rust Types (Two Layers)

**1. 5etools Types (typify-generated)**
- Generated from JSON schemas via typify
- Match 5etools structure exactly
- Used for: deserializing 5etools JSON, storing in `data` blob
- Example: `Monster` with full stat block, abilities, actions, etc.

**2. DB Row Types (Diesel)**
- `Queryable`/`Insertable` structs for database operations
- Indexed columns extracted from 5etools type + JSON blob
- Example: `MonsterRow { id, name, source, cr, creature_type, ..., data: JsonValue }`

**Flow:** 5etools JSON → typify type → extract indexed fields → DB row

### Import Pipeline
- Parse 5etools JSON format
- Extract indexed fields to columns
- Store full entity in data blob
- Update FTS indexes
- Track source in CatalogSource

## Alternatives Considered

**Single mega-table vs separate tables**: Chose separate tables for type safety and targeted indexes. A single "catalog_entities" table would simplify schema but lose query optimization.

**Store only JSON vs indexed columns**: Chose hybrid. JSON-only would require deserializing for every filter. Indexed columns enable efficient SQL queries while JSON preserves full fidelity.

## Implementation Plan

### Phase 1: Foundation
- [ ] Create mimir-core crate structure
- [ ] Create mimir-services crate structure
- [ ] Migration 001 with catalog_sources table
- [ ] CatalogSource Rust types and DAL

### Phase 2: Core Entities
- [ ] Monster, Item, Spell tables and types
- [ ] FTS5 indexes for these entities
- [ ] Token image path support on Monster

### Phase 3: Character Entities
- [ ] Class, Race, Background, Feat tables
- [ ] FTS5 indexes for these entities
- [ ] Types for character creation support

### Phase 4: Remaining Entities
- [ ] Trap, Action, Condition, Language, Deity
- [ ] Vehicle, Object, Reward, OptionalFeature
- [ ] Psionic, VariantRule, Cult, Table
- [ ] FTS5 indexes for all remaining entities

### Phase 5: Import Pipeline (mimir-services)
- [ ] 5etools JSON parser
- [ ] Import service with token extraction
- [ ] Source management
- [ ] Unified search service