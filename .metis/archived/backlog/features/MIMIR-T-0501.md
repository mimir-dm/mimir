---
id: add-module-reordering-support
level: task
title: "Add module reordering support"
short_code: "MIMIR-T-0501"
created_at: 2026-01-31T03:35:08.997172+00:00
updated_at: 2026-01-31T13:43:48.096175+00:00
parent: 
blocked_by: []
archived: true

tags:
  - "#task"
  - "#feature"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: NULL
---

# Add module reordering support

## Objective

Allow modules within a campaign to be reordered by swapping `module_number` values. Currently modules are always ordered by creation time (auto-incrementing `module_number`), with no way to rearrange them.

## Backlog Details

- **Type**: Feature
- **Priority**: P2 - Medium
- **Effort**: S

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] DAL: `reorder_module(conn, campaign_id, module_id, new_position)` renumbers modules in a transaction, handling the UNIQUE constraint
- [ ] Service: `reorder()` method exposed on ModuleService
- [ ] MCP: `reorder_module` tool accepting module short code and target position
- [ ] Tauri: `reorder_module` command exposed to frontend
- [ ] Frontend: drag-and-drop or move up/down controls on module sidebar list
- [ ] Module numbers are contiguous after reorder (no gaps)
- [ ] Existing tests pass, new unit tests for reorder logic

## Implementation Notes

### Current State
- `module_number` is an integer field on the `modules` table with `UNIQUE(campaign_id, module_number)`
- DAL orders by `module_number ASC` — this is already the display order
- `next_module_number()` finds `max + 1` for new modules
- Frontend displays `#N` labels from `module_number`

### Technical Approach
- Reorder within a transaction: temporarily set the moving module's number to a sentinel (e.g., -1), shift affected modules up or down by 1, then set the moved module to its target position
- Renumber to keep modules contiguous (1, 2, 3...) after any reorder
- Expose via MCP as `reorder_module(module_id, new_position)` where position is 1-indexed

### Key Files
- `crates/mimir-core/src/dal/campaign/module.rs` — DAL reorder function
- `crates/mimir-core/src/services/module.rs` — service method
- `crates/mimir-mcp/src/tools/module.rs` — MCP tool
- `crates/mimir/src-tauri/src/commands/module.rs` — Tauri command
- `crates/mimir/frontend/src/features/campaigns/components/dashboard/ModulesTab.vue` — UI

## Status Updates

*To be added during implementation*