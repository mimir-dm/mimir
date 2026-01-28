---
id: consolidate-composables
level: task
title: "Consolidate Composables"
short_code: "MIMIR-T-0452"
created_at: 2026-01-28T03:54:04.912494+00:00
updated_at: 2026-01-28T04:26:03.712433+00:00
parent: MIMIR-I-0049
blocked_by: []
archived: true

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: MIMIR-I-0049
---

# Consolidate Composables

## Parent Initiative

[[MIMIR-I-0049]] - Frontend Organizational Debt Cleanup

## Objective

Merge fragmented composables from multiple locations into a single organized structure with clear subdomain groupings.

## Current State

Composables are split across 3 locations:

**`src/composables/` (10 files):**
- useTokens, useTokenVision, usePlayerDisplay, useDmMapWindow
- useFog, useLightSources, useUvttMap
- useVisibilityPolygon, useVisionCalculation, useDashboardLink

**`src/shared/composables/` (3 files):**
- useApiCall, useDataEvents, usePagination

**`src/features/{feature}/composables/`:**
- campaigns: useDashboardState
- characters: useLevelUp
- modules: useModuleMaps, useModuleMonsters, usePlayNotes
- sources: 20+ catalog composables, useBookContent, useSearch, etc.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `src/shared/composables/` merged into `src/composables/`
- [ ] `src/composables/` organized by subdomain (map/, api/, windows/)
- [ ] Feature-specific composables remain in features (clear ownership)
- [ ] All imports updated
- [ ] No broken imports (app builds successfully)

## Implementation Notes

### Target Structure

```
src/composables/
├── map/
│   ├── useFog.ts
│   ├── useLightSources.ts
│   ├── useTokens.ts
│   ├── useTokenVision.ts
│   ├── useUvttMap.ts
│   ├── useVisibilityPolygon.ts
│   └── useVisionCalculation.ts
├── windows/
│   ├── useDmMapWindow.ts
│   └── usePlayerDisplay.ts
├── api/
│   ├── useApiCall.ts        (from shared/)
│   └── useDataEvents.ts     (from shared/)
├── useDashboardLink.ts
├── usePagination.ts         (from shared/)
└── index.ts
```

### Decision: Feature vs Shared

Keep in features if:
- Only used within that feature
- Tightly coupled to feature's domain

Move to shared if:
- Used by 2+ features
- Generic pattern (pagination, API calls)

## Status Updates

### Session 1 - Completed
- Created `src/composables/map/` and `src/composables/windows/` subdirectories
- Moved 7 map composables to `map/`:
  - useFog, useLightSources, useTokens, useTokenVision
  - useUvttMap, useVisibilityPolygon, useVisionCalculation
- Moved 2 window composables to `windows/`:
  - useDmMapWindow, usePlayerDisplay
- Moved 3 shared composables from `src/shared/composables/` to `src/composables/`:
  - useApiCall, useDataEvents, usePagination
- Removed empty `src/shared/composables/` directory
- Updated imports in 14 files
- Build passes

**Final structure:**
```
src/composables/
├── map/           (7 files)
├── windows/       (2 files)
├── useApiCall.ts
├── useDashboardLink.ts
├── useDataEvents.ts
└── usePagination.ts
```