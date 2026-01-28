---
id: mcp-context-and-serverhandler
level: task
title: "MCP Context and ServerHandler"
short_code: "MIMIR-T-0462"
created_at: 2026-01-28T04:06:30.647777+00:00
updated_at: 2026-01-28T04:35:24.305614+00:00
parent: MIMIR-I-0050
blocked_by: []
archived: true

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: MIMIR-I-0050
---

# MCP Context and ServerHandler

## Parent Initiative

[[MIMIR-I-0050]]

## Objective

Implement the `McpContext` struct for managing database connections and the `ServerHandler` trait implementation that routes MCP tool calls to the appropriate handlers.

**Reference**: `mimir-dm-bu/mimir-dm-mcp/src/handler.rs`

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `McpContext` struct with database connection management
- [ ] `McpContext::new()` initializes database from standard paths
- [ ] `ServerHandler` implementation with `call_tool` routing
- [ ] Tool routing pattern using match on tool name
- [ ] Proper error handling with MCP error types
- [ ] Server responds to `initialize` and `tools/list` requests

## Implementation Notes

### McpContext
```rust
pub struct McpContext {
    pub db: Mutex<SqliteConnection>,
    pub paths: AppPaths,
    pub active_campaign_id: Mutex<Option<String>>,
}
```

### ServerHandler Pattern
The `call_tool` method routes based on tool name:
```rust
async fn call_tool(&self, name: &str, args: Value) -> Result<CallToolResult> {
    match name {
        "list_campaigns" => tools::campaign::list_campaigns(&self.context, args).await,
        "set_active_campaign" => tools::campaign::set_active_campaign(&self.context, args).await,
        // ... etc
        _ => Err(McpError::ToolNotFound(name.to_string())),
    }
}
```

### Dependencies
- Depends on: MIMIR-T-0461 (Crate Setup)
- Blocks: All tool implementation tasks

## Status Updates

*To be added during implementation*