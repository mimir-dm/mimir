---
id: pass-1-14-fix-reference-pages
level: task
title: "Pass 1.14: Fix reference pages — dashboard, module-prep, play-mode, token-setup, keyboard shortcuts"
short_code: "MIMIR-T-0610"
created_at: 2026-03-13T13:50:28.436050+00:00
updated_at: 2026-03-13T14:21:02.672443+00:00
parent: MIMIR-I-0061
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: MIMIR-I-0061
---

# Pass 1.14: Fix reference pages — dashboard, module-prep, play-mode, token-setup, keyboard shortcuts

## Parent Initiative

[[MIMIR-I-0061]]

## Objective

Fix 5 UI reference pages with known inaccuracies from audit MIMIR-T-0581.

## Scope

### `reference/ui/campaign-dashboard.md`
**Issues (MIMIR-T-0581):**
1. Missing Homebrew tab — only lists 4 tabs. Add Homebrew section.
2. Document stages fiction — claims 4 stages (Pre-Production/Production/Post-Production/Archive). Remove entirely.
3. Campaign tab sub-sections wrong — lists "World Building" and "Timeline" as separate sections. Describe actual `WorldTab.vue` layout.

**Verification:** `useDashboardState.ts`, `WorldTab.vue`, `HomebrewTab.vue`.

### `reference/ui/module-prep-view.md`
**Issue (MIMIR-T-0581):** Lists "Dangers" section. Should be "Monsters" (`ModuleMonsters.vue`). Also describe homebrew monster support.

### `reference/ui/play-mode.md`
**Issue (MIMIR-T-0581):** Map Toolbar table lists "Add Token" and "Add PCs" buttons. Same phantom buttons as play mode how-to guides. Fix per findings from T-0607.

### `reference/ui/token-setup-modal.md`
**Issue (MIMIR-T-0581):** "Object" → "Marker" handled by T-0597, but verify surrounding description makes sense.

### `reference/keyboard-shortcuts.md`
**Issues (MIMIR-T-0581):**
1. Play mode shortcuts wrong — lists `Space` (play/pause) and `B` (blackout) which don't exist
2. Actual shortcuts from `DmMapViewer.vue`: `+`/`=` (zoom in), `-` (zoom out), `0` (reset), `h`/`H` (toggle visibility), `d`/`D` (toggle dead), `Escape` (close/deselect)
3. Missing h/d token manipulation shortcuts

**Verification:** `DmMapViewer.vue` keydown handler.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] campaign-dashboard.md: Homebrew tab section added, document stages removed, Campaign tab matches WorldTab.vue
- [ ] module-prep-view.md: "Dangers" → "Monsters" with homebrew support noted
- [ ] play-mode.md: Phantom buttons resolved
- [ ] token-setup-modal.md: Marker description correct
- [ ] keyboard-shortcuts.md: All shortcuts match actual keydown handlers
- [ ] Every change verified against source code

## Status Updates

### 2026-03-13: Completed

**campaign-dashboard.md — full rewrite:**
- Removed fictional document stages (Pre-Production/Production/Post-Production/Archive)
- Removed fictional "World Building" and "Timeline" sections
- Campaign tab accurately describes two-panel layout: document sidebar + maps on left, editor/preview on right (verified `WorldTab.vue`)
- Added Homebrew tab section
- Added Sources button to header
- Fixed archive extension to `.mimir-campaign.tar.gz`

**module-prep-view.md:**
- "Dangers" → "Monsters" already done by T-0597
- Added homebrew monster mention

**play-mode.md:**
- Removed "Add Token" from toolbar table (doesn't exist)
- Added "Print" button (verified in `DmMapViewer.vue`)

**token-setup-modal.md:**
- Added Module Monsters quick-select section (verified `TokenPalette.vue`)

**keyboard-shortcuts.md — full rewrite:**
- Removed fictional `Space` (play/pause) and `B` (blackout) shortcuts
- Removed fictional `F11` (fullscreen) and `Delete` (delete token)
- Removed `Cmd+S` (auto-save handles it, no explicit save handler)
- Added actual shortcuts from `DmMapViewer.vue` lines 1845-1876:
  - `h`/`H` toggle visibility, `d`/`D` toggle dead
  - `+`/`=` zoom in, `-` zoom out, `0` reset
  - `Escape` close/deselect
- Added note about input field exclusion