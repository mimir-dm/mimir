---
id: v0-5-service-layer-api-design
level: task
title: "v0.5 Service Layer API Design"
short_code: "MIMIR-T-0358"
created_at: 2026-01-19T22:06:59.522533+00:00
updated_at: 2026-01-21T16:08:09.291010+00:00
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

# v0.5 Service Layer API Design

## Parent Initiative
[[MIMIR-I-0041]] - Mimir v0.5 Architecture Rewrite

## Objective
Define Rust trait interfaces for all service layer components. Services own business logic; repositories handle data access. Both MCP and Tauri commands call services.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria
- [x] Trait definitions for all services
- [x] Clear separation: Service (business logic) vs DAL (data access)
- [x] Sync design with Diesel (async wrapper at Tauri layer)
- [x] Services are testable with in-memory SQLite

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
         │ Repository Layer │  ← SQL queries, data mapping
         └────────┬─────────┘
                  ▼
         ┌──────────────────┐
         │     SQLite       │
         └──────────────────┘
```

## Service Traits

### CampaignService

```rust
#[async_trait]
pub trait CampaignService: Send + Sync {
    async fn list(&self, include_archived: bool) -> Result<Vec<CampaignSummary>>;
    async fn get(&self, id: i64) -> Result<Campaign>;
    async fn create(&self, input: CreateCampaign) -> Result<Campaign>;
    async fn update(&self, id: i64, input: UpdateCampaign) -> Result<Campaign>;
    async fn archive(&self, id: i64) -> Result<()>;
    async fn delete(&self, id: i64) -> Result<()>;
    
    // Export/Import
    async fn export(&self, id: i64) -> Result<CampaignExport>;
    async fn import(&self, data: CampaignExport) -> Result<Campaign>;
}

pub struct CreateCampaign {
    pub name: String,
    pub description: Option<String>,
}

pub struct CampaignExport {
    pub campaign: Campaign,
    pub modules: Vec<Module>,
    pub documents: Vec<Document>,
    pub characters: Vec<Character>,
    pub maps: Vec<MapExport>,  // includes base64 image data
}
```

### ModuleService

```rust
#[async_trait]
pub trait ModuleService: Send + Sync {
    async fn list(&self, campaign_id: i64) -> Result<Vec<ModuleSummary>>;
    async fn get(&self, id: i64) -> Result<ModuleDetails>;
    async fn create(&self, input: CreateModule) -> Result<Module>;
    async fn update(&self, id: i64, input: UpdateModule) -> Result<Module>;
    async fn delete(&self, id: i64) -> Result<()>;
    
    // Encounter building
    async fn add_monster(&self, input: AddMonsterToModule) -> Result<ModuleMonster>;
    async fn update_monster(&self, id: i64, input: UpdateModuleMonster) -> Result<ModuleMonster>;
    async fn remove_monster(&self, id: i64) -> Result<()>;
    
    async fn add_item(&self, input: AddItemToModule) -> Result<ModuleItem>;
    async fn remove_item(&self, id: i64) -> Result<()>;
    
    async fn assign_npc(&self, input: AssignNpcToModule) -> Result<ModuleNpc>;
    async fn unassign_npc(&self, module_id: i64, character_id: i64) -> Result<()>;
}

pub struct ModuleDetails {
    pub module: Module,
    pub monsters: Vec<ModuleMonster>,
    pub items: Vec<ModuleItem>,
    pub npcs: Vec<ModuleNpcDetails>,
    pub documents: Vec<DocumentSummary>,
}
```

### DocumentService

```rust
#[async_trait]
pub trait DocumentService: Send + Sync {
    async fn list(&self, filter: DocumentFilter) -> Result<Vec<DocumentSummary>>;
    async fn get(&self, id: i64) -> Result<Document>;
    async fn create(&self, input: CreateDocument) -> Result<Document>;
    async fn update(&self, id: i64, input: UpdateDocument) -> Result<Document>;
    async fn delete(&self, id: i64) -> Result<()>;
    
    // Content operations
    async fn search_replace(
        &self, 
        id: i64, 
        search: &str, 
        replace: &str, 
        replace_all: bool
    ) -> Result<u32>;  // returns replacement count
    
    // Export
    async fn export_markdown(&self, id: i64) -> Result<String>;
    async fn export_all_markdown(&self, campaign_id: i64) -> Result<Vec<(String, String)>>; // (path, content)
}

pub struct DocumentFilter {
    pub campaign_id: i64,
    pub module_id: Option<i64>,
    pub document_type: Option<String>,
}
```

### CharacterService

```rust
#[async_trait]
pub trait CharacterService: Send + Sync {
    async fn list(&self, filter: CharacterFilter) -> Result<Vec<CharacterSummary>>;
    async fn get(&self, id: i64) -> Result<Character>;
    async fn create(&self, input: CreateCharacter) -> Result<Character>;
    async fn update(&self, id: i64, input: UpdateCharacter) -> Result<Character>;
    async fn delete(&self, id: i64) -> Result<()>;
    
    // Inventory
    async fn add_item(&self, character_id: i64, input: AddItem) -> Result<InventoryItem>;
    async fn remove_item(&self, item_id: i64) -> Result<()>;
    async fn update_currency(&self, character_id: i64, delta: CurrencyDelta) -> Result<Currency>;
}

