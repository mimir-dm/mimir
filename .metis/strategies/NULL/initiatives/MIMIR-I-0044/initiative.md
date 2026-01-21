---
id: implement-service-layer-for-mimir
level: initiative
title: "Implement Service Layer for mimir-core"
short_code: "MIMIR-I-0044"
created_at: 2026-01-21T02:35:10.393385+00:00
updated_at: 2026-01-21T03:02:14.400921+00:00
parent: MIMIR-V-0001
blocked_by: []
archived: false

tags:
  - "#initiative"
  - "#phase/decompose"


exit_criteria_met: false
estimated_complexity: L
strategy_id: NULL
initiative_id: implement-service-layer-for-mimir
---

# Implement Service Layer for mimir-core Initiative

## Context

mimir-core currently has:
- **DAL layer** (`src/dal/`) - raw database operations for all entities
- **Models layer** (`src/models/`) - data structures for catalog and campaign entities
- **Import layer** (`src/import/`) - 5etools data import

Missing: A **services layer** that provides business logic, validation, and a clean API for both MCP and Tauri consumers.

### Findings from Previous Implementation (mimir-dm-bu)

The previous mimir-dm-core had a comprehensive services layer with these patterns:

1. **Stateful Services**: Held `&'a mut SqliteConnection` reference
2. **CatalogService Trait**: Generic interface with associated types (Filters, Summary, Full)
3. **Business Logic**: Validation, file I/O, directory creation, stage transitions lived in services
4. **Query Limits**: `DEFAULT_QUERY_LIMIT = 1000` to prevent memory issues
5. **Boxed Queries**: Dynamic filtering with Diesel's `into_boxed()`

## Goals & Non-Goals

**Goals:**
- Implement content management services for campaign authoring workflows
- Provide clean API for MCP server and Tauri commands to consume
- Encapsulate business logic (validation, file operations) in services
- Enable testability via trait-based design
- Support both sync and async patterns (sync Diesel, async-wrapped for Tauri)

**Non-Goals:**
- Play state services (fog of war, token movement, lighting calculations)
- Real-time VTT features (WebSocket updates, spatial queries)
- PDF export services (separate initiative)

## Scope

### In Scope - Content Management Services

| Service | Responsibility |
|---------|---------------|
| `CampaignService` | Create, list, get, update, archive campaigns |
| `ModuleService` | Module CRUD, assign monsters/NPCs/items to modules |
| `CharacterService` | PC/NPC management, inventory, class/level tracking |
| `DocumentService` | Campaign/module document CRUD |
| `CatalogService` | Search and retrieve catalog entities (monsters, spells, items, etc.) |

### Out of Scope - Play State Services (Future Initiative)

- `FogOfWarService` - reveal/hide map areas
- `TokenService` - token placement and movement
- `LightSourceService` - dynamic lighting
- `MapService` (VTT aspects) - real-time map state management

## Architecture

```
┌──────────────────┐     ┌──────────────────┐
│   MCP Server     │     │  Tauri Commands  │
└────────┬─────────┘     └────────┬─────────┘
         │                        │
         └──────────┬─────────────┘
                    ▼
         ┌──────────────────┐
         │  Service Layer   │  ← Business logic, validation
         └────────┬─────────┘
                  ▼
         ┌──────────────────┐
         │    DAL Layer     │  ← SQL queries, data mapping
         └────────┬─────────┘
                  ▼
         ┌──────────────────┐
         │     SQLite       │
         └──────────────────┘
```

## Detailed Design

### Service Pattern

Based on ADR MIMIR-A-0005, services are stateful with connection field:

```rust
pub struct CampaignService<'a> {
    conn: &'a mut SqliteConnection,
}

impl<'a> CampaignService<'a> {
    pub fn new(conn: &'a mut SqliteConnection) -> Self {
        Self { conn }
    }
    
    pub fn list(&mut self, include_archived: bool) -> Result<Vec<Campaign>> { ... }
    pub fn get(&mut self, id: i32) -> Result<Option<Campaign>> { ... }
    pub fn create(&mut self, input: CreateCampaign) -> Result<Campaign> { ... }
    // ...
}
```

