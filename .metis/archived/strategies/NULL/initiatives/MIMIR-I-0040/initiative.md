---
id: database-only-document-storage
level: initiative
title: "Database-Only Document Storage"
short_code: "MIMIR-I-0040"
created_at: 2026-01-19T21:15:52.386006+00:00
updated_at: 2026-01-19T21:58:47.798869+00:00
parent: MIMIR-V-0001
blocked_by: []
archived: true

tags:
  - "#initiative"
  - "#phase/completed"


exit_criteria_met: false
estimated_complexity: M
strategy_id: NULL
initiative_id: database-only-document-storage
---

# Database-Only Document Storage Initiative

## Context

The current architecture stores document content as files on the filesystem with metadata in SQLite. This hybrid approach has critical data integrity issues:

**Root Cause Analysis (from code review):**
1. **Non-atomic file writes** - `fs::write()` truncates before writing. A crash mid-write leaves empty/corrupted files.
2. **File/DB desync** - Files are written BEFORE database commits. If DB insert fails, orphaned files exist. If file write partially succeeds, DB still commits.
3. **No transaction spanning file+DB** - Operations are not atomic across the two storage systems.
4. **Silent error swallowing** - API layer returns `Ok(ApiResponse::error(...))`, making frontend think saves succeeded when they failed.
5. **No file locking** - Concurrent writes can corrupt data.

**Impact:** Campaign data has been lost 4+ times due to these issues.

**Affected code paths:**
- `DocumentService::save_document_file()` - `crates/mimir-dm-core/src/services/document_service.rs:333`
- `DocumentService::create_document_from_template()` - lines 270, 284
- `DocumentService::create_user_document()` - lines 389, 404
- `ModuleService` - similar patterns at lines 166, 358
- MCP `edit_document` tool - `crates/mimir-dm-mcp/src/tools/document.rs:265-330`

## Goals & Non-Goals

**Goals:**
- Eliminate data loss from file/DB desynchronization
- Make all document operations atomic via SQLite transactions
- Enable trivial backup/restore (single DB file + asset directories)

**Non-Goals:**
- External file editing (VS Code, etc.) - can add export feature later if needed
- Changing the document data model or types
- Migrating binary assets - maps, tokens, images, uploaded files remain on filesystem (one consistent method for binary assets)
- Document versioning (can add later if needed)
- Automatic migration of existing campaigns - users will export/re-import

## Architecture

### Current State
```
┌─────────────┐     ┌──────────────┐
│   SQLite    │     │  Filesystem  │
│  (metadata) │     │  (content)   │
└──────┬──────┘     └──────┬───────┘
       │                   │
       │    NOT ATOMIC     │
       └─────────┬─────────┘
                 │
         ┌───────▼───────┐
         │DocumentService│
         └───────────────┘
```

### Target State
```
┌─────────────────────────────┐
│          SQLite             │
│  (metadata + content)       │
│  WAL mode = crash recovery  │
│  Transactions = atomicity   │
└──────────────┬──────────────┘
               │
       ┌───────▼───────┐
       │DocumentService│
       └───────────────┘
```

### Schema Changes

```sql
-- Migration 046: Add content storage to documents
ALTER TABLE documents ADD COLUMN content TEXT;
-- Only markdown content stored in DB; images/binary remain on filesystem

-- Note: file_path column retained for binary files (images, uploaded assets)
-- For markdown documents, file_path becomes unused/nullable
```

### What Stays on Filesystem
- Maps (UVTT files, map images)
- Tokens
- Uploaded images (PNG, JPG, WEBP, GIF, SVG document types)
- Any other binary assets

### What Moves to Database
- Markdown document content only (`file_type = 'markdown'`)

## Detailed Design

### Phase 1: Schema Migration
- Add `content` TEXT column to `documents` table
- Keep `file_path` column for binary files (images)

### Phase 2: Update Backend Services
- `DocumentService::save_document_file()` → `DocumentRepository::update_content()` for markdown
- `DocumentService::read_document_file()` → `DocumentRepository::find_by_id().content` for markdown
- `DocumentService::create_*()` → Single INSERT with content for markdown
- `DocumentService::delete_document()` → Single DELETE (no file cleanup for markdown)
- Binary files (images) continue to use filesystem via `file_path`
- Add conditional logic: `if file_type == "markdown"` → use DB, else → use filesystem

### Phase 3: Update MCP Server
- `edit_document` tool reads/writes via DocumentService (DB-backed for markdown)
- `read_document` tool returns content from DB for markdown
- Binary documents continue to use file_path

### Phase 4: Update Frontend
- `DocumentService.ts` - already uses API, should just work
- Verify caching behavior still correct

### Phase 5: Character Versions Cleanup
- `character_versions` table already has `character_data` TEXT column
- Remove usage of `file_path` column in character version operations
- Stop writing character JSON files to disk

### Phase 6: Test & Verify
- Integration tests for all document CRUD paths
- Verify markdown save/load works atomically
- Verify image upload still works via filesystem
- Test campaign export/import still functions

## Alternatives Considered

### 1. Atomic File Writes (temp file + rename)
```rust
let temp_path = path.with_extension("tmp");
fs::write(&temp_path, content)?;
fs::rename(&temp_path, &path)?;
```
**Rejected:** Still doesn't solve file/DB desync. Two storage systems remain.

### 2. Write-Ahead Log for Files
Custom journaling for file operations.
**Rejected:** Reinventing SQLite poorly. Significant complexity.

### 3. Embedded Document DB (e.g., sled, redb)
Replace SQLite with a document database.
**Rejected:** Loses SQL query capabilities. Migration complexity. SQLite is battle-tested.

### 4. Keep Hybrid but Add Transactions
Use SQLite to track file state, rollback on failure.
**Rejected:** Still complex, still two systems to keep in sync.

**Conclusion:** Moving content into SQLite is the simplest, most robust solution.

## Key Decisions

| Question | Decision | Rationale |
|----------|----------|-----------|
| Binary content (images)? | Stay on filesystem | One consistent method for all binary assets (maps, tokens, images) |
| Character versions? | Use existing `character_data` column | Data already in DB, file_path is redundant |
| Campaign directories needed? | Yes, for binary assets | Maps, tokens, images, uploaded files |
| Migration strategy? | No migration, clean break | Users export old campaigns, import fresh. Simpler than dual-source logic |
| Rollback plan? | Archive/import is recovery | No need for complex rollback - export handles it |
| Document versioning? | Not now | Can add later if needed, keeps scope small |

## Implementation Plan

1. **Schema Migration** - Add `content` TEXT column to documents table
2. **DocumentService Update** - Replace fs operations with DB operations for markdown
3. **ModuleService Update** - Same treatment for module overview markdown files
4. **MCP Server Update** - Update document tools to use DB-backed content
5. **Character Versions Cleanup** - Stop using file_path, rely on character_data column
6. **Frontend Verification** - Ensure frontend still works correctly
7. **Integration Testing** - Test all document CRUD paths, verify atomicity