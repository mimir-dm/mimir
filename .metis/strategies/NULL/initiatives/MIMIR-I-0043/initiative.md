---
id: v0-5-campaign-data-model
level: initiative
title: "v0.5 Campaign Data Model Implementation"
short_code: "MIMIR-I-0043"
created_at: 2026-01-20T02:02:37.484288+00:00
updated_at: 2026-01-20T02:02:37.484288+00:00
parent: MIMIR-V-0001
blocked_by: []
archived: false

tags:
  - "#initiative"
  - "#phase/discovery"


exit_criteria_met: false
estimated_complexity: XL
strategy_id: NULL
initiative_id: v0-5-campaign-data-model
---

# v0.5 Campaign Data Model Implementation Initiative

## Context

The Campaign data model is the **user-created, mutable content** in Mimir - everything the DM creates and modifies during campaign prep and play. This is distinct from the Catalog (read-only reference data).

**Dependency:** Requires [[MIMIR-I-0042]] (Catalog) as campaign entities reference catalog data.

**Reference:** Ontology defined in [[MIMIR-T-0357]]

## Goals & Non-Goals

**Goals:**
- Implement all campaign entity types with proper relationships
- Database-first storage (markdown content in DB, not filesystem)
- Campaign rule sets (which source books are in play)
- Character model with multiclass, feats, inventory, spells, proficiencies
- Map system with token placements, traps, fog, lighting
- Module system for encounter organization
- FTS5 search on documents
- Campaign import/export for DM sharing

**Non-Goals:**
- Catalog entities (separate initiative [[MIMIR-I-0042]])
- MCP tools (separate initiative, depends on this)
- Tauri UI (separate initiative)
- PDF export (separate initiative)

## Campaign Entities

### Core Hierarchy

| Entity | Purpose | Key Fields |
|--------|---------|------------|
| Campaign | Top-level container | name, description, archived_at |
| CampaignSource | Rule set (allowed books) | campaign_id, source_code |
| Module | Adventure chapters | campaign_id, name, module_number |
| Document | Markdown content | campaign_id, module_id?, title, content |

### Character System

| Entity | Purpose | Key Fields |
|--------|---------|------------|
| Character | PC or NPC | name, is_npc, player_name, classes[], abilities, combat stats, currency, personality, npc fields |
| CharacterInventory | Held items | character_id, item_name, item_source, quantity, equipped, attuned |
| CharacterProficiency | Skills, saves, etc. | character_id, proficiency_type, name, expertise |
| CharacterSpell | Known/prepared spells | character_id, spell_name, spell_source, prepared, source_class |

### Map System

| Entity | Purpose | Key Fields |
|--------|---------|------------|
| Map | Tactical maps | campaign_id, module_id?, name, uvtt_path, image_path, grid config, los_walls[] |
| TokenPlacement | Placed tokens | map_id, name, token_type, size, x, y, monster_id?, character_id? |
| MapTrap | Placed traps | map_id, trap_name, trap_source, x, y, triggered, visible |
| FogArea | Fog of war | map_id, x, y, width, height |
| LightSource | Lighting | map_id, x, y, light_type, radius_ft, color |

### Module Content

| Entity | Purpose | Key Fields |
|--------|---------|------------|
| ModuleMonster | Encounter monsters | module_id, monster_name, monster_source, quantity, encounter_tag |
| ModuleNpc | NPCs in module | module_id, character_id, role, encounter_tag |

## Architecture

### Crate Structure (extends mimir-core)
```
crates/mimir-core/
├── src/
│   ├── models/
│   │   ├── catalog/          # From MIMIR-I-0042
│   │   └── campaign/         # NEW
│   │       ├── mod.rs
│   │       ├── campaign.rs
│   │       ├── campaign_source.rs
│   │       ├── module.rs
│   │       ├── document.rs
│   │       ├── character.rs
│   │       ├── character_inventory.rs
│   │       ├── character_proficiency.rs
│   │       ├── character_spell.rs
│   │       ├── map.rs
│   │       ├── token_placement.rs
│   │       ├── map_trap.rs
│   │       ├── fog_area.rs
│   │       ├── light_source.rs
│   │       ├── module_monster.rs
│   │       └── module_npc.rs
│   ├── dal/
│   │   ├── catalog.rs
│   │   └── campaign.rs       # NEW
│   └── services/
│       ├── catalog.rs
│       ├── campaign.rs       # NEW
│       ├── character.rs      # NEW
│       └── map.rs            # NEW
└── migrations/
    ├── 001_catalog_schema/   # From MIMIR-I-0042
    └── 002_campaign_schema/  # NEW
```

### Key Design Decisions

**Database-first:** Document.content stored in SQLite, not filesystem. Enables ACID compliance, search, and portable export.

**Unified Character:** Single table for PC and NPC with `is_npc` flag. NPCs have additional fields (role, location, faction).

**Typed JSON:** Character.classes and Character.feats are JSON arrays with typed Rust structs (ClassLevel[], Feature[]).

**Rule Sets:** CampaignSource join table defines which source books are allowed, constraining character creation and filtering catalog searches.

## Detailed Design

### Migration 002: Campaign Schema
- Campaign, CampaignSource tables
- Module table with unique (campaign_id, module_number)
- Document table with content column + FTS5
- Character with all stat columns + JSON for classes/feats
- CharacterInventory, CharacterProficiency, CharacterSpell
- Map with UVTT support, grid config, los_walls JSON
- TokenPlacement, MapTrap, FogArea, LightSource
- ModuleMonster, ModuleNpc
- All foreign keys with CASCADE deletes
- Indexes on common query paths

### Value Objects (JSON embedded)
```rust
// Character.classes column
pub struct ClassLevel {
    pub class: String,
    pub level: i32,
    pub subclass: Option<String>,
}

// Character.feats column  
pub struct Feature {
    pub name: String,
    pub source: String,
    pub feature_type: FeatureType,
    pub description: Option<String>,
}

// Map.los_walls column
pub struct Wall {
    pub start: Point,
    pub end: Point,
}
```

### Services
- **CampaignService:** CRUD for campaigns, modules, documents
- **CharacterService:** Character lifecycle, inventory, spells, proficiencies
- **MapService:** Map management, token placement, fog/lighting

## Alternatives Considered

**Separate PC/NPC tables:** Rejected. Unified model is simpler, NPCs can become PCs (captured enemies, converted allies).

**Normalized classes (separate table):** Rejected. JSON array is simpler for multiclass and keeps character as single unit for export.

**Filesystem for documents:** Rejected. Database-first enables ACID, search, and clean export/import.

## Implementation Plan

### Phase 1: Core Entities
- [ ] Migration 002 with Campaign, CampaignSource, Module, Document
- [ ] Document FTS5 index
- [ ] Campaign/Module/Document Rust types and DAL
- [ ] CampaignService

### Phase 2: Character System
- [ ] Character table with all columns
- [ ] CharacterInventory, CharacterProficiency, CharacterSpell tables
- [ ] ClassLevel, Feature value objects
- [ ] CharacterService

### Phase 3: Map System
- [ ] Map table with UVTT/grid/lighting fields
- [ ] TokenPlacement, MapTrap, FogArea, LightSource tables
- [ ] Wall, Point value objects
- [ ] MapService

### Phase 4: Module Content
- [ ] ModuleMonster, ModuleNpc tables
- [ ] Encounter organization queries

### Phase 5: Export/Import
- [ ] Campaign serialization format
- [ ] Export service
- [ ] Import service with conflict resolution