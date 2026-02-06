---
id: standardize-frontend-event-naming
level: task
title: "Standardize frontend event naming and extract shared homebrew CSS"
short_code: "MIMIR-T-0513"
created_at: 2026-02-02T01:25:06.111202+00:00
updated_at: 2026-02-05T04:18:44.673953+00:00
parent: 
blocked_by: []
archived: false

tags:
  - "#task"
  - "#tech-debt"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: NULL
---

# Standardize frontend event naming and extract shared homebrew CSS

## Objective

Standardize frontend data event naming conventions and extract duplicated homebrew CSS into a shared stylesheet.

## Backlog Item Details

### Type
- [x] Tech Debt - Code improvement or refactoring

### Priority
- [x] P3 - Low (when time permits)

### Technical Debt Impact
- **Current Problems**: Event naming is inconsistent — items use `homebrew:created` (colon separator) while monsters use `homebrew-monster:created` (hyphen-colon). Homebrew components share ~95% identical CSS (`.homebrew-card`, `.homebrew-list`, `.homebrew-detail`, `.modal-lg`, `.clone-results`) but each defines its own copy in scoped styles.
- **Benefits of Fixing**: Consistent event naming prevents subtle listener bugs. Shared CSS reduces maintenance burden and ensures visual consistency when styles change.
- **Risk Assessment**: Very low — renaming events and extracting CSS are straightforward find-and-replace operations.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] All homebrew events use consistent naming pattern (e.g., `homebrew-item:created`, `homebrew-monster:created`, `homebrew-spell:created`)
- [ ] `DataEventPayloads` type in `dataEvents.ts` updated to match
- [ ] All event listeners updated to use new names
- [ ] Shared homebrew CSS extracted to `assets/styles/components/homebrew.css`
- [ ] Homebrew components import shared styles and only define entity-specific overrides in scoped CSS
- [ ] No visual regressions

## Implementation Notes

### Files to Modify
- `crates/mimir/frontend/src/utils/dataEvents.ts` — rename event types
- `crates/mimir/frontend/src/services/HomebrewService.ts` — update emitted event names
- `crates/mimir/frontend/src/services/HomebrewMonsterService.ts` — update emitted event names
- `crates/mimir/frontend/src/services/HomebrewSpellService.ts` — update emitted event names
- New: `crates/mimir/frontend/src/assets/styles/components/homebrew.css`
- `HomebrewTab.vue`, `HomebrewMonstersSubTab.vue`, `HomebrewSpellsSubTab.vue` — remove duplicate CSS, import shared

## Status Updates

### 2026-02-04: Implementation complete

**Event naming standardization:**
- Updated `dataEvents.ts`: renamed `homebrew:*` to `homebrew-item:*` for consistency with `homebrew-monster:*` and `homebrew-spell:*`
- Updated `HomebrewService.ts`: changed `eventPrefix` from `'homebrew'` to `'homebrew-item'`
- Updated `HomebrewTab.vue`: updated event listeners to use new `homebrew-item:*` names
- Also added missing `module:reordered` event type discovered during TypeScript check

**Shared CSS extraction:**
- Created `frontend/src/assets/styles/components/homebrew.css` with ~200 lines of shared styles
- Added import to `main.css`
- Renamed all shared class names with `homebrew-` prefix to avoid scoped CSS conflicts:
  - `.tab-header` → `.homebrew-tab-header`
  - `.loading-state` → `.homebrew-loading-state`
  - `.empty-state` → `.homebrew-empty-state`
  - `.card-header` → `.homebrew-card-header`
  - `.card-name` → `.homebrew-card-name`
  - `.card-meta` → `.homebrew-card-meta`
  - `.detail-header` → `.homebrew-detail-header`
  - `.clone-results` → `.homebrew-clone-results`
  - etc.

**Updated components:**
- `HomebrewTab.vue`: Updated all class names, reduced scoped CSS from ~300 lines to ~60 (only sub-tabs + item-specific form sections)
- `HomebrewMonstersSubTab.vue`: Updated all class names, reduced scoped CSS from ~200 lines to ~15
- `HomebrewSpellsSubTab.vue`: Updated all class names, reduced scoped CSS from ~200 lines to ~15

**Verification:**
- `npx vue-tsc --noEmit` passes
- `cargo check -p mimir` passes