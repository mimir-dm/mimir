---
id: audit-developer-documentation
level: task
title: "Audit developer documentation against current architecture"
short_code: "MIMIR-T-0583"
created_at: 2026-03-11T23:13:28.823010+00:00
updated_at: 2026-03-11T23:13:28.823010+00:00
parent: MIMIR-I-0059
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/todo"


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

*To be added during implementation*