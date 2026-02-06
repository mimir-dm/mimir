---
id: standardize-mcp-tool-response
level: task
title: "Standardize MCP tool response format"
short_code: "MIMIR-T-0514"
created_at: 2026-02-02T01:25:07.072005+00:00
updated_at: 2026-02-05T04:26:49.089619+00:00
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

# Standardize MCP tool response format

## Objective

Create a standardized response wrapper for MCP tools, analogous to Tauri's `ApiResponse<T>`. Currently each MCP tool constructs ad-hoc JSON responses with inconsistent key names and structures.

## Backlog Item Details

### Type
- [x] Tech Debt - Code improvement or refactoring

### Priority
- [x] P2 - Medium (nice to have)

### Technical Debt Impact
- **Current Problems**: Each MCP tool manually constructs `Ok(json!({ "status": "created", "campaign": {...} }))` with different key names per entity. No consistent envelope. Some use singular keys ("monster"), some could drift to plural. Response construction is verbose and error-prone.
- **Benefits of Fixing**: Consistent API contract for AI consumers. Less boilerplate per tool. Plugin documentation can describe one response format instead of per-tool documentation.
- **Risk Assessment**: Low — purely additive. Existing tools can be migrated incrementally.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [x] `McpResponse` helper in mimir-mcp with methods like `created(entity_type, data)`, `ok(data)`, `list(entity_type, items, count)`
- [x] All MCP tools use `McpResponse` instead of ad-hoc `json!()` construction
- [x] Response format is documented in plugin tool-parameter-reference.md
- [x] All 24 MCP tests pass (test count increased from 16)

## Implementation Notes

### Technical Approach
```rust
pub struct McpResponse;

impl McpResponse {
    pub fn created(entity: &str, data: Value) -> Result<Value, McpError> {
        Ok(json!({ "status": "created", entity: data }))
    }
    pub fn ok(data: Value) -> Result<Value, McpError> {
        Ok(json!({ "status": "ok", "data": data }))
    }
    pub fn list(entity: &str, items: Vec<Value>, count: usize) -> Result<Value, McpError> {
        Ok(json!({ entity: items, "count": count }))
    }
    pub fn deleted(entity: &str, id: &str) -> Result<Value, McpError> {
        Ok(json!({ "status": "deleted", "id": id }))
    }
}
```

## Status Updates

### 2026-02-05: Implementation Complete

**Created `McpResponse` helper** (`crates/mimir-mcp/src/response.rs`):
- `list(key, items)` - List responses with count
- `get(key, data)` - Single entity retrieval
- `created(key, data)` - Create operation
- `updated(key, data)` - Update operation
- `deleted(id)` - Delete operation
- `added(key, data)` - Add to collection
- `removed(id)` - Remove from collection
- `success(data)` - Special operations (level_up)
- `ok(data)` - Pass-through for complex responses

**Files Updated**:
- `campaign.rs` - 11 tool implementations
- `module.rs` - 6 tool implementations
- `homebrew.rs` - 5 tool implementations
- `homebrew_monster.rs` - 5 tool implementations
- `homebrew_spell.rs` - 5 tool implementations
- `document.rs` - 5 tool implementations
- `character.rs` - 11 tool implementations
- `catalog.rs` - 8 tool implementations
- `map.rs` - 8 tool implementations

**Documentation**: Response format documented in `tool-parameter-reference.md`

**Tests**: All 24 MCP tests pass (updated 2 tests for new response format)