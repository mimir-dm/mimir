# Mimir Architecture

Architecture documentation following the [C4 model](https://c4model.com/): Context → Containers → Components → Code.

## Level 1: System Context

Mimir is a local-first desktop application for D&D 5e campaign management. It has no cloud dependencies — all data stays on the user's machine.

```
                                    ┌─────────────────────┐
                                    │    Dungeon Master    │
                                    │      (User)          │
                                    └──────────┬──────────┘
                                               │
                          ┌────────────────────┼────────────────────┐
                          │                    │                    │
                          ▼                    ▼                    ▼
                   ┌─────────────┐     ┌─────────────┐     ┌─────────────┐
                   │   Mimir     │     │  Claude Code │     │ Dungeondraft│
                   │ Desktop App │     │  (via MCP)   │     │  (Map Tool) │
                   └──────┬──────┘     └──────┬──────┘     └──────┬──────┘
                          │                   │                    │
                          │                   │                    │
                          ▼                   ▼                    ▼
                   ┌─────────────┐     ┌─────────────┐     ┌─────────────┐
                   │   SQLite    │     │  MCP Sidecar │     │ .dd2vtt /   │
                   │  Database   │◀────│   Server     │     │ .uvtt files │
                   └─────────────┘     └─────────────┘     └─────────────┘
```

**Users:** Dungeon Masters preparing and running D&D 5e campaigns.

**External systems:**
- **Claude Code** — AI assistant that manages campaigns via the MCP sidecar server
- **Dungeondraft** — Map authoring tool; Mimir imports UVTT maps and generates `.dungeondraft_map` files
- **5etools** — Source of D&D 5e catalog data (monsters, spells, items, etc.), imported into the database at build time

## Level 2: Container Diagram

The system is a Cargo workspace with 5 crates, a Vue 3 frontend, and a SQLite database.

```
┌──────────────────────────────────────────────────────────┐
│                    Mimir Desktop App                      │
│  ┌────────────────────────┐  ┌────────────────────────┐  │
│  │   Vue 3 Frontend       │  │   Tauri Shell (mimir)  │  │
│  │   TypeScript + Pinia   │──│   Rust command handlers │  │
│  │                        │  │                        │  │
│  │   4 feature modules:   │  │   22 command modules   │  │
│  │   campaigns, modules,  │  │   AppState management  │  │
│  │   characters, sources  │  │                        │  │
│  └────────────────────────┘  └───────────┬────────────┘  │
│                                          │               │
│              ┌───────────────────────────┼───────────┐   │
│              │                           │           │   │
│              ▼                           ▼           │   │
│  ┌────────────────────────┐  ┌──────────────────┐    │   │
│  │   mimir-core           │  │   mimir-print    │    │   │
│  │   Domain logic, DAL,   │  │   Typst-based    │    │   │
│  │   services, migrations │  │   PDF generation │    │   │
│  └───────────┬────────────┘  └──────────────────┘    │   │
│              │                                       │   │
│              ▼                                       │   │
│  ┌────────────────────────┐                          │   │
│  │   SQLite Database      │                          │   │
│  │   56 tables, WAL mode  │                          │   │
│  └────────────────────────┘                          │   │
└──────────────────────────────────────────────────────┘   │
                                                           │
┌──────────────────────────────────────────────────────┐   │
│   MCP Sidecar (mimir-mcp)                            │   │
│   Separate binary, launched by Tauri externalBin     │◀──┘
│   142 tools across 10 categories                     │
│   Depends on: mimir-core, mimir-mapgen               │
└──────────────────────────────────────────────────────┘

┌──────────────────────────────────────────────────────┐
│   Map Generator CLI (mimir-mapgen)                   │
│   Standalone binary + library                        │
│   Procedural terrain, rooms, polygons → Dungeondraft │
│   No database dependency                             │
└──────────────────────────────────────────────────────┘
```

### Container responsibilities

| Container | Technology | Purpose |
|-----------|-----------|---------|
| **Vue 3 Frontend** | TypeScript, Pinia, Tiptap, Tailwind | UI: campaign dashboard, character sheets, play mode, player display |
| **Tauri Shell (mimir)** | Rust, Tauri 2 | IPC command handlers, AppState, window management |
| **mimir-core** | Rust, Diesel ORM | Domain models, services, data access layer, migrations |
| **mimir-print** | Rust, Typst | PDF generation: character sheets, spell cards, map prints |
| **mimir-mcp** | Rust, rust-mcp-sdk | MCP server exposing campaign management to AI assistants |
| **mimir-mapgen** | Rust, noise-rs, clap | Procedural map generation with YAML config or biome presets |
| **SQLite Database** | SQLite 3, WAL mode | 56 tables: campaigns, characters, maps, D&D 5e catalog |

### Crate dependency graph

```
mimir ──────► mimir-core
  │
  └─────────► mimir-print

mimir-mcp ──► mimir-core
  │
  └─────────► mimir-mapgen

(mimir-core, mimir-print, mimir-mapgen have no internal crate dependencies)
```

## Level 3: Component Diagrams

### mimir-core — Domain Logic

The shared library containing all business logic and data access.

```
┌─────────────────────────────────────────────────────────┐
│                        mimir-core                        │
│                                                          │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐  │
│  │   Models     │  │   Services   │  │    DAL       │  │
│  │              │  │              │  │              │  │
│  │  campaign/   │  │  Campaign    │  │  Repository  │  │
│  │  catalog/    │◀─│  Character   │──│  traits      │  │
│  │  character/  │  │  Module      │  │              │  │
│  │              │  │  Document    │  │  Diesel      │  │
│  │  Domain      │  │  Map, Token  │  │  queries     │  │
│  │  types and   │  │  Archive     │  │              │  │
│  │  enums       │  │  Asset       │  └──────────────┘  │
│  │              │  │  Homebrew    │                     │
│  └──────────────┘  │              │  ┌──────────────┐  │
│                    │  24 catalog  │  │  Migrations  │  │
│                    │  services    │  │  29 migrations│  │
│                    └──────────────┘  └──────────────┘  │
└─────────────────────────────────────────────────────────┘
```

**Service layer pattern** (per [ADR-0005](/.metis/adr/MIMIR-A-0005.md)):

Services are stateful structs that borrow a database connection:

```rust
pub struct CampaignService<'a> {
    pub conn: &'a mut SqliteConnection,
}

impl<'a> CampaignService<'a> {
    pub fn new(conn: &'a mut SqliteConnection) -> Self {
        Self { conn }
    }

    pub fn create(&mut self, input: CreateCampaignInput) -> ServiceResult<Campaign> {
        // Business logic + database access
    }
}
```

**Service inventory:**

| Category | Services |
|----------|----------|
| Campaign | CampaignService, ModuleService, DocumentService, ArchiveService, AssetService |
| Character | CharacterService, HomebrewService |
| Map | MapService, TokenService |
| Catalog (24) | SpellService, MonsterService, ItemService, ClassService, RaceService, BackgroundService, FeatService, ConditionService, SubclassService, and 15 more |

### Tauri Shell — Command Layer

The `mimir` crate bridges the Vue frontend to core services via Tauri IPC commands.

```
┌──────────────────────────────────────────────────────────┐
│                      mimir (Tauri)                        │
│                                                           │
│  ┌─────────────┐    ┌──────────────────────────────────┐ │
│  │  AppState    │    │         Commands (22 modules)    │ │
│  │             │    │                                  │ │
│  │  AppPaths   │    │  campaign     character    map   │ │
│  │  (data_dir, │◀───│  module       document     print │ │
│  │   db_path,  │    │  archive      homebrew*3   token │ │
│  │   is_dev)   │    │  asset        source       dev   │ │
│  │             │    │  dm_map       player_display     │ │
│  └─────────────┘    │  catalog/ (8 sub-modules)        │ │
│                     └──────────────────────────────────┘ │
│                                                           │
│  ┌──────────────────────────────────────────────────────┐ │
│  │  Sidecar Management                                  │ │
│  │  Launches mimir-mcp as externalBin process           │ │
│  │  Configured in tauri.conf.json                       │ │
│  └──────────────────────────────────────────────────────┘ │
└──────────────────────────────────────────────────────────┘
```

**Request flow:**

```
Vue Component
    │  invoke('get_campaign', { id: 42 })
    ▼
Tauri IPC
    │  Deserialize params, inject State<AppState>
    ▼
Command Handler (commands/campaign.rs)
    │  let mut conn = state.get_connection()?;
    │  let mut svc = CampaignService::new(&mut conn);
    ▼
CampaignService (mimir-core)
    │  svc.get_by_id(42)?
    ▼
Diesel Query → SQLite
    │
    ▼
Result<ApiResponse<Campaign>, ApiError> → JSON → Frontend
```

### Vue 3 Frontend

Feature-based organization with 4 domain modules.

```
frontend/src/
├── app/                    # App bootstrap, router, root layout
├── features/
│   ├── campaigns/          # Dashboard, module listing, homebrew tab
│   │   ├── views/          # CampaignDashboardView, etc.
│   │   ├── components/     # WorldTab, ModulesTab, PCsTab, NPCsTab, HomebrewTab
│   │   └── composables/    # useDashboardState
│   ├── characters/         # Creation wizard, character sheet, level-up
│   │   ├── views/          # CharacterListView, CharacterSheetView
│   │   └── components/     # CharacterCreationWizard, LevelUpDialog
│   ├── modules/            # Module prep, play mode, player display
│   │   ├── views/          # ModulePrepView, ModulePlayView
│   │   └── components/     # ModuleMonsters, ModuleNPCs, ModuleMaps
│   └── sources/            # D&D 5e catalog browser
│       └── composables/    # useCatalogSearch (instantiated 22×)
├── components/             # Shared: TokenPalette, DmMapViewer, print dialogs
├── stores/                 # Pinia: campaigns, characters, theme
├── services/               # Tauri command wrappers
└── types/                  # TypeScript interfaces (generated from Rust)
```

**Key architectural patterns:**
- Composition API with `<script setup>` exclusively
- Pinia setup stores (function-based, not options-based)
- Tauri `invoke()` for all backend communication
- `ApiResponse<T>` wrapper for consistent error handling
- 3 themes: light, dark, hyper (CSS variables + Pinia persistence)

### mimir-mcp — MCP Sidecar Server

Runs as a separate process, sharing the same SQLite database.

```
┌──────────────────────────────────────────────────────────┐
│                       mimir-mcp                           │
│                                                           │
│  ┌──────────────┐    ┌──────────────────────────────────┐ │
│  │  MCP Server   │    │     Tools (142 total)            │ │
│  │              │    │                                  │ │
│  │  rust-mcp-   │    │  character (26)   campaign (20) │ │
│  │  sdk 0.8     │    │  catalog (16)     map (16)      │ │
│  │              │    │  module (16)      document (12) │ │
│  │  stdio +     │◀───│  homebrew (10)    h_monster (10)│ │
│  │  streamable  │    │  h_spell (10)     mapgen (6)    │ │
│  │  HTTP        │    │                                  │ │
│  └──────────────┘    └──────────────────────────────────┘ │
│                                                           │
│  ┌──────────────┐    ┌──────────────────────────────────┐ │
│  │  Database     │    │  Claude Code Plugin              │ │
│  │  Context      │    │  plugin/ directory with:         │ │
│  │              │    │  - plugin.json (manifest)        │ │
│  │  Shares same  │    │  - skills/ (mapgen, mimir-dm)   │ │
│  │  SQLite DB    │    │  - agents/ (campaign authoring)  │ │
│  │  as Tauri app │    │  - hooks/ (event handlers)       │ │
│  └──────────────┘    └──────────────────────────────────┘ │
└──────────────────────────────────────────────────────────┘
```

**Sidecar lifecycle:**
1. Tauri launches `mimir-mcp` binary via `externalBin` configuration
2. Binary built to `crates/mimir/binaries/mimir-mcp-{target-triple}`
3. Build scripts: `scripts/build-sidecar.{sh,ps1,mjs}`
4. Must be built before `cargo build -p mimir` (Tauri requirement)

### mimir-mapgen — Procedural Map Generator

Standalone library + CLI for generating Dungeondraft-compatible maps.

```
┌──────────────────────────────────────────────────────────┐
│                      mimir-mapgen                         │
│                                                           │
│  CLI Interface (clap)          Pipeline                   │
│  ┌──────────────────┐         ┌──────────────────────┐   │
│  │ generate          │         │  YAML Config         │   │
│  │   --config / --preset      │       │              │   │
│  │   --output         │────────│       ▼              │   │
│  │   --seed           │        │  Noise Generation    │   │
│  │ validate           │        │       │              │   │
│  │ list-presets        │        │       ▼              │   │
│  └──────────────────┘         │  Terrain Blending    │   │
│                               │       │              │   │
│  Core Modules (15)            │       ▼              │   │
│  ┌──────────────────┐         │  Object Placement    │   │
│  │ noise_gen         │         │  (trees, clutter)    │   │
│  │ terrain           │         │       │              │   │
│  │ objects            │         │       ▼              │   │
│  │ paths (roads,      │         │  Paths & Water      │   │
│  │   rivers, meander) │         │  (exclusion zones)  │   │
│  │ water             │         │       │              │   │
│  │ rooms             │         │       ▼              │   │
│  │ polygons          │         │  Rooms / Polygons    │   │
│  │ elevation         │         │       │              │   │
│  │ contour           │         │       ▼              │   │
│  │ biomes (12        │         │  Dungeondraft        │   │
│  │   presets)         │         │  Format Output       │   │
│  │ format            │         │  (.dungeondraft_map) │   │
│  │ pipeline          │         └──────────────────────┘   │
│  │ distribution      │                                    │
│  │ curves            │                                    │
│  │ assets            │                                    │
│  └──────────────────┘                                    │
└──────────────────────────────────────────────────────────┘
```

**No database dependency.** Operates entirely on YAML config input → file output.

### mimir-print — PDF Generation

Typst-based document builder with pluggable sections.

```
┌──────────────────────────────────────────────────────────┐
│                       mimir-print                         │
│                                                           │
│  ┌──────────────┐    ┌──────────────────────────────────┐ │
│  │ PrintService  │    │  Renderable Sections             │ │
│  │              │    │                                  │ │
│  │ Template     │    │  CharacterSection (2-page sheet) │ │
│  │ resolution + │    │  CharacterBattleCardSection      │ │
│  │ PDF compile  │    │  SpellCardsSection (3×3 grid)    │ │
│  │              │    │  EquipmentCardsSection            │ │
│  └──────────────┘    │  MonsterCardSection               │ │
│                      │  TrapCardSection                  │ │
│  ┌──────────────┐    │  TiledMapSection + MapPreview    │ │
│  │ DocumentBuilder   │  TokenCutoutSection               │ │
│  │              │    │  MarkdownSection                  │ │
│  │ Assembles    │    └──────────────────────────────────┘ │
│  │ sections into│                                        │
│  │ multi-part   │    ┌──────────────────────────────────┐ │
│  │ documents    │    │  Support Modules                  │ │
│  └──────────────┘    │  markdown → Typst converter      │ │
│                      │  map_renderer (grid, LOS, tokens) │ │
│                      │  Typst World (font/file resolver) │ │
│                      └──────────────────────────────────┘ │
└──────────────────────────────────────────────────────────┘
```

## Level 4: Key Design Decisions

Detailed rationale lives in ADRs. Summary:

| Decision | ADR | Rationale |
|----------|-----|-----------|
| Denormalized catalog data | [ADR-0001](/.metis/adr/MIMIR-A-0001.md) | Child entities (subclasses, subraces) embed full parent data to avoid joins on read |
| Multi-ruleset content | [ADR-0002](/.metis/adr/MIMIR-A-0002.md) | Content organized by rule system and source for future expansion |
| JSON columns | [ADR-0003](/.metis/adr/MIMIR-A-0003.md) | Variable/complex fields (spell components, monster traits) stored as JSON |
| Stateful services | [ADR-0005](/.metis/adr/MIMIR-A-0005.md) | Services borrow `&mut SqliteConnection` for lifetime-scoped transactions |

### Database Schema (56 tables)

| Category | Tables | Examples |
|----------|--------|----------|
| Campaign (7) | campaigns, modules, documents, campaign_sources, campaign_assets, campaign_homebrew_* | Core user data |
| Character (8) | characters, character_classes, character_spells, character_inventory, character_feats, character_features, character_proficiencies, character_sources | PC/NPC data |
| Map & Tokens (8) | maps, token_placements, light_sources, fog_revealed_areas, map_pois, map_traps, module_monsters, module_npcs | Spatial data |
| D&D Catalog (28) | spells, monsters, items, classes, races, backgrounds, feats, conditions, subclasses, etc. | Immutable 5etools data |
| Management (5) | catalog_sources, books, spell_classes, spell_subclasses, item_attunement_classes | Relationships & metadata |

### Data Paths

| Environment | Database | Assets |
|------------|----------|--------|
| **Production** | `~/Library/Application Support/com.mimir.app/data/mimir.db` | `~/Library/Application Support/com.mimir.app/assets/` |
| **Development** | `~/Library/Application Support/com.mimir.app/dev/data/mimir.db` | `~/Library/Application Support/com.mimir.app/dev/assets/` |

Dev mode activated by debug assertions or `MIMIR_DEV` environment variable. Paths shown are macOS; Linux uses `~/.local/share/`, Windows uses `%APPDATA%`.

## Directory Structure

```
mimir/
├── Cargo.toml                     # Workspace manifest (5 crates)
├── crates/
│   ├── mimir/                     # Tauri desktop application
│   │   ├── src/
│   │   │   ├── main.rs            # Entry point, plugin registration
│   │   │   ├── state.rs           # AppState, AppPaths, dev/prod separation
│   │   │   └── commands/          # 22 Tauri command modules
│   │   │       └── catalog/       # 8 sub-modules for D&D catalog queries
│   │   ├── frontend/              # Vue 3 + TypeScript application
│   │   │   └── src/
│   │   │       ├── features/      # campaigns/, characters/, modules/, sources/
│   │   │       ├── components/    # Shared: TokenPalette, DmMapViewer, print dialogs
│   │   │       └── stores/        # Pinia: campaigns, characters, theme
│   │   ├── binaries/              # Sidecar output (mimir-mcp-{target})
│   │   └── tauri.conf.json        # Tauri config with externalBin sidecar
│   │
│   ├── mimir-core/                # Domain logic library
│   │   └── src/
│   │       ├── models/            # Domain types (catalog/, campaign/)
│   │       ├── services/          # 9 core + 24 catalog services
│   │       │   └── catalog/       # Spell, Monster, Item, Class, Race, etc.
│   │       ├── dal/               # Data access layer
│   │       └── migrations/        # 29 Diesel migrations
│   │
│   ├── mimir-mcp/                 # MCP sidecar server
│   │   ├── src/
│   │   │   ├── main.rs            # Server bootstrap (stdio + HTTP)
│   │   │   └── tools/             # 10 tool modules (142 tools total)
│   │   └── plugin/                # Claude Code plugin definition
│   │       ├── plugin.json        # Manifest
│   │       ├── skills/            # mapgen, mimir-dm, mimir-campaign
│   │       └── agents/            # Campaign authoring agent
│   │
│   ├── mimir-print/               # PDF generation library
│   │   └── src/
│   │       ├── service.rs         # PrintService
│   │       ├── builder.rs         # DocumentBuilder
│   │       ├── sections/          # 9 renderable section types
│   │       └── map_renderer.rs    # Map image rendering
│   │
│   └── mimir-mapgen/              # Map generation library + CLI
│       ├── src/
│       │   ├── main.rs            # CLI (generate, validate, list-presets)
│       │   ├── pipeline.rs        # MapConfig, generation orchestration
│       │   ├── biomes.rs          # 12 biome presets
│       │   └── (13 more modules)  # noise, terrain, paths, water, rooms, etc.
│       └── examples/              # YAML configs + generated maps
│
├── docs/                          # mdBook documentation
├── scripts/                       # Build scripts (sidecar, CI)
└── .metis/                        # Project management (ADRs, initiatives)
```
