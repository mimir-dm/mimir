# Mimir Architecture

This document provides a high-level overview of the Mimir codebase architecture for developers.

## Crate Overview

Mimir is organized as a Cargo workspace with 7 crates:

```
┌─────────────────────────────────────────────────────────┐
│                    mimir-dm (Main App)                  │
│                   Tauri Desktop Application             │
└─────────────────────────┬───────────────────────────────┘
                          │
        ┌─────────────────┼─────────────────┐
        ▼                 ▼                 ▼
┌───────────────┐ ┌───────────────┐ ┌───────────────┐
│ mimir-dm-core │ │ mimir-dm-llm  │ │ mimir-dm-print│
│  Core Logic   │ │  LLM Layer    │ │  PDF Export   │
└───────────────┘ └───────────────┘ └───────────────┘

Standalone Utilities:
├── mimir-dm-agent-test    (Agent test harness)
├── mimir-5etools-splitter (Data processing)
└── mimir-llm-eval         (LLM benchmarking)
```

### mimir-dm-core

The heart of the system containing:
- **Models**: Domain types for campaigns, characters, D&D catalog data
- **Services**: Business logic (28 services)
- **DAL**: Data Access Layer with repository traits
- **Migrations**: 35 Diesel migrations for SQLite schema

### mimir-dm-llm

LLM provider abstraction layer:
- **Traits**: `LlmProvider` interface for chat operations
- **Providers**: Ollama implementation (OpenAI-compatible)
- **Config**: Model configuration, rate limiting

### mimir-dm-print

PDF generation using Typst:
- Character sheets
- Campaign documents
- Markdown to Typst conversion

### mimir-dm

Tauri desktop application:
- **Commands**: 50+ Tauri command handlers
- **Services**: LLM integration, context management
- **State**: Consolidated `AppState` struct

## Request Flow

```
┌──────────┐    ┌─────────────┐    ┌─────────────┐    ┌──────────┐
│ Frontend │───▶│   Tauri     │───▶│   Service   │───▶│ Database │
│ (Vue 3)  │    │  Command    │    │   Layer     │    │ (SQLite) │
└──────────┘    └─────────────┘    └─────────────┘    └──────────┘
     ▲                │                   │
     │                ▼                   │
     │         ┌─────────────┐           │
     └─────────│  AppState   │◀──────────┘
               │  (db, llm)  │
               └─────────────┘
```

### Typical Command Flow

1. **Frontend** invokes Tauri command (e.g., `get_campaign`)
2. **Tauri Command** extracts `State<AppState>` and parameters
3. **Service** performs business logic, accesses database
4. **Response** returned to frontend as JSON

Example:
```rust
#[tauri::command]
pub async fn get_campaign(
    campaign_id: i32,
    state: State<'_, AppState>,
) -> Result<ApiResponse<Campaign>, ApiError> {
    let mut conn = state.db.get_connection()?;
    let campaign = CampaignService::get_by_id(&mut conn, campaign_id)?;
    Ok(ApiResponse::success(campaign))
}
```

## Database Design

See [ADR-0001](/.metis/adr/MIMIR-A-0001.md) for denormalized design rationale.

### Key Design Decisions

- **Denormalized**: Child entities (subclasses, subraces) contain full parent data
- **JSON Columns**: Complex/variable data stored as JSON (see [ADR-0003](/.metis/adr/MIMIR-A-0003.md))
- **Multi-Ruleset**: Content organized by rule system and source (see [ADR-0002](/.metis/adr/MIMIR-A-0002.md))

### Schema Overview (37 tables)

**Campaign Management:**
- `campaigns`, `modules`, `sessions`
- `documents`, `template_documents`
- `workflow_cards`

**D&D Catalog:**
- `catalog_*` tables for each entity type (spells, monsters, items, etc.)
- `uploaded_books`, `catalog_sources`

**Characters:**
- `players`, `characters`, `character_versions`
- `campaign_players` (links players to campaigns)

## Service Layer

Services follow the stateful pattern per [ADR-0005](/.metis/adr/MIMIR-A-0005.md):

```rust
pub struct SpellService<'a> {
    pub conn: &'a mut SqliteConnection,
}

impl<'a> SpellService<'a> {
    pub fn new(conn: &'a mut SqliteConnection) -> Self {
        Self { conn }
    }

    pub fn search(&mut self, filters: SpellFilters) -> Result<Vec<SpellSummary>> {
        // Query logic
    }
}
```

