---
id: document-rendering-tests-campaign
level: task
title: "Document rendering tests — campaign documents, module documents, markdown rendering"
short_code: "MIMIR-T-0545"
created_at: 2026-03-10T01:31:30.388356+00:00
updated_at: 2026-03-10T01:58:30.994785+00:00
parent: MIMIR-I-0056
blocked_by: []
archived: true

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: MIMIR-I-0056
---

# Document rendering tests — campaign documents, module documents, markdown rendering

**Phase 3** — Campaign & Module Coverage

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[MIMIR-I-0056]]

## Objective

Write Vitest component tests for document rendering — both campaign-level and module-level documents. Test that markdown content renders correctly, document ordering is preserved, and the document viewer handles various content types (plain text, markdown with headers/lists/tables, embedded references).

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] Campaign documents render with correct titles and markdown content
- [ ] Module documents render in their configured sort order
- [ ] Markdown headings, lists, tables, and code blocks render correctly
- [ ] 5etools cross-references (`{@spell fireball}`, `{@creature goblin}`) are parsed and rendered as links/modals
- [ ] Document search filters results correctly
- [ ] Empty document state renders appropriately
- [ ] Document editor saves content via correct invoke call
- [ ] All tests pass in CI

## Key Components

- Document viewer/reader component
- Markdown rendering pipeline
- 5etools reference parser (`textFormatting.ts`)
- Document list with ordering

## Implementation Notes

Create fixture documents with various markdown content and embedded 5etools references. Test both the rendering output and the reference parsing logic. The `textFormatting.ts` utility in `features/sources/utils/` handles 5etools tag parsing.

## Status Updates

### 2026-03-09
- Created 3 test files covering document rendering:
  - `textFormatting.test.ts` — 47 tests (bold/italic/bi, dice rolls, cross-references for spell/creature/item/condition/class/feat/race/background/action, skills/senses, DC checks, attack types, recharge/chance/note, edge cases, formatEntries with nested structures)
  - `contentRenderer.test.ts` — 20 tests (renderSection with h1, paragraphs, section/entries/inset/list/table/quote/image/item/statblock types, 5etools tag processing, depth-based headers)
  - `ModuleDocumentsPanel.test.ts` — 10 tests (title formatting with kebab-case and underscores, multiple docs, empty state, create/select/delete events)
- All 77 tests passing across 3 files