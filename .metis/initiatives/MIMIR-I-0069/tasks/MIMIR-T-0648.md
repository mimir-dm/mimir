---
id: registry-core-pilot-migration
level: task
title: "Registry core + pilot migration (module tools)"
short_code: "MIMIR-T-0648"
created_at: 2026-07-08T11:12:10.120642+00:00
updated_at: 2026-07-08T11:17:37.155615+00:00
parent: MIMIR-I-0069
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/active"


exit_criteria_met: false
initiative_id: MIMIR-I-0069
---

# Registry core + pilot migration (module tools)

## Parent Initiative

[[MIMIR-I-0069]] — MCP tool registry with typed arguments

## Objective **[REQUIRED]**

Build the registry module and prove it on the module tool family (9 tools:
create/list/get/update/delete module, add/update/remove monster, add_item stub).
This task carries all the design risk; the rest of the initiative is mechanical
migration.

## Acceptance Criteria

## Acceptance Criteria **[REQUIRED]**

- [ ] A tool is defined in ONE place: typed arg struct (`#[derive(Deserialize)]`) + handler + name/description, registered once
- [ ] JSON `ToolInputSchema` is generated from the same arg struct that deserializes the call (no second copy of the contract)
- [ ] `MimirHandler::handle_list_tools_request` and `handle_call_tool_request` consume the registry; migrated tools have no `get_tools()` vec entry or `execute_tool()` match arm
- [ ] Unmigrated families keep working through the legacy path during the transition (hybrid dispatch)
- [ ] Wire compatibility: all existing functional tests in `handler.rs` and `tests/functional_stdio.rs` pass UNCHANGED — same tool names, arg names, required fields, and response shapes
- [ ] Registry rejects unknown tool names with the same `ToolNotFound` behavior

## Implementation Notes

- Key decision to settle here: schema generation approach (small hand-rolled derive vs `schemars` dependency vs declarative macro emitting both struct and schema). Prefer the lightest thing that keeps required/optional and descriptions adjacent to the struct fields.
- Async handler storage: `Box<dyn Fn(&Arc<McpContext>, Args) -> BoxFuture<...>>` per tool, or an enum dispatch — pick for simplicity, not generality.
- Module family is the pilot because it has the freshest tests (increment parity, validation, hang guard from MIMIR-T-0643 work).

## Status Updates **[REQUIRED]**

*To be added during implementation*