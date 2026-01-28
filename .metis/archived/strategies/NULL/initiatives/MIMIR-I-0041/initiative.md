---
id: mimir-v0-5-architecture-rewrite
level: initiative
title: "Mimir v0.5 Architecture Rewrite"
short_code: "MIMIR-I-0041"
created_at: 2026-01-19T22:03:08.110297+00:00
updated_at: 2026-01-28T20:54:32.145751+00:00
parent: MIMIR-V-0001
blocked_by: []
archived: true

tags:
  - "#initiative"
  - "#phase/completed"


exit_criteria_met: false
estimated_complexity: XL
strategy_id: NULL
initiative_id: mimir-v0-5-architecture-rewrite
---

# Mimir v0.5 Architecture Rewrite Initiative

*This template includes sections for various types of initiatives. Delete sections that don't apply to your specific use case.*

## Context

Mimir has grown organically and accumulated significant technical debt:
- Filesystem-first storage for markdown documents (no ACID compliance)
- Character versioning system adding complexity without clear value
- Session as a first-class entity (now just play notes)
- Kanban/workflow boards for campaign/module progression (unused complexity)
- Repetitive code patterns across the stack
- Frontend state scattered rather than centralized

This is a **full rewrite** embracing breaking changes to establish a clean foundation.

## Goals & Non-Goals

**Goals:**
- Database-first storage for all editable content (markdown in DB, ACID compliant)
- Simplified data model (no versioning, no sessions entity, no workflow boards)
- Backend (Rust) as single source of truth for all state
- Pinia stores in frontend for reactive state mirroring backend
- Clean, minimal codebase - only what's needed
- Preserve core functionality: campaigns, modules, characters, encounters, maps, catalog, export

**Non-Goals:**
- Backwards compatibility with v1 data (migration can be separate effort)
- Preserving existing code structure
- Feature parity day one (rebuild incrementally)
- Workflow/kanban tracking for campaigns/modules
- Character versioning/history

## Simplified Data Model

### Core Entities

```
CAMPAIGN
├── id, name, description
├── 1:N → MODULE
├── 1:N → DOCUMENT (campaign-level)
├── 1:N → CHARACTER
└── 1:N → MAP (campaign-level)

MODULE
├── id, campaign_id, name, module_number
├── 1:N → DOCUMENT (module-level)
├── 1:N → MAP (module-level)
├── 1:N → MODULE_MONSTER (encounter building)
├── 1:N → MODULE_ITEM (treasure/loot)
└── 1:N → MODULE_NPC (NPC assignments)

CHARACTER
├── id, campaign_id, name, is_npc
├── Normalized stats (no JSON blob, no versioning)
├── abilities, hp, ac, speed, proficiencies
├── inventory[], currency
└── NPC fields: role, location, faction, notes

DOCUMENT
├── id, campaign_id, module_id (optional)
├── title, content (markdown stored in DB)
└── No file_path, no workflow status

MAP
├── id, campaign_id, module_id (optional)
├── name, image_path (binary stays on filesystem)
├── grid config, ambient lighting
├── 1:N → TOKEN
├── 1:N → FOG_AREA
└── 1:N → LIGHT_SOURCE

TOKEN
├── id, map_id, name, token_type, size
├── x, y position, visibility
├── monster_id OR character_id (optional links)
└── vision_type, vision_range

CATALOG (read-only reference)
├── MONSTER, ITEM, SPELL, CLASS, RACE, etc.
└── Full 5e content with search indexes
```

### Dropped from v1
- Session entity (play notes are just documents)
- CharacterVersion (no history tracking)
- Workflow status on Campaign/Module
- Document requirements and exit criteria
- file_path on documents (content in DB)

## Architecture

### Core Principles

1. **Database-first**: All editable content (markdown) lives in SQLite. No filesystem writes for documents.
2. **Backend owns state**: Rust backend is the single source of truth. All mutations go through backend.
3. **Frontend mirrors state**: Pinia stores reflect backend state. Optimistic updates with rollback on failure.
4. **Assets on filesystem**: Binaries (images, maps) stay on disk. DB stores paths to them.
5. **Export/Import for sharing**: Campaigns serialize to portable format for DM sharing.

### Layer Diagram

