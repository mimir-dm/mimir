---
id: delete-campaign-documents-via
level: task
title: "Delete Campaign Documents via Sidebar"
short_code: "MIMIR-T-0330"
created_at: 2026-01-07T10:15:37.553902+00:00
updated_at: 2026-01-07T14:43:11.460855+00:00
parent: 
blocked_by: []
archived: false

tags:
  - "#task"
  - "#feature"
  - "#phase/todo"


exit_criteria_met: false
strategy_id: NULL
initiative_id: NULL
---

# Delete Campaign Documents via Sidebar

## Objective

Allow users to delete unneeded campaign documents directly from the sidebar, removing both the database record and the file on disk.

## Backlog Item Details

### Type
- [x] Feature - New functionality or enhancement  

### Priority
- [x] P2 - Medium (nice to have)

### Business Justification
- **User Value**: Users can clean up optional documents they don't need (safety_tools, house_rules, etc.) without leaving the app or manually deleting files
- **Effort Estimate**: S

## Acceptance Criteria

## Acceptance Criteria

- [ ] Right-click context menu on documents in sidebar shows "Delete" option
- [ ] Confirmation dialog warns user this will delete both the database record and file
- [ ] Deleting a document removes it from the documents table
- [ ] Deleting a document removes the .md file from disk
- [ ] UI updates immediately to reflect deletion
- [ ] Cannot delete documents that are marked as required for the current stage
- [ ] Success/error toast notification after deletion

## Implementation Notes

### Technical Approach
1. Add context menu to document list items in sidebar
2. Create `delete_document` Tauri command that:
   - Gets document by ID
   - Deletes file at `file_path`
   - Deletes database record
3. Add confirmation dialog component
4. Emit event to refresh document list after deletion

### Files to Modify
- `DocumentList.vue` or sidebar component - add context menu
- `document_service.rs` - add delete method
- Tauri commands - add `delete_document` command

### Considerations
- Should soft-delete (archive) vs hard-delete? Recommend hard-delete since files can be regenerated from templates
- Consider whether to check if document is required for current stage before allowing deletion

## Status Updates

*To be added during implementation*