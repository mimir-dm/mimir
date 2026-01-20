# mimir-dm-core

## Purpose & Boundaries

The `mimir-dm-core` crate is the heart of the Mimir D&D Campaign Assistant. It provides the core business logic, domain models, and data persistence layer. This crate implements domain-driven design patterns with a clear separation between D&D rules reference data and campaign management systems.

### Responsibilities

- **Domain Models**: Core entities for both rules (D&D 5e) and campaigns
- **Business Services**: High-level operations for campaign and template management
- **Data Persistence**: SQLite database with Diesel ORM, including migrations
- **Workflow Management**: Board definitions and state transitions for campaigns/modules/sessions
- **Document System**: Template engine and document generation with frontmatter support
- **Search Infrastructure**: Full-text search (FTS5) and vector similarity search
- **Data Seeding**: Initial template and reference data population

### What This Crate Does NOT Do

- No UI or presentation logic
- No LLM inference (depends on mimir-dm-llm for embeddings only)
- No agent orchestration
- No network operations beyond database connections
- No direct user interaction

## Architecture

The crate is organized into two distinct domains:

### 1. Catalog Domain (`models::catalog`, `services::*_service`)
Comprehensive D&D 5e reference data with database-backed services:
- **20+ Content Categories** - Classes, races, backgrounds, spells, items, monsters, etc.
- **Multiple Source Books** - PHB, DMG, MM, Xanathar's, Tasha's, and more
- **Advanced Search** - Full-text and vector similarity search
- **Rich Metadata** - Sources, types, levels, classifications
- **Service Layer** - Individual services for each content category

### 2. Campaign Domain (`models::campaign`, `dal::campaign`)
Dynamic campaign management and story organization:
- Campaigns and their lifecycle
- Modules (story arcs)
- Sessions (individual game sessions)
- Documents and templates
- Workflow cards and board states

## Layout

```
src/
├── lib.rs                 # Crate root with public exports
├── connection.rs          # Database connection management
├── schema.rs              # Diesel schema definitions (generated)
├── error.rs               # Error types and Result type alias
│
├── models/                # Domain models (split by domain)
│   ├── catalog/          # D&D reference data models (20+ types)
│   │   ├── action.rs          # Actions and activities
│   │   ├── background.rs      # Character backgrounds
│   │   ├── book.rs            # Source books and publications
│   │   ├── class.rs           # Classes, subclasses, and features
│   │   ├── condition.rs       # Conditions and effects
│   │   ├── cult.rs            # Cults and organizations
│   │   ├── deity.rs           # Deities and pantheons
│   │   ├── feat.rs            # Feats and abilities
│   │   ├── item.rs            # Equipment and magic items
│   │   ├── language.rs        # Languages and scripts
│   │   ├── monster.rs         # Monsters and NPCs
│   │   ├── object.rs          # Objects and structures
│   │   ├── optionalfeature.rs # Optional features and variants
│   │   ├── psionic.rs         # Psionic abilities
│   │   ├── race.rs            # Races and lineages
│   │   ├── reward.rs          # Rewards and treasures
│   │   ├── spell.rs           # Spells and magic
│   │   ├── trap.rs            # Traps and hazards
│   │   ├── variant_rule.rs    # Variant rules
│   │   ├── vehicle.rs         # Vehicles and mounts
│   │   └── mod.rs             # Module exports
│   └── campaign/         # Campaign management models
│       ├── campaigns.rs  # Campaign lifecycle
│       ├── modules.rs    # Story arc management
│       ├── sessions.rs   # Game session tracking
│       ├── documents.rs  # Campaign documents
│       └── ...
│
├── dal/                   # Data Access Layer (Repository pattern)
│   ├── traits.rs         # Repository trait definitions
│   ├── catalog/         # Repositories for catalog data
│   └── campaign/        # Repositories for campaign data
│
├── domain/               # Domain logic and business rules
│   └── boards/          # Workflow board definitions
│       ├── campaign_board.rs
│       ├── module_board.rs
│       └── session_board.rs
│
├── services/             # Business service layer (26 services)
│   ├── campaign_service.rs   # Campaign lifecycle operations
│   ├── template_service.rs   # Document and template generation
│   ├── catalog_service.rs    # Central catalog management
│   ├── module_service.rs     # Module and session management
│   ├── *_service.rs          # Individual catalog services (20+ files)
│   │   ├── action_service.rs      # Actions and activities
│   │   ├── background_service.rs  # Character backgrounds
│   │   ├── class_service.rs       # Classes and subclasses
│   │   ├── condition_service.rs   # Conditions and effects
│   │   ├── deity_service.rs       # Deities and pantheons
│   │   ├── feat_service.rs        # Feats and abilities
│   │   ├── item_service.rs        # Items and equipment
│   │   ├── language_service.rs    # Languages and scripts
│   │   ├── monster_service.rs     # Monsters and NPCs
│   │   ├── spell_service.rs       # Spells and magic
│   │   └── ... (additional services)
│
├── seed/                 # Data initialization
│   ├── template_loader.rs   # Load templates from filesystem
│   └── template_seeder.rs   # Seed initial templates
│
└── migrations/           # Diesel SQL migrations
```

## Key Features

### Domain-Driven Design
- Clear separation between catalog reference and campaign management
- Rich domain models with business logic
- Repository pattern for data access
- Service layer for complex operations

### Database Infrastructure
- **SQLite** with **Diesel ORM** for type-safe queries
- Automatic schema migrations
- Connection pooling
- Transaction support

### Search Capabilities
- **Full-Text Search (FTS5)** - Fast text queries across catalog content
- **Vector Similarity Search** - sqlite-vec integration for semantic search
- **Hybrid Search Strategy** - Combines text and vector search for optimal results
- **Embedding Storage** - Integrated with LLM providers for content embeddings
- **Advanced Filtering** - Search by source, type, level, and other attributes

### Workflow System
- Board definitions for campaigns, modules, and sessions
- Stage transitions with validation
- Required document tracking
- Progress monitoring

### Template Engine
- Markdown templates with YAML frontmatter
- Variable substitution using Tera
- Template versioning and validation
- Automatic document generation



## Usage

```rust
use mimir_dm_core::{
    establish_connection, 
    run_migrations,
    services::CampaignService,
};

// Initialize database
let mut conn = establish_connection("path/to/db.sqlite")?;
run_migrations(&mut conn)?;

// Use services for business operations
let campaign_service = CampaignService::new();
let campaign = campaign_service.create_campaign(
    &mut conn,
    "The Lost Mines",
    "/path/to/campaigns"
)?;

// Access domain models directly
use mimir_dm_core::models::campaign::Campaign;
use mimir_dm_core::models::catalog::Class;

// Use DAL for data access
use mimir_dm_core::dal::campaign::campaigns::CampaignRepository;
let repo = CampaignRepository::new(&mut conn);
let all_campaigns = repo.list()?;
```

## Testing

The crate includes comprehensive tests:
- Unit tests for domain models
- Integration tests for repositories
- Service layer tests with test databases
- Template validation tests

Run tests with:
```bash
cargo test -p mimir-dm-core
```

