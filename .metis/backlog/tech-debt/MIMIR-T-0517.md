---
id: extract-moduletype-parsing-to-enum
level: task
title: "Extract ModuleType parsing to enum impl"
short_code: "MIMIR-T-0517"
created_at: 2026-02-04T14:06:27.317182+00:00
updated_at: 2026-02-04T15:11:14.369799+00:00
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

# Extract ModuleType parsing to enum impl

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[Parent Initiative]]

## Objective

Move `parse_module_type` function to `impl From<Option<&str>> for ModuleType` on the enum itself, eliminating duplicate parsing logic in Tauri commands and MCP tools.

## Backlog Item Details

### Type
- [x] Tech Debt - Code improvement or refactoring

### Priority
- [x] P1 - High (quick win, reduces duplication)

### Technical Debt Impact
- **Current Problems**: Identical `parse_module_type` functions in `commands/module.rs:58-67` and `tools/module.rs:187-194`
- **Benefits of Fixing**: Single source of truth for module type parsing
- **Risk Assessment**: Low - simple refactor

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `ModuleType` enum has `impl From<Option<&str>> for ModuleType`
- [ ] Tauri `commands/module.rs` uses `ModuleType::from()` 
- [ ] MCP `tools/module.rs` uses `ModuleType::from()`
- [ ] Both `parse_module_type` functions deleted
- [ ] All tests pass

## Implementation Notes

### Files to Modify
- `mimir-core/src/models/campaign/module.rs` - Add `From` impl
- `mimir/src/commands/module.rs` - Remove `parse_module_type`, use `From`
- `mimir-mcp/src/tools/module.rs` - Remove `parse_module_type`, use `From`

## Status Updates

### 2026-02-04: Complete

**Changes Made:**

1. **Added `impl From<Option<&str>> for ModuleType`** in `mimir-core/src/services/module.rs:64-75`
   - Centralizes module type parsing logic
   - Returns `ModuleType::General` for `None` or unrecognized values

2. **Updated Tauri commands** in `mimir/src/commands/module.rs`
   - Removed `parse_module_type` function (lines 57-67)
   - Changed `parse_module_type(request.module_type.as_deref())` to `ModuleType::from(request.module_type.as_deref())`

3. **Updated MCP tools** in `mimir-mcp/src/tools/module.rs`
   - Replaced inline match expression (lines 187-194) with `ModuleType::from(...)`

4. **Added unit test** `test_module_type_from_str` verifying all parsing cases

**Verification:**
- `cargo check -p mimir-core -p mimir -p mimir-mcp` - passes
- `cargo test -p mimir-core --lib services::module` - 19 tests pass
- `cargo test -p mimir-mcp` - 16 tests pass
- No remaining `parse_module_type` functions in codebase