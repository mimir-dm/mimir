---
id: convert-boolean-as-integer-fields
level: task
title: "Convert boolean-as-integer fields to proper bool types"
short_code: "MIMIR-T-0298"
created_at: 2026-01-03T15:08:53.922108+00:00
updated_at: 2026-01-04T14:35:30.135778+00:00
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

# Convert boolean-as-integer fields to proper bool types

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[Parent Initiative]]

## Objective

Convert boolean fields stored as `i32` to proper `bool` types in Diesel models for type safety and cleaner code.

## Backlog Item Details

### Type
- [x] Tech Debt - Code improvement or refactoring

### Priority
- [x] P2 - Medium (nice to have)

### Technical Debt Impact
- **Current Problems**: SQLite stores booleans as integers, but Diesel supports `bool` mapping. Current code requires helper methods like `is_npc()` to convert. Inconsistent - `maps.fog_enabled` is already `bool`.
- **Benefits of Fixing**: Direct boolean semantics, remove helper methods, consistent typing across models.
- **Risk Assessment**: Low risk - straightforward refactor with good test coverage.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `characters.is_npc` uses `bool` type in Rust model
- [ ] `players.active` uses `bool` type in Rust model
- [ ] Helper methods like `is_npc()` removed (no longer needed)
- [ ] All existing tests pass
- [ ] Schema uses Diesel's `Bool` type mapping

## Test Cases **[CONDITIONAL: Testing Task]**

{Delete unless this is a testing task}

### Test Case 1: {Test Case Name}
- **Test ID**: TC-001
- **Preconditions**: {What must be true before testing}
- **Steps**: 
  1. {Step 1}
  2. {Step 2}
  3. {Step 3}
- **Expected Results**: {What should happen}
- **Actual Results**: {To be filled during execution}
- **Status**: {Pass/Fail/Blocked}

### Test Case 2: {Test Case Name}
- **Test ID**: TC-002
- **Preconditions**: {What must be true before testing}
- **Steps**: 
  1. {Step 1}
  2. {Step 2}
- **Expected Results**: {What should happen}
- **Actual Results**: {To be filled during execution}
- **Status**: {Pass/Fail/Blocked}

## Documentation Sections **[CONDITIONAL: Documentation Task]**

{Delete unless this is a documentation task}

### User Guide Content
- **Feature Description**: {What this feature does and why it's useful}
- **Prerequisites**: {What users need before using this feature}
- **Step-by-Step Instructions**:
  1. {Step 1 with screenshots/examples}
  2. {Step 2 with screenshots/examples}
  3. {Step 3 with screenshots/examples}

### Troubleshooting Guide
- **Common Issue 1**: {Problem description and solution}
- **Common Issue 2**: {Problem description and solution}
- **Error Messages**: {List of error messages and what they mean}

### API Documentation **[CONDITIONAL: API Documentation]**
- **Endpoint**: {API endpoint description}
- **Parameters**: {Required and optional parameters}
- **Example Request**: {Code example}
- **Example Response**: {Expected response format}

## Implementation Notes

### Technical Approach
1. Update `schema.rs` to use `Bool` type for affected columns
2. Update model structs to use `bool` instead of `i32`
3. Remove helper methods like `is_npc()`
4. Update all call sites (services, API handlers)
5. Verify with `cargo check` and `cargo test`

### Files Affected
- `crates/mimir-dm-core/src/schema.rs`
- `crates/mimir-dm-core/src/models/character/mod.rs`
- `crates/mimir-dm-core/src/models/player/mod.rs`
- Various services that use these fields

### Effort Estimate
2-3 hours

## Status Updates **[REQUIRED]**

*To be added during implementation*