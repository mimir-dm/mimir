---
id: comprehensive-test-coverage
level: initiative
title: "Comprehensive Test Coverage"
short_code: "MIMIR-I-0055"
created_at: 2026-03-08T22:47:41.255596+00:00
updated_at: 2026-03-09T01:27:30.581042+00:00
parent: MIMIR-V-0001
blocked_by: []
archived: true

tags:
  - "#initiative"
  - "#phase/completed"


exit_criteria_met: false
estimated_complexity: M
initiative_id: comprehensive-test-coverage
---

# Comprehensive Test Coverage Initiative

## Context

Mimir currently has partial test coverage — some Rust service modules (document, campaign, module, map, character) have unit tests, but others (homebrew, archive, MCP handler) have none. The frontend has a handful of service tests (DocumentService, ModuleService, PrintService, boardConfigService) but nothing for homebrew services. There is no frontend test step in CI, so frontend tests don't gate PRs.

## Goals & Non-Goals

**Goals:**
- Unit tests for all Rust service modules in mimir-core (homebrew, archive, token)
- Integration tests for MCP handler (homebrew CRUD, error cases)
- Frontend service tests for homebrew factory (items, monsters, spells)
- Frontend tests running in CI pipeline
- Coverage reporting for Rust crates

**Non-Goals:**
- Full E2E browser testing (future initiative)
- 100% coverage target — focus on critical paths and recent bug areas
- Visual regression testing

## Detailed Design

### Rust Tests
- Use existing `setup_test_db()` pattern with in-memory SQLite
- HomebrewService: CRUD for items/monsters/spells + data enrichment + validation
- ArchiveService: export/import round-trip, manifest verification
- MCP Handler: tool dispatch, error handling, active campaign requirement

### Frontend Tests
- Use Vitest + jsdom with existing `mockTauri` utilities
- Test `createHomebrewService` factory pattern (covers all 3 entity types)
- Mock `@tauri-apps/api/core` invoke calls

### CI Integration
- Add `test-frontend` job to `.github/workflows/ci.yml`
- Run `npm ci && npm test` in `crates/mimir/frontend`

## Implementation Plan

1. Rust HomebrewService unit tests (~30 tests) — partially written
2. Rust ArchiveService tests (export/import round-trip)
3. MCP handler integration tests (homebrew CRUD + error cases)
4. Frontend createHomebrewService factory tests
5. Add frontend test job to CI pipeline