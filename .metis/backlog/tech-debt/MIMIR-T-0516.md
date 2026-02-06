---
id: generate-typescript-types-from
level: task
title: "Generate TypeScript types from Rust structs with ts-rs"
short_code: "MIMIR-T-0516"
created_at: 2026-02-02T01:25:08.982941+00:00
updated_at: 2026-02-06T03:54:10.916982+00:00
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

## Acceptance Criteria

- [x] `ts-rs` added as dev dependency to mimir-core
- [x] Key model structs derive `TS` (Campaign, Module, Character, Document, Map, HomebrewItem, HomebrewMonster, HomebrewSpell, and their Create/Update variants)
- [x] Build step generates `.ts` files into `frontend/src/types/generated/`
- [x] Frontend imports generated types instead of hand-written duplicates
- [x] Hand-written type files removed or reduced to frontend-only types (UI state, form models)
- [x] `vue-tsc --noEmit` passes

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

### Session 2026-02-05 - COMPLETED

**Work completed:**

1. **Added ts-rs dependency**
   - Added `ts-rs = "10"` to workspace Cargo.toml
   - Added `ts-rs = { workspace = true }` to mimir-core dev-dependencies

2. **Added TS derive macros to 12 model structs:**
   - Campaign, Module, Document, Map
   - Character, CharacterResponse, CharacterClass, CharacterProficiency, CharacterInventory
   - CampaignHomebrewItem, CampaignHomebrewMonster, CampaignHomebrewSpell

3. **Generated TypeScript types** via `cargo test -p mimir-core --lib`
   - Types generated to `crates/mimir-core/bindings/` then copied to `frontend/src/types/generated/`
   - Created `index.ts` to re-export all generated types

4. **Updated frontend to use generated types:**
   - `api.ts` - Campaign, Module, Document now imported from generated
   - `character.ts` - CharacterProficiency, CharacterClass, CharacterInventory, CharacterResponse imported from generated
   - `index.ts` - Added Map and Homebrew type exports
   - `HomebrewService.ts`, `HomebrewMonsterService.ts`, `HomebrewSpellService.ts` - Now use generated types
   - `dataEvents.ts` - Updated character event payload to allow null campaign_id
   - `ClassSelectionStep.vue` - Added missing `starting_class` field to CharacterClass objects

5. **Verification:**
   - `cargo test -p mimir-core --lib` - 812 tests pass
   - `npx vue-tsc --noEmit` - No TypeScript errors

**Files modified:**
- `Cargo.toml` (workspace)
- `crates/mimir-core/Cargo.toml`
- `crates/mimir-core/src/models/campaign/*.rs` (12 files)
- `crates/mimir/frontend/src/types/generated/*.ts` (13 files created)
- `crates/mimir/frontend/src/types/api.ts`
- `crates/mimir/frontend/src/types/character.ts`
- `crates/mimir/frontend/src/types/index.ts`
- `crates/mimir/frontend/src/services/Homebrew*.ts` (3 files)
- `crates/mimir/frontend/src/utils/dataEvents.ts`
- `crates/mimir/frontend/src/features/characters/components/levelup/steps/ClassSelectionStep.vue`

**Notes:**
- Used `#[cfg_attr(test, derive(TS))]` to only generate types during test runs
- Export path set to `bindings/` relative to mimir-core crate
- Hand-written request types (CreatePcRequest, UpdateCharacterRequest, etc.) kept in character.ts as they're frontend-only