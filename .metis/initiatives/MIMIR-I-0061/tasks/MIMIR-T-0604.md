---
id: pass-1-8-fix-character-docs-create
level: task
title: "Pass 1.8: Fix character docs — create-pc, create-npc, print-character-sheet"
short_code: "MIMIR-T-0604"
created_at: 2026-03-13T13:50:18.009650+00:00
updated_at: 2026-03-13T14:12:27.543638+00:00
parent: MIMIR-I-0061
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: MIMIR-I-0061
---

# Pass 1.8: Fix character docs — create-pc, create-npc, print-character-sheet

## Parent Initiative

[[MIMIR-I-0061]]

## Objective

Fix three character how-to pages with known inaccuracies from audit MIMIR-T-0580.

## Scope

### `how-to/characters/create-pc.md`

**Issues (MIMIR-T-0580):**
1. Says wizard has 5 steps (Basics, Race, Class, Abilities, Review). Actual `CharacterCreationWizard.vue` has **7 steps**: Basics, Race, Class, **Background**, Abilities, **Skills**, Review. Background (step 4) and Skills (step 6) are completely missing.
2. No mention of the 4-tab character sheet layout after creation: Character, Equipment, Spells (casters only), Details.

**Fix:** Add Background and Skills step sections. Add "After Creation" section describing the character sheet tabs.

**Verification:** `CharacterCreationWizard.vue` — step definitions and order. `CharacterSheetView.vue` — tab layout.

### `how-to/characters/create-npc.md`

**Issue (MIMIR-T-0580):** Says "only a name is required" which is true, but omits optional fields: Race, Role, Location, Faction.

**Fix:** List all optional fields with brief descriptions.

**Verification:** NPC creation form component — check for optional field inputs.

### `how-to/characters/print-character-sheet.md`

**Issue (MIMIR-T-0580):** Only describes Compact Sheet and Spell Cards. Actual `CharacterPrintDialog.vue` has **4 options**: Compact Sheet (2-page), **Battle Card** (half-page combat reference), Spell Cards, **Equipment Cards**.

**Fix:** Add Battle Card and Equipment Cards sections with descriptions.

**Verification:** `CharacterPrintDialog.vue` — all print option definitions.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] create-pc.md describes all 7 wizard steps in correct order
- [ ] create-pc.md has "After Creation" section with character sheet tabs
- [ ] create-npc.md lists all optional fields (Race, Role, Location, Faction)
- [ ] print-character-sheet.md describes all 4 print options
- [ ] All changes verified against source code with file:line citations

## Status Updates

### 2026-03-13: Completed

**create-pc.md:**
- Fixed wizard from 5 steps to 7: added Background (step 4) and Skills (step 6)
- Verified against `CharacterCreationWizard.vue` step definitions
- Added "After Creation" section describing 4-tab character sheet layout (Character, Equipment, Spells, Details)
- Verified tabs against `CharacterSheetView.vue` line 262

**create-npc.md:**
- Added optional fields: Race, Role, Location, Faction
- Verified against NPC creation form (CharacterCreationWizard with npcOnly=true)

**print-character-sheet.md:**
- Added Battle Card (half-page combat reference) and Equipment Cards (weapons, magic items, special ammo)
- Organized into "Character Sheets" and "Cards" sections matching dialog layout
- Verified all 4 options against `CharacterPrintDialog.vue`