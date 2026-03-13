---
id: audit-how-to-guides-campaigns-maps
level: task
title: "Audit how-to guides: campaigns, maps, and modules"
short_code: "MIMIR-T-0579"
created_at: 2026-03-11T23:13:28.421185+00:00
updated_at: 2026-03-13T12:32:37.112665+00:00
parent: MIMIR-I-0059
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: MIMIR-I-0059
---

# Audit how-to guides: campaigns, maps, and modules

## Parent Initiative

[[MIMIR-I-0059]]

## Objective

Review the 11 how-to guide pages covering campaigns, maps, and modules for accuracy against the current codebase. These are task-oriented guides that users follow to accomplish specific goals — incorrect steps will directly frustrate users.

## Scope

### Campaigns (3 pages)
- `docs/src/how-to/campaigns/create-campaign.md`
- `docs/src/how-to/campaigns/manage-documents.md`
- `docs/src/how-to/campaigns/export-campaign.md`

### Maps (6 pages)
- `docs/src/how-to/maps/upload-map.md`
- `docs/src/how-to/maps/configure-grid.md`
- `docs/src/how-to/maps/place-tokens.md`
- `docs/src/how-to/maps/manage-light-sources.md`
- `docs/src/how-to/maps/print-map.md`

### Modules (3 pages)
- `docs/src/how-to/modules/create-module.md`
- `docs/src/how-to/modules/add-monsters.md`
- `docs/src/how-to/modules/module-documents.md`

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] Each how-to page read and cross-referenced against current Vue components, Tauri commands, and service layer
- [ ] Verify all steps match current UI (button labels, form fields, navigation paths)
- [ ] Check export/import campaign flow against current `CampaignService` capabilities
- [ ] Verify map upload flow — does it still match UVTT processing pipeline?
- [ ] Check token placement steps against current `TokenService` and token setup modal
- [ ] Verify print/PDF flow against current `mimir-print` crate capabilities
- [ ] Check whether `add-monsters.md` covers homebrew monsters (added in v0.6.1)
- [ ] Identify where screenshots would help — especially for map grid config, token placement, and light source management
- [ ] Produce findings report listing inaccuracies, missing content, and screenshot requests

## Screenshot Candidates

Pay special attention to these — they are likely areas where a screenshot would significantly help:
- Campaign creation dialog
- Map upload flow (UVTT file selection)
- Grid configuration overlay
- Token setup modal (recently updated)
- Light source placement UI
- Print preview / PDF export dialog
- Module monster list (with homebrew monsters)

## Status Updates

### Audit Completed 2026-03-12

---

#### Campaigns

**create-campaign.md — No issues.** Steps match current UI: campaign selector → + New Campaign, name + description fields, Create Campaign button. Accurate.

**manage-documents.md — 2 issues found**

1. **INACCURACY: Document templates don't exist** — Guide says "Mimir provides templates for common document types: Session notes, Location descriptions, NPC profiles, Encounter plans." The actual `CreateDocumentModal.vue` only has a title input and optional file upload. There is no template selection. Fix: Remove the "Document Templates" section entirely, or note that documents start blank.

2. **INACCURACY: Document stages are wrong** — Guide says stages are "Draft, Review, Complete." The actual `Document` type has NO stage/phase/status field at all — documents are simply created and edited with auto-save. There are no stage transitions. Fix: Remove "Document Stages" section or replace with accurate description of auto-save behavior.

**export-campaign.md — Accurate.** Export format is `.tar.gz` (confirmed: `placeholder="Select a .mimir-campaign.tar.gz file"`). Import is available in campaign selector dropdown. Export button in campaign dashboard header. All steps match.

---

#### Maps

**upload-map.md — 1 issue found**

1. **INACCURACY: Grid Size and Grid Offset fields don't exist in upload dialog** — Step 5 says "Configure map settings: Grid Size, Grid Offset". The actual `MapUploadModal.vue` only has: file picker, Map Name input, and read-only UVTT metadata display. Grid configuration happens separately in the Token Setup modal (see configure-grid.md). Fix: Remove grid fields from upload steps, note that grid is configured after upload via Token Setup.

**configure-grid.md — Accurate.** Grid configuration is correctly described as happening in the Token Setup modal. Steps match: Grid button → Grid Size/X Offset/Y Offset → Save. Note about UVTT files auto-configuring is correct.

**place-tokens.md — 1 issue found**

1. **INACCURACY: "Objects" should be "Marker"** — Token palette lists "Objects" but the actual UI has "Marker" (📍 icon). Fix: Replace "Objects - Generic object tokens" with "Markers - Points of interest."

**manage-light-sources.md — Accurate.** Light types (Torch 20/40, Lantern 30/60, Candle 5/10), placement flow, Lit/Unlit toggle, right-click in play mode — all match current code.

**print-map.md — 3 issues found**

1. **INACCURACY: Print is NOT in Play Mode toolbar** — Guide says "Open Play Mode → Click Print in the toolbar." Print is actually accessed from the module dashboard via the "PDF" action button on module rows, NOT from play mode. Fix: Correct the entry point.

2. **INACCURACY: Print options are wrong** — Guide lists "Include Grid, Include Tokens, Token Cutouts, Scale." Actual `MapPrintDialog.vue` has two sections (Preview/Play) with: Include Grid, Include LOS Walls, Starting Positions, Token Cutouts. There is NO "Scale" option and NO "Include Tokens" toggle (token cutouts is separate). Fix: Update options list.

3. **INACCURACY: Tiled printing doesn't exist** — Guide describes "Enable tiled mode, set paper size, generates pages that align." This feature does not exist in `MapPrintDialog.vue`. Fix: Remove tiled printing section entirely.

---

#### Modules

**create-module.md — Accurate.** Module creation dialog verified: Name, Type (all 6 options match), Description. Action buttons (Play, Open, PDF) confirmed in `ModulesTable.vue`.

**add-monsters.md — 1 issue found**

1. **MISSING FEATURE: Homebrew monsters not mentioned** — The module monsters system (`ModuleMonsters.vue`) supports both catalog AND homebrew monsters. Guide only describes catalog search. Fix: Add a note about homebrew monsters being available if created in the Homebrew tab.

**module-documents.md — 2 issues found (same as manage-documents.md)**

1. **INACCURACY: Templates don't exist** — Same issue as campaign documents.
2. **INACCURACY: Document stages wrong** — Says "In Progress, Complete" but documents have no stage system.

---

#### Cross-Cutting Issues

1. **"Objects" vs "Marker"** — Appears in both `place-tokens.md` and tutorial `02-first-module.md`. Token palette type is "Marker" not "Objects."
2. **Document templates and stages** — Referenced in both `manage-documents.md` and `module-documents.md`. Neither templates nor stages exist.
3. **Homebrew not mentioned** — None of the how-to guides mention homebrew items, monsters, or spells as options alongside catalog content.
4. **No screenshots exist** — All `![image](...)` references point to non-existent files.

#### Screenshot Recommendations
- Map upload modal (showing actual fields)
- Token Setup modal with Marker type visible
- Print/PDF dialog with actual options
- Module dashboard with Play/Open/PDF buttons
- Module monsters panel showing homebrew monster support