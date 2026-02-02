---
id: standardize-mcp-tool-response
level: task
title: "Standardize MCP tool response format"
short_code: "MIMIR-T-0514"
created_at: 2026-02-02T01:25:07.072005+00:00
updated_at: 2026-02-02T01:25:07.072005+00:00
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

- [ ] `McpResponse` helper in mimir-mcp with methods like `created(entity_type, data)`, `ok(data)`, `list(entity_type, items, count)`
- [ ] All MCP tools use `McpResponse` instead of ad-hoc `json!()` construction
- [ ] Response format is documented in plugin tool-parameter-reference.md
- [ ] All 16 MCP tests pass

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

*To be added during implementation*