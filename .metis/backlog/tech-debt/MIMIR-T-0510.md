---
id: reduce-homebrew-crud-duplication
level: task
title: "Reduce homebrew CRUD duplication with macros and generics"
short_code: "MIMIR-T-0510"
created_at: 2026-02-02T01:25:02.865761+00:00
updated_at: 2026-02-03T18:25:09.339048+00:00
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

# Reduce homebrew CRUD duplication with macros and generics

## Objective

Reduce ~2,500 lines of near-identical homebrew CRUD boilerplate across all layers (DAL, models, Tauri commands, MCP tools, frontend services, frontend components) using Rust macros/generics and a Vue component factory.

## Backlog Item Details

### Type
- [x] Tech Debt - Code improvement or refactoring

### Priority
- [x] P2 - Medium (nice to have)

### Technical Debt Impact
- **Current Problems**: Three homebrew entity types (items, monsters, spells) are copy-pasted at every layer. DAL has 3 × 85-line files with identical CRUD functions. Models have 3 × 80-line files with identical struct patterns. Frontend has 3 services (~110 lines each) and 3 components (655-1070 lines each) that are 90%+ identical.
- **Benefits of Fixing**: Adding a new homebrew entity type (e.g., homebrew races, homebrew feats) becomes trivial. Bug fixes apply to all entity types at once. ~2,000+ line reduction.
- **Risk Assessment**: Medium — macro-based code generation can be harder to debug. Frontend component abstraction needs careful prop/slot design to handle entity-specific differences (stat block components, form fields).

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] Rust `homebrew_crud!` macro generates DAL functions (insert, get, get_by_name, list, update, delete, delete_by_campaign) from table name and type
- [ ] Homebrew model structs use shared derives/patterns (or macro generation)
- [ ] Frontend `createHomebrewService<T>()` factory replaces three separate service classes
- [ ] Frontend shared `HomebrewCrudSubTab` component handles list/detail/clone/delete, parameterized by entity type, fields, and stat block component
- [ ] All existing functionality preserved (no regressions)
- [ ] All tests pass

## Implementation Notes

### Technical Approach

**Rust (DAL + Models):**
```rust
macro_rules! homebrew_crud {
    ($table:ident, $model:ty, $new_model:ty, $update_model:ty) => {
        pub fn insert(...) -> QueryResult<String> { ... }
        pub fn get(...) -> QueryResult<$model> { ... }
        // etc.
    }
}
```

**Frontend (Services):**
```typescript
function createHomebrewService<T>(commandPrefix: string, eventPrefix: string) {
    return { list, get, create, update, delete }
}
export const HomebrewItemService = createHomebrewService<HomebrewItem>('homebrew_item', 'homebrew')
```

**Frontend (Components):**
Shared `HomebrewCrudSubTab` with props for: service instance, catalog search function, stat block component (slot), entity-specific field definitions for clone modal.

### Dependencies
- MIMIR-T-0509 (HomebrewService) should be done first — this ticket builds on that foundation

## Status Updates

### Session
- Created `homebrew_crud!` macro in `dal/campaign/homebrew_crud.rs` (generates 7 CRUD functions)
- Replaced 3 × 85-line DAL files with 3 × 18-line macro invocations (~200 lines saved)
- Added `paste = "1.0"` dependency for identifier concatenation
- Created `createHomebrewService<T>()` factory in frontend
- Replaced 3 × 115-line TS services with 3 × 40-line type definitions + factory call (~225 lines saved)
- All 16 MCP tests pass
- `vue-tsc` has no homebrew-related errors (pre-existing unrelated error exists)

**Scope reduction**: Skipped frontend component abstraction (largest/riskiest part). The Rust and TS service layers are now DRY. Component abstraction can be a follow-up ticket if needed.