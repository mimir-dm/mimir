---
id: generate-typescript-types-from
level: task
title: "Generate TypeScript types from Rust structs with ts-rs"
short_code: "MIMIR-T-0516"
created_at: 2026-02-02T01:25:08.982941+00:00
updated_at: 2026-02-02T01:25:08.982941+00:00
parent: 
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/backlog"
  - "#tech-debt"


exit_criteria_met: false
strategy_id: NULL
initiative_id: NULL
---

# Generate TypeScript types from Rust structs with ts-rs

## Objective

Use the `ts-rs` crate to auto-generate TypeScript type definitions from Rust structs, eliminating the risk of type drift between backend models and frontend interfaces.

## Backlog Item Details

### Type
- [x] Tech Debt - Code improvement or refactoring

### Priority
- [x] P3 - Low (when time permits)

### Technical Debt Impact
- **Current Problems**: Rust models in mimir-core and TypeScript interfaces in `frontend/src/types/` are maintained independently. When a field is added to a Rust struct, someone must remember to update the corresponding TS interface. Mismatches cause runtime errors that the compiler can't catch.
- **Benefits of Fixing**: Single source of truth for types. Adding a field to a Rust struct automatically updates the TS definition. Eliminates an entire class of bugs.
- **Risk Assessment**: Low-medium. `ts-rs` is mature but adds a build step. Generated types may need post-processing for Tauri command request/response wrappers. Some Rust types (Diesel-specific, lifetimed) may not be directly exportable.

## Acceptance Criteria

- [ ] `ts-rs` added as dev dependency to mimir-core
- [ ] Key model structs derive `TS` (Campaign, Module, Character, Document, Map, HomebrewItem, HomebrewMonster, HomebrewSpell, and their Create/Update variants)
- [ ] Build step generates `.ts` files into `frontend/src/types/generated/`
- [ ] Frontend imports generated types instead of hand-written duplicates
- [ ] Hand-written type files removed or reduced to frontend-only types (UI state, form models)
- [ ] `vue-tsc --noEmit` passes

## Implementation Notes

### Technical Approach
1. Add `ts-rs = "10"` to mimir-core dev-dependencies
2. Add `#[derive(TS)]` and `#[ts(export)]` to model structs
3. Run `cargo test` to generate bindings (ts-rs generates during test runs)
4. Create an angreal task or npm script to copy generated types to frontend
5. Update frontend imports incrementally

### Risk Considerations
- Lifetimed structs (`NewCampaign<'a>`) may not work with ts-rs directly — may need owned variants or manual type overrides
- `Option<Option<&str>>` patterns (used in update structs) may need special handling
- Start with read models (query structs) and expand to write models later

## Status Updates

*To be added during implementation*