```
┌─────────────────────────────────────────────────────────┐
│  MCP Server (Claude Code Plugin)                        │
│  - 25 tools: campaigns, modules, characters, catalog    │
└─────────────────────┬───────────────────────────────────┘
                      │
┌─────────────────────┴───────────────────────────────────┐
│  Tauri Commands (Desktop App)                           │
│  - Same operations as MCP, UI-specific extras           │
└─────────────────────┬───────────────────────────────────┘
                      │
┌─────────────────────┴───────────────────────────────────┐
│  Service Layer (Rust)                                   │
│  - CampaignService, ModuleService, CharacterService     │
│  - DocumentService, MapService, CatalogService          │
│  - ExportService (PDF, Markdown, Campaign blob)         │
└─────────────────────┬───────────────────────────────────┘
                      │
┌─────────────────────┴───────────────────────────────────┐
│  Repository Layer (DAL)                                 │
│  - Clean SQL queries, no ORM bloat                      │
│  - SQLite with FTS5 for full-text search                │
└─────────────────────┬───────────────────────────────────┘
                      │
┌─────────────────────┴───────────────────────────────────┐
│  Storage                                                │
│  - SQLite: campaigns, modules, characters, documents    │
│  - Filesystem: map images, token images, exports        │
└─────────────────────────────────────────────────────────┘
```

### Frontend Architecture (Vue + Pinia)

```
┌─────────────────────────────────────────────────────────┐
│  Views (Pages)                                          │
│  - CampaignView, ModuleView, CharacterView, MapEditor   │
└─────────────────────┬───────────────────────────────────┘
                      │
┌─────────────────────┴───────────────────────────────────┐
│  Pinia Stores (State)                                   │
│  - useCampaignStore, useModuleStore, useCharacterStore  │
│  - useMapStore, useCatalogStore                         │
│  - Stores own all state, components just render         │
└─────────────────────┬───────────────────────────────────┘
                      │
┌─────────────────────┴───────────────────────────────────┐
│  Tauri IPC                                              │
│  - invoke() calls to Rust backend                       │
│  - Events for backend → frontend notifications          │
└─────────────────────────────────────────────────────────┘
```

### Import/Export

**Campaign Export** (for sharing):
- Serialize campaign + modules + documents + characters to JSON
- Include map images as base64 or as separate files in ZIP
- Portable format other DMs can import

**Markdown Export**:
- Export any document as .md file
- Export all campaign documents to folder structure

**PDF Export** (preserved from v1):
- Character sheets, monster stat blocks, maps, encounters

## MCP Tools (Preserved Interface)

The 25 MCP tools remain the public API contract. Internal implementation changes but tool signatures stay compatible:

**Campaign (3)**: list_campaigns, set_active_campaign, get_campaign_details
**Module (6)**: create_module, list_modules, get_module_details, add_monster_to_module, add_item_to_module, update_module_monster
**Character (7)**: list_characters, get_character, create_character, edit_character, assign_npc_to_module, add_item_to_character, update_character_currency
**Document (6)**: list_documents, read_document, edit_document, create_user_document, create_document_from_template, list_templates
**Catalog (3)**: search_monsters, search_items, search_traps

**New tools to add:**
- `export_campaign` - Export campaign as shareable blob
- `import_campaign` - Import campaign from blob
- `export_document_markdown` - Export document as .md file

## Feature Scope

### Must Have (MVP)
- Campaign CRUD
- Module CRUD  
- Document CRUD (DB-stored markdown)
- Character CRUD (normalized, no versioning)
- Catalog search (monsters, items, traps)
- MCP tool interface
- Basic Tauri desktop shell
- LLM chat assistant (consumes MCP tools - same interface as Claude Code)

### Should Have
- Map editor with tokens, fog, lighting
- PDF export (character sheets, stat blocks)
- Campaign import/export for sharing
- Markdown export

### Could Have (Post-MVP)
- Advanced catalog search (spells, classes, races)
- Token image management

## Alternatives Considered

**Incremental refactor vs full rewrite**: Rejected incremental approach because the architectural changes (DB-first, no versioning, simplified model) touch every layer. Easier to build clean than migrate piecemeal.

**Keep filesystem for documents**: Rejected because ACID compliance matters for data integrity. DB-first with export capability gives us both reliability and portability.

**ORM (SeaORM, Diesel)**: Rejected in favor of raw SQL with sqlx. Simpler, more control, less magic. The schema is straightforward enough that an ORM adds complexity without benefit.

## Implementation Plan

### Phase 1: Planning Documents
Create detailed design documents before writing code:
1. **Database Schema Design** - Full DDL for v2 schema
2. **Service Layer API Design** - Rust trait definitions for each service
3. **MCP Tool Specification** - Finalized tool signatures with examples
4. **Pinia Store Design** - Store structure, actions, state shape
5. **Migration Strategy** - How to move v1 data to v2 (separate from rewrite)

### Phase 2: Foundation
- New crate structure (clean workspace)
- Database schema + migrations
- Repository layer with tests
- Service layer skeleton

### Phase 3: Core Features
- Campaign/Module/Document CRUD
- Character CRUD (simplified)
- Catalog search
- MCP server with core tools

### Phase 4: Frontend
- Tauri shell
- Pinia stores
- Basic views (campaigns, modules, documents, characters)

### Phase 5: Advanced Features
- Map editor
- PDF export
- Campaign import/export

## Progress

*To be updated as work progresses*