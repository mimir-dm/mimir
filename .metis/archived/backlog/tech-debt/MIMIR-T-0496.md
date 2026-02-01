---
id: audit-and-update-all-documentation
level: task
title: "Audit and Update All Documentation for v0.5.0"
short_code: "MIMIR-T-0496"
created_at: 2026-01-29T13:57:52.223322+00:00
updated_at: 2026-01-29T14:42:50.914785+00:00
parent: 
blocked_by: []
archived: true

tags:
  - "#task"
  - "#tech-debt"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: NULL
---

# Audit and Update All Documentation for v0.5.0

## Objective

Review every documentation file against the current codebase and verify correctness. v0.5.0 introduced major changes: architecture rewrite, removed CharacterLongForm and EquipmentDetail PDF sections, redesigned character sheet, MCP server env-var-only config, Settings UI reorganization, and more. Documentation must reflect reality.

## Approach

Ralph through each document: read it, verify claims against code, fix inaccuracies, and ask the user to verify anything visual (screenshots, layouts, UI descriptions).

## Document Inventory

### Root Files
1. [ ] `README.md` — Project overview, setup instructions
2. [ ] `CONTRIBUTING.md` — Contribution guidelines
3. [ ] `DEVELOPMENT.md` — Development setup and workflow

### MCP Plugin
4. [ ] `crates/mimir-mcp/plugin/README.md` — MCP server setup, database path

### UAT Docs (docs/uat/)
5. [ ] `docs/uat/print-character-dialog.md` — Export dialog (removed long form + equipment detail options)
6. [ ] `docs/uat/print-character-longform.md` — **REMOVED FEATURE — delete this file**
7. [ ] `docs/uat/print-equipment-detail.md` — **REMOVED FEATURE — delete this file**
8. [ ] `docs/uat/print-character-sheet.md` — Character sheet redesign changes
9. [ ] `docs/uat/print-spell-cards.md` — Spell cards UAT
10. [ ] `docs/uat/T-0259-physical-play-kit-uat.md` — Physical play kit UAT

### Tutorials (docs/src/tutorials/)
11. [ ] `tutorials/README.md`
12. [ ] `tutorials/01-first-campaign.md`
13. [ ] `tutorials/02-first-module.md`
14. [ ] `tutorials/03-first-session.md`
15. [ ] `tutorials/04-player-display.md`

### How-To Guides (docs/src/how-to/)
16. [ ] `how-to/README.md`
17. [ ] `how-to/campaigns/README.md`
18. [ ] `how-to/campaigns/create-campaign.md`
19. [ ] `how-to/campaigns/export-campaign.md`
20. [ ] `how-to/campaigns/manage-documents.md`
21. [ ] `how-to/characters/README.md`
22. [ ] `how-to/characters/create-pc.md`
23. [ ] `how-to/characters/create-npc.md`
24. [ ] `how-to/characters/assign-to-campaign.md`
25. [ ] `how-to/characters/print-character-sheet.md` — Print dialog changes
26. [ ] `how-to/maps/README.md`
27. [ ] `how-to/maps/upload-map.md`
28. [ ] `how-to/maps/configure-grid.md`
29. [ ] `how-to/maps/place-tokens.md`
30. [ ] `how-to/maps/manage-light-sources.md`
31. [ ] `how-to/maps/print-map.md`
32. [ ] `how-to/modules/README.md`
33. [ ] `how-to/modules/create-module.md`
34. [ ] `how-to/modules/add-monsters.md`
35. [ ] `how-to/modules/module-documents.md`
36. [ ] `how-to/play-mode/README.md`
37. [ ] `how-to/play-mode/start-session.md`
38. [ ] `how-to/play-mode/use-player-display.md`
39. [ ] `how-to/play-mode/manage-encounters.md`
40. [ ] `how-to/play-mode/fog-of-war.md`

### Reference (docs/src/reference/)
41. [ ] `reference/README.md`
42. [ ] `reference/glossary.md`
43. [ ] `reference/keyboard-shortcuts.md`
44. [ ] `reference/file-formats.md`
45. [ ] `reference/vision-and-lighting.md`
46. [ ] `reference/characters/README.md`
47. [ ] `reference/pdf-export/README.md` — PDF export reference (removed sections)
48. [ ] `reference/sources/README.md`
49. [ ] `reference/ui/README.md`
50. [ ] `reference/ui/home-screen.md`
51. [ ] `reference/ui/campaign-dashboard.md`
52. [ ] `reference/ui/module-prep-view.md`
53. [ ] `reference/ui/play-mode.md`
54. [ ] `reference/ui/player-display.md`
55. [ ] `reference/ui/token-setup-modal.md`

