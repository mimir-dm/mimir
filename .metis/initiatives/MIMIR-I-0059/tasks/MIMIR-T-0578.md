---
id: audit-tutorials-for-accuracy
level: task
title: "Audit tutorials for accuracy against current UI and workflows"
short_code: "MIMIR-T-0578"
created_at: 2026-03-11T23:13:28.320314+00:00
updated_at: 2026-03-13T02:45:48.237957+00:00
parent: MIMIR-I-0059
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: MIMIR-I-0059
---

# Audit tutorials for accuracy against current UI and workflows

## Parent Initiative

[[MIMIR-I-0059]]

## Objective

Review all 4 tutorial pages for accuracy against the current Mimir UI and workflows. Tutorials are the first thing new users read, so accuracy is critical.

## Scope

Review these files against the current codebase and UI:
- `docs/src/tutorials/01-first-campaign.md` — Create and explore campaigns
- `docs/src/tutorials/02-first-module.md` — Build adventures with maps/monsters
- `docs/src/tutorials/03-first-session.md` — Run encounters in Play Mode
- `docs/src/tutorials/04-player-display.md` — Set up second screen for players

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] Each tutorial page read in full and cross-referenced against current Vue components and Tauri commands
- [ ] Every step-by-step instruction verified against current UI flow (button names, menu locations, form fields)
- [ ] Flag any instructions that reference removed, renamed, or relocated UI elements
- [ ] Check that tutorials mention features added since they were written (e.g., homebrew, spell management)
- [ ] Identify steps where a screenshot would help the reader — note exactly what the screenshot should show
- [ ] Produce a findings report as a status update listing: inaccuracies found, missing content, screenshot requests

## How to Review

For each tutorial page:
1. Read the doc page
2. Trace the described workflow through the frontend code (`crates/mimir/frontend/src/`)
3. Verify component names, route paths, and Tauri command invocations match what the doc describes
4. Check whether new features (homebrew items, spells, character inventory) should be mentioned
5. Note any places where a screenshot would clarify a step — describe what the screenshot should capture

## Status Updates

### Audit Completed 2026-03-12

#### 01-first-campaign.md — 2 issues found

1. **INACCURACY: Dashboard has 5 tabs, not 4** — Tutorial says "four tabs" but the dashboard now has 5: Campaign, Modules, NPCs, PCs, and **Homebrew** (flask icon). The Homebrew tab is not mentioned anywhere in the tutorial. Fix: Update "four dashboard tabs" to "five", add a brief Homebrew tab section explaining it's for creating custom items, monsters, and spells.

2. **MISSING FEATURE: Campaign Sources** — The campaign dashboard header has a `CampaignSourcesModal` for selecting which D&D source books are enabled. Not mentioned in tutorial. Consider adding a brief mention in Step 5.

**Accurate sections:** Home screen header (skull, campaign selector, Characters, Reference, Settings), campaign creation form (name + description), Export Archive button, all Quick Reference entries.

#### 02-first-module.md — 3 issues found

1. **INACCURACY: Token type "Object" should be "Marker"** — Tutorial says token palette has "Object" tokens but the UI has `Marker` (📍 icon) instead. Fix: Replace "Object" with "Marker" throughout.

2. **INACCURACY: Map upload dialog fields** — Tutorial says upload dialog has "Grid Size" and "Grid Offset" fields. The actual `MapUploadModal.vue` only has file picker and Map Name input. Grid configuration happens elsewhere (in the map viewer/token setup modal). Fix: Remove grid config from upload step, or move to a separate step that references where grid config actually lives.

3. **MISSING FEATURE: Module Monsters Quick Select** — Token palette has a "Module Monsters" section at the top for quick access to monsters already added to the module. Not mentioned in tutorial.

**Accurate sections:** Module creation dialog (name, type with all 6 options, description), map file types (PNG, JPG, WebP, UVTT), light sources (Torch/Lantern/Candle with correct ranges), token options (size/color/visibility), overall workflow.

#### 03-first-session.md — Minor issues only

1. **POSSIBLE INACCURACY: "Add Token" and "Add PCs" toolbar buttons** — Tutorial mentions these in "Map Controls > Token Management" but no matching code found in `ModulePlayView.vue`. These may have been removed or relocated. Needs manual UI verification.

2. **MISSING FEATURE: Monster Stats Panel** — Tutorial briefly mentions clicking a monster shows stats, but doesn't describe the `MonsterStatsPanel` component that slides in from the right. Could add more detail.

**Accurate sections:** Play mode entry (two methods), header buttons (Back to Prep, Player Display, Blackout, End Session), sidebar (Monsters with quantity + tags, Maps with active indicator), fog of war mechanics, LOS controls (Fog/Token modes), ambient light levels (Bright/Dim/Dark), viewport sync, session notes panel with auto-save.

#### 04-player-display.md — No issues found

All described features verified against code:
- Player Display button in header ✅
- Blackout mode (conditional on display open) ✅
- View sync and Push View ✅
- Two-screen comparison table accurate ✅
- Fog of war and ambient lighting descriptions accurate ✅

#### Cross-Cutting Gaps (applies to all tutorials)

1. **Homebrew system entirely absent** — No tutorial mentions homebrew items, monsters, or spells. This is a major feature with its own dashboard tab.
2. **Character spell management absent** — Character sheet now has a Spells tab. Tutorials don't mention it.
3. **Character inventory absent** — Character sheet now has an Equipment tab with InventoryManager. Tutorials don't mention it.
4. **No screenshots exist** — All `![image](../images/tutorials/*.png)` references point to non-existent files. Screenshots need to be captured for: home screen, campaign dashboard (showing 5 tabs), modules tab, token setup modal, play mode, player display views.

#### Screenshot Recommendations

- `home-screen.png` — Home screen with skull and campaign selector visible
- `campaign-dashboard.png` — Dashboard showing all 5 tabs, with Homebrew tab visible
- `module-tab.png` — Modules listing with Play/Open/PDF action buttons
- `token-setup.png` — Token setup modal showing palette with Monster/NPC/Trap/Marker sections
- `play-mode.png` — Full play mode with sidebar, map, and notes panel
- `player-display-header.png` — Play mode header showing Player Display button
- `dm-view.png` / `player-view-fog.png` / `player-view-token-los.png` — Comparison views