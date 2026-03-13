---
id: audit-reference-pages-ui-reference
level: task
title: "Audit reference pages: UI reference, keyboard shortcuts, file formats, glossary"
short_code: "MIMIR-T-0581"
created_at: 2026-03-11T23:13:28.622267+00:00
updated_at: 2026-03-13T12:36:06.931807+00:00
parent: MIMIR-I-0059
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: MIMIR-I-0059
---

# Audit reference pages: UI reference, keyboard shortcuts, file formats, glossary

## Parent Initiative

[[MIMIR-I-0059]]

## Objective

Review all reference pages for accuracy — UI reference (6 pages), keyboard shortcuts, file formats, and glossary. Reference docs are where users go to look things up, so completeness and accuracy matter most here.

## Scope

### UI Reference (6 pages)
- `docs/src/reference/ui/home-screen.md`
- `docs/src/reference/ui/campaign-dashboard.md`
- `docs/src/reference/ui/module-prep-view.md`
- `docs/src/reference/ui/play-mode.md`
- `docs/src/reference/ui/token-setup-modal.md`
- `docs/src/reference/ui/player-display.md`

### Other Reference (4 pages)
- `docs/src/reference/keyboard-shortcuts.md`
- `docs/src/reference/file-formats.md`
- `docs/src/reference/vision-and-lighting.md`
- `docs/src/reference/glossary.md`

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] Each UI reference page verified against current Vue components — check every described element still exists
- [ ] Verify keyboard shortcuts against current keybinding handlers in frontend code
- [ ] Check file formats reference covers UVTT import, campaign export (.mimir), and any new formats
- [ ] Verify glossary includes all current terminology (homebrew, spell slots, sidecar, MCP, etc.)
- [ ] Check whether UI reference pages need updating for homebrew panels, spell management, inventory views
- [ ] Identify where screenshots would help — UI reference pages are prime candidates
- [ ] Produce findings report

## Screenshot Candidates

UI reference pages should ideally each have an annotated screenshot showing the key elements described:
- Home screen with campaign list
- Campaign dashboard with all tabs visible
- Module prep view showing monsters, items, documents tabs
- Play mode with encounter tracker, map, and token list
- Token setup modal (recently updated with new fields)
- Player display window

## Status Updates

### Audit Completed 2026-03-12

---

#### UI Reference Pages

**home-screen.md — Accurate.** Header bar items (Mimir logo, Campaign Selector, Characters, Reference, Settings gear) all match. Import Campaign is also available in campaign selector dropdown but not mentioned — minor gap.

**campaign-dashboard.md — 3 issues found**

1. **INACCURACY: Missing Homebrew tab** — Lists only 4 tabs (Campaign, Modules, NPCs, PCs). The dashboard now has 5 tabs including Homebrew (flask icon). Fix: Add Homebrew tab section.

2. **INACCURACY: Document stages are wrong** — Claims 4 stages: "Pre-Production, Production, Post-Production, Archive." Documents have no stage/phase system in the actual code — the `Document` type has no status field. Fix: Remove "Document Stages" section or replace with accurate behavior (auto-save, no stages).

3. **INACCURACY: Campaign tab sub-sections** — Lists "World Building" and "Timeline" as separate sections. The Campaign tab is `WorldTab.vue` which contains documents and campaign notes — not separate "World Building" and "Timeline" views. Fix: Describe the actual WorldTab layout.

**module-prep-view.md — 1 issue found**

1. **INACCURACY: "Dangers" section** — Lists a "Dangers" section for monsters. The actual module prep view uses `ModuleMonsters.vue` — typically labeled "Monsters" not "Dangers." Fix: Rename to "Monsters" and describe the actual component (search, quantity, encounter tags, homebrew monster support).

**play-mode.md — 1 issue found**

1. **POSSIBLE INACCURACY: "Add Token" and "Add PCs" in toolbar** — Map Toolbar table lists these buttons. Not found in `ModulePlayView.vue` code. Same issue flagged in how-to guides. Fix: Verify manually; remove if not present.

**token-setup-modal.md — 1 issue found**

1. **INACCURACY: "Object" should be "Marker"** — Token palette lists "Object - Generic object tokens." Actual code has "Marker" (📍 icon). Fix: Replace "Object" with "Marker" and update description.

**player-display.md — Accurate.** Opening flow, fog of war modes (Fog/Token), DM controls (Blackout, Reveal Map, Sync, Push View, Ambient Light), physical setup tips — all match current code.

---

#### Other Reference Pages

**keyboard-shortcuts.md — 2 issues found**

1. **INACCURACY: Play Mode shortcuts wrong** — Lists `Space` (toggle play/pause) and `B` (toggle blackout). The actual `DmMapViewer.vue` keydown handler has: `+`/`=` (zoom in), `-` (zoom out), `0` (reset view), `h`/`H` (toggle token visibility), `d`/`D` (toggle dead state), `Escape` (close menu/deselect). No `Space` or `B` shortcuts exist. Fix: Replace with actual shortcuts.

2. **MISSING: Token manipulation shortcuts** — The `h` key toggles selected token visibility and `d` toggles dead state. These are not documented. Fix: Add to Token Manipulation section.

**file-formats.md — 1 issue found**

1. **MISSING: Dungeondraft map format** — The mapgen system now generates `.dungeondraft_map` files. This format should be mentioned alongside UVTT. Also missing mention of `.mimir-campaign.tar.gz` as the specific archive extension. Fix: Add Dungeondraft map format entry and clarify archive extension.

**vision-and-lighting.md — Accurate.** Vision types (Normal, Darkvision), ambient light levels, light source ranges, fog of war modes, wall occlusion, PC-only vision — all match current implementation. Well-written reference page.

**glossary.md — 3 missing terms**

1. **Missing: Homebrew** — "Custom content (items, monsters, spells) created by the DM, stored alongside catalog content."
2. **Missing: Sidecar** — "The MCP server binary that runs alongside Mimir, enabling AI-assisted campaign management."
3. **Missing: MCP (Model Context Protocol)** — "Protocol used by the Mimir sidecar to expose campaign management tools to AI assistants."
4. **INACCURACY: Campaign Dashboard definition** — Says "with tabs for Campaign, Modules, NPCs, and PCs" but should include Homebrew tab.

---

#### Cross-Cutting Issues

1. **"Object" vs "Marker"** — Appears in token-setup-modal.md (same issue as tutorials and how-to guides).
2. **Document stages are a fiction** — Referenced in campaign-dashboard.md with 4 stages, in how-to guides with 3 stages, in module docs with 2 stages. None exist in code. This needs a consistent fix across all docs.
3. **Homebrew missing everywhere** — Dashboard reference, glossary, and module prep view don't mention it.
4. **No screenshots exist** — All image references point to non-existent files. UI reference pages are the highest priority for screenshots.

#### Screenshot Recommendations (Priority Order)
1. Campaign dashboard showing all 5 tabs (Homebrew visible)
2. Token setup modal with Marker type
3. Play mode full view
4. Player display in Fog and Token modes
5. Module prep view with Monsters section