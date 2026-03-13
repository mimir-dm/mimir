---
id: pass-1-11-fix-play-mode-docs-start
level: task
title: "Pass 1.11: Fix play mode docs — start-session and manage-encounters"
short_code: "MIMIR-T-0607"
created_at: 2026-03-13T13:50:24.476836+00:00
updated_at: 2026-03-13T14:16:29.351642+00:00
parent: MIMIR-I-0061
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: MIMIR-I-0061
---

# Pass 1.11: Fix play mode docs — start-session and manage-encounters

## Parent Initiative

[[MIMIR-I-0061]]

## Objective

Fix `how-to/play-mode/start-session.md` and `how-to/play-mode/manage-encounters.md` — resolve the phantom "Add Token" and "Add PCs" buttons that were not found in `ModulePlayView.vue` during audit.

## Scope

Both pages reference "Add Token" and "Add PCs" buttons in the play mode toolbar. These were NOT found in `ModulePlayView.vue` during the audit (MIMIR-T-0578, MIMIR-T-0580).

### Investigation Required

Before fixing, you MUST do a thorough code search:
1. Search ALL play mode components for "Add Token", "Add PCs", or similar button labels
2. Check `DmMapViewer.vue`, `ModulePlayView.vue`, and any toolbar child components
3. Check if these are context menu items rather than toolbar buttons
4. Check if token addition happens through a different mechanism (e.g., right-click on map, drag from sidebar)

### Possible Outcomes

**If buttons exist somewhere:** Update the docs to describe their actual location and behavior.

**If buttons don't exist:** Remove the references and describe the actual token management workflow in play mode. Tokens may be added during prep mode only, not play mode.

### Pages

- `how-to/play-mode/start-session.md` — references "Add PCs" and "Add Token" in toolbar
- `how-to/play-mode/manage-encounters.md` — references same buttons

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] Thorough code search completed for Add Token/Add PCs across all play mode components
- [ ] Finding documented in status updates with file:line citations
- [ ] Both pages updated to reflect actual play mode token management
- [ ] No phantom UI elements remain in either page

## Status Updates

### 2026-03-13: Completed

**Code search results (from T-0598 investigation):**
- "Add PCs" button: EXISTS in `DmMapViewer.vue` lines 26-36 (`addAllPCsToMap()`)
- "Add Token" button: DOES NOT EXIST anywhere in play mode components

**start-session.md:**
- Removed "Add Token Button" section (phantom)
- Renamed "Quick Start Options" to "Adding PCs to the Map"
- Kept "Add PCs" description (verified)

**manage-encounters.md:**
- Removed "Quick Add Token" section (phantom)
- Kept "Add PCs" description
- Added note that monster tokens are placed during prep in Token Setup, not during play