---
id: v0-5-campaign-data-model
level: initiative
title: "v0.5 Campaign Data Model Implementation"
short_code: "MIMIR-I-0043"
created_at: 2026-01-20T02:02:37.484288+00:00
updated_at: 2026-01-20T21:57:36.239485+00:00
parent: MIMIR-V-0001
blocked_by: []
archived: true

tags:
  - "#initiative"
  - "#phase/completed"


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
| Module | Adventure chapter container | campaign_id, name, module_number |
| Document | Markdown content | campaign_id, module_id (optional), title, content, `type` (freeform string, app defines valid types) |
| Map | Tactical/regional maps | campaign_id, module_id (optional), name, uvtt_path, `lighting_mode` |
| CampaignAsset | User-uploaded images | campaign_id, filename, mime_type, blob_path, uploaded_at |

**Hierarchy:** Campaign contains Documents, Modules, Maps, Assets. Modules contain their own Documents and Maps (via optional module_id).

### Module Content

| Entity | Purpose | Key Fields |
|--------|---------|------------|
| ModuleMonster | Module's monster roster | module_id, monster_name, monster_source, quantity, `encounter_tag` (optional) |
| ModuleNpc | Module's NPC roster | module_id, character_id, role, `encounter_tag` (optional) |

**encounter_tag** ties roster entries to narrative elements in documents (e.g., "ambush", "throne_room").

### Character System

**Design principle:** Store joinable data, calculate derived values (AC, HP, etc.) at view time.

| Entity | Purpose | Key Fields |
|--------|---------|------------|
| Character | PC or NPC | name, is_npc, player_name, str/dex/con/int/wis/cha, currency (cp/sp/ep/gp/pp), personality (traits/ideals/bonds/flaws), NPC fields (role/location/faction) |
| CharacterClass | Class levels | character_id, class_name, class_source, level, subclass_name, subclass_source, `starting_class` flag |
| CharacterFeat | Selected feats | character_id, feat_name, feat_source, `source_type` (asi/race/class/bonus) |
| CharacterInventory | Equipment | character_id, item_name, item_source, quantity, `equipped` flag, `attuned` flag, notes |
| CharacterProficiency | Skills, saves, etc. | character_id, proficiency_type, name, `expertise` flag |
| CharacterSpell | Known spells | character_id, spell_name, spell_source, source_class, `equipped` flag (prepared/chosen) |

### Map System

**Design principle:** UVTT file is source of truth for grid/walls/base lighting. DB stores initial play state and additions. PC placement happens at play time (Pinia store).

| Entity | Purpose | Key Fields |
|--------|---------|------------|
| Map | Tactical maps | campaign_id, module_id?, name, uvtt_path, `lighting_mode` (bright/dim/dark/token_los), `environment` (indoor/outdoor/underground) |
| TokenPlacement | Monster + NPC initial positions | map_id, name, size, x, y, `monster_id` OR `character_id` (one or other), `faction_color` (optional hex), notes |
| MapTrap | Placed traps | map_id, trap_name, trap_source, x, y, `visible` (initial visibility), notes |
| LightSource | Additional lights beyond UVTT | map_id, x, y, radius_ft, color, `visible`, notes |

**Dropped:** FogArea table - handled by map-level lighting_mode instead.



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