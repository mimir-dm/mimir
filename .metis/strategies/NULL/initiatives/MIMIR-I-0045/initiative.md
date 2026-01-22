---
id: implement-tauri-application
level: initiative
title: "Implement Tauri Application"
short_code: "MIMIR-I-0045"
created_at: 2026-01-21T16:12:06.522184+00:00
updated_at: 2026-01-21T16:13:05.412966+00:00
parent: MIMIR-V-0001
blocked_by: []
archived: false

tags:
  - "#initiative"
  - "#phase/decompose"


exit_criteria_met: false
estimated_complexity: XL
strategy_id: NULL
initiative_id: implement-tauri-application
---

# Implement Tauri Application Initiative

## Context

The `mimir-core` service layer is now complete with:
- Campaign, Module, Document, Character services
- Asset and Map services for binary files
- Full catalog services (Monster, Spell, Item, Race, etc.)
- 146 passing tests

**Investigation Findings (T-0360, T-0363):**
The existing frontend (`mimir-dm-bu/mimir-dm/frontend/`) is ~70% aligned with v0.5 design:
- Pinia stores are well-structured and reusable (campaign 90%, character 70%)
- Vue 3 + Vite + TypeScript stack matches target
- Most views and routes exist (gaps: /documents, /maps, /catalog routes)
- Key specialized components are production-ready (DmMapViewer, token rendering)

This initiative **migrates** the existing frontend to work with `mimir-core` rather than building from scratch.

## Goals & Non-Goals

**Goals:**
- Create `mimir-tauri` crate with Tauri v2 commands wrapping `mimir-core` services
- Implement Vue 3 + Pinia frontend per designs in T-0360 and T-0363
- Support core campaign authoring workflows (campaigns, modules, characters, documents)
- Catalog search and browsing
- Map viewing (static display, no real-time VTT features yet)

**Non-Goals:**
- LLM chat assistant (deferred per user request)
- MCP server integration (deferred)
- Real-time VTT features (fog of war, token movement, lighting calculations)
- PDF export (separate initiative)
- Play state management

## Architecture

### Overview

```
┌──────────────────────────────────────────────────────────────┐
│                     Tauri Application                        │
├──────────────────────────────────────────────────────────────┤
│  Frontend (Vue 3 + TypeScript)                               │
│  ┌────────────────────────────────────────────────────────┐  │
│  │  Views          │  Stores (Pinia)  │  Components       │  │
│  │  - Campaigns    │  - campaign      │  - Cards          │  │
│  │  - Modules      │  - module        │  - Lists          │  │
│  │  - Characters   │  - character     │  - Editor         │  │
│  │  - Documents    │  - document      │  - Modals         │  │
│  │  - Maps         │  - map           │  - Navigation     │  │
│  │  - Catalog      │  - catalog       │                   │  │
│  └────────────────────────────────────────────────────────┘  │
│                            │ invoke()                        │
├────────────────────────────┼─────────────────────────────────┤
│  Rust Backend              ▼                                 │
│  ┌────────────────────────────────────────────────────────┐  │
│  │  Tauri Commands (crates/mimir-tauri/src/commands/)     │  │
│  │  - campaign.rs, module.rs, character.rs, document.rs   │  │
│  │  - map.rs, asset.rs, catalog.rs                        │  │
│  └────────────────────────────────────────────────────────┘  │
│                            │                                 │
│  ┌────────────────────────────────────────────────────────┐  │
│  │  mimir-core (crates/mimir-core/)                       │  │
│  │  - Services: Campaign, Module, Character, Document...  │  │
│  │  - DAL: Database operations                            │  │
│  │  - Models: Data structures                             │  │
│  └────────────────────────────────────────────────────────┘  │
│                            │                                 │
│  ┌────────────────────────────────────────────────────────┐  │
│  │  SQLite Database                                       │  │
│  └────────────────────────────────────────────────────────┘  │
└──────────────────────────────────────────────────────────────┘
```

### State Management

