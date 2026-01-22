---
id: move-composable-state-to-pinia
level: task
title: "Move composable state to Pinia stores"
short_code: "MIMIR-T-0409"
created_at: 2026-01-21T16:34:58.910824+00:00
updated_at: 2026-01-21T16:34:58.910824+00:00
parent: MIMIR-I-0045
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/todo"


exit_criteria_met: false
strategy_id: NULL
initiative_id: MIMIR-I-0045
---

# Move composable state to Pinia stores

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[MIMIR-I-0045]]

## Objective

Refactor composables that maintain independent state to instead use Pinia stores. Investigation found 30+ composables in `src/features/sources/` duplicating state that should live in stores.

## Acceptance Criteria

- [ ] Audit all composables in `src/composables/` and `src/features/*/composables/`
- [ ] Identify composables with state that belongs in stores
- [ ] Move state to appropriate Pinia stores
- [ ] Refactor composables to be thin wrappers around store access
- [ ] Keep utility composables that don't manage state (fog, tokens, vision calculations)

## Implementation Notes

### Problem
Many composables manage their own reactive state independently:
```typescript
// Current (problematic)
export function useMonsters() {
  const monsters = ref<Monster[]>([]);  // Independent state!
  const loading = ref(false);
  // ...
}
```

### Solution
Composables should access store state:
```typescript
// Target (correct)
export function useMonsters() {
  const catalogStore = useCatalogStore();
  return {
    monsters: computed(() => catalogStore.monsterResults),
    loading: computed(() => catalogStore.loading),
    search: catalogStore.searchMonsters,
  };
}
```

### Composables to Refactor
- `src/features/sources/composables/catalog/*.ts` - Move state to useCatalogStore
- Any composable with `ref()` state that duplicates store concerns

### Composables to Keep (utility/stateless)
- `useFog.ts` - Map fog calculations
- `useTokens.ts` - Token rendering logic
- `useVisibilityPolygon.ts` - Vision math
- `useVisionCalculation.ts` - LOS calculations
- `useLightSources.ts` - Lighting logic
- `useUvttMap.ts` - UVTT parsing

### Dependencies
- Blocked by: [[MIMIR-T-0405]] (store migration)

## Status Updates

*To be added during implementation*