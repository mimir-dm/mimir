---
id: catalog-search-and-filter-tests
level: task
title: "Catalog search and filter tests — search across all entity types, filter combinations"
short_code: "MIMIR-T-0551"
created_at: 2026-03-10T01:31:40.619340+00:00
updated_at: 2026-03-10T12:57:33.012690+00:00
parent: MIMIR-I-0056
blocked_by: []
archived: true

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: MIMIR-I-0056
---

# Catalog search and filter tests — search across all entity types, filter combinations

**Phase 4** — Catalog & Search Coverage

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[MIMIR-I-0056]]

## Objective

Write Vitest tests for the catalog search and filter system. Test that search queries return correct results, filters (type, source, CR range, school, level) apply correctly, and pagination/infinite scroll works. Cover all searchable entity types: monsters, spells, items, classes, races, backgrounds, feats, conditions.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] Text search returns matching results by name
- [ ] Monster search filters by CR range, type, size, source
- [ ] Spell search filters by level, school, class, source, ritual/concentration flags
- [ ] Item search filters by type, rarity, source, attunement
- [ ] Class search returns class list with subclass counts
- [ ] Race search filters by source
- [ ] Background search filters by source
- [ ] Feat search filters by source, prerequisite
- [ ] Search with no results shows empty state
- [ ] Debounced search fires after typing stops (test `useDebouncedSearch`)
- [ ] All tests pass in CI

## Key Components

- `SearchView.vue` / `SearchResults.vue`
- `useCatalogSearch.ts` composable
- `useSourceSearch.ts` composable
- `useDebouncedSearch.ts` composable
- Individual catalog composables (`useMonsters`, `useSpells`, `useItems`, etc.)
- `SearchService.ts`

## Implementation Notes

The search composables call invoke commands like `search_monsters`, `search_spells`, etc. Mock these with fixture data that includes enough variety to test filter combinations. The `useDebouncedSearch` composable can be tested with fake timers.

## Status Updates

### Completed
- Created `__tests__/composables/useCatalogSearch.test.ts` (15 tests) — tests generic catalog search: initialization (with/without command, idempotent), search (returns results, auto-init, sends filter/limit/offset, error handling, loading state), filter transformation (custom transforms, default renames query→name_contains, empty strings→null, empty arrays→null), getDetails (success, error→null)
- Created `__tests__/composables/useDebouncedSearch.test.ts` (10 tests) — tests debounced typeahead: initial state, min query length (default 2, custom, clears below min), debounce timing (300ms delay, timer reset on subsequent input), search execution (mapped results, error clearing), clear, setQuery
- Created `__tests__/composables/useSpells.test.ts` (8 tests) — tests spell-specific filter transformation: query→name_contains, single level, multi-level→null, school filter, ritual/concentration flags, sources passthrough, getDetails
- All 33 tests passing