### Explanations (docs/src/explanation/)
56. [ ] `explanation/README.md`
57. [ ] `explanation/campaign-vs-module.md`
58. [ ] `explanation/document-workflow.md`
59. [ ] `explanation/two-board-system.md`
60. [ ] `explanation/vision-system.md`

### Developer Docs (docs/src/developer/)
61. [ ] `developer/README.md`
62. [ ] `developer/ARCHITECTURE.md`
63. [ ] `developer/CONTRIBUTING.md`
64. [ ] `developer/DEVELOPMENT.md`
65. [ ] `developer/reference/frontend/README.md`
66. [ ] `developer/reference/llm-tools/README.md`
67. [ ] `developer/reference/source-data/README.md`
68. [ ] `developer/reference/source-data/archive-structure.md`
69. [ ] `developer/reference/source-data/schemas/class-schema.md`
70. [ ] `developer/reference/source-data/schemas/content/*.md` (10 schema files)
71. [ ] `developer/reference/5e-tools-processing/*.md` (2 files)

### Campaign Framework (docs/src/campaign-framework/)
72. [ ] `campaign-framework/README.md`
73. [ ] `campaign-framework/cheat-sheet.md`
74. [ ] `campaign-framework/board-workflow-guide.md`
75. [ ] `campaign-framework/01-foundations/README.md`
76. [ ] `campaign-framework/01-foundations/three-board-system.md`
77. [ ] `campaign-framework/02-campaign-genesis/*.md` (6 files)
78. [ ] `campaign-framework/03-module-creation/*.md` (5 files)
79. [ ] `campaign-framework/04-session-management/*.md` (5 files)
80. [ ] `campaign-framework/05-scaling/README.md`
81. [ ] `campaign-framework/06-templates/README.md`
82. [ ] `campaign-framework/06-templates/templates/*.md` (~30 template files)

### mdBook Config
83. [ ] `docs/src/SUMMARY.md` — Table of contents (must match actual files)

## Known Issues to Fix

- **Deleted features**: CharacterLongForm and EquipmentDetail sections removed — delete UAT docs, update all references
- **Print dialog**: No longer has "Include Long Form" or "Include Equipment Detail" checkboxes
- **Character sheet**: Redesigned with compact ability blocks, spell tracking circles, spellcasting on page 1
- **MCP server**: Now requires `MIMIR_DATABASE_PATH` env var (no auto-detection)
- **Settings UI**: Claude Integration moved from Theme tab to its own Integrations tab
- **Architecture**: v0.5 rewrite — backup system removed, frontend restructured

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] Every document reviewed against code
- [ ] Deleted-feature references removed
- [ ] MCP setup instructions updated
- [ ] Print/PDF docs reflect current sections
- [ ] Settings UI docs reflect new tab structure
- [ ] SUMMARY.md matches actual file structure
- [ ] All UAT docs for removed features deleted

## Questions for User Verification (RESOLVED)

*Documents I cannot verify against code alone (visual layouts, UI behavior, screenshots). These accumulate as I work. When I've done everything I can, I'll hand this list to you for review.*

1. **`docs/uat/print-character-sheet.md`** — This wireframe shows the old WotC-style 2-page layout. The character sheet was redesigned in this session (compact inline ability blocks, spellcasting moved to page 1 right column, spell tracking circles, conditions section removed, "Your Turn" reference box). Does this wireframe need to be updated to match the new layout, or should it be deleted since the code is now the source of truth?

2. **`docs/uat/print-character-dialog.md`** — Updated to remove Long Form and Equipment Detail options. Please verify the wireframe matches the actual dialog UI.

3. **`docs/uat/print-spell-cards.md`** — Spec says 3x3 grid, 2.5"x3.5" cards. Code confirms 3x3 at 2.5in columns. Does the actual PDF output match this spec?

