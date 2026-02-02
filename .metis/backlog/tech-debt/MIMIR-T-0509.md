---
id: create-homebrewservice-in-mimir
level: task
title: "Create HomebrewService in mimir-core"
short_code: "MIMIR-T-0509"
created_at: 2026-02-02T01:25:01.800499+00:00
updated_at: 2026-02-02T01:41:12.993356+00:00
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

# Create HomebrewService in mimir-core

## Objective

Create a `HomebrewService` in `mimir-core/src/services/` that centralizes all homebrew CRUD logic (items, monsters, spells). Currently homebrew Tauri commands call the DAL directly, bypassing the service layer that all other entities use. This causes logic duplication between Tauri and MCP, missing JSON validation in Tauri, and inconsistent timestamp formatting.

## Backlog Item Details

### Type
- [x] Tech Debt - Code improvement or refactoring

### Priority
- [x] P1 - High (important for user experience)

### Technical Debt Impact
- **Current Problems**: Homebrew commands in `crates/mimir/src/commands/homebrew*.rs` call DAL directly. MCP tools in `crates/mimir-mcp/src/tools/homebrew*.rs` duplicate the same logic independently. MCP validates JSON data, Tauri doesn't — invalid JSON can reach the database. UUID generation and timestamp formatting happen in the command layer instead of the service layer.
- **Benefits of Fixing**: Single source of truth for homebrew business logic. Both Tauri and MCP become thin adapters. JSON validation happens once. Consistent timestamp formatting. Easier to add new consumers (CLI, REST API).
- **Risk Assessment**: Low risk — straightforward extraction of existing logic into a new service following the established `CampaignService` pattern.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `HomebrewService` struct in `mimir-core/src/services/` with methods for list/get/create/update/delete for items, monsters, and spells
- [ ] Service handles UUID generation, JSON validation of `data` field, and timestamp formatting
- [ ] Tauri homebrew commands refactored to call `HomebrewService` instead of DAL directly
- [ ] MCP homebrew tools refactored to call `HomebrewService` instead of DAL directly
- [ ] All 16 MCP tests still pass
- [ ] `cargo check` passes for all crates
- [ ] `vue-tsc --noEmit` passes

## Implementation Notes

### Technical Approach
Follow the `CampaignService` pattern in `crates/mimir-core/src/services/campaign.rs`:
- Service takes `&mut SqliteConnection` in constructor
- Methods accept typed input structs and return `ServiceResult<T>`
- Validate `data` field is valid JSON in create/update methods
- Generate UUIDs and timestamps in the service, not the command layer

### Files to Modify
- New: `crates/mimir-core/src/services/homebrew.rs`
- Modify: `crates/mimir-core/src/services/mod.rs` (register module)
- Modify: `crates/mimir/src/commands/homebrew.rs`, `homebrew_monster.rs`, `homebrew_spell.rs`
- Modify: `crates/mimir-mcp/src/tools/homebrew.rs`, `homebrew_monster.rs`, `homebrew_spell.rs`

## Status Updates

### Session
- Created `crates/mimir-core/src/services/homebrew.rs` with `HomebrewService<'a>` struct
- Input structs: `Create/Update HomebrewItem/Monster/Spell Input` (6 total)
- Methods: list, get, get_by_name, create, update, delete for all 3 entity types (18 methods)
- JSON validation via `validate_json()` on create and update
- UUID generation and RFC3339 timestamps in service layer
- Registered in `services/mod.rs` with all public exports
- `cargo check -p mimir-core` passes
- Refactored all 3 Tauri command files to use HomebrewService (removed direct DAL calls, uuid, chrono imports)
- Refactored all 3 MCP tool files to use HomebrewService (removed direct DAL calls, uuid, chrono imports)
- Added `From<ServiceError> for McpError` in `error.rs` for clean error propagation
- `cargo check` passes for mimir-core, mimir, mimir-mcp
- All 16 MCP tests pass
- Tauri commands now get JSON validation they were missing before
- Timestamps now use consistent RFC3339 format everywhere (was `%Y-%m-%d %H:%M:%S` in Tauri)