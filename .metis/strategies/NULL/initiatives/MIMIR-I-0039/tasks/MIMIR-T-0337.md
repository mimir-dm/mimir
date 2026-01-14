---
id: mcp-server-test-coverage
level: task
title: "MCP Server Test Coverage"
short_code: "MIMIR-T-0337"
created_at: 2026-01-14T01:50:48.741449+00:00
updated_at: 2026-01-14T03:51:53.645042+00:00
parent: MIMIR-I-0039
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: MIMIR-I-0039
---

# MCP Server Test Coverage

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[MIMIR-I-0039]]

## Objective

Add integration tests for the MCP (Model Context Protocol) server in `mimir-dm-mcp`, ensuring all tool handlers work correctly.

## Scope

**Target: `crates/mimir-dm-mcp/`**

The MCP server provides tools for campaign management through the Model Context Protocol. Each tool handler needs tests to verify:
- Correct parameter handling
- Proper response formatting
- Error handling for invalid inputs
- Integration with `mimir-dm-core` services

**MCP Tools to Test:**
- Campaign tools (list, create, read, update, delete)
- Module tools
- Document tools
- Search functionality
- Any other exposed MCP tools

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [x] Create test infrastructure for MCP tool handlers
- [x] Add tests for each MCP tool's happy path
- [x] Add tests for error conditions (invalid params, missing data)
- [x] Verify JSON-RPC response format is correct
- [x] Test tool discovery/listing
- [x] Tests can run without external MCP client
- [x] Document test patterns for future tool additions

## Implementation Notes

### Technical Approach

**Test Setup:**
```rust
// Create test database and MCP server instance
let db = TestDatabase::new();
let server = MimirMcpServer::new(db.connection());

// Call tool handler directly
let result = server.handle_tool_call("campaign_list", json!({})).await;

// Verify response
assert!(result.is_ok());
let campaigns: Vec<Campaign> = serde_json::from_value(result.unwrap())?;
```

**Key Scenarios:**
1. Tool discovery returns all available tools
2. Each tool returns valid JSON-RPC responses
3. Invalid parameters return appropriate errors
4. Database operations through tools work correctly

### Files to Create
- `tests/mcp_tools.rs` - Main test file
- `tests/common/mod.rs` - Test utilities if needed

### Dependencies
- Depends on `mimir-dm-core` test utilities (`TestDatabase`)
- May need `rust-mcp-sdk` test helpers

### Risk Considerations
- MCP protocol version compatibility
- Async test handling for tool calls
- Mock vs real database trade-offs

### Testing with angreal

Run MCP server tests:
```bash
# Run core tests (includes mimir-dm-mcp)
angreal test unit --core

# Check coverage for MCP tool handlers
angreal test coverage --core --open
```

Coverage reports output to `target/coverage/tarpaulin-report.html`

## Status Updates **[REQUIRED]**

### Session 2026-01-14

**Work Completed:**
1. Explored MCP server structure in `crates/mimir-dm-mcp/`:
   - `McpContext` wraps database path and active campaign state with async Mutex
   - `MimirHandler` implements `ServerHandler` trait with 19 tools
   - Tools organized by category: campaign, module, document, character, catalog

2. Created test infrastructure:
   - `tests/common/mod.rs` - `TestMcpEnv` struct with file-based SQLite database
   - Template seeding via `seed_templates()` for document creation
   - Helper methods: `create_campaign()`, `set_active_campaign()`

3. Created comprehensive test suite in `tests/mcp_tools.rs`:
   - **Campaign tools**: 7 tests (list, create, get, update, archive, restore, details)
   - **Module tools**: 7 tests (list, create, get, update, delete, transition, details)
   - **Document tools**: 6 tests (list, get, create, edit, template list)
   - **Character tools**: 8 tests (list, create NPC, create PC, get, update currency, update inventory)
   - **Tool discovery**: 5 tests (total count, categories, schemas, campaign tools, metadata)

4. Fixed multiple struct field mismatches by reading actual source definitions:
   - `CreateModuleInput`: only has `name` and `module_type`
   - `CreateNpcInput`: `race` is `String` not `Option<String>`
   - `ListCharactersInput`: uses `character_type` field
   - `GetCharacterResponse`: fields directly on response, not nested

**Test Results:**
```
running 33 tests
test result: ok. 33 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

**Files Created:**
- `crates/mimir-dm-mcp/tests/common/mod.rs` - Test infrastructure
- `crates/mimir-dm-mcp/tests/mcp_tools.rs` - 33 comprehensive tests