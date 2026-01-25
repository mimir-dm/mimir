---
id: implement-book-content-reading
level: task
title: "Implement Book Content Reading Support"
short_code: "MIMIR-T-0410"
created_at: 2026-01-22T02:29:59.865276+00:00
updated_at: 2026-01-25T01:03:03.214153+00:00
parent: 
blocked_by: []
archived: true

tags:
  - "#task"
  - "#feature"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: NULL
---

# Implement Book Content Reading Support

Enable reading mode in the Reference view by importing and serving 5etools book content.

## Objective

Add backend support for storing and retrieving book content from imported 5etools data, enabling the "Reading" mode in the Reference view to display full book chapters and sections.

## Backlog Item Details

### Type
- [x] Feature - New functionality or enhancement  

### Priority
- [ ] P1 - High (important for user experience)

### Business Justification
- **User Value**: Users can read full book content (PHB chapters, DMG sections, etc.) directly in the app instead of just searching the catalog
- **Business Value**: Completes the Reference view functionality - currently only Catalog mode works
- **Effort Estimate**: M (Medium)

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [x] `books` table added to database schema storing book content as JSON
- [x] Book content imported during catalog import from `data/book/book-{source}.json` files
- [x] Book images copied from `img/book/` to assets directory during import (uses existing copy_images)
- [x] Image paths in book content updated to reference local asset paths
- [x] `get_book_content` Tauri command returns book sections for a given source code
- [ ] Frontend Reading mode displays book table of contents and section content
- [ ] Clicking a book in Reading mode loads and displays its chapters/sections
- [ ] Images render correctly in book content view

## Implementation Notes

### Technical Approach

#### 1. Database Migration
Add `books` table to store book content:
```sql
CREATE TABLE books (
    id INTEGER PRIMARY KEY,
    source TEXT NOT NULL UNIQUE,  -- Source code (PHB, DMG, etc.)
    name TEXT NOT NULL,           -- Display name
    data TEXT NOT NULL,           -- Full book content JSON (sections/entries)
    contents TEXT,                -- Table of contents JSON
    cover_path TEXT,              -- Path to cover image
    FOREIGN KEY (source) REFERENCES catalog_sources(code)
);
```

#### 2. Import Service Updates
- Add book content collection in `collector.rs`:
  - Read `data/book/book-{source}.json` files
  - Extract the `data` array (sections with entries)
- Add `import_book()` in `service.rs` following existing entity pattern
- Store full JSON in `data` field, extract `contents` for TOC
- **Image handling** (extend existing `images.rs` logic):
  - Copy book images from `img/book/{SOURCE}/` to `assets/catalog/book/{SOURCE}/`
  - Rewrite image `href.path` entries in book JSON to use local paths
  - Handle cover images from `img/covers/` directory
  - Use existing `ImageCopier` pattern from monster token imports

#### 3. DAL Layer
- Add `dal/catalog/book.rs` with:
  - `insert_book()` - Insert book content
  - `get_book_by_source()` - Retrieve by source code
  - `list_books()` - List all available books

#### 4. Tauri Command
Add `get_book_content` command in `commands/source.rs`:
```rust
#[tauri::command]
pub fn get_book_content(
    state: State<'_, AppState>,
    book_id: String,
) -> ApiResponse<BookContent>
```

#### 5. Frontend (already exists)
- `useBookContent.ts` already calls `get_book_content` with `bookId`
- `useBookLibrary.ts` already loads catalog sources for display
- May need minor adjustments to content rendering

### Key Files to Modify
- `crates/mimir-core/src/schema.rs` - Add books table
- `crates/mimir-core/migrations/` - New migration
- `crates/mimir-core/src/dal/catalog/mod.rs` - Export book module
- `crates/mimir-core/src/dal/catalog/book.rs` - New file
- `crates/mimir-core/src/models/catalog/mod.rs` - Book model
- `crates/mimir-core/src/import/collector.rs` - Collect book content
- `crates/mimir-core/src/import/service.rs` - Import books
- `crates/mimir-core/src/import/images.rs` - Book image copying & path rewriting
- `crates/mimir/src/commands/source.rs` - get_book_content command
- `crates/mimir/src/main.rs` - Register command

### 5etools Book Structure
Book content files (`data/book/book-phb.json`) contain:
```json
{
  "data": [
    {
      "type": "section",
      "name": "Chapter 1: Step-by-Step Characters",
      "page": 11,
      "id": "001",
      "entries": [...]
    }
  ]
}
```