```rust
// Tauri AppState
pub struct AppState {
    pub db: Mutex<SqliteConnection>,
    pub app_data_dir: PathBuf,
}
```

Commands acquire the connection, create services, and invoke methods:

```rust
#[tauri::command]
pub fn list_campaigns(state: State<AppState>, include_archived: bool) -> Result<Vec<Campaign>, String> {
    let mut conn = state.db.lock().map_err(|e| e.to_string())?;
    let mut service = CampaignService::new(&mut conn);
    service.list(include_archived).map_err(|e| e.to_string())
}
```

## Detailed Design

### Tauri Commands by Domain

| Domain | Commands | mimir-core Service |
|--------|----------|-------------------|
| Campaign | list, get, create, update, archive, delete | CampaignService |
| Module | list, get, create, update, delete | ModuleService |
| Character | list_pcs, list_npcs, get, create, update, delete, inventory ops | CharacterService |
| Document | list, get, create, update, delete | DocumentService |
| Map | list, get, create, update, delete, read_uvtt | MapService |
| Asset | upload, get, delete, list | AssetService |
| Catalog | search_monsters, search_spells, search_items, get_* | CatalogEntityService impls |

### Frontend Structure

```
frontend/
├── src/
│   ├── main.ts
│   ├── App.vue
│   ├── router/
│   │   └── index.ts
│   ├── stores/
│   │   ├── campaign.ts
│   │   ├── module.ts
│   │   ├── character.ts
│   │   ├── document.ts
│   │   ├── map.ts
│   │   └── catalog.ts
│   ├── views/
│   │   ├── CampaignList.vue
│   │   ├── CampaignDetail.vue
│   │   ├── ModuleList.vue
│   │   ├── ModuleDetail.vue
│   │   ├── CharacterList.vue
│   │   ├── CharacterDetail.vue
│   │   ├── DocumentList.vue
│   │   ├── DocumentEditor.vue
│   │   ├── MapList.vue
│   │   ├── MapViewer.vue
│   │   └── CatalogSearch.vue
│   ├── components/
│   │   ├── shell/
│   │   ├── common/
│   │   └── domain/
│   └── types/
├── index.html
├── package.json
├── vite.config.ts
└── tsconfig.json
```

## Implementation Plan (Migration Approach)

### Phase 1: Rust Backend Setup
- Create `crates/mimir` with Tauri v2 dependencies
- Set up AppState with database connection using `mimir-core`
- Implement Tauri commands wrapping `mimir-core` services
- Run migrations on app startup

### Phase 2: Frontend Migration
- Copy existing frontend from `mimir-dm-bu/mimir-dm/frontend/`
- Update Tauri command invocations to match new command signatures
- Add ApiResponse unwrapping adapter (or update backend to return direct types)
- Verify stores work with new backend

### Phase 3: Architecture Alignment
- Add sidebar navigation per v0.5 design (currently header-only)
- Consolidate shared components to `src/components/common/`
- Move composable state into Pinia stores (30+ composables with duplicate state)
- Add missing routes: `/documents`, `/maps`, `/catalog`

### Phase 4: Cleanup & Polish
- Remove unused code from old implementation
- Update TypeScript types to match `mimir-core` models
- Test all views with new backend
- Address any data model differences

**Estimated Timeline:** 2-3 weeks for full v0.5 compliance

### Migration Priorities

**Reuse (60%):**
- All Pinia store patterns
- Tauri invocation infrastructure
- Specialized components (DmMapViewer, PlayerDisplay, tokens)
- Feature components (campaigns, modules, characters, sources)
- Services and utilities

**Refactor (40%):**
- ApiResponse wrapper handling
- App shell structure (add sidebar)
- Router configuration (top-level routes)
- Component organization (consolidate shared)
- Chat store (flatten sub-stores, deferred)

## Related Documents

- [[MIMIR-T-0360]] - Pinia Store Design
- [[MIMIR-T-0363]] - UI Architecture and Views Design
- [[MIMIR-I-0044]] - Service Layer Implementation (completed)
- [[MIMIR-A-0005]] - Service Layer Pattern Standardization