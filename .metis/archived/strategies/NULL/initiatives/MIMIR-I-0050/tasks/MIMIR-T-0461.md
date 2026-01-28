---
id: mcp-crate-setup-and-dependencies
level: task
title: "MCP Crate Setup and Dependencies"
short_code: "MIMIR-T-0461"
created_at: 2026-01-28T04:06:30.000318+00:00
updated_at: 2026-01-28T04:35:03.978427+00:00
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

# MCP Crate Setup and Dependencies

## Parent Initiative

[[MIMIR-I-0050]]

## Objective

Create the `mimir-mcp` crate with proper workspace configuration and dependencies for building an MCP server using `rust-mcp-sdk`.

**Reference**: `mimir-dm-bu/mimir-dm-mcp/Cargo.toml`

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] New `crates/mimir-mcp/` directory created
- [ ] `Cargo.toml` with `rust-mcp-sdk` dependency and `mimir-core` dependency
- [ ] Added to workspace in root `Cargo.toml`
- [ ] Basic `src/lib.rs` with module structure: `mod context; mod handler; mod tools;`
- [ ] Compiles successfully with `cargo build -p mimir-mcp`

## Implementation Notes

### Crate Structure
```
crates/mimir-mcp/
├── Cargo.toml
├── src/
│   ├── lib.rs
│   ├── context.rs      # Database and state management
│   ├── handler.rs      # ServerHandler implementation
│   └── tools/
│       ├── mod.rs
│       ├── campaign.rs
│       ├── module.rs
│       ├── document.rs
│       ├── character.rs
│       └── catalog.rs
└── plugin/             # Claude Code plugin files
    ├── plugin.json
    └── README.md
```

### Key Dependencies
- `rust-mcp-sdk` - MCP protocol implementation
- `mimir-core` - Data access and services
- `serde`, `serde_json` - Serialization
- `tokio` - Async runtime
- `tracing` - Logging

### Dependencies
This is the foundational task - all other MCP tasks depend on this.

## Status Updates

*To be added during implementation*