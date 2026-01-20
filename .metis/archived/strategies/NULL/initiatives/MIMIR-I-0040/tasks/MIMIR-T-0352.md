---
id: update-documentservice-for-db
level: task
title: "Update DocumentService for DB-Backed Markdown Storage"
short_code: "MIMIR-T-0352"
created_at: 2026-01-19T21:27:10.322548+00:00
updated_at: 2026-01-19T21:27:10.322548+00:00
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

# Update DocumentService for DB-Backed Markdown Storage

## Parent Initiative

[[MIMIR-I-0040]] - Database-Only Document Storage

## Objective

Update DocumentService to read/write markdown content from the database instead of the filesystem, while preserving filesystem operations for binary files (images).

## Acceptance Criteria

## Acceptance Criteria

- [ ] `save_document_file()` writes to `content` column for markdown documents
- [ ] `read_document_file()` reads from `content` column for markdown documents
- [ ] Binary files (images) continue to use filesystem via `file_path`
- [ ] `create_document_from_template()` stores content in DB, not file
- [ ] `create_user_document()` stores content in DB, not file
- [ ] `delete_document()` no longer needs file cleanup for markdown
- [ ] All operations wrapped in transactions for atomicity
- [ ] No `fs::write()` calls for markdown documents
- [ ] Existing tests pass or are updated

## Implementation Notes

### Key Changes

```rust
// Conditional logic based on file type
pub fn save_document_content(&mut self, doc_id: i32, content: &str) -> Result<()> {
    let doc = DocumentRepository::find_by_id(self.conn, doc_id)?;
    if doc.file_type == "markdown" {
        // Write to DB
        DocumentRepository::update_content(self.conn, doc_id, content)
    } else {
        // Write to filesystem (images)
        fs::write(&doc.file_path, content)
    }
}
```

### Files to Modify
- `crates/mimir-dm-core/src/services/document_service.rs`
- `crates/mimir-dm-core/src/dal/campaign/documents.rs` (add update_content, content field handling)

### Methods to Update
- `save_document_file()` → conditional DB/file logic
- `read_document_file()` → conditional DB/file logic
- `create_document_from_template()` → INSERT with content
- `create_user_document()` → INSERT with content
- `upload_document()` → DB for markdown, file for binary
- `delete_document()` → simplify, only delete file for binary

### Dependencies
- MIMIR-T-0351 (Schema Migration) must be complete first

## Status Updates

*To be added during implementation*