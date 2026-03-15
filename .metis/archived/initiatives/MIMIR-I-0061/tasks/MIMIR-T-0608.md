---
id: pass-1-12-write-homebrew-how-to
level: task
title: "Pass 1.12: Write homebrew how-to section (4 pages)"
short_code: "MIMIR-T-0608"
created_at: 2026-03-13T13:50:25.750548+00:00
updated_at: 2026-03-13T14:18:03.789787+00:00
parent: MIMIR-I-0061
blocked_by: []
archived: true

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: MIMIR-I-0061
---

# Pass 1.12: Write homebrew how-to section (4 pages)

## Parent Initiative

[[MIMIR-I-0061]]

## Objective

Write the complete homebrew how-to section: 4 new pages covering the homebrew system that currently has zero user-facing documentation.

## Scope

Create the `docs/src/how-to/homebrew/` directory with 4 pages. The homebrew system (items, monsters, spells) is a major feature with zero user-facing docs. Gap analysis in MIMIR-T-0584 has the full system inventory.

### Page 1: `how-to/homebrew/README.md` — Overview
- What homebrew content is and why you'd create it
- The 3 types: items, monsters, spells
- Where to find it: Homebrew tab (5th tab on campaign dashboard)
- Campaign-scoped: homebrew belongs to a specific campaign
- Links to the 3 creation pages

### Page 2: `how-to/homebrew/create-item.md` — Create Homebrew Items
- Navigate to Homebrew tab → Items sub-tab
- Two paths: create from scratch vs clone-from-catalog
- Clone workflow: search catalog → select → modify fields → save
- Form fields: name, item_type, rarity, data (structured for weapons/armor, JSON for others)
- Editing and deleting (with inventory reference warning)

### Page 3: `how-to/homebrew/create-monster.md` — Create Homebrew Monsters
- Navigate to Homebrew tab → Monsters sub-tab
- Clone-from-catalog workflow (search by name/CR/type)
- JSON data editing with stat block preview
- Adding homebrew monsters to module encounter lists
- Deleting (with module reference warning)

### Page 4: `how-to/homebrew/create-spell.md` — Create Homebrew Spells
- Navigate to Homebrew tab → Spells sub-tab
- Clone-from-catalog workflow
- JSON data editing with spell stat block preview
- Assigning to characters via character spell management
- Deleting

### Verification Sources
- `HomebrewTab.vue` — tab layout, sub-tabs
- `HomebrewItemForm.vue` / `HomebrewMonsterForm.vue` / `HomebrewSpellForm.vue` — form fields
- `HomebrewService` in mimir-core — CRUD operations, clone-from-catalog
- Tauri commands: `create_homebrew_*`, `update_homebrew_*`, `delete_homebrew_*`
- `ModuleMonsters.vue` — homebrew monster integration

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] 4 pages created in `docs/src/how-to/homebrew/`
- [ ] All form fields verified against actual Vue components
- [ ] Clone-from-catalog workflow verified against service layer
- [ ] Delete warnings verified (reference checks in service layer)
- [ ] Cross-links to related pages (character inventory, module monsters)
- [ ] Consistent style with existing how-to pages

## Status Updates

### 2026-03-13: Completed
Created 4 pages in `docs/src/how-to/homebrew/`:

**README.md (overview):** 3 types, clone-from-catalog concept, integration points, HB badge

**create-item.md:** 
- Create from scratch OR clone from catalog (items support both)
- All common fields: name, item_type (10 options), rarity (6 options), weight, value, attunement, description
- Weapon-specific fields: category, bonus, damage, range, 10 properties
- Armor-specific fields: AC, bonus, strength, stealth disadvantage
- Delete warning shows affected character inventories
- Verified against `HomebrewTab.vue` items section

**create-monster.md:**
- Clone-only (no create from scratch — verified, no "Create" button exists)
- JSON data editing for stat block
- Detail pane shows CR, size, creature type
- Delete warning shows affected modules
- Verified against `HomebrewMonstersSubTab.vue`

**create-spell.md:**
- Clone-only (no create from scratch)
- JSON data editing
- Detail pane shows level and school
- Simple delete confirmation (no usage warnings)
- Verified against `HomebrewSpellsSubTab.vue`