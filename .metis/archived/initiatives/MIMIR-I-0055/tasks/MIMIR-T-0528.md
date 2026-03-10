---
id: mcp-handler-integration-tests
level: task
title: "MCP handler integration tests"
short_code: "MIMIR-T-0528"
created_at: 2026-03-08T22:48:30.904723+00:00
updated_at: 2026-03-09T01:27:24.341335+00:00
parent: MIMIR-I-0055
blocked_by: []
archived: true

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: MIMIR-I-0055
---

# MCP handler integration tests

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[MIMIR-I-0055]]

## Objective
Add integration tests to `crates/mimir-mcp/src/handler.rs` covering homebrew monster/spell/item CRUD lifecycle, error handling, and argument validation through the MCP tool dispatch layer.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria
- [x] Homebrew monster CRUD lifecycle (create, list, get, update, delete)
- [x] Homebrew spell CRUD lifecycle (with level/school fields)
- [x] Homebrew item CRUD lifecycle (with item_type/rarity fields)
- [x] Not-found errors for get/update/delete with invalid IDs
- [x] Missing required fields (name, data) return InvalidArguments
- [x] Missing id on get/update/delete returns InvalidArguments
- [x] List/create without active campaign returns NoActiveCampaign
- [x] All tests pass — 32 total (8 new + 24 existing)

## Implementation Notes
- File: `crates/mimir-mcp/src/handler.rs` (existing test module with `test_ctx()`, `setup_campaign()`, `call_ok()`, `call_err()` helpers)
- Append ~12 new async tests to existing module
- Uses tokio::test, exercises full tool dispatch path

## Status Updates
- 2026-03-08: Added 8 integration tests to handler.rs. All 32 pass.
  - CRUD lifecycles: monster, spell, item (full create→list→get→update→delete)
  - Error cases: not-found for fake IDs, missing name/data on create, missing id on get/update/delete
  - Auth: list and create fail with NoActiveCampaign when no campaign set
  - Also added `use serde_json::json;` import to test module