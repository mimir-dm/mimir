---
id: v0-5-catalog-entity-types-and
level: task
title: "v0.5 Catalog Entity Types and Import"
short_code: "MIMIR-T-0365"
created_at: 2026-01-20T01:22:50.766446+00:00
updated_at: 2026-01-20T01:22:50.766446+00:00
parent: MIMIR-I-0041
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/todo"


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
- [ ] Source management service design
- [ ] Import pipeline specification
- [ ] 5eTools JSON parsing
- [ ] Source enable/disable UI flow

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

*To be updated during implementation*