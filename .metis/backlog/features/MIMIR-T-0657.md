---
id: mcp-title-support-in-edit-document
level: task
title: "MCP: title support in edit_document"
short_code: "MIMIR-T-0657"
created_at: 2026-07-10T09:54:20.822430+00:00
updated_at: 2026-07-10T09:54:20.822430+00:00
parent: 
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/backlog"
  - "#feature"


exit_criteria_met: false
initiative_id: NULL
---

# MCP: title support in edit_document

## Objective **[REQUIRED]**

Agents cannot rename documents. `DocumentService::UpdateDocumentInput` supports
`title` and the Tauri `update_document` command passes it through, but the MCP
`edit_document` tool only exposes content search/replace. Discovered 2026-07-10
when the "Player Secrets" → "DM Secrets" rename required direct SQL (via
Python's FTS5-capable sqlite3 — the system CLI can't write `documents` because
of the FTS sync triggers).

## Backlog Item Details

### Type
- [x] Feature - New functionality or enhancement

### Priority
- [ ] P2 - Medium (nice to have)

### Business Justification
- **User Value**: agents authoring documents can fix titles without DB surgery
- **Effort Estimate**: S — one optional field on the registry arg struct

## Acceptance Criteria **[REQUIRED]**

- [ ] `EditDocumentArgs` gains optional `title`; `search`/`replace` become optional as a pair (loosening required is wire-compatible — existing callers always pass them)
- [ ] Validation: at least one of (search+replace) or title must be provided; search without replace (or vice versa) is InvalidArguments
- [ ] Handler passes title into `UpdateDocumentInput`; response includes the new title
- [ ] Functional test: rename-only, content-only, and combined edits
- [ ] Tool description updated so agents discover the capability

## Status Updates **[REQUIRED]**

*To be added during implementation*
