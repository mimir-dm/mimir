---
id: automated-gui-testing-to-catch
level: initiative
title: "Automated GUI testing to catch regressions and validate data flow"
short_code: "MIMIR-I-0056"
created_at: 2026-03-09T14:13:34.394669+00:00
updated_at: 2026-03-10T19:50:44.015629+00:00
parent: MIMIR-V-0001
blocked_by: []
archived: true

tags:
  - "#initiative"
  - "#phase/completed"


exit_criteria_met: false
estimated_complexity: L
initiative_id: automated-gui-testing-to-catch
---

# Automated GUI testing to catch regressions and validate data flow Initiative

## Context

We keep shipping bugs that would be caught by basic automated testing of the GUI. The character sheet alone has had multiple issues: background details not loading, class features missing, subclass features showing wrong content, equipment not rendering, homebrew items using wrong source codes. Every fix is discovered manually by clicking through the app, and there's no safety net to prevent regressions.

The app is a Tauri v2 desktop app with a Vue 3 frontend and Rust backend. The frontend calls Tauri commands (`invoke`) that return `ApiResponse<T>` objects. Most bugs occur at the boundary between backend data and frontend rendering — the data exists but the frontend fails silently, shows stale state, or doesn't handle the response correctly.

**Current test coverage:**
- Rust unit tests for services/DAL (good coverage)
- Vitest unit tests for a few frontend components (minimal)
- CI runs both Rust and frontend tests
- Zero integration tests that verify the full invoke → render pipeline
- Zero end-to-end tests that validate what the user actually sees

## The Problem

The bugs we keep hitting fall into predictable categories:

1. **Data flow failures**: Backend returns data but frontend silently drops it (wrong field name, missing null check, error not surfaced)
2. **Catalog lookup failures**: Character references a source/name combo that doesn't resolve (PHB vs XPHB, "HB" vs "PHB" for homebrew)
3. **Feature rendering**: Subclass features missing child entries, class features not expanding, descriptions empty
4. **Cross-concern data**: Inventory items, equipment cards, print/PDF export all need the same data pipelines to work

These are all testable without a full browser — most can be caught by testing the Vue components with mocked Tauri invoke responses, and some by integration-testing the Rust commands against a test database.

## Goals

- Catch data flow bugs automatically before they reach the user
- Cover the highest-value pages: character sheet, campaign dashboard, module view
- Make it easy to add regression tests when bugs are found ("fix + test" workflow)
- Run in CI on every PR

## Non-Goals

- 100% coverage — focus on the paths that actually break
- Visual/screenshot testing (nice to have later, not the priority)
- Performance testing
- Testing the Tauri window management / native OS integration

## Testing Strategy

### Layer 1: Vue Component Tests (Vitest + Vue Test Utils)

The highest-value, lowest-cost layer. Mock `@tauri-apps/api/core` invoke calls and verify components render the right content for given data.

**What to test:**
- Character sheet tabs render correctly given known class/background/race data
- Feature list includes subclass child features (Fast Hands, Improved Critical)
- Feature expand loads correct detail (class feature vs subclass feature)
- Equipment section resolves items from both catalog and homebrew sources
- Error states display when invoke returns `{ success: false }`
- Background details render from parsed data blob
- Saving throws show proficiency bonus for proficient saves

**Approach:**
- Create test fixtures with realistic 5etools data blobs (Fighter/PHB, Thief/Rogue, etc.)
- Mock `invoke` to return `ApiResponse` fixtures for each command
- Mount components and assert DOM content

### Layer 2: Rust Integration Tests (cargo test)

Test the Tauri command layer against a real SQLite database with seeded catalog data. This catches the "data is in the DB but the query doesn't find it" class of bugs.

**What to test:**
- `get_class_by_name("Fighter", "PHB")` returns data with `classFeatures` array
- `get_background_by_name("Soldier", "PHB")` returns data with entries
- `get_subclass_feature("Fast Hands", "Thief", "PHB")` returns the feature (not the subclass intro)
- `list_subclass_features("Champion", "PHB")` returns child features like "Improved Critical"
- `get_item_by_name` with source "HB" routes to homebrew lookup
- Character enrichment (`get_character`) populates classes array correctly
- Print/export commands produce non-empty output for characters with homebrew items

### Layer 3: End-to-End Tests (WebDriver / Playwright — future)

Full browser tests against the running dev app. Higher cost, but catches real integration issues.

**Deferred** — focus on Layer 1 and 2 first, which cover 90% of the bugs we've hit.

## Alternatives Considered

1. **Only e2e tests (Playwright/WebDriver)**: Slow, flaky, hard to debug, requires running the full app. The bugs we're hitting are testable at the component and integration level.

2. **Snapshot testing**: Catches unexpected changes but doesn't validate correctness. A snapshot of broken HTML would just become the new baseline.

3. **Manual QA checklist**: This is what we're doing today and it's not working. We need automated regression prevention.

## Design Decisions

