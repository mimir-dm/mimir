---
id: v0-5-catalog-entity-types-and
level: task
title: "v0.5 Catalog Entity Types and Import"
short_code: "MIMIR-T-0365"
created_at: 2026-01-20T01:22:50.766446+00:00
updated_at: 2026-01-28T03:52:06.979468+00:00
parent: MIMIR-I-0041
blocked_by: []
archived: true

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: MIMIR-I-0041
---

# v0.5 Catalog Import Pipeline

## Parent Initiative
[[MIMIR-I-0041]] - Mimir v0.5 Architecture Rewrite

## Objective
Design the import pipeline for loading 5e content from source files into the catalog tables, and the source management service for enabling/disabling content sources.

Note: Catalog entity Rust types are defined in [[MIMIR-T-0357]] Database Schema Design.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria
- [x] Source management service design
- [x] Import pipeline specification
- [x] 5eTools JSON parsing
- [x] Source filtering UI - Implemented at campaign and character level (MIMIR-T-0449, MIMIR-T-0450) rather than global enable/disable

## Implementation Status

**IMPLEMENTED** - Core import pipeline exists in `mimir-core/src/import/`:
- `discovery.rs` - Discovers books from 5etools data directories
- `filter.rs` - Filters entities by source code
- `srd.rs` - SRD content identification
- `collector.rs` - Generic entity collection
- `service.rs` - Orchestration with DB insertion and FTS indexing

### Entity Types (26 total, 26 imported)

| Entity | Table | Import | Notes |
|--------|-------|--------|-------|
| Monsters | ✅ | ✅ | CR, type, size extracted |
| Spells | ✅ | ✅ | Level, school, ritual, concentration, class lists |
| Items | ✅ | ✅ | Type, rarity, attunement classes |
| Classes | ✅ | ✅ | Core classes |
| Subclasses | ✅ | ✅ | With parent class tracking |
| Races | ✅ | ✅ | Player races |
| Backgrounds | ✅ | ✅ | Character backgrounds |
| Feats | ✅ | ✅ | Character feats |
| Skills | ✅ | ✅ | With ability field |
| Senses | ✅ | ✅ | Creature senses |
| Languages | ✅ | ✅ | With type classification |
| Actions | ✅ | ✅ | Combat actions |
| Conditions | ✅ | ✅ | Game conditions |
| Diseases | ✅ | ✅ | Disease entries |
| Traps | ✅ | ✅ | With trap tier |
| Hazards | ✅ | ✅ | Environmental hazards |
| Objects | ✅ | ✅ | With object type |
| Vehicles | ✅ | ✅ | With vehicle type |
| Deities | ✅ | ✅ | With pantheon |
| Cults | ✅ | ✅ | Cults and boons |
| Optional Features | ✅ | ✅ | With feature_type (EI, MM, etc.) |
| Psionics | ✅ | ✅ | With psionic_type and order |
| Rewards | ✅ | ✅ | With reward_type (blessing, boon, charm) |
| Variant Rules | ✅ | ✅ | With rule_type |
| Catalog Tables | ✅ | ✅ | Roll tables, trinkets, etc. |
| Spell Lists | ✅ | ✅ | Class/subclass spell assignments |

### Special Features
- Token image importing from 5etools img directory
- FTS indexing of entries and fluff
- Transaction safety with per-source SAVEPOINTs
- Spell-class and item-attunement join tables

## Source Management Service

```rust
#[async_trait]
pub trait SourceService: Send + Sync {
    /// List all imported sources
    async fn list_sources(&self) -> Result<Vec<CatalogSource>>;
    
    /// Enable/disable a source
    async fn set_source_enabled(&self, code: &str, enabled: bool) -> Result<()>;
    
    /// Import content from a source file
    async fn import_source(&self, path: &Path, source_type: SourceType) -> Result<ImportResult>;
    
    /// Get enabled sources (for filtering catalog queries)
    async fn get_enabled_sources(&self) -> Result<Vec<String>>;
}

pub struct ImportResult {
    pub source_code: String,
    pub monsters_imported: usize,
    pub items_imported: usize,
    pub spells_imported: usize,
    pub classes_imported: usize,
    pub races_imported: usize,
    pub feats_imported: usize,
    pub backgrounds_imported: usize,
    pub traps_imported: usize,
}
```

## Import Pipeline

### Source File Format

Support for 5eTools JSON format:
```
data/
├── bestiary/
│   ├── bestiary-mm.json
│   ├── bestiary-vgm.json
│   └── ...
├── spells/
│   ├── spells-phb.json
│   └── ...
├── items/
│   ├── items-base.json
│   ├── items-dmg.json
│   └── ...
├── class/
│   ├── class-fighter.json
│   └── ...
└── ...
```

### Import Flow

```
1. User selects source file(s) or directory
2. Parser identifies content type (monsters, spells, etc.)
3. Parser extracts source code from file
4. Create/update catalog_sources entry
5. For each entity:
   a. Extract indexed fields to columns
   b. Store full JSON in data column
   c. Update FTS index
6. Return ImportResult summary
```

## UI: Source Management

```
Settings > Sources

┌─────────────────────────────────────────────────────────┐
│  Catalog Sources                          [+ Import]    │
├─────────────────────────────────────────────────────────┤
│  Core Rules                                             │
│  ┌─────────────────────────────────────────────────┐    │
│  │ [✓] PHB - Player's Handbook                     │    │
│  │ [✓] MM  - Monster Manual                        │    │
│  │ [✓] DMG - Dungeon Master's Guide                │    │
│  └─────────────────────────────────────────────────┘    │
│                                                         │
│  Supplements                                            │
│  ┌─────────────────────────────────────────────────┐    │
│  │ [✓] XGE - Xanathar's Guide to Everything        │    │
│  │ [ ] TCE - Tasha's Cauldron of Everything        │    │
│  │ [✓] VGM - Volo's Guide to Monsters              │    │
│  └─────────────────────────────────────────────────┘    │
│                                                         │
│  Disabled sources are excluded from catalog searches    │
└─────────────────────────────────────────────────────────┘
```

## Dependencies

- Depends on: [[MIMIR-T-0357]] Database Schema (catalog tables and entity types)
- Related: [[MIMIR-T-0358]] Service Layer (CatalogService search methods)
- Related: [[MIMIR-T-0359]] MCP Tools (search_* tools for catalog)

## Progress

### 2026-01-20: Completed 5 missing catalog entities
- Created migration `018_remaining_catalog` with 5 new tables:
  - `optional_features` (feature_type for EI/MM/etc.)
  - `psionics` (psionic_type and psionic_order)
  - `rewards` (reward_type for blessing/boon/charm)
  - `variant_rules` (rule_type)
  - `catalog_tables` (roll tables, trinkets, etc.)
- Created Rust models: OptionalFeature, Psionic, Reward, VariantRule, CatalogTable
- Created DAL functions with full CRUD + batch insert
- Updated import service to persist all 5 entity types
- All 576 tests passing