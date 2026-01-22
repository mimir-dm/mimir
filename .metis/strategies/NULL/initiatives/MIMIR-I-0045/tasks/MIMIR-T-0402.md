---
id: implement-document-tauri-commands
level: task
title: "Implement Document Tauri commands"
short_code: "MIMIR-T-0402"
created_at: 2026-01-21T16:34:48.647753+00:00
updated_at: 2026-01-21T18:03:59.671474+00:00
parent: MIMIR-I-0045
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: MIMIR-I-0045
---

# Implement Document Tauri commands

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[MIMIR-I-0045]]

## Objective

Implement Tauri commands wrapping `mimir-core` DocumentService for markdown document management. Documents are stored in database (not filesystem).

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [x] List documents (campaign-level and module-level filtering)
- [x] Get document with full content
- [x] Create document
- [x] Update document content
- [x] Delete document
- [x] Search documents (full-text search)

## Implementation Notes

### Commands to Implement

```rust
#[tauri::command] fn list_documents(state, campaign_id: String, module_id: Option<String>) -> Result<Vec<Document>>
#[tauri::command] fn get_document(state, id: String) -> Result<Document>
#[tauri::command] fn create_document(state, input: CreateDocumentInput) -> Result<Document>
#[tauri::command] fn update_document(state, id: String, input: UpdateDocumentInput) -> Result<Document>
#[tauri::command] fn delete_document(state, id: String) -> Result<()>
```

### Key Difference from v1
- Content stored in DB `content` column, not filesystem
- No `file_path` field

### Dependencies
- Blocked by: [[MIMIR-T-0399]] (Rust backend setup)

## Status Updates

### 2026-01-21: Implementation Complete

Created `crates/mimir/src/commands/document.rs` with 9 Tauri commands:

**List Commands (3)**:
- `list_campaign_documents` - Campaign-level documents (not in any module)
- `list_module_documents` - Documents for a specific module
- `list_documents_by_type` - Filter by doc_type

**CRUD Commands (4)**:
- `get_document` - Get by ID with full content
- `create_document` - Create campaign-level or module-level document
- `update_document` - Update title, content, or doc_type
- `delete_document` - Permanent delete

**Search Commands (2)**:
- `search_documents` - Full-text search across campaign
- `search_module_documents` - Full-text search within a module

Note: Move commands omitted by design - documents should be created in the correct location.

All commands registered in `main.rs`, build passes.