---
id: pass-1-10-write-manage-inventory
level: task
title: "Pass 1.10: Write manage-inventory.md and level-up.md how-to pages"
short_code: "MIMIR-T-0606"
created_at: 2026-03-13T13:50:19.907517+00:00
updated_at: 2026-03-13T14:15:56.616196+00:00
parent: MIMIR-I-0061
blocked_by: []
archived: true

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: MIMIR-I-0061
---

# Pass 1.10: Write manage-inventory.md and level-up.md how-to pages

## Parent Initiative

[[MIMIR-I-0061]]

## Objective

Write two new how-to pages: `how-to/characters/manage-inventory.md` (Equipment tab, adding/removing items, equip/attune) and `how-to/characters/level-up.md` (8-step level-up dialog).

## Scope

### `how-to/characters/manage-inventory.md` (NEW)

Cover the Equipment tab and inventory management workflow:
1. Accessing the Equipment tab on a character sheet
2. Adding items from catalog search
3. Adding homebrew items (if created in Homebrew tab)
4. Equip/unequip and attune/unattune toggles
5. Removing items from inventory
6. How homebrew items display with "HB" badge

**Verification:** `InventoryManager.vue`, `CharacterSheetView.vue` (Equipment tab), `add_item_to_character` / `remove_item_from_character` / `update_character_inventory` Tauri commands.

### `how-to/characters/level-up.md` (NEW)

Cover the level-up dialog workflow. `LevelUpDialog.vue` has 8 steps:
1. Class — select class for the new level (multiclass support)
2. Subclass — choose subclass if at appropriate level
3. HP — roll or take average for hit points
4. ASI — ability score improvements at applicable levels
5. Spells — choose new spells if spellcaster
6. Feature Choices — select from class feature options
7. Features Display — review all features gained
8. Review — summary before confirming

**Verification:** `LevelUpDialog.vue` — step definitions and order, form fields per step.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] manage-inventory.md created with complete add/remove/equip workflow
- [ ] manage-inventory.md covers homebrew items and HB badge
- [ ] level-up.md created with all 8 steps documented
- [ ] All UI elements verified against Vue components
- [ ] Both pages match tone of existing character how-to pages

## Status Updates

### 2026-03-13: Completed

**manage-inventory.md created:**
- Three sub-tabs: Inventory, Equipment, Currency (verified `InventoryManager.vue`)
- Adding items: search with 2-char minimum, catalog + homebrew results, HB badge
- Equipment: checkbox equip/unequip toggles
- Attunement: 3-item limit enforced, separate section for magic items
- Currency: pp/gp/sp/cp with total gold calculation

**level-up.md created:**
- 8 steps documented with conditional visibility noted (verified `LevelUpDialog.vue`)
- Steps: Select Class, Choose Subclass, Hit Points, ASI, Spells, Features, Summary, Review
- HP options: Average/Roll/Manual
- Feature choices: Fighting Style, Metamagic, Maneuvers, Invocations, Expertise
- Navigation: checkmarks, back/next, revisitable steps
- Note: Spells step in level-up DOES allow selecting new cantrips/spells (different from character sheet Spells tab which is read-only reference)