---
id: migrate-map-homebrew-mapgen
level: task
title: "Migrate map, homebrew, mapgen, catalog families"
short_code: "MIMIR-T-0651"
created_at: 2026-07-08T11:12:14.109303+00:00
updated_at: 2026-07-08T11:31:21.507987+00:00
parent: MIMIR-I-0069
blocked_by: [MIMIR-T-0648]
archived: false

tags:
  - "#task"
  - "#phase/active"


exit_criteria_met: false
initiative_id: MIMIR-I-0069
---

# Migrate map, homebrew, mapgen, catalog families

## Parent Initiative

[[MIMIR-I-0069]] — MCP tool registry with typed arguments

## Objective **[REQUIRED]**

Migrate the remaining families: map (8 tools), homebrew (5, content_type-dispatched),
mapgen (3, no McpContext needed), catalog (1, category-dispatched). After this task
the legacy dispatch path is empty.

## Acceptance Criteria

## Acceptance Criteria **[REQUIRED]**

- [ ] All remaining tools registered with typed args; `execute_tool()` match and `get_tools()` vec are empty/deleted
- [ ] Homebrew's `content_type` and catalog's `category` dispatch stay inside their handlers (enum-typed field on the arg struct, not three/eight separate tools)
- [ ] Mapgen tools work without a DB context — registry must support context-free handlers cleanly
- [ ] Wire compatibility: homebrew CRUD lifecycles, mapgen preset/validate/generate, map error-path tests, catalog empty-DB tests pass unchanged

## Implementation Notes

- `content_type`/`category` as `#[serde(rename_all = "lowercase")]` enums gives free validation with better error messages than today's string checks — but error TEXT changes must be checked against tests that assert on messages.

## Status Updates **[REQUIRED]**

*To be added during implementation*