### Service Categories

| Category | Services | Purpose |
|----------|----------|---------|
| Campaign | CampaignService, ModuleService, SessionService | Campaign management |
| Catalog | SpellService, MonsterService, ItemService, etc. | D&D reference data |
| Character | CharacterService, CharacterCreationService | PC/NPC management |
| Content | DocumentService, TemplateService | Document management |

## LLM Integration

```
┌──────────────┐    ┌──────────────┐    ┌──────────────┐
│  LlmService  │───▶│ ChatProcessor│───▶│   Provider   │
│  (Tauri)     │    │  (Tools)     │    │  (Ollama)    │
└──────────────┘    └──────────────┘    └──────────────┘
                           │
                    ┌──────┴──────┐
                    │ ToolRegistry │
                    │ (15+ tools)  │
                    └─────────────┘
```

### Tool System

Tools enable the LLM to interact with the application:
- **Character Tools**: Cast spells, check slots, manage inventory
- **Query Tools**: Search spells, monsters, items
- **Campaign Tools**: Access session notes, module info

## Key Patterns

### Error Handling

```rust
// Application-level errors
pub enum MimirError {
    Database(DbError),
    Print(String),
    Llm(String),
    NotFound(String),
    // ...
}

// Database-level errors
pub enum DbError {
    NotFound { entity_type, id },
    ConstraintViolation { field, message },
    Query(diesel::result::Error),
    // ...
}
```

### State Management

`AppState` consolidates all shared state:

```rust
pub struct AppState {
    pub db: Arc<DatabaseService>,       // Database connection pool
    pub paths: Arc<AppPaths>,           // Application paths
    pub context: ContextState,          // Conversation context
    pub sessions: SessionManager,       // Chat sessions
    pub llm: Arc<Mutex<Option<LlmService>>>, // LLM service
}
```

## Directory Structure

```
crates/
├── mimir-dm/                    # Main Tauri app
│   ├── src/
│   │   ├── main.rs             # Entry point
│   │   ├── state.rs            # AppState
│   │   ├── commands/           # Tauri commands
│   │   │   ├── catalog/        # D&D reference commands
│   │   │   ├── campaign/       # Campaign management
│   │   │   ├── character/      # Character operations
│   │   │   └── chat/           # LLM chat commands
│   │   └── services/           # Business logic
│   │       ├── llm/            # LLM integration
│   │       └── tools/          # LLM tool implementations
│   └── frontend/               # Vue 3 frontend
│
├── mimir-dm-core/              # Core business logic
│   ├── src/
│   │   ├── models/             # Domain models
│   │   │   ├── catalog/        # D&D entity types
│   │   │   └── campaign/       # Campaign types
│   │   ├── services/           # Business logic services
│   │   ├── dal/                # Data access layer
│   │   └── migrations/         # Database migrations
│
├── mimir-dm-llm/               # LLM abstraction
│   └── src/
│       ├── providers/          # Provider implementations
│       └── traits/             # Provider interface
│
└── mimir-dm-print/             # PDF generation
    └── src/
        └── templates/          # Typst templates
```

## Development Guidelines

### Adding a New Catalog Entity

1. Create model in `mimir-dm-core/src/models/catalog/`
2. Add migration in `mimir-dm-core/migrations/`
3. Create service in `mimir-dm-core/src/services/`
4. Add Tauri commands in `mimir-dm/src/commands/catalog/`
5. Update schema.rs after migration

### Adding a New Tool

1. Implement tool trait in `mimir-dm/src/services/tools/`
2. Register in `ToolRegistry`
3. Add tests in corresponding `*_test.rs` file

### Testing

```bash
# Run all tests
cargo test --workspace

# Run core tests only
cargo test -p mimir-dm-core

# Run specific test
cargo test test_name
```

## Related Documentation

- [ADR-0001: Denormalized Database Design](/.metis/adr/MIMIR-A-0001.md)
- [ADR-0002: Multi-Ruleset Architecture](/.metis/adr/MIMIR-A-0002.md)
- [ADR-0003: JSON Storage Strategy](/.metis/adr/MIMIR-A-0003.md)
- [ADR-0005: Service Layer Pattern](/.metis/adr/MIMIR-A-0005.md)
