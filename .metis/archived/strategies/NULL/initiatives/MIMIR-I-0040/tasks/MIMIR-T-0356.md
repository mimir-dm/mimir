---
id: integration-testing-for-db-only
level: task
title: "Integration Testing for DB-Only Document Storage"
short_code: "MIMIR-T-0356"
created_at: 2026-01-19T21:27:11.298606+00:00
updated_at: 2026-01-19T21:27:11.298606+00:00
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

# Integration Testing for DB-Only Document Storage

## Parent Initiative

[[MIMIR-I-0040]] - Database-Only Document Storage

## Objective

Verify all document CRUD operations work correctly with DB-only storage and are atomic.

## Acceptance Criteria

## Acceptance Criteria

- [ ] Create markdown document → content in DB, no file created
- [ ] Read markdown document → content from DB
- [ ] Update markdown document → atomic DB update
- [ ] Delete markdown document → DB record removed, no file cleanup needed
- [ ] Upload image → file created on disk, file_path in DB
- [ ] Campaign export includes all document content
- [ ] Campaign import restores documents correctly
- [ ] Crash during save doesn't corrupt data (atomicity test)
- [ ] All existing tests pass
- [ ] MCP document operations work correctly

## Test Cases

### TC-001: Markdown Document Lifecycle
- **Preconditions**: Fresh campaign created
- **Steps**:
  1. Create markdown document via UI
  2. Edit and save content
  3. Read document back
  4. Delete document
- **Expected**: All operations succeed, no orphaned files

### TC-002: Image Upload
- **Preconditions**: Fresh campaign created
- **Steps**:
  1. Upload PNG image as document
  2. Verify file exists on disk
  3. Verify file_path in DB points to file
  4. Delete image document
  5. Verify file removed from disk
- **Expected**: Binary files still use filesystem correctly

### TC-003: Atomicity
- **Preconditions**: Document with content exists
- **Steps**:
  1. Begin save operation
  2. Simulate crash/interruption (if possible)
  3. Restart application
  4. Check document state
- **Expected**: Either old content or new content, never partial/empty

### TC-004: MCP Edit
- **Preconditions**: Campaign with documents
- **Steps**:
  1. Use MCP edit_document tool
  2. Verify content updated in DB
  3. Read document via UI
- **Expected**: Content matches, no file I/O for markdown

## Dependencies
- All other tasks in initiative must be complete

## Status Updates

*To be added during implementation*