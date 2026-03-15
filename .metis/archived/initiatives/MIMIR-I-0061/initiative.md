---
id: documentation-correctness
level: initiative
title: "Documentation correctness: systematic fix, gap fill, and adversarial verification"
short_code: "MIMIR-I-0061"
created_at: 2026-03-13T13:44:40.050315+00:00
updated_at: 2026-03-14T18:45:01.671447+00:00
parent: MIMIR-V-0001
blocked_by: []
archived: true

tags:
  - "#initiative"
  - "#phase/completed"


exit_criteria_met: false
estimated_complexity: XL
initiative_id: documentation-correctness
---

# Documentation correctness: systematic fix, gap fill, and adversarial verification Initiative

## Context

MIMIR-I-0059 completed a comprehensive audit of all 40+ user-facing documentation pages and produced detailed findings across 8 tasks (MIMIR-T-0578 through MIMIR-T-0585). The audit was **read-only** — it identified ~30 inaccuracies, several pages built on entirely fictional features, major feature gaps (homebrew, MCP, mapgen), and zero screenshots across the entire docs site.

This initiative takes those findings and **fixes everything**, then verifies the fixes are correct through an independent adversarial review pass. The goal is documentation that a user can trust completely — every button name, every form field, every keyboard shortcut, every workflow step verified against the actual source code.

### Audit Summary (from MIMIR-I-0059)

**Cross-cutting issues affecting many pages:**
- Document stages/templates are fiction — referenced in 5+ pages, exist nowhere in code
- "Object" token type should be "Marker" — wrong in 4 pages
- Homebrew tab missing from all dashboard references (docs say 4 tabs, actual is 5)
- Zero screenshots exist — all image references point to non-existent files

**Pages requiring correction (25 pages with known issues):**

