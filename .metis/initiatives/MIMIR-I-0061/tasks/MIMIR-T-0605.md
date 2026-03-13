---
id: pass-1-9-write-manage-spells-md
level: task
title: "Pass 1.9: Write manage-spells.md how-to page"
short_code: "MIMIR-T-0605"
created_at: 2026-03-13T13:50:18.882170+00:00
updated_at: 2026-03-13T14:13:42.331505+00:00
parent: MIMIR-I-0061
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: MIMIR-I-0061
---

# Pass 1.9: Write manage-spells.md how-to page

## Parent Initiative

[[MIMIR-I-0061]]

## Objective

Write new page `how-to/characters/manage-spells.md` covering the character spell management workflow (Spells tab, adding/removing spells, spell slot tracking).

## Scope

Write `docs/src/how-to/characters/manage-spells.md` — a new how-to page for character spell management, a feature added in v0.6.0 with zero documentation.

### Page Content

1. **Prerequisites** — Character must be a spellcasting class (wizard, cleric, etc.)
2. **Accessing the Spells tab** — Open character sheet → Spells tab (only visible for casters)
3. **Adding spells** — Search catalog, select spells, add to character's spell list
4. **Homebrew spells** — Note that homebrew spells appear alongside catalog spells if created
5. **Spell slots** — How spell slot tracking works (if applicable in UI)
6. **Removing spells** — How to remove spells from a character

### Verification Sources
- `SpellsSection.vue` or equivalent — spell management UI
- `CharacterSheetView.vue` — Spells tab visibility and layout
- `add_character_spell` / `remove_character_spell` Tauri commands
- `CharacterSpellService` in mimir-core — service capabilities

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] Page created at `docs/src/how-to/characters/manage-spells.md`
- [ ] All described UI elements verified against Vue components
- [ ] Spell add/remove workflow matches actual implementation
- [ ] Homebrew spell integration mentioned
- [ ] Matches tone and structure of existing character how-to pages

## Status Updates

### 2026-03-13: Completed
Created `docs/src/how-to/characters/manage-spells.md`.

**Key finding:** The Spells tab is **read-only** — it displays available class spells but has no add/remove UI. Spell selection happens during character creation (SpellSelector.vue) and level-up, not on the character sheet itself. Wrote the page to accurately reflect this.

**Content:**
- Spells tab visibility (casters only, verified `CharacterSheetView.vue` lines 48-54)
- Three sections: Spellcasting Stats, Spell Slots, Available Spells (verified `SpellsSection.vue`)
- Multiclass support noted
- Spell card details (school, tags: HB/R/C, expandable description)
- Spell selection via creation wizard and level-up
- Homebrew spell integration (verified `useSpellManagement.ts` line 405 — `list_homebrew_spells`)

**Post-completion correction:** User clarified spell selection is NOT a thing in Mimir — it's a table activity managed by the player. Mimir shows all possible class spells as a reference. Removed "Selecting Spells" section and SpellSelector references. Also fixed create-pc.md Spells tab description.