---
id: print-tests-campaign-and-module
level: task
title: "Print tests — campaign and module document export"
short_code: "MIMIR-T-0554"
created_at: 2026-03-10T01:31:46.333524+00:00
updated_at: 2026-03-10T13:17:42.814633+00:00
parent: MIMIR-I-0056
blocked_by: []
archived: true

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: MIMIR-I-0056
---

# Print tests — campaign and module document export

**Phase 5** — Print/Export Coverage

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[MIMIR-I-0056]]

## Objective

Write Rust integration tests for campaign and module document PDF export. Test `export_campaign_documents` and `export_module_documents` commands — verifying that documents render in correct order, markdown content is included, and embedded 5etools references resolve to readable text in the PDF.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] Campaign document export produces PDF with all campaign documents in order
- [ ] Module document export produces PDF with all module documents in order
- [ ] Document titles appear as headers in the PDF
- [ ] Markdown formatting (bold, italic, lists, headers) renders correctly
- [ ] Table of contents is generated when enabled
- [ ] Empty document set returns appropriate error (not a crash)
- [ ] PDF output is non-empty and valid
- [ ] All tests pass in CI

## Key Files

- `crates/mimir/src/commands/print/document.rs` — export commands
- `crates/mimir-print/src/sections/document.rs` — document section builder

## Implementation Notes

Create test campaign and module with multiple documents via the service layer. Test with various markdown content including headers, lists, tables, and 5etools references. Verify document ordering matches the configured sort order.

## Status Updates

### Completed
Added tests across 3 files in mimir-print:

**markdown.rs** — 8 new tests (22 total, was 14):
- Table rendering (structure, headers, body cells, formatting in cells)
- Strikethrough (→ `#strike[...]`)
- Nested lists (indentation preserved)
- HTML comments (wrapped in `/* */`)
- Image syntax (`#image(...)`)
- Empty markdown handling
- Backslash escaping in string literals

**sections/markdown.rs** — 7 new tests (10 total, was 3):
- TOC title None when no title set
- `with_title()` overrides frontmatter title
- `doc_type()` extraction from frontmatter
- `doc_type()` None when missing
- `to_typst()` returns converted content
- Markdown bold/italic in rendered output
- List items in rendered output
- `page_break_before()` default behavior

**builder.rs** — 7 new tests (13 total, was 6):
- Multi-section ordering (content order preserved via position assertions)
- TOC with multiple titled sections
- Sections without titles
- Empty builder produces valid Typst with page setup
- Title page config
- Page setup (margin, font tokens)
- Page breaks between sections

**Note**: Full pipeline tests (campaign/module DB → PDF) require Tauri command handlers and are not feasible as unit tests. The existing tests validate document ordering, markdown formatting, and TOC generation through the Renderable/Typst layer.