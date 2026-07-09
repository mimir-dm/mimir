---
id: cleanup-dissolve-bookkeeping-tests
level: task
title: "Cleanup: dissolve bookkeeping tests + ServiceError From impl"
short_code: "MIMIR-T-0652"
created_at: 2026-07-08T11:12:14.949077+00:00
updated_at: 2026-07-08T11:38:19.333242+00:00
parent: MIMIR-I-0069
blocked_by: [MIMIR-T-0649, MIMIR-T-0650, MIMIR-T-0651]
archived: false

tags:
  - "#task"
  - "#phase/active"


exit_criteria_met: false
initiative_id: MIMIR-I-0069
---

# Cleanup: dissolve bookkeeping tests + ServiceError From impl

## Parent Initiative

[[MIMIR-I-0069]] — MCP tool registry with typed arguments

## Objective **[REQUIRED]**

Finish the deepening: remove the scaffolding the registry makes structurally
unnecessary, and centralize error mapping — closing MIMIR-I-0073 as absorbed.

## Acceptance Criteria

## Acceptance Criteria **[REQUIRED]**

- [ ] `EXPECTED_TOOLS`, `published_tools_match_expected_count`, `every_published_tool_has_a_route`, `no_duplicate_tool_names` deleted or reduced to trivial registry invariants (duplicate-name check can live in registry construction)
- [ ] `impl From<ServiceError> for McpError` in `mimir-mcp/src/error.rs`; all remaining per-handler `match e { ServiceError::... }` blocks deleted (registry dispatch or `?` conversions handle it)
- [ ] Dead helpers removed (`create_properties` if unused, legacy dispatch plumbing)
- [ ] Full workspace suite green; stdio e2e green
- [ ] MIMIR-I-0073 transitioned to completed with a note: absorbed by this task

## Status Updates **[REQUIRED]**

- 2026-07-08: COMPLETE on `feat/mcp-tool-registry` (commit 2823592).
  EXPECTED_TOOLS + count/route/published tests deleted; no_duplicate_tool_names
  and all_tools_have_descriptions kept as real invariants. Error mapping
  centralized: new `McpError::caller_fault` in error.rs for the
  InvalidArguments-classifying families (module, character); pre-existing
  `From<ServiceError>` impl retained for homebrew's NotFound-variant contract —
  full unification deliberately NOT done because the two families publish
  different not-found classifications, both pinned by tests. All per-handler
  match blocks deleted. create_properties removed. Net −134 lines.
  MIMIR-I-0073 closed as absorbed. Full suite + stdio e2e green.