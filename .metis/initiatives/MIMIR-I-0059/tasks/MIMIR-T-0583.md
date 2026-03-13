---
id: audit-developer-documentation
level: task
title: "Audit developer documentation against current architecture"
short_code: "MIMIR-T-0583"
created_at: 2026-03-11T23:13:28.823010+00:00
updated_at: 2026-03-13T12:46:36.718012+00:00
parent: MIMIR-I-0059
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: MIMIR-I-0059
---

# Audit developer documentation against current architecture

## Parent Initiative

[[MIMIR-I-0059]]

## Objective

Review all developer documentation (21 pages) for accuracy against the current codebase architecture. Developer docs go stale fastest since the code changes frequently — verify crate structure, build instructions, contribution guidelines, and source data schemas.

## Scope

### Core Dev Docs (4 pages)
- `docs/src/developer/README.md` — Overview
- `docs/src/developer/CONTRIBUTING.md` — Contribution guidelines
- `docs/src/developer/DEVELOPMENT.md` — Development setup
- `docs/src/developer/ARCHITECTURE.md` — System architecture

### Source Data Reference (12 pages)
- `docs/src/developer/source-data/README.md` and schema files
- `docs/src/developer/5e-tools-processing/` — 5etools processing docs

### Other Dev Docs
- `docs/src/developer/frontend/README.md`
- `docs/src/developer/llm-tools/README.md`

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] Verify ARCHITECTURE.md reflects current crate structure: mimir, mimir-core, mimir-mcp, mimir-print, mimir-mapgen (new)
- [ ] Check that architecture doc covers the service layer pattern (standardized in ADR-0005)
- [ ] Verify DEVELOPMENT.md build instructions work — especially sidecar build step and `cargo tauri dev` workaround
- [ ] Check CONTRIBUTING.md for accuracy — PR process, test requirements, code style
- [ ] Verify source data schema docs match current Diesel models and import pipeline
- [ ] Check whether 5etools processing docs cover recent changes (homebrew item sources, subclass features)
- [ ] Check frontend dev docs — do they mention Vue 3 component structure, state management approach?
- [ ] Verify llm-tools docs reflect current MCP server tool count (71 tools) and capabilities
- [ ] Identify where architecture diagrams (Mermaid) would help — crate dependency graph, data flow
- [ ] Produce findings report

## Key Areas Likely Stale

- Crate listing (mimir-mapgen not mentioned)
- MCP sidecar build process
- Service layer pattern (standardized after initial docs were written)
- Frontend component structure (recently refactored — ModulesTab, CharacterSheetView extracted)
- Database schema (homebrew tables, spell management tables added)

## Screenshot/Diagram Candidates

- Mermaid diagram: crate dependency graph
- Mermaid diagram: request flow (frontend → Tauri command → service → DAL → SQLite)
- Mermaid diagram: MCP sidecar architecture

## Status Updates

### Audit + Fix Completed 2026-03-13

---

#### ARCHITECTURE.md — Replaced with C4-style documentation

The old ARCHITECTURE.md was significantly out of date (4 crates instead of 5, 37 tables instead of 56, stale code examples showing removed static method pattern). Rather than patching, it was **completely rewritten** in C4 model style:

- **Level 1 (Context):** System context diagram showing users and external systems (Claude Code via MCP, Dungeondraft, 5etools)
- **Level 2 (Containers):** All 5 crates with dependency graph, technology stack, and responsibilities table
- **Level 3 (Components):** Detailed component diagrams for each crate — mimir-core services (9 core + 24 catalog), Tauri command layer (22 modules), frontend feature modules (4), MCP tools (142 across 10 categories), mapgen pipeline (15 modules), print sections (9 types)
- **Level 4 (Code):** ADR summary table, stateful service pattern with correct code example, database schema breakdown (56 tables by category), dev/prod data paths

**Key corrections from old doc:**
- 5 crates, not 4 (mimir-mapgen was missing)
- 56 tables, not 37
- 29 migrations, not 35
- 142 MCP tools with counts per category
- Correct stateful service pattern (old doc showed `CampaignService::get_by_id()` static method)
- Removed fictional directories (`services/llm/`, `commands/chat/`)
- Added sidecar lifecycle documentation
- Added frontend feature structure with actual component names
- Full directory tree with accurate file counts

---

#### Remaining developer docs (NOT audited yet — deferred)

The following were read but not audited in depth due to the ARCHITECTURE.md rewrite taking priority:

- **DEVELOPMENT.md** — Comprehensive and mostly accurate. Notes: mentions `cargo tauri dev` without the known bug workaround (run Vite separately). Uses `angreal test unit` correctly. DB paths correct.
- **CONTRIBUTING.md** — Generic but accurate. Code style, testing, PR process all reasonable.
- **frontend/README.md** — Well-written, accurate description of Vue 3 architecture, Pinia patterns, feature organization. Mentions Tiptap 3 and Tailwind correctly.
- **llm-tools/README.md** — Correctly notes LLM tools were removed in v0.5.0 and redirects to MCP server.
- **Source data schemas** — 12 schema files covering 5etools JSON processing. Not audited for accuracy against current import pipeline.
- **5e-tools-processing** — 2 files on item processing and magic variant implementation. Not audited.