---
id: migrate-campaign-document-tool
level: task
title: "Migrate campaign + document tool families"
short_code: "MIMIR-T-0649"
created_at: 2026-07-08T11:12:11.428410+00:00
updated_at: 2026-07-08T11:12:11.428410+00:00
parent: MIMIR-I-0069
blocked_by: ["MIMIR-T-0648"]
archived: false

tags:
  - "#task"
  - "#phase/todo"


exit_criteria_met: false
initiative_id: MIMIR-I-0069
---

# Migrate campaign + document tool families

## Parent Initiative

[[MIMIR-I-0069]] — MCP tool registry with typed arguments

## Objective **[REQUIRED]**

Move the campaign family (11 tools incl. export/import/preview archive) and the
document family (6 tools) onto the registry established in [[MIMIR-T-0648]].

## Acceptance Criteria **[REQUIRED]**

- [ ] All 17 tools defined via typed arg structs, registered once; their `get_tools()` entries and match arms deleted
- [ ] `set_active_campaign` / `get_active_campaign` session-state behavior unchanged (context mutation stays in handlers, not the registry)
- [ ] Wire compatibility: existing campaign/document functional tests and the stdio e2e pass unchanged
- [ ] `crates/mimir-mcp/src/tools/campaign.rs` and `document.rs` shrink to arg structs + handlers only (no hand-built schemas)

## Implementation Notes

- `export_campaign`/`import_campaign` take filesystem paths — keep the same arg names and error text.
- `edit_document` has MCP-only search/replace pre-logic; that stays in the handler (presentation), just with typed args.

## Status Updates **[REQUIRED]**

*To be added during implementation*
