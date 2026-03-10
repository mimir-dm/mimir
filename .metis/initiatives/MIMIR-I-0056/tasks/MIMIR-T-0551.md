---
id: catalog-search-and-filter-tests
level: task
title: "Catalog search and filter tests — search across all entity types, filter combinations"
short_code: "MIMIR-T-0551"
created_at: 2026-03-10T01:31:40.619340+00:00
updated_at: 2026-03-10T01:31:40.619340+00:00
parent: MIMIR-I-0056
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/todo"


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

*To be added during implementation*