| Page | Issues |
|------|--------|
| `tutorials/01-first-campaign.md` | 4 tabs → 5, missing Campaign Sources |
| `tutorials/02-first-module.md` | Object→Marker, phantom grid fields in upload, missing Module Monsters quick-select |
| `tutorials/03-first-session.md` | "Add Token"/"Add PCs" buttons not found in code |
| `how-to/campaigns/manage-documents.md` | Templates don't exist, stages don't exist |
| `how-to/maps/upload-map.md` | Grid Size/Grid Offset fields don't exist in upload dialog |
| `how-to/maps/place-tokens.md` | Object→Marker |
| `how-to/maps/print-map.md` | Wrong entry point (not in play mode), wrong options, tiled printing doesn't exist |
| `how-to/modules/add-monsters.md` | Missing homebrew monster support |
| `how-to/modules/module-documents.md` | Templates and stages don't exist |
| `how-to/characters/create-pc.md` | 5 wizard steps → 7 (missing Background, Skills) |
| `how-to/characters/create-npc.md` | Missing optional fields (Race, Role, Location, Faction) |
| `how-to/characters/print-character-sheet.md` | Missing Battle Card and Equipment Cards print options |
| `how-to/play-mode/start-session.md` | "Add PCs"/"Add Token" buttons not found |
| `how-to/play-mode/manage-encounters.md` | Same phantom buttons |
| `reference/ui/campaign-dashboard.md` | Missing Homebrew tab, fictional document stages, wrong Campaign tab sub-sections |
| `reference/ui/module-prep-view.md` | "Dangers" should be "Monsters" |
| `reference/ui/play-mode.md` | Phantom "Add Token"/"Add PCs" toolbar buttons |
| `reference/ui/token-setup-modal.md` | Object→Marker |
| `reference/keyboard-shortcuts.md` | Play mode shortcuts wrong (Space, B don't exist), missing h/d token shortcuts |
| `reference/file-formats.md` | Missing .dungeondraft_map and .mimir-campaign.tar.gz |
| `reference/glossary.md` | Missing Homebrew, Sidecar, MCP terms; dashboard definition says 4 tabs |
| `explanation/two-board-system.md` | "Dangers"→"Monsters", missing Homebrew tab |
| `explanation/document-workflow.md` | ENTIRE PAGE IS FICTION — stage-based workflow and templates don't exist |
| `developer/DEVELOPMENT.md` | Missing cargo tauri dev bug workaround |

**Pages verified accurate (no changes needed):**
- `tutorials/04-player-display.md`
- `how-to/campaigns/create-campaign.md`
- `how-to/campaigns/export-campaign.md`
- `how-to/maps/configure-grid.md`
- `how-to/maps/manage-light-sources.md`
- `how-to/characters/assign-to-campaign.md`
- `how-to/play-mode/fog-of-war.md`
- `how-to/play-mode/use-player-display.md`
- `reference/ui/home-screen.md`
- `reference/ui/player-display.md`
- `reference/vision-and-lighting.md`
- `explanation/campaign-vs-module.md`
- `explanation/vision-system.md`
- `developer/ARCHITECTURE.md` (rewritten during audit)
- `developer/CONTRIBUTING.md`
- `developer/reference/frontend/README.md`
- `developer/reference/llm-tools/README.md`

**New pages needed (13 pages):**
1. `how-to/homebrew/README.md` — Homebrew overview
2. `how-to/homebrew/create-item.md` — Create homebrew items
3. `how-to/homebrew/create-monster.md` — Create homebrew monsters
4. `how-to/homebrew/create-spell.md` — Create homebrew spells
5. `how-to/characters/manage-spells.md` — Character spell management
6. `how-to/characters/manage-inventory.md` — Character inventory management
7. `how-to/characters/level-up.md` — Level-up workflow (8-step dialog)
8. `how-to/maps/generate-map.md` — Generate maps with mapgen
9. `how-to/ai-assistant/README.md` — Using Mimir with Claude Code
10. `explanation/homebrew-system.md` — Homebrew concepts
11. `developer/reference/mcp-server/README.md` — MCP server overview
12. `developer/reference/mcp-server/tool-reference.md` — MCP tool reference (60+ tools)
13. `developer/reference/mapgen/README.md` — Mapgen reference

## Goals & Non-Goals

**Goals:**
- Fix every known inaccuracy identified in the MIMIR-I-0059 audit
- Write all 13 missing documentation pages
- Update SUMMARY.md navigation to include new sections
- Add glossary terms for all missing concepts
- Ensure every factual claim in every page can be traced to source code
- Independently verify all fixes through an adversarial review pass
- Ensure cross-page consistency (same terms, same feature descriptions, same tab counts everywhere)

**Non-Goals:**
- Screenshots — these require a running app and manual capture; they are out of scope for this initiative (a separate initiative should handle screenshot capture)
- Campaign Framework section (`docs/src/campaign-framework/`) — this is DM methodology content, not software documentation; it describes how to run D&D campaigns, not how to use Mimir features, and was not part of the audit
- Source data schema docs (`developer/reference/source-data/`) — not audited, not addressed here
- 5etools processing docs — not audited, not addressed here
- Visual design or formatting improvements — we're fixing correctness, not aesthetics

## Detailed Design

### Strategy: Three-Pass Verification

The core innovation of this initiative is **three independent passes** over the documentation, each with a different purpose. This means every page gets scrutinized at least twice by independent review.

#### Pass 1: Systematic Fix & Fill (Tasks 1–19)

Fix all known inaccuracies and write all missing pages. Each task:
- Receives a specific list of pages and the known issues from the audit
- Must fix every listed issue
- Must verify every factual claim against source code (cite file:line)
- Must update internal cross-references if page content changes
- Logs which source files were checked in the task's status updates

Organized by documentation section for coherent editing:

**Cross-cutting fixes (1 task):**
1. Global terminology and consistency fixes — "Object"→"Marker" across all pages, 4-tabs→5-tabs everywhere, "Dangers"→"Monsters", glossary additions, file-formats additions

**Tutorials (2 tasks):**
2. Fix tutorials 01–03 (04 is accurate, skip it)
3. Write missing tutorial content: homebrew tab mentions, character spell/inventory mentions

**How-to: Campaigns & Modules (2 tasks):**
4. Fix manage-documents.md — remove fictional templates and stages, rewrite around actual auto-save workflow
5. Fix module docs: add-monsters.md (homebrew mention), module-documents.md (remove fictional stages)

**How-to: Maps (2 tasks):**
6. Fix map docs: upload-map.md (remove phantom grid fields), print-map.md (correct entry point, options, remove tiled printing), place-tokens.md (Object→Marker)
7. Write generate-map.md — new how-to page for mapgen

**How-to: Characters (3 tasks):**
8. Fix create-pc.md (7 wizard steps, character sheet tabs), create-npc.md (optional fields), print-character-sheet.md (4 print options)
9. Write manage-spells.md — new how-to for character spell management
10. Write manage-inventory.md and level-up.md — new how-to pages

**How-to: Play Mode (1 task):**
11. Fix start-session.md and manage-encounters.md — resolve phantom "Add Token"/"Add PCs" buttons

**How-to: New Sections (2 tasks):**
12. Write homebrew how-to section: README + create-item + create-monster + create-spell (4 pages)
13. Write AI assistant how-to: README + setup guide

**Reference (2 tasks):**
14. Fix reference pages: campaign-dashboard.md (Homebrew tab, remove fictional stages, fix Campaign tab), module-prep-view.md (Dangers→Monsters), play-mode.md (phantom buttons), token-setup-modal.md (Object→Marker), keyboard-shortcuts.md (correct play mode shortcuts)
15. Fix glossary and file-formats

**Explanation (1 task):**
16. Fix two-board-system.md; rewrite or remove document-workflow.md (entirely fictional); write homebrew-system.md explanation

**Developer (2 tasks):**
17. Fix DEVELOPMENT.md (cargo tauri dev workaround); update developer README links
18. Write MCP server docs (overview + tool reference) and mapgen reference

**Navigation (1 task):**
19. Update SUMMARY.md with all new sections and pages; verify all internal links resolve

#### Pass 2: Cross-Cutting Consistency Audit (Tasks 20–22)

After all fixes and new pages are written, independent verification passes:

20. **Terminology consistency scan** — Read every page and verify: "Marker" never "Object", "Monsters" never "Dangers", dashboard always says 5 tabs, document workflow never mentions stages/templates, all feature names match UI component labels. Build a canonical terminology table and check every page against it.

21. **Feature coverage matrix** — Build a matrix of all major Mimir features (campaigns, modules, documents, characters, spells, inventory, level-up, homebrew items/monsters/spells, maps, tokens, light sources, fog of war, player display, play mode, print/PDF, export/import, MCP, mapgen) and verify each feature is covered by at least: one how-to page, one reference mention, and one glossary entry where appropriate. Flag gaps.

22. **Navigation and link integrity** — Verify SUMMARY.md matches actual file system. Check every `[link](path)` in every page resolves. Check every image reference. Verify section READMEs link to their child pages. Verify breadcrumb-style "see also" links are bidirectional.

#### Pass 3: Adversarial Verification (Tasks 23–27)

An independent re-read of every page with the explicit goal of **trying to find remaining errors**. The agent executing these tasks should assume the documentation is wrong and attempt to disprove every claim.

23. **Adversarial review: Tutorials** — Re-read all 4 tutorials. For every step, open the corresponding Vue component and verify the step is possible. Check: button exists, form field exists, navigation path works, result matches description. Log every claim checked and the source file that confirms it.

24. **Adversarial review: How-to guides (campaigns, maps, modules)** — Same approach for all how-to pages in these sections. Every "Click X" instruction must map to a real UI element. Every "you will see Y" must match component render output.

25. **Adversarial review: How-to guides (characters, play mode, homebrew, AI assistant)** — Same approach. Pay special attention to the newly written pages which haven't been through a prior audit cycle.

26. **Adversarial review: Reference pages** — Verify every keyboard shortcut by finding the keydown handler. Verify every UI element described in reference pages exists in the component. Verify every file format entry against actual code. Verify every glossary definition is accurate and complete.

27. **Adversarial review: Explanation and developer pages** — Verify conceptual explanations match implementation. Check architecture claims against actual crate structure. Verify code examples compile conceptually. Check that developer setup instructions actually work.

### Task Execution Rules

Every task in this initiative MUST follow these rules:

1. **Read before write** — Always read the current page content before making changes
2. **Cite sources** — When verifying a claim, log the source file and line number in the task's status updates (e.g., "Verified 5 tabs: useDashboardState.ts:13-19")
3. **No guessing** — If a claim can't be verified from source code, flag it rather than assuming it's correct
4. **Preserve accurate content** — Don't rewrite sections that are already correct; only change what needs changing
5. **Match existing style** — New pages should match the tone and structure of the best existing pages (vision-and-lighting.md and campaign-vs-module.md are good models)
6. **Test navigation** — After writing new pages, verify they appear correctly in SUMMARY.md

### Verification Standard

A page is considered **verified correct** when:
- Every UI element name matches the actual component label/text
- Every form field mentioned exists in the component template
- Every keyboard shortcut maps to an actual keydown/keypress handler
- Every workflow step is achievable in the current UI
- Every feature mentioned exists and works as described
- No features available in the UI are missing from the documentation (for that page's scope)
- All cross-references to other pages point to existing content
- Terminology is consistent with the canonical terminology table

## Alternatives Considered

**1. Patch-and-ship (minimal fix):** Fix only the most egregious errors (document stages, Object→Marker) and skip new pages. Rejected because this leaves major features (homebrew, MCP, mapgen) completely undocumented and doesn't provide confidence in overall correctness.

**2. Rewrite from scratch:** Delete all docs and rewrite every page. Rejected because 17 pages were verified accurate in the audit — rewriting them wastes effort and risks introducing new errors.

**3. Fix without verification pass:** Do Pass 1 only, skip Passes 2 and 3. Rejected because the user explicitly requested maximum scrutiny. The adversarial verification pass is what catches errors introduced during the fix phase and provides high confidence in the final result.

**4. Single-page-per-task (40+ tasks):** Give every single page its own task. Rejected because many fixes are trivial (a single word change) and grouping related pages allows the agent to maintain context about the section being fixed. However, the adversarial review tasks (Pass 3) DO cover every page independently.

## Implementation Plan

### Phase 1: Discovery & Design (current phase)
- Review audit findings ✅
- Design three-pass strategy ✅
- Get human approval on scope and approach

### Phase 2: Decomposition
- Create all ~27 tasks under this initiative
- Ensure task descriptions include specific audit findings and source file pointers

### Phase 3: Execution — Pass 1 (Systematic Fix & Fill)
- Tasks 1–19 executed serially
- Each task logs source file citations in status updates
- Cross-cutting fix (Task 1) executes first to establish baseline terminology

### Phase 4: Execution — Pass 2 (Cross-Cutting Consistency)
- Tasks 20–22 execute after all Pass 1 tasks complete
- These read every page holistically looking for gaps and inconsistencies

### Phase 5: Execution — Pass 3 (Adversarial Verification)
- Tasks 23–27 execute after Pass 2
- Independent agents attempt to disprove every claim
- Any issues found get fixed in-task (not deferred)

### Phase 6: Final Review
- Human reviews a sample of pages for quality
- SUMMARY.md verified against filesystem
- `angreal docs build` confirms site builds without errors
- `angreal docs check` confirms no broken links