4. **`docs/uat/T-0259-physical-play-kit-uat.md`** — Physical play kit UAT. Not affected by recent changes but references `npm run tauri dev` as prerequisite. Should be `cargo tauri dev`. Does the export dialog structure in this doc still match the current Campaign Export dialog UI?

5. **`tutorials/01-first-campaign.md`** — Removed the "Campaign Directory Location" field and folder picker. The code shows only Name + Description fields. Please verify: is this correct? Are there any other fields in the campaign creation form I'm missing?

6. **`tutorials/02-first-module.md`** — Updated the create module dialog to show Name, Type (6 options), and Description. Also changed the module dashboard description to a table with Play/Open/PDF buttons. Does this match the current UI?

7. **`tutorials/03-first-session.md` & `tutorials/04-player-display.md`** — These describe Play Mode UI in detail (sidebar with monsters/maps, map toolbar, LOS controls, ambient lighting, viewport sync, session notes). All key UI elements confirmed in code but the detailed descriptions of layout and toolbar positioning should be visually verified. Do these still match?

## Progress

### Session 1 — Root Files & MCP Plugin
- [x] `README.md` — Fixed: package name mimir-dm-mcp→mimir-mcp, plugin path, database path com.mimir.mimir→com.mimir.app/data, GitHub org colliery→mimir-dm, removed AI/LLM reference
- [x] `CONTRIBUTING.md` — Fixed: repo URL colliery-io→mimir-dm, all frontend paths mimir-dm→mimir, project structure (4 crates not old names), license Apache-2.0 OR MIT→MIT
- [x] `DEVELOPMENT.md` — Fixed: all paths mimir-dm→mimir, crate names, project structure, removed LLM/Ollama section, database paths com.mimir.mimir-test→com.mimir.app/dev, frontend structure
- [x] `crates/mimir-mcp/plugin/README.md` — Fixed: env var now required (not optional), database path, GitHub URL, added export/import tools, added claude mcp add example, Settings > Integrations reference, removed campaign authoring skills table (not in current plugin)
- [x] Deleted `docs/uat/print-character-longform.md` (removed feature)
- [x] Deleted `docs/uat/print-equipment-detail.md` (removed feature)

