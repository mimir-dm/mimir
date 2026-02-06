---
id: create-tokenservice-and-mcp-token
level: task
title: "Create TokenService and MCP token tools"
short_code: "MIMIR-T-0521"
created_at: 2026-02-04T14:06:30.640892+00:00
updated_at: 2026-02-04T15:05:00.731254+00:00
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

# Create TokenService and MCP token tools

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[Parent Initiative]]

## Objective

Extract 514 lines of token management code from `commands/module.rs` into a proper `TokenService` in mimir-core, then create MCP tools for token operations (currently no MCP token support exists).

## Backlog Item Details **[CONDITIONAL: Backlog Item]**

{Delete this section when task is assigned to an initiative}

### Type
- [ ] Bug - Production issue that needs fixing
- [ ] Feature - New functionality or enhancement  
- [ ] Tech Debt - Code improvement or refactoring
- [ ] Chore - Maintenance or setup work

### Priority
- [ ] P0 - Critical (blocks users/revenue)
- [ ] P1 - High (important for user experience)
- [ ] P2 - Medium (nice to have)
- [ ] P3 - Low (when time permits)

### Impact Assessment **[CONDITIONAL: Bug]**
- **Affected Users**: {Number/percentage of users affected}
- **Reproduction Steps**: 
  1. {Step 1}
  2. {Step 2}
  3. {Step 3}
- **Expected vs Actual**: {What should happen vs what happens}

### Business Justification **[CONDITIONAL: Feature]**
- **User Value**: {Why users need this}
- **Business Value**: {Impact on metrics/revenue}
- **Effort Estimate**: {Rough size - S/M/L/XL}

### Technical Debt Impact **[CONDITIONAL: Tech Debt]**
- **Current Problems**: {What's difficult/slow/buggy now}
- **Benefits of Fixing**: {What improves after refactoring}
- **Risk Assessment**: {Risks of not addressing this}

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria **[REQUIRED]**

- [ ] {Specific, testable requirement 1}
- [ ] {Specific, testable requirement 2}
- [ ] {Specific, testable requirement 3}

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

## Implementation Notes **[CONDITIONAL: Technical Task]**

{Keep for technical tasks, delete for non-technical. Technical details, approach, or important considerations}

### Technical Approach
{How this will be implemented}

### Dependencies
{Other tasks or systems this depends on}

### Risk Considerations
{Technical risks and mitigation strategies}

## Status Updates **[REQUIRED]**

### 2026-02-04: Completed

**TokenService created** (`mimir-core/src/services/token.rs`):
- `CreateTokenInput` / `UpdateTokenInput` - input structs with builders
- `TokenResponse` - enriched response with resolved name, type, size, and coordinates
- Service methods:
  - `list(map_id)` / `list_visible(map_id)` - list tokens with enrichment
  - `get(id)` - get single token enriched
  - `create(input)` - create with validation
  - `update(id, input)` - general update
  - `update_position(id, x, y)` - optimized drag operation
  - `update_vision(id, ...)` - D&D 5e vision settings
  - `toggle_visibility(id)` - toggle hidden state
  - `delete(id)` - delete with existence check
  - `count(map_id)` - count tokens on map
- Helper methods: `get_grid_size()`, `enrich()`, `resolve_names()`, `normalize_size_code()`

**Tauri commands updated** (`mimir/src/commands/module.rs`):
- Removed ~400 lines of inline token logic
- All 8 token commands now delegate to `TokenService`
- Removed redundant imports and helper functions

**MCP tools updated** (`mimir-mcp/src/tools/map.rs`):
- `add_token_to_map` - now uses TokenService, returns enriched token
- `list_tokens_on_map` - now returns enriched tokens with names/types/sizes
- `get_map` - tokens section now enriched
- `remove_token` - now uses TokenService

**Benefits:**
- Single source of truth for token enrichment logic
- MCP tools now return same quality data as Tauri commands
- Consistent validation across both consumers
- Reduced code duplication (~350 lines removed)

**Verification:**
- All crates compile: `cargo check -p mimir-core -p mimir -p mimir-mcp`
- All mimir-core tests pass