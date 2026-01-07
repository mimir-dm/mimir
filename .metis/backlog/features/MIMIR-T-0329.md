---
id: arbitrary-document-creation-for
level: task
title: "Arbitrary Document Creation for Campaigns and Modules"
short_code: "MIMIR-T-0329"
created_at: 2026-01-07T09:32:30.095083+00:00
updated_at: 2026-01-07T09:32:30.095083+00:00
parent: 
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/backlog"
  - "#feature"


exit_criteria_met: false
strategy_id: NULL
initiative_id: NULL
---

# Arbitrary Document Creation for Campaigns and Modules

## Objective

Allow users to create arbitrary markdown documents at the campaign or module level without being constrained to predefined templates. This enables freeform note-taking, custom content types, and flexible organization.

## Backlog Item Details

### Type
- [x] Feature - New functionality or enhancement  

### Priority
- [ ] P1 - High (important for user experience)

### Business Justification
- **User Value**: DMs often need to create custom documents that don't fit existing templates (e.g., custom lore pages, puzzle solutions, DM notes, custom tables)
- **Business Value**: Removes friction and makes Mimir more flexible for diverse campaign styles
- **Effort Estimate**: M (upload, preview, export - gallery cut from scope)

## Acceptance Criteria

### Document Creation
- [ ] Add "New Document" button/action at campaign level
- [ ] Add "New Document" button/action at module level
- [ ] Prompt for document title and optional document type/category
- [ ] Create blank markdown file with minimal frontmatter
- [ ] Support for custom folder placement within campaign structure
- [ ] Add MCP tool for creating arbitrary documents programmatically

### File Upload Support
- [ ] Upload existing markdown files (.md)
- [ ] Upload image files (PNG, JPG, WEBP, GIF, SVG)
- [ ] Validate file types before accepting upload
- [ ] Store uploaded files in appropriate folder (`resources/images/` for images)
- [ ] Register uploaded files in documents table
- [ ] Support drag-and-drop upload

### Future Enhancements (cut from initial scope)
- ~~Thumbnail grid view for images in a dedicated gallery section~~
- ~~Copy markdown image reference to clipboard for embedding in documents~~
- Consider image optimization/resizing on upload for large files

### Visual Differentiation
- [ ] User-created documents show distinct visual indicator in sidebar/lists
- [ ] Differentiate from template-generated documents (e.g., badge, icon, or styling)
- [ ] Consider using `is_user_created` or `source` field in documents table

### Preview & Viewing
- [ ] Preview function detects file type (markdown vs image)
- [ ] Markdown files render in editor/preview as normal
- [ ] Image files (PNG/SVG) display in image viewer component
- [ ] Smooth switching between document types in preview pane

### Export & Printing
- [ ] User-created markdown documents included in PDF exports
- [ ] Uploaded images included in PDF exports (with appropriate sizing)
- [ ] User documents included in campaign archive exports
- [ ] Maintain document ordering in exports

## Implementation Notes

### Technical Approach

**Document Creation:**
1. Add `create_document` method to DocumentService for blank documents
2. Accept parameters: title, level (campaign/module), optional module_id, optional subfolder
3. Generate appropriate file path based on level and folder
4. Create minimal frontmatter with title and timestamps
5. Add Tauri command and frontend UI
6. Add `create_document` MCP tool

**File Upload:**
1. Add `upload_document` Tauri command accepting file path and metadata
2. Copy file to campaign/module folder with sanitized filename
3. For markdown: parse frontmatter if present, otherwise generate
4. For images: store as-is, use filename as title
5. Register in documents table with `file_type` field (markdown/image)

**Preview System:**
1. Add `file_type` column to documents table (markdown, png, svg, etc.)
2. DocumentEditor/Preview component checks file_type
3. Route to appropriate viewer: TipTap for markdown, img element for images
4. Consider using a unified DocumentViewer component with type switching

### Database Changes
```sql
ALTER TABLE documents ADD COLUMN file_type TEXT DEFAULT 'markdown';
ALTER TABLE documents ADD COLUMN is_user_created BOOLEAN DEFAULT FALSE;
```

### Document Levels
- Campaign level: stored in campaign root or custom subfolder
- Module level: stored in `modules/<module_name>/` folder
- Images: consider `resources/` or `images/` subfolder

### Frontmatter Structure (Markdown)
```yaml
---
title: "My Custom Document"
document_type: "custom"
created_at: "2026-01-07"
updated_at: "2026-01-07"
---
```

### Export Integration
- Modify PrintService to handle image documents
- Typst templates need image embedding support
- Archive export already copies files; ensure new documents included

### Dependencies
- Existing DocumentService and document infrastructure
- PrintService for PDF generation
- Archive export functionality

### Risk Considerations
- Need to handle filename sanitization for titles
- Avoid path collisions with existing files
- Large images may need resizing for PDF export
- SVG rendering in Typst may have limitations

## Status Updates

*To be added during implementation*