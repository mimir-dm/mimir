---
id: audit-how-to-guides-characters-and
level: task
title: "Audit how-to guides: characters and play mode"
short_code: "MIMIR-T-0580"
created_at: 2026-03-11T23:13:28.523096+00:00
updated_at: 2026-03-13T12:34:56.078338+00:00
parent: MIMIR-I-0059
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: MIMIR-I-0059
---

# Audit how-to guides: characters and play mode

## Parent Initiative

[[MIMIR-I-0059]]

## Objective

Review the 9 how-to guide pages covering characters and play mode for accuracy. These guides cover the most interactive parts of Mimir — character management and live session features — where the UI has evolved significantly.

## Scope

### Characters (4 pages)
- `docs/src/how-to/characters/create-pc.md`
- `docs/src/how-to/characters/create-npc.md`
- `docs/src/how-to/characters/assign-to-campaign.md`
- `docs/src/how-to/characters/print-character-sheet.md`

### Play Mode (5 pages)
- `docs/src/how-to/play-mode/start-session.md`
- `docs/src/how-to/play-mode/manage-encounters.md`
- `docs/src/how-to/play-mode/fog-of-war.md`
- `docs/src/how-to/play-mode/use-player-display.md`

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] Each page read and cross-referenced against current Vue components and Tauri commands
- [ ] Verify character creation steps match current form fields (race, class, background, ability scores)
- [ ] Check if character docs mention spell management (added in v0.6.0) — likely missing
- [ ] Check if character docs mention inventory management and item assignment — likely missing
- [ ] Verify NPC creation flow matches current implementation
- [ ] Check print character sheet guide against current `mimir-print` PDF output
- [ ] Verify play mode steps: session start, encounter management, initiative tracking
- [ ] Check fog of war docs against current implementation
- [ ] Verify player display setup and WebSocket connection flow
- [ ] Identify where screenshots would help — especially character sheet, play mode encounter panel, fog of war controls
- [ ] Produce findings report

## Screenshot Candidates

- Character creation form (PC and NPC variants)
- Character sheet view with spells tab (new)
- Character inventory panel (new)
- Play mode encounter tracker
- Fog of war controls overlay
- Player display connection setup
- Printed character sheet PDF example

## Status Updates

### Audit Completed 2026-03-12

---

#### Characters

**create-pc.md — 2 issues found**

1. **INACCURACY: Wizard has 7 steps, not 5** — Guide says wizard has 5 steps (Basics, Race, Class, Abilities, Review). The actual `CharacterCreationWizard.vue` has 7 steps: Basics, Race, Class, **Background**, Abilities, **Skills**, Review. Background and Skills steps are completely missing from the documentation. Fix: Add Background (step 4) and Skills (step 6) sections.

2. **MISSING FEATURES: Character sheet tabs not mentioned** — Guide says "Open the character sheet, edit sections directly" but doesn't mention the 4-tab layout: Character, Equipment, Spells (casters only), Details. The Spells and Equipment tabs are major features. Fix: Add "After Creation" section describing the character sheet tabs.

**create-npc.md — 1 issue found**

1. **INCOMPLETE: NPC creation has more fields than documented** — Guide says "only a name is required" which is true, but omits the optional fields visible in the NPC creation form: Race, Role (e.g. "Merchant"), Location (e.g. "Tavern"), and Faction (e.g. "Thieves Guild"). Fix: List optional fields.

**assign-to-campaign.md — Accurate.** Both assignment methods verified: "Add Existing" button on PCs tab opens `AddCharacterModal`, and character list view has campaign assignment dropdown. NPC assignment to modules via `NpcSelectorModal` also confirmed.

**print-character-sheet.md — 1 issue found**

1. **INCOMPLETE: Missing 2 print options** — Guide describes "Compact Sheet" and "Spell Cards". The actual `CharacterPrintDialog.vue` has 4 options: Compact Sheet (2-page), **Battle Card** (half-page combat reference), Spell Cards, and **Equipment Cards** (weapon/magic item cards). Fix: Add Battle Card and Equipment Cards sections.

---

#### Play Mode

**start-session.md — 1 issue found**

1. **POSSIBLE INACCURACY: "Add PCs" and "Add Token" buttons** — Guide describes "Add PCs" and "Add Token" buttons in play mode. These buttons were NOT found in `ModulePlayView.vue` during code review. They may have been removed or relocated. Fix: Verify manually in running app; if removed, delete these sections.

**manage-encounters.md — 1 issue found**

1. **SAME ISSUE: "Add Token" and "Add PCs" buttons** — Guide references "Click Add Token in the toolbar" and "Click Add PCs." Same concern as start-session.md — these buttons not found in play mode code. Fix: Same as above.

**fog-of-war.md — Accurate.** Vision modes (Fog/Token), ambient light levels (Bright/Dim/Dark), Reveal Map toggle, wall occlusion for UVTT maps — all match current code. LOS toggle confirmed in ModulePlayView.vue.

**use-player-display.md — Accurate.** Player Display button, Blackout mode, viewport sync/Push View, map switching — all confirmed. Clean and accurate guide.

---

#### Cross-Cutting Gaps

1. **Character spell management undocumented** — CharacterSheetView has a Spells tab with SpellsSection component for managing character spells (added v0.6.0). No how-to guide exists for this workflow.

2. **Character inventory undocumented** — CharacterSheetView has an Equipment tab with InventoryManager dialog for managing items, equip/attune status. No how-to guide exists.

3. **Level-up dialog undocumented** — LevelUpDialog.vue has 8 steps (Class, Subclass, HP, ASI, Spells, Feature Choices, Features Display, Review). Only mentioned in passing as "Level up uses a separate dialog" in create-pc.md. Needs its own how-to guide.

4. **"Add Token"/"Add PCs" may not exist** — Referenced in start-session.md and manage-encounters.md but not found in code. Needs manual verification.

5. **No screenshots exist** — All image references point to non-existent files.

#### Screenshot Recommendations
- Character creation wizard showing all 7 steps
- NPC creation form with optional fields visible
- Character sheet with Equipment and Spells tabs
- Character print dialog showing all 4 options
- Level-up dialog
- Play mode (verify Add Token/Add PCs presence)