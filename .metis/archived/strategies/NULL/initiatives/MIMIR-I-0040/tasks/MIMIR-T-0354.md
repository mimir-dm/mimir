---
id: update-mcp-server-document-tools
level: task
title: "Update MCP Server Document Tools for DB Storage"
short_code: "MIMIR-T-0354"
created_at: 2026-01-19T21:27:10.813026+00:00
updated_at: 2026-01-19T21:27:10.813026+00:00
parent: MIMIR-I-0040
blocked_by: []
archived: true

tags:
  - "#task"
  - "#phase/todo"


exit_criteria_met: false
strategy_id: NULL
initiative_id: MIMIR-I-0040
---

# Update MCP Server Document Tools for DB Storage

## Parent Initiative

[[MIMIR-I-0040]] - Database-Only Document Storage

## Objective

Update MCP server document tools to use DB-backed content storage for markdown documents.

## Acceptance Criteria

## Acceptance Criteria

- [ ] `read_document` tool returns content from DB for markdown
- [ ] `edit_document` tool reads/writes via DB for markdown
- [ ] `create_document` tool (if exists) stores content in DB
- [ ] Binary document handling unchanged (uses file_path)
- [ ] All MCP document operations are atomic
- [ ] MCP server tests pass

## Implementation Notes

### Key Changes
The MCP server uses DocumentService internally. After MIMIR-T-0352, most changes should be inherited. Verify:
- MCP tools call DocumentService methods (not direct file I/O)
- Error handling properly propagates DB errors
- Content is returned correctly in tool responses

### Files to Modify
- `crates/mimir-dm-mcp/src/tools/document.rs`

### Tools to Review
- `ReadDocument::execute()` - verify uses DocumentService
- `EditDocument::execute()` - verify uses DocumentService for read/write
- Any other document-related tools

### Dependencies
- MIMIR-T-0351 (Schema Migration)
- MIMIR-T-0352 (DocumentService Update)

## Status Updates

*To be added during implementation*