---
id: implement-documentservice-for-crud
level: task
title: "Implement DocumentService for CRUD operations"
short_code: "MIMIR-T-0396"
created_at: 2026-01-21T03:02:30.681507+00:00
updated_at: 2026-01-21T13:24:10.467375+00:00
parent: MIMIR-I-0044
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: MIMIR-I-0044
---

# Implement DocumentService for CRUD operations

## Parent Initiative

[[MIMIR-I-0044]] - Implement Service Layer for mimir-core

## Objective

Implement `DocumentService` for managing campaign and module documents. This provides CRUD operations for the markdown documents created with campaigns/modules, plus support for creating ad-hoc blank documents.

## Acceptance Criteria

## Acceptance Criteria

- [x] `DocumentService` struct with stateful connection pattern
- [x] `list_for_campaign()` - list all campaign-level documents
- [x] `list_for_module()` - list all module documents
- [x] `get()` - get document by ID
- [x] `update()` - update document title and/or content
- [x] `delete()` - delete a document
- [x] `create()` - create ad-hoc document for campaign or module (named `create` with builder pattern input)
- [x] Update sets `updated_at` timestamp
- [x] Unit tests for all operations (16 tests)

### Additional Methods Implemented
- `list_by_type()` - list documents by doc_type
- `move_to_module()` - move document to a module
- `move_to_campaign()` - move document out of module to campaign level
- `exists()` - check if document exists
- `count_for_campaign()` - count all documents for campaign
- `count_for_module()` - count documents in a module
- `search()` - full-text search across campaign documents
- `search_in_module()` - full-text search within module

## Implementation Notes

### Files to Create

```
crates/mimir-core/src/services/
├── mod.rs              # Add document module export
├── document.rs         # DocumentService implementation
```

### Also Required: Document DAL

If not already present, create `dal/campaign/document.rs` with:
- `insert_document()`
- `get_document()`
- `list_documents_for_campaign()`
- `list_documents_for_module()`
- `update_document()`
- `delete_document()`

### DocumentService API

```rust
pub struct DocumentService<'a> {
    conn: &'a mut SqliteConnection,
}

pub struct CreateBlankDocument {
    pub campaign_id: i32,
    pub module_id: Option<i32>,
    pub title: String,
}

pub struct UpdateDocument {
    pub title: Option<String>,
    pub content: Option<String>,
}

impl<'a> DocumentService<'a> {
    pub fn new(conn: &'a mut SqliteConnection) -> Self;
    
    /// List documents for a campaign (campaign-level only, no module docs)
    pub fn list_for_campaign(&mut self, campaign_id: i32) -> ServiceResult<Vec<Document>>;
    
    /// List documents for a specific module
    pub fn list_for_module(&mut self, module_id: i32) -> ServiceResult<Vec<Document>>;
    
    /// Get document by ID
    pub fn get(&mut self, id: &str) -> ServiceResult<Option<Document>>;
    
    /// Update document title and/or content
    pub fn update(&mut self, id: &str, input: UpdateDocument) -> ServiceResult<Document>;
    
    /// Delete a document
    pub fn delete(&mut self, id: &str) -> ServiceResult<()>;
    
    /// Create a blank document (ad-hoc user document)
    pub fn create_blank(&mut self, input: CreateBlankDocument) -> ServiceResult<Document>;
}
```

### Document Schema Reference

```rust
pub struct Document {
    pub id: String,           // UUID
    pub campaign_id: String,
    pub module_id: Option<String>,
    pub title: String,
    pub content: String,      // Full markdown content
    pub doc_type: String,     // e.g., "campaign_pitch", "user_document"
    pub created_at: String,
    pub updated_at: String,
}
```

### Blank Document Creation

```rust
pub fn create_blank(&mut self, input: CreateBlankDocument) -> ServiceResult<Document> {
    let new_doc = NewDocument {
        id: uuid::Uuid::new_v4().to_string(),
        campaign_id: input.campaign_id.to_string(),
        module_id: input.module_id.map(|id| id.to_string()),
        title: input.title,
        content: String::new(),  // Blank content
        doc_type: "user_document".to_string(),
    };
    dal::document::insert(self.conn, &new_doc)?;
    Ok(Document::from(new_doc))
}
```

### Dependencies

- MIMIR-T-0390 (ServiceError type)
- Existing or new `dal::document` module

## Status Updates

*To be added during implementation*