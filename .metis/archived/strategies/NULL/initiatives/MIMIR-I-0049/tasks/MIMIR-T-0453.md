---
id: consolidate-utils
level: task
title: "Consolidate Utils"
short_code: "MIMIR-T-0453"
created_at: 2026-01-28T03:54:05.083045+00:00
updated_at: 2026-01-28T04:33:31.245897+00:00
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

# Consolidate Utils

## Parent Initiative

[[MIMIR-I-0049]] - Frontend Organizational Debt Cleanup

## Objective

Consolidate fragmented utility functions into a single location with clear organization.

## Current State

Utils are spread across 3 locations:

**`src/utils/` (1 file):**
- characterUtils.ts (505 lines)

**`src/shared/utils/` (6 files):**
- api.ts, dataEvents.ts, debounce.ts, debug.ts, formatters.ts, windows.ts

**`src/features/sources/utils/`:**
- textFormatting.ts
- renderers/contentRenderer.ts

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] Single utils location established (`src/utils/` or `src/shared/utils/`)
- [ ] characterUtils.ts moved to appropriate location
- [ ] Empty directories removed
- [ ] All imports updated
- [ ] No broken imports (app builds successfully)

## Implementation Notes

### Decision: Where to Put Utils

**Option A: Use `src/utils/`**
- Move everything from `src/shared/utils/` to `src/utils/`
- Remove `src/shared/utils/`

**Option B: Use `src/shared/utils/`**
- Move `src/utils/characterUtils.ts` to `src/shared/utils/`
- Remove `src/utils/`

**Recommendation:** Option A - `src/utils/` is more discoverable and consistent with common conventions.

### Target Structure

```
src/utils/
├── api.ts
├── character.ts      (renamed from characterUtils.ts)
├── dataEvents.ts
├── debounce.ts
├── debug.ts
├── formatters.ts
└── windows.ts
```

### Feature-Specific Utils

Keep `src/features/sources/utils/` as-is since textFormatting and contentRenderer are source-specific.

## Status Updates

### Session 1 - Completed
- Moved 6 files from `src/shared/utils/` to `src/utils/`:
  - api.ts, dataEvents.ts, debounce.ts, debug.ts, formatters.ts, windows.ts
- Updated imports in 11+ files from `@/shared/utils/` to `@/utils/`
- Fixed several broken relative imports discovered during build:
  - AppHeader.vue, DocumentEditor.vue, useApiCall.ts
  - monsterFormatterEnhanced.ts, spellFormatterEnhanced.ts
  - CatalogTable.vue, itemConfig.ts, spellConfig.ts
- Removed empty `src/shared/utils/` directory
- Build passes

**Final structure:**
```
src/utils/
├── api.ts
├── characterUtils.ts
├── dataEvents.ts
├── debounce.ts
├── debug.ts
├── formatters.ts
└── windows.ts
```