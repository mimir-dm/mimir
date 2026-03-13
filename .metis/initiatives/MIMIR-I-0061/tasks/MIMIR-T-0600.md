---
id: pass-1-4-fix-manage-documents-md
level: task
title: "Pass 1.4: Fix manage-documents.md — remove fictional templates and stages"
short_code: "MIMIR-T-0600"
created_at: 2026-03-13T13:50:10.853939+00:00
updated_at: 2026-03-13T14:07:14.608519+00:00
parent: MIMIR-I-0061
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: MIMIR-I-0061
---

# Pass 1.4: Fix manage-documents.md — remove fictional templates and stages

## Parent Initiative

[[MIMIR-I-0061]]

## Objective

Rewrite `how-to/campaigns/manage-documents.md` to remove the fictional document templates and stages system. The current page describes Draft/Review/Complete stages and 4 document templates — none of which exist in the code. Documents are simply created with a title and auto-saved.

## Scope

### `how-to/campaigns/manage-documents.md` — Major rewrite

**Issues from audit (MIMIR-T-0579):**

1. **Document Templates section is fiction** — Claims Mimir provides templates for "Session notes, Location descriptions, NPC profiles, Encounter plans." The actual `CreateDocumentModal.vue` only has a title input and optional file upload. There is no template selection UI. **Remove the entire "Document Templates" section.**

2. **Document Stages section is fiction** — Claims stages "Draft, Review, Complete" with stage transitions. The `Document` model has NO stage/phase/status field whatsoever. Documents are created and auto-saved. **Remove the entire "Document Stages" section.**

**What to write instead:**
- Document creation: title input, optional file attachment
- Auto-save behavior: changes saved automatically as you type
- Document ordering: drag-and-drop reordering (verify `reorder_document` Tauri command exists)
- Rich text editing: describe the actual editor capabilities (Tiptap 3 — verify in frontend code)
- Campaign vs module documents: documents can belong to either

**Verification sources:**
- `CreateDocumentModal.vue` — creation form fields
- `DocumentEditor.vue` or equivalent — editor capabilities
- `Document` struct in mimir-core — actual fields (no stage/status)
- Tauri commands: `create_document`, `edit_document`, `reorder_document`, `delete_document`

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] Zero mentions of document templates in the page
- [ ] Zero mentions of document stages/phases (Draft/Review/Complete)
- [ ] Page accurately describes actual document creation flow (title + optional file)
- [ ] Auto-save behavior documented
- [ ] All described features verified against source code
- [ ] Page reads as a coherent how-to guide, not a patched-up remnant

## Status Updates

### 2026-03-13: Completed
Full rewrite of manage-documents.md:

**Removed:**
- Fictional "Document Templates" section (Session notes, Location descriptions, NPC profiles, Encounter plans templates — none exist in `CreateDocumentModal.vue`)
- Fictional "Document Stages" section (Draft/Review/Complete — no stage field exists in Document model)
- "Create documents from templates or blank" reference

**Added (all verified against source):**
- Accurate creation flow: title input + optional file upload with two modes (New Document / Upload File)
- Supported upload formats: .md, .png, .jpg, .jpeg, .webp, .gif, .svg (from `CreateDocumentModal.vue`)
- Rich text editor capabilities: headings, formatting, lists, tables, undo/redo (from `DocumentEditor.vue` — Tiptap 3)
- Auto-save with 1000ms debounce + status indicator (from `DocumentEditor.vue` line 336)
- PDF export feature (from editor toolbar)
- Reordering via up/down arrow buttons, not drag-and-drop (from `DocumentSidebar.vue` lines 69-82)
- Campaign vs module document scope explanation

**Verified against:** `CreateDocumentModal.vue`, `DocumentEditor.vue`, `DocumentSidebar.vue`, `Document` struct in mimir-core (no stage/status field).