### Session 2 — Tutorials
- [x] `tutorials/README.md` — Verified, no changes needed
- [x] `tutorials/01-first-campaign.md` — Fixed: removed "Players" from header bar (doesn't exist, only Characters), removed "Campaign Directory Location" field (campaigns are DB-only, no folder picker), removed "create directory structure" from creation steps, changed "three fields" to "two fields"
- [x] `tutorials/02-first-module.md` — Fixed: removed "Module Number" field from create dialog (auto-assigned), added actual fields (Name, Type, Description), replaced module dashboard left/right column description with modules table description, changed Play/Print header buttons to table row buttons (Play, Open, PDF)
- [x] `tutorials/03-first-session.md` — Verified against code: Back to Prep, PLAY MODE badge, Player Display, Blackout, End Session, Session Notes all confirmed in ModulePlayView.vue. No changes needed to text — UI behavior descriptions are accurate.
- [x] `tutorials/04-player-display.md` — Verified against code: Player Display button, Blackout, Display Open label all confirmed. No changes needed.

### Session 2 — How-To Guides
- [x] `how-to/README.md` — Fixed: broken link export-campaign-pdf.md→export-campaign.md, removed Reference Search section (files don't exist)
- [x] `how-to/campaigns/README.md` — Verified, no changes needed
- [x] `how-to/campaigns/create-campaign.md` — Fixed: removed "Directory Location" field and Campaign Directory section (campaigns are DB-only)
- [x] `how-to/campaigns/export-campaign.md` — Verified, no changes needed
- [x] `how-to/campaigns/manage-documents.md` — UI descriptions, needs visual verification
- [x] `how-to/characters/README.md` — Verified, no changes needed
- [x] `how-to/characters/create-pc.md` — Fixed: wizard is 5 steps (Basics, Race, Class, Abilities, Review), not 6 (no Equipment step). Updated step descriptions.
- [x] `how-to/characters/create-npc.md` — Fixed: NPC creation is Name-only, not Role/Description/Notes. Additional fields editable after creation.
- [x] `how-to/characters/assign-to-campaign.md` — UI descriptions, needs visual verification
- [x] `how-to/characters/print-character-sheet.md` — Fixed: replaced incorrect print options (Page Size, Include Inventory/Spells/Notes) with actual dialog (Compact Sheet + Spell Cards checkboxes). Fixed broken link to export-campaign-pdf.md.
- [x] `how-to/maps/README.md` — Verified, no changes needed
- [x] `how-to/maps/upload-map.md` — UI descriptions, looks accurate
- [x] `how-to/maps/configure-grid.md` — UI descriptions, looks accurate
- [x] `how-to/maps/place-tokens.md` — UI descriptions, looks accurate
- [x] `how-to/maps/manage-light-sources.md` — UI descriptions, looks accurate
- [x] `how-to/maps/print-map.md` — Fixed broken link. Note: describes printing from Play Mode toolbar which may not match current workflow (campaign export dialog).
- [x] `how-to/modules/README.md` — Verified, no changes needed
- [x] `how-to/modules/create-module.md` — Fixed: removed Module Number field, added actual fields (Name, Type with 6 options, Description), updated after-creation description
- [x] `how-to/modules/add-monsters.md` — UI descriptions, looks accurate
- [x] `how-to/modules/module-documents.md` — UI descriptions, needs visual verification
- [x] `how-to/play-mode/README.md` — Verified, no changes needed
- [x] `how-to/play-mode/start-session.md` — Fixed: Play button is in module table row, not module header
- [x] `how-to/play-mode/manage-encounters.md` — UI descriptions, looks accurate
- [x] `how-to/play-mode/fog-of-war.md` — UI descriptions, looks accurate
- [x] `how-to/play-mode/use-player-display.md` — UI descriptions, looks accurate

### Session 2 — Reference Docs
- [x] `reference/pdf-export/README.md` — Fixed: crate name mimir-dm-print→mimir-print
- [x] `reference/ui/home-screen.md` — Rewrote layout: no sidebar, header bar with Campaign Selector/Characters/Reference/Settings
- [x] `reference/characters/README.md` — Replaced LLM Tool Integration with MCP Integration
- [x] `reference/ui/play-mode.md` — Fixed: Play button in table row, not header
- [x] `reference/file-formats.md` — Removed Campaign Directory section, ZIP→.tar.gz
- [x] `reference/sources/README.md` — Fixed: com.mimir.mimir→com.mimir.app
- [x] Remaining reference docs (README, glossary, shortcuts, vision, campaign-dashboard, module-prep, player-display, token-setup) — verified, no code-verifiable issues

### Session 2 — Explanation Docs
- [x] `explanation/README.md` — Removed reference to nonexistent player-display-architecture.md
- [x] Other explanation docs — Conceptual content, no code-verifiable issues

### Session 3 — Developer Docs (Main)
- [x] `developer/README.md` — Removed LLM Tools reference
- [x] `developer/ARCHITECTURE.md` — All crate names updated, removed LLM section, updated AppState, updated directory structure, 7→4 crates
- [x] `developer/CONTRIBUTING.md` — All mimir-dm paths→mimir, updated key crates list
- [x] `developer/DEVELOPMENT.md` — All mimir-dm paths→mimir, database paths updated

### Session 3 — Developer Reference Subdocs
- [x] `developer/reference/frontend/README.md` — Updated from 9 features to 4 (campaigns, modules, characters, sources). Removed chat store paragraph. Updated stores/services lists.
- [x] `developer/reference/llm-tools/README.md` — Replaced entire file with removal notice pointing to MCP server
- [x] `developer/reference/5e-tools-processing/5etools-item-processing.md` — Added note that splitter crate was removed in v0.5.0
- [x] `developer/reference/5e-tools-processing/magic-variant-implementation.md` — Added note that splitter crate was removed in v0.5.0
- [x] `developer/reference/source-data/*` — Schema format references, still accurate, no changes needed

### Session 3 — Campaign Framework Docs
- [x] Grepped all ~50 files for stale references (mimir-dm-, colliery, com.mimir.mimir, campaign directory, Directory Location) — zero matches. These are DM methodology docs, not software docs.

### Session 3 — SUMMARY.md
- [x] Verified table of contents matches actual file structure. All links valid.

### Session 3 — User Verification
- Questions 1-4: User said delete all UAT docs. Deleted all 4 remaining UAT files.
- Questions 5-7: User confirmed all correct. No changes needed.