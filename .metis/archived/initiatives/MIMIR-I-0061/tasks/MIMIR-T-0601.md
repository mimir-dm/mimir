---
id: pass-1-5-fix-module-docs-add
level: task
title: "Pass 1.5: Fix module docs — add-monsters.md and module-documents.md"
short_code: "MIMIR-T-0601"
created_at: 2026-03-13T13:50:11.473759+00:00
updated_at: 2026-03-13T14:07:53.907115+00:00
parent: MIMIR-I-0061
blocked_by: []
archived: true

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: MIMIR-I-0061
---

# Pass 1.5: Fix module docs — add-monsters.md and module-documents.md

## Parent Initiative

[[MIMIR-I-0061]]

## Objective

Fix `how-to/modules/add-monsters.md` to mention homebrew monster support, and fix `how-to/modules/module-documents.md` to remove fictional templates and stages (same issue as campaign documents).

## Scope

### `how-to/modules/add-monsters.md`

**Issue from audit (MIMIR-T-0579):** Guide only describes catalog monster search. The module monsters system (`ModuleMonsters.vue`) supports both catalog AND homebrew monsters. Add a note/section about homebrew monsters being available if created in the Homebrew tab.

**Verification:** `ModuleMonsters.vue` — check how homebrew monsters appear in search alongside catalog results.

### `how-to/modules/module-documents.md`

**Issues from audit (MIMIR-T-0579):** Same fictional templates and stages as `manage-documents.md`:
1. Claims templates exist — they don't
2. Claims stages "In Progress, Complete" — no stage system exists

Apply the same fix pattern as T-0600: remove fictional sections, describe actual auto-save document workflow.

**Verification:** Same sources as T-0600 — `CreateDocumentModal.vue`, `Document` model.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] add-monsters.md mentions homebrew monsters alongside catalog monsters
- [ ] module-documents.md has zero mentions of templates or stages
- [ ] module-documents.md accurately describes actual document workflow
- [ ] All changes verified against source code

## Status Updates

### 2026-03-13: Completed

**add-monsters.md:**
- Added homebrew monster mention in intro and search results description
- Added "Module Monsters Quick Select" section describing the quick-select buttons in TokenPalette
- Added homebrew tip

**module-documents.md — full rewrite:**
- Removed fictional "Document Templates" section (pre-filled sections, prompts — none exist)
- Removed fictional "Document Stages" section (In Progress/Complete — no stage field in Document model)
- Removed fictional "Organizing Documents" section (sorting by stage — doesn't exist)
- Added accurate creation flow (New Document / Upload File modes)
- Added editor capabilities (headings, formatting, lists, tables)
- Added auto-save behavior
- Added reordering via up/down arrows
- Added PDF export mention

All verified against: `CreateDocumentModal.vue`, `DocumentEditor.vue`, `DocumentSidebar.vue`, `TokenPalette.vue`.