### CatalogService Trait

Generic trait for catalog entity access:

```rust
pub trait CatalogEntityService {
    type Entity;
    type Filter: Default;
    type Summary;
    
    fn search(&mut self, filter: Self::Filter) -> Result<Vec<Self::Summary>>;
    fn get_by_name_and_source(&mut self, name: &str, source: &str) -> Result<Option<Self::Entity>>;
    fn get_sources(&mut self) -> Result<Vec<String>>;
}
```

### Error Handling

```rust
#[derive(Debug, thiserror::Error)]
pub enum ServiceError {
    #[error("Not found: {entity_type} with id {id}")]
    NotFound { entity_type: String, id: String },
    
    #[error("Validation error: {0}")]
    Validation(String),
    
    #[error("Database error: {0}")]
    Database(#[from] diesel::result::Error),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}
```

## Testing Strategy

- Unit tests with in-memory SQLite using `test_utils::setup_test_db_with_sources()`
- Service tests verify business logic (validation, error cases)
- DAL tests already exist; service tests focus on orchestration
- Template loading tests use bundled templates (no mocking needed)
- Asset services tested with `tempfile` for disk operations

## Design Decisions (Finalized)

### Database as Source of Truth

**Documents are stored entirely in the database** - no file system storage for campaign/module documents. The existing `documents` table schema:

```rust
pub struct Document {
    pub id: String,
    pub campaign_id: String,
    pub module_id: Option<String>,
    pub title: String,
    pub content: String,        // Full markdown content
    pub doc_type: String,       // e.g., "campaign_pitch", "module_overview"
    pub created_at: String,
    pub updated_at: String,
}
```

### No Tera Templating

Templates are **static markdown guides** - no dynamic rendering required. When creating a campaign or module:
1. Read template content from disk (bundled with app)
2. Insert directly into `documents` table as initial content
3. User edits in place, no variable substitution needed

### Templates Hardcoded in Application (No Database Seeding)

Templates are **bundled static files**, not database records:

- Templates live at `docs/src/campaign-framework/06-templates/templates/` in source
- At build time, templates are embedded in the binary (via `include_str!` or similar)
- No `templates` table in database, no seeding process needed
- Template content is read from embedded resources when creating campaigns/modules
- Future path: Allow user customization by checking app data directory first, falling back to bundled templates

### All Documents Created Immediately

No progressive disclosure - all relevant templates created at once. Users delete what they don't need.

**Campaign Creation** creates 11 documents:

| Document | Purpose |
|----------|---------|
| `campaign_pitch` | One-page pitch to excite players |
| `starting_scenario` | Player-facing introduction to the opening situation |
| `world_primer` | Campaign setting foundation |
| `character_guidelines` | Help players create appropriate characters |
| `table_expectations` | Social contract and play style expectations |
| `character_integration` | Session Zero party building worksheet |
| `campaign_bible` | Master reference document (grows over time) |
| `safety_tools` | Lines, veils, and safety mechanisms |
| `house_rules` | Campaign-specific rule modifications |
| `player_secrets` | DM-only player/character information |
| `faction_overview` | Major factions and their relationships |

**Module Creation** creates 2 documents:

| Document | Purpose |
|----------|---------|
| `module_overview` | Main planning document (or type-specific variant) |
| `play_notes` | Blank document for tracking what happened during play |

Module type determines which overview template:
- `general` → `module_overview.md`
- `mystery` → `module_mystery.md`
- `dungeon` → `module_dungeon.md`
- `heist` → `module_heist.md`
- `horror` → `module_horror.md`
- `political` → `module_political.md`

**Additional Artifacts**: Both campaigns and modules support:
- "Create blank document" - user-named document with empty content
- "Upload image" - binary asset storage via `campaign_assets` table

No other templated documents for modules - users create what they need.