### 5etools Image Structure
Images in entries use this format:
```json
{
  "type": "image",
  "href": {
    "type": "internal",
    "path": "book/PHB/001-intro.webp"
  },
  "title": "Image Title"
}
```

Image source locations:
- Book images: `img/book/{SOURCE}/*.webp`
- Cover images: `img/covers/{source}.webp`
- Adventure images: `img/adventure/{SOURCE}/*.webp`

During import, rewrite `href.path` to local asset path (e.g., `assets://catalog/book/PHB/001-intro.webp`)

### Dependencies
- Catalog import must run first to populate `catalog_sources`
- Frontend content renderer must handle entry types (section, image, table, etc.)

## Status Updates

### Session 1 - Backend Implementation Complete

**Completed:**
1. **Database Migration** (`migrations/019_books/up.sql`)
   - Created `books` table with id, source, name, data, contents, cover_path
   - Added foreign key constraint to catalog_sources
   - Created index on source column

2. **Schema Updates** (`schema.rs`)
   - Added books table definition to diesel schema
   - Added joinable and allow_tables_to_appear_in_same_query macros

3. **Book Model** (`models/catalog/book.rs`)
   - `Book` - Queryable struct for database reads
   - `NewBook` - Insertable struct for database writes
   - `BookContent` - API response struct with parsed JSON data
   - Builder pattern with `with_contents()` and `with_cover_path()`

4. **Book DAL** (`dal/catalog/book.rs`)
   - `insert_book()` - Insert new book
   - `upsert_book()` - Insert or update existing
   - `get_book_by_source()` - Get by source code
   - `list_books()` - List all books
   - `delete_book()`, `book_exists()`, `count_books()`
   - Updated `delete_source_cascade` to include books table

5. **Import Collector** (`import/collector.rs`)
   - Added `book_content` and `book_contents_toc` fields to `CollectedEntities`
   - Added `collect_book_content()` function to load from `data/book/book-{source}.json`
   - Extracts TOC from `books.json` metadata

6. **Import Service** (`import/service.rs`)
   - Added `import_book()` method to import book content
   - Added `rewrite_book_image_paths()` to update image paths
   - Cover image detection for webp/jpg/png formats

7. **Tauri Commands** (`commands/source.rs`)
   - `list_books` - Returns all books with readable content
   - `get_book_content` - Returns full book content for reading
   - Registered in `main.rs`

**Build Status:** Compiles successfully

**Next Steps:**
- Test with actual 5etools import
- Verify book images are copied correctly
- Test frontend integration

### Session 2 - Image Import & Frontend Integration

**Key Discovery:**
5etools images are in a **separate repository** (5etools-img), not in the main data zip.
The image archive structure is `5etools-img-X.Y.Z/` containing:
- `book/` - Book images
- `bestiary/` - Monster tokens
- `covers/` - Book covers
- `adventure/` - Adventure images

**Completed:**

8. **Separate Image Import Command** (`commands/source.rs`)
   - Added `import_catalog_images` Tauri command
   - Extracts image zip to temp directory
   - Uses `find_img_directory()` to locate img root (handles nested structure)
   - Calls `mimir_core::import::copy_images()` to copy ALL images to `assets/catalog/`
   - Registered command in `main.rs`

9. **Frontend Import Images Button** (`BookManagementModal.vue`)
   - Added "Import Images" button to modal footer
   - Added `handleImportImages()` function to invoke the new command
   - Uses same dialog pattern as data import

10. **Logging/Tracing** (`main.rs`, `Cargo.toml`)
   - Added `tracing-subscriber` dependency with env-filter feature
   - Initialize tracing in main() for proper log output
   - Use `RUST_LOG=mimir=debug` for detailed logs

11. **Content Rendering Fixes** (`contentRenderer.ts`, `textFormatting.ts`)
   - Added handling for `type: "item"` entries (name/entry pairs)
   - Added handling for `type: "options"`, `type: "inline"`, `type: "inlineBlock"`
   - Added `{@5etoolsImg}` tag processing

**Build Status:** Compiles successfully, frontend type-checks pass

**User Testing Status:**
- Books import and display in Reading mode
- Book content renders correctly
- Images not loading - user needs to import from separate 5etools-img archive

**Ready for Testing:**
1. User downloads 5etools-img archive
2. Click "Import Images" in Manage Catalog Sources modal
3. Select the 5etools-img zip file
4. Images should copy to assets/catalog/ and display in book content