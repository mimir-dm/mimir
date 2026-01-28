---
id: mcp-document-tools
level: task
title: "MCP Document Tools"
short_code: "MIMIR-T-0465"
created_at: 2026-01-28T04:06:32.610430+00:00
updated_at: 2026-01-28T04:39:45.909662+00:00
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

# MCP Document Tools

## Parent Initiative

[[MIMIR-I-0050]]

## Objective

Implement MCP tools for managing module documents (narrative content like backstory, read-aloud text, DM notes).

**Reference**: `mimir-dm-bu/mimir-dm-mcp/src/tools/` (document handlers)

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `list_documents` - Returns all documents in a module
- [ ] `read_document` - Returns full document content by ID
- [ ] `edit_document` - Updates document content via search/replace
- [ ] `create_document` - Creates new document in a module
- [ ] Tool schemas registered in ServerHandler
- [ ] Supports document types: backstory, read_aloud, dm_notes, description, custom

## Tools Specification

### list_documents
- **Parameters**: `module_id: string`
- **Returns**: Array of `{id, title, document_type, sort_order}`
- **Uses**: `DocumentService::list_by_module()`

### read_document
- **Parameters**: `document_id: string`
- **Returns**: Full document with content
- **Uses**: `DocumentService::get()`

### edit_document
- **Parameters**: `document_id, search: string, replace: string`
- **Returns**: Updated document
- **Uses**: String replacement on `content` field + `DocumentService::update()`

### create_document
- **Parameters**: `module_id, title, document_type, content?`
- **Returns**: Created document
- **Uses**: `DocumentService::create()`

## Dependencies
- Depends on: MIMIR-T-0461, MIMIR-T-0462, MIMIR-T-0464 (modules must exist)

## Status Updates

*To be added during implementation*