### Module Types Preserved

Module types guide which overview template is used:

| Type | Template |
|------|----------|
| `mystery` | module_mystery.md |
| `dungeon` | module_dungeon.md |
| `heist` | module_heist.md |
| `horror` | module_horror.md |
| `political` | module_political.md |
| (default) | module_overview.md |

### Binary Asset Storage

Binary assets (images, maps) stored on disk in app data directory:

```
{app_data_dir}/
├── images/           # Uploaded images, campaign art
└── maps/             # UVTT map files (.dd2vtt)
```

Database stores references (file paths) to these assets:
- `maps` table has `uvtt_asset_id` column referencing `campaign_assets`
- Images referenced by relative path in document content

## Schema Adjustments Required

Review of current `mimir-core` schema against design decisions:

### Current Schema Status

| Table | Status | Notes |
|-------|--------|-------|
| `campaigns` | ✓ Complete | id, name, description, archived_at, timestamps |
| `modules` | ✓ Complete | module_type not stored (transient at creation) |
| `documents` | ✓ Complete | id, campaign_id, module_id, title, content, doc_type |
| `characters` | ⚠ Needs Update | Missing race/background catalog references |
| `character_classes` | ✓ Complete | Multiclass support with subclasses |
| `character_feats` | ✓ Complete | |
| `character_inventory` | ✓ Complete | |
| `character_proficiencies` | ✓ Complete | |
| `character_spells` | ✓ Complete | |
| `module_monsters` | ✓ Complete | Catalog monster references with customization |
| `module_npcs` | ✓ Complete | Full NPC info including stat_block JSON |
| `campaign_assets` | ✓ Complete | Binary asset storage with blob_path |
| `maps` | ✓ Complete | UVTT map support with lighting_mode |

### Module Types - No Schema Change Needed

Module types are **not stored in the database**. They are passed at module creation time to select which template documents to generate. The type is a transient input, not persistent state.

**Module Type → Template Mapping** (handled in service layer):
- `general` → `module_overview.md`
- `mystery` → `module_mystery.md`
- `dungeon` → `module_dungeon.md`
- `heist` → `module_heist.md`
- `horror` → `module_horror.md`
- `political` → `module_political.md`

### Characters Table: Add Race/Background Columns

Since this is unreleased, we edit the existing migration directly (no ALTER TABLE needed).

Add to `characters` table in existing migration:

```sql
race_name TEXT,
race_source TEXT,
background_name TEXT,
background_source TEXT
```

These columns link characters to catalog entities:
- `race_name` + `race_source` → `races` table lookup
- `background_name` + `background_source` → `backgrounds` table lookup

### Model Updates Required

**`src/models/campaign/character.rs`:**
- Add `race_name: Option<String>`, `race_source: Option<String>` to `Character`
- Add `background_name: Option<String>`, `background_source: Option<String>` to `Character`
- Add corresponding fields to `NewCharacter` and `UpdateCharacter`
- Add `with_race()` and `with_background()` builder methods

## Implementation Plan

### Phase 1: Foundation
- Create `src/services/mod.rs` module structure
- Implement `ServiceError` type
- Implement `CatalogService` trait and first implementations (MonsterService, SpellService)

### Phase 2: Campaign Services  
- `CampaignService` - campaign CRUD, creates initial documents from embedded templates
- `ModuleService` - module CRUD, creates documents based on module type
- `DocumentService` - document CRUD (list, get, update, delete)
- `templates` module - exposes embedded markdown templates via `include_str!`

### Phase 3: Character Services
- `CharacterService` - character CRUD, inventory management
- Integration with catalog for class/race lookup

### Phase 4: Asset Services
- `MapService` - UVTT map upload, storage, retrieval
- Image upload utilities for campaign/module assets

### Dependencies
- Templates at `docs/src/campaign-framework/06-templates/templates/` (embedded at compile time)
- Existing DAL layer for database operations
- No additional crate dependencies (removed Tera, no template seeding)