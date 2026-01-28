---
id: claude-code-plugin-structure
level: task
title: "Claude Code Plugin Structure"
short_code: "MIMIR-T-0468"
created_at: 2026-01-28T04:06:34.769972+00:00
updated_at: 2026-01-28T04:48:29.505474+00:00
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

# Claude Code Plugin Structure

## Parent Initiative

[[MIMIR-I-0050]]

## Objective

Create the Claude Code plugin configuration files that register the MCP server and provide usage documentation for the AI assistant.

**Reference**: `mimir-dm-bu/mimir-dm-mcp/plugin/`

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `plugin/plugin.json` - MCP server registration with stdio transport
- [ ] `plugin/README.md` - Tool documentation for Claude's context
- [ ] Plugin can be installed via `claude plugin add ./plugin`
- [ ] MCP server binary builds and runs standalone
- [ ] `main.rs` with stdio server startup

## Plugin Structure

### plugin/plugin.json
```json
{
  "name": "mimir-dm",
  "description": "D&D 5e Campaign Management for Dungeon Masters",
  "mcp": {
    "servers": {
      "mimir": {
        "command": "mimir-mcp",
        "args": [],
        "transport": "stdio"
      }
    }
  }
}
```

### plugin/README.md
Documentation that gets injected into Claude's context:
- Overview of Mimir capabilities
- Tool categories and usage patterns
- Workflow examples (creating module, populating encounters)
- Common patterns for DM authoring

### Binary Entry Point
`src/main.rs`:
```rust
#[tokio::main]
async fn main() {
    let handler = MimirHandler::new();
    run_stdio_server(handler).await;
}
```

## Installation Flow
1. Build: `cargo build -p mimir-mcp --release`
2. Install: `claude plugin add ./crates/mimir-mcp/plugin`
3. Verify: `claude mcp list` shows mimir tools

## Dependencies
- Depends on: All other MCP tasks (tools must be implemented)
- This is the final integration task

## Status Updates

*To be added during implementation*