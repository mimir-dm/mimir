---
id: add-sort-ordering-for-user-created
level: task
title: "Add sort ordering for user-created campaign documents"
short_code: "MIMIR-T-0525"
created_at: 2026-02-14T15:19:57.457967+00:00
updated_at: 2026-02-16T03:27:34.468057+00:00
parent: 
blocked_by: []
archived: false

tags:
  - "#task"
  - "#feature"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: NULL
---

# Add sort ordering for user-created campaign documents

## Objective

Add `sort_order` support to user-created campaign documents so they can be manually reordered, following the same pattern used for module reordering.

Template documents (campaign_pitch, world_primer, etc.) keep their existing hardcoded sort order in the frontend. This only applies to user-created documents.

## Details

- **Priority**: P2
- **Effort**: S

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `sort_order` column added to `documents` table via migration
- [ ] `reorder_document` DAL function implemented (following `reorder_module` pattern)
- [ ] Tauri command and MCP tool for reordering documents
- [ ] User-created documents in DocumentSidebar are ordered by `sort_order`
- [ ] New user documents get the next available sort_order automatically
- [ ] Unit tests for reorder logic

## Implementation Notes

### Reference Pattern
- `reorder_module()` in `crates/mimir-core/src/dal/campaign/module.rs` (lines 86-163)
- Uses sentinel value (-1), shift affected rows, place at new position

### Key Files
- Schema: `crates/mimir-core/migrations/` (new migration)
- DAL: `crates/mimir-core/src/dal/campaign/document.rs`
- Model: `crates/mimir-core/src/models/campaign/document.rs`
- Frontend: `crates/mimir/frontend/src/features/campaigns/components/DocumentSidebar.vue`

## Status Updates

### Implementation Complete
- Migration 027: Added `sort_order INTEGER NOT NULL DEFAULT 0` to documents table
- Schema: Updated diesel schema with `sort_order -> Integer`
- Model: Added `sort_order` field to `Document`, `NewDocument` (with `with_sort_order` builder)
- DAL: Added `reorder_document`, `next_campaign_document_sort_order`, `next_module_document_sort_order`, `count_campaign_level_documents`; updated all list queries to order by `sort_order` first
- Service: `DocumentService::create` auto-assigns next sort_order; added `reorder` method
- Tauri: `reorder_document` command registered in main.rs
- MCP: `reorder_document` tool definition and handler
- Frontend: Updated `Document` ts-rs type, `DocumentService.reorder()`, `DocumentSidebar.vue` user docs sorted by `sort_order` with up/down reorder buttons
- Tests: 4 new DAL tests (move down, move up, no-op, invalid position), all 828 mimir-core tests pass, all 24 MCP tests pass