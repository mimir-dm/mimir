---
id: pass-1-2-fix-tutorials-01-03
level: task
title: "Pass 1.2: Fix tutorials 01-03 against current UI"
short_code: "MIMIR-T-0598"
created_at: 2026-03-13T13:50:08.577601+00:00
updated_at: 2026-03-13T14:04:10.874948+00:00
parent: MIMIR-I-0061
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: MIMIR-I-0061
---

# Pass 1.2: Fix tutorials 01-03 against current UI

## Parent Initiative

[[MIMIR-I-0061]]

## Objective

Fix all known inaccuracies in tutorials 01–03. Tutorial 04 (player-display) was verified accurate — skip it.

## Scope

### `tutorials/01-first-campaign.md`

**Issues from audit (MIMIR-T-0578):**
1. Says "four dashboard tabs" — should be 5 (Homebrew tab missing). Note: the cross-cutting fix (T-0597) handles the tab count, but this task must also add descriptive content about what the Homebrew tab does.
2. No mention of Campaign Sources modal (`CampaignSourcesModal`). Add brief mention in the dashboard exploration section.

**Verification sources:** `useDashboardState.ts` (tab list), `CampaignSourcesModal.vue` (sources feature).

### `tutorials/02-first-module.md`

**Issues from audit (MIMIR-T-0578):**
1. Token type "Object" → "Marker" — handled by cross-cutting fix T-0597, but verify context around the mention still reads correctly after rename.
2. Map upload dialog claims "Grid Size" and "Grid Offset" fields. `MapUploadModal.vue` only has file picker + Map Name input. Remove phantom grid fields from upload step. Note that grid config happens in Token Setup modal (see `configure-grid.md`).
3. Missing Module Monsters Quick Select — the token palette has a "Module Monsters" section for quick access to monsters already added to the module. Add mention.

**Verification sources:** `MapUploadModal.vue` (upload fields), `TokenPalette.vue` (Module Monsters section, Marker type).

### `tutorials/03-first-session.md`

**Issues from audit (MIMIR-T-0578):**
1. "Add Token" and "Add PCs" toolbar buttons mentioned in "Map Controls > Token Management" — NOT found in `ModulePlayView.vue`. Must verify against running app or trace all child components. If buttons don't exist, remove the references. If they exist in a child component, correct the description.

**Verification sources:** `ModulePlayView.vue`, `DmMapViewer.vue`, any toolbar child components.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] Tutorial 01: Dashboard describes 5 tabs with Homebrew mentioned
- [ ] Tutorial 01: Campaign Sources feature mentioned
- [ ] Tutorial 02: Map upload step only references actual form fields (file picker + name)
- [ ] Tutorial 02: Module Monsters quick-select mentioned in token section
- [ ] Tutorial 03: "Add Token"/"Add PCs" buttons resolved (fixed or removed based on code verification)
- [ ] All changes verified against source code with file:line citations logged
- [ ] Read each tutorial end-to-end after fixes to ensure flow still makes sense

## Status Updates

### 2026-03-13: Completed
All three tutorials verified against source code and fixed:

**Tutorial 01 (first-campaign.md):**
- Added Homebrew tab description (was completely missing)
- Added Campaign Sources modal mention + description (verified via `CampaignSourcesModal.vue` and `CampaignDashboardView.vue`)
- Added Sources to quick reference table

**Tutorial 02 (first-module.md):**
- Removed phantom "Grid Size" and "Grid Offset" fields from map upload step (verified `MapUploadModal.vue` only has file picker + name input)
- Added note that grid config happens later in Token Setup
- Added "Module Monsters" quick-select section to Token Palette description (verified in `TokenPalette.vue` lines 11-28)

**Tutorial 03 (first-session.md):**
- Removed "Add Token" button reference (does not exist — verified `DmMapViewer.vue` toolbar)
- Kept "Add PCs" button (confirmed exists in `DmMapViewer.vue` lines 26-36)
- Renamed section header from "Token Management" to "Add PCs"

All verified against source: `MapUploadModal.vue`, `TokenPalette.vue`, `DmMapViewer.vue`, `CampaignSourcesModal.vue`, `CampaignDashboardView.vue`.