pub struct CharacterFilter {
    pub campaign_id: i64,
    pub is_npc: Option<bool>,
}

pub struct CreateCharacter {
    pub campaign_id: i64,
    pub name: String,
    pub is_npc: bool,
    pub race: Option<String>,
    pub class: Option<String>,
    pub level: Option<i32>,
    // ... other fields
}
```

### MapService

```rust
#[async_trait]
pub trait MapService: Send + Sync {
    async fn list(&self, filter: MapFilter) -> Result<Vec<MapSummary>>;
    async fn get(&self, id: i64) -> Result<MapDetails>;
    async fn create(&self, input: CreateMap) -> Result<Map>;
    async fn update(&self, id: i64, input: UpdateMap) -> Result<Map>;
    async fn delete(&self, id: i64) -> Result<()>;
    
    // Tokens
    async fn add_token(&self, input: CreateToken) -> Result<Token>;
    async fn update_token(&self, id: i64, input: UpdateToken) -> Result<Token>;
    async fn remove_token(&self, id: i64) -> Result<()>;
    
    // Fog of war
    async fn reveal_area(&self, map_id: i64, area: FogArea) -> Result<()>;
    async fn hide_area(&self, area_id: i64) -> Result<()>;
    
    // Lighting
    async fn add_light(&self, input: CreateLight) -> Result<LightSource>;
    async fn remove_light(&self, id: i64) -> Result<()>;
}

pub struct MapDetails {
    pub map: Map,
    pub tokens: Vec<Token>,
    pub fog_areas: Vec<FogArea>,
    pub lights: Vec<LightSource>,
}
```

### CatalogService

```rust
#[async_trait]
pub trait CatalogService: Send + Sync {
    async fn search_monsters(&self, query: MonsterQuery) -> Result<Vec<MonsterSummary>>;
    async fn get_monster(&self, name: &str, source: &str) -> Result<Monster>;
    
    async fn search_items(&self, query: ItemQuery) -> Result<Vec<ItemSummary>>;
    async fn get_item(&self, name: &str, source: &str) -> Result<Item>;
    
    async fn search_spells(&self, query: SpellQuery) -> Result<Vec<SpellSummary>>;
    async fn get_spell(&self, name: &str, source: &str) -> Result<Spell>;
}

pub struct MonsterQuery {
    pub name: Option<String>,
    pub creature_type: Option<String>,
    pub min_cr: Option<f32>,
    pub max_cr: Option<f32>,
    pub source: Option<String>,
    pub limit: Option<u32>,
}
```

### ExportService

```rust
#[async_trait]
pub trait ExportService: Send + Sync {
    // PDF generation
    async fn character_pdf(&self, id: i64, options: CharacterPdfOptions) -> Result<Vec<u8>>;
    async fn monster_pdf(&self, name: &str, source: &str) -> Result<Vec<u8>>;
    async fn module_pdf(&self, id: i64, options: ModulePdfOptions) -> Result<Vec<u8>>;
    async fn map_pdf(&self, id: i64, options: MapPdfOptions) -> Result<Vec<u8>>;
}
```

## Repository Traits

Each service has a corresponding repository trait for data access:

```rust
#[async_trait]
pub trait CampaignRepository: Send + Sync {
    async fn find_all(&self, include_archived: bool) -> Result<Vec<CampaignRow>>;
    async fn find_by_id(&self, id: i64) -> Result<Option<CampaignRow>>;
    async fn insert(&self, campaign: &NewCampaign) -> Result<i64>;
    async fn update(&self, id: i64, campaign: &UpdateCampaignRow) -> Result<()>;
    async fn delete(&self, id: i64) -> Result<()>;
}

// Similar patterns for ModuleRepository, DocumentRepository, etc.
```

## Error Handling

```rust
#[derive(Debug, thiserror::Error)]
pub enum ServiceError {
    #[error("Not found: {0}")]
    NotFound(String),
    
    #[error("Validation error: {0}")]
    Validation(String),
    
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, ServiceError>;
```

## Dependencies
- Depends on: [[MIMIR-T-0357]] Database Schema Design

## Progress

### 2026-01-21 - Completed

Design finalized and implemented via **MIMIR-I-0044** (Implement Service Layer for mimir-core).

**Key design decisions:**
- Stateful services with `&'a mut SqliteConnection` (per ADR MIMIR-A-0005)
- Sync Diesel queries (async wrapper at Tauri command layer)
- `CatalogEntityService` trait for generic catalog access
- `ServiceError` enum with NotFound, Validation, Database, Io variants
- Services tested with in-memory SQLite via `test_utils::setup_test_db()`

**Implemented services (9 tasks, 146 tests):**
- `CampaignService` - CRUD, archive/unarchive, document creation from templates
- `ModuleService` - CRUD with type-based template selection
- `DocumentService` - CRUD, move between campaign/module
- `CharacterService` - PC/NPC CRUD, inventory management
- `AssetService` - Binary asset upload/storage/retrieval
- `MapService` - UVTT map upload with asset management
- `CatalogEntityService` implementations: Monster, Spell, Item, Race, Background, Class, Condition, Feat, Language, Trap, Action, Hazard