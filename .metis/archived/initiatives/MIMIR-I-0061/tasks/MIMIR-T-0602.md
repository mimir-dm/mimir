---
id: pass-1-6-fix-map-docs-upload-print
level: task
title: "Pass 1.6: Fix map docs — upload, print, and place-tokens"
short_code: "MIMIR-T-0602"
created_at: 2026-03-13T13:50:15.361808+00:00
updated_at: 2026-03-13T14:09:38.630221+00:00
parent: MIMIR-I-0061
blocked_by: []
archived: true

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: MIMIR-I-0061
---

# Pass 1.6: Fix map docs — upload, print, and place-tokens

## Parent Initiative

[[MIMIR-I-0061]]

## Objective

Fix three map how-to pages with known inaccuracies.

## Scope

### `how-to/maps/upload-map.md`

**Issue (MIMIR-T-0579):** Step 5 says "Configure map settings: Grid Size, Grid Offset." The actual `MapUploadModal.vue` only has: file picker, Map Name input, and read-only UVTT metadata display. Grid config happens separately in Token Setup modal.

**Fix:** Remove grid fields from upload steps. Add a note pointing to `configure-grid.md` for grid configuration after upload.

**Verification:** `MapUploadModal.vue` — confirm actual form fields.

### `how-to/maps/print-map.md`

**Issues (MIMIR-T-0579):** Three problems:
1. Says entry point is "Open Play Mode → Click Print in toolbar." Print is actually accessed from the module dashboard via the "PDF" action button on module rows, NOT from play mode.
2. Print options listed as "Include Grid, Include Tokens, Token Cutouts, Scale." Actual `MapPrintDialog.vue` has: Include Grid, Include LOS Walls, Starting Positions, Token Cutouts. No "Scale" or "Include Tokens."
3. Describes "Tiled printing" feature that doesn't exist.

**Fix:** Correct entry point, update options list, remove tiled printing section.

**Verification:** `MapPrintDialog.vue` — all print options. `ModulesTable.vue` — PDF action button location.

### `how-to/maps/place-tokens.md`

**Issue (MIMIR-T-0579):** Lists "Objects" as a token type. Should be "Marker." Note: the global fix T-0597 handles the word replacement, but this task must verify the surrounding description still makes sense and update "Generic object tokens" to "Points of interest" or similar.

**Verification:** `TokenPalette.vue` — Marker type and description.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] upload-map.md: No mention of Grid Size or Grid Offset in upload dialog steps
- [ ] upload-map.md: Points to configure-grid.md for post-upload grid setup
- [ ] print-map.md: Entry point is module dashboard PDF button, not play mode toolbar
- [ ] print-map.md: Options match actual MapPrintDialog.vue (Include Grid, Include LOS Walls, Starting Positions, Token Cutouts)
- [ ] print-map.md: Zero mention of tiled printing
- [ ] place-tokens.md: Marker type described correctly with appropriate description
- [ ] All changes verified against source code

## Status Updates

### 2026-03-13: Completed

**upload-map.md:**
- Removed "Grid Size" and "Grid Offset" from upload steps (verified `MapUploadModal.vue` — only file picker + name)
- Replaced "Image Files" section with note pointing to configure-grid.md
- Removed grid size tips (not relevant to upload)

**print-map.md — full rewrite:**
- Entry point is correct: Play Mode toolbar print button (verified `DmMapViewer.vue` lines 116-129)
- Note: the module dashboard "PDF" button is a separate `ModuleExportDialog`, not map printing
- Rewrote options to match actual two-section layout (verified `MapPrintDialog.vue`):
  - Preview section: Grid, LOS Walls, Starting Positions
  - Play section: Grid, LOS Walls, Token Cutouts (tiled at 1"=5ft scale)
- Removed fictional standalone "Scale" option and "Include Tokens" option
- Tiled printing actually DOES exist as part of the Play section — corrected to describe accurately
- Added estimated page count mention

**place-tokens.md:**
- Fixed intro line: "objects" → full list of token types
- Marker description already correct from T-0597

All verified against: `MapUploadModal.vue`, `MapPrintDialog.vue`, `DmMapViewer.vue`, `TokenPalette.vue`.