### Test Fixture Strategy
- Extract real SRD data blobs from the production database for test fixtures (Fighter, Rogue, Wizard, Cleric — all PHB/SRD content is open-source under the OGL/SRD)
- SRD-only constraint means we can safely commit these fixtures to the repo without IP concerns
- Fixtures live in dedicated test data directories (e.g., `frontend/__tests__/fixtures/`, `crates/mimir-core/tests/fixtures/`)
- Include realistic 5etools JSON blobs — not minimal stubs — so tests catch real parsing edge cases

### Test Data Seeding
- Rust integration tests use dedicated SRD-based seed data (not dev seed)
- Seed script creates a test DB with: SRD classes, subclasses, backgrounds, races, items, monsters, spells + a test campaign with characters covering key scenarios (multiclass, homebrew items, subclass features)
- Frontend component tests mock `invoke` with the same fixture data for consistency

### CI Configuration
- Frontend component tests (Vitest): fast, runs on every PR (existing `test-frontend` job)
- Rust integration tests: separate "heavy" CI job, runs **post-merge to main** only
- Unit tests continue to run on every PR as they do today

### Coverage Phasing
This is a long-running initiative. Each phase adds coverage for a major area of the app, and each phase is self-contained (shippable independently).

## Implementation Plan

### Phase 1: Test Infrastructure & Fixtures — COMPLETE
- ~~Extract SRD data blobs from production DB~~ — **MIMIR-T-0533** (12 classes, 12 subclasses, 170 class features, 89 subclass features, 7 backgrounds, 17 races, 44 items, 43 spells, 17 monsters, 4 test characters, 3 homebrew items)
- ~~Create frontend test fixture files with mock invoke responses~~ — **MIMIR-T-0534** (invoke router with persistent/one-shot/sequence/handler/reject modes, typed fixture lookups, mount helpers; 48 tests)
- ~~Create Rust integration test harness with SRD-seeded test database~~ — **MIMIR-T-0535** (in-memory SQLite, dynamic source discovery, 11 smoke tests)
- ~~Set up CI "heavy" job for post-merge integration tests~~ — **MIMIR-T-0536** (post-merge to main only, runs Rust integration + full frontend suite)

### Phase 2: Character Sheet Coverage — IN PROGRESS (3/4 tasks complete)
- ~~**CharacterStatsTab + CharacterDetailsTab**~~ — **MIMIR-T-0537** (73 tests: ability scores, combat stats, saving throws, skills, attacks, proficiencies, class features, spellcasting, personality, background, NPC details, features by level)
- ~~**EquipmentSection + SpellsSection**~~ — **MIMIR-T-0538** (43 tests: currency, equipped items, inventory, item detail caching, spellcasting stats, spell slots, available spells, spell formatting, non-spellcaster)
- ~~**CharacterSheetView integration**~~ — **MIMIR-T-0539** (25 tests: full load flow, catalog lookups, speed enrichment, class/subclass features, multiclass, tab navigation, inventory, error handling, loading state)
- **Regression tests** — **MIMIR-T-0540** (todo, not started)

**Total test coverage added: 350 tests across 16 files (141 new component/integration tests + helpers)**

### Phase 3: Campaign & Module Coverage
- **MIMIR-T-0543** — Campaign dashboard tests (campaign list, source management, character list)
- **MIMIR-T-0544** — Module view tests (module list, documents, monsters, NPCs, maps)
- **MIMIR-T-0545** — Document rendering tests (markdown, 5etools references, ordering)
- **MIMIR-T-0546** — Module CRUD tests (create, update, delete, reorder)
- **MIMIR-T-0547** — Token management tests (palette, setup, vision, light sources)

### Phase 4: Catalog & Search Coverage
- **MIMIR-T-0548** — Monster detail rendering (stat blocks, actions, legendary, lair, spellcasting)
- **MIMIR-T-0549** — Spell detail rendering (components, duration, range, school, classes)
- **MIMIR-T-0550** — Item detail rendering (weapons, armor, magic items, type codes)
- **MIMIR-T-0551** — Catalog search and filter (all entity types, debounced search)
- **MIMIR-T-0552** — Cross-reference rendering (@spell, @creature, @item, @dice, etc.)

### Phase 5: Print/Export Coverage
- **MIMIR-T-0553** — Character sheet PDF generation and content validation
- **MIMIR-T-0554** — Campaign and module document export
- **MIMIR-T-0555** — Monster cards and trap cards (catalog + homebrew)
- **MIMIR-T-0556** — Equipment cards (catalog + homebrew, type code mapping)
- **MIMIR-T-0557** — Map printing and token cutout sheets

### Phase 6: Homebrew & Advanced Features
- **MIMIR-T-0558** — Homebrew CRUD tests (item/monster/spell create/edit/delete, formToDataJson)
- **MIMIR-T-0559** — Homebrew rendering tests (display in catalog, inventory, modules)
- **MIMIR-T-0560** — Campaign archive export/import round-trip validation
- **MIMIR-T-0561** — Player display window (tokens, map, DM events, visibility)
- **MIMIR-T-0562** — Map features (fog of war, light sources, traps, POIs, UVTT parsing)
- **MIMIR-T-0563** — Level-up flow (class selection, HP, ASI/feats, subclass, spells)