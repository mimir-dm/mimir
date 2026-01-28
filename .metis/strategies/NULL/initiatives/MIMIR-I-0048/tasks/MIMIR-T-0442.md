---
id: backend-spell-and-feature-updates
level: task
title: "Backend - Spell and Feature Updates"
short_code: "MIMIR-T-0442"
created_at: 2026-01-27T21:15:10.116109+00:00
updated_at: 2026-01-27T22:28:06.901651+00:00
parent: MIMIR-I-0048
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: MIMIR-I-0048
---

# Backend - Spell and Feature Updates

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[MIMIR-I-0048]]

## Objective

Extend the level up service to handle spell changes (new spells, cantrips, spell swaps) and feat additions during level up.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [x] Add new spells to `character_spells` table for Spells Known casters
- [x] Support Wizard spellbook additions (2 spells per level)
- [x] Support spell swap (remove one, add one) for Spells Known casters
- [x] Add new cantrips to `character_spells` table (note: spell_level not needed - comes from catalog)
- [x] Add feat to `character_feats` table when feat chosen over ASI (already implemented in T-0441)
- [ ] Validate spell levels don't exceed class spell slot progression (deferred to frontend - catalog provides validation data)
- [ ] Validate cantrip count doesn't exceed class cantrip known limit (deferred to frontend - catalog provides limits)

## Implementation Notes

### Spellcasting Types

| Type | Classes | Level-Up Behavior |
|------|---------|-------------------|
| Spells Known | Bard, Ranger, Sorcerer, Warlock | Learn per table, swap 1 |
| Third Caster Known | Eldritch Knight, Arcane Trickster | Learn per table, swap 1 |
| Wizard Spellbook | Wizard | Add 2 spells to spellbook |
| Prepared | Cleric, Druid, Paladin | No spell selection needed |

### SpellChanges Structure
```rust
pub struct SpellChanges {
    pub new_spells: Vec<SpellReference>,
    pub new_cantrips: Vec<SpellReference>,
    pub swap_out: Option<SpellReference>,
    pub swap_in: Option<SpellReference>,
}
```

### Database Operations
- Insert into `character_spells` for new spells/cantrips
- Delete + Insert for spell swap
- Insert into `character_feats` for feat selection

### Dependencies
- MIMIR-T-0441 (Core Level Up Command)

## Status Updates

### Session 2026-01-27

**Completed Implementation:**

1. **DAL Functions Added** (`character_spell.rs`):
   - `find_character_spell_by_name` - Finds a spell by name and source class for swap operations
   - `count_spells_by_class` - Counts spells for a character by source class

2. **New Types Added** (`services/character.rs`):
   - `SpellChanges` - Container for new_spells, new_cantrips, swap_out, swap_in
   - `SpellReference` - Spell name + source reference

3. **LevelUpRequest Extended**:
   - Added `spell_changes: Option<SpellChanges>` field

4. **Spell Handling in `level_up` Method** (Step 8):
   - Spell swap: Validates swap_out exists, deletes it, inserts swap_in
   - New spells: Adds each spell (skips duplicates via `character_knows_spell`)
   - New cantrips: Adds each cantrip (skips duplicates)

5. **Exports Updated** (`services/mod.rs`):
   - Added `SpellChanges` and `SpellReference` to public exports

**Design Decisions:**
- Spell level NOT stored in character_spells - comes from catalog lookup
- Duplicate prevention: silently skips if character already knows a spell
- Validation of spell levels/cantrip limits deferred to frontend (has catalog access)

**Verification:**
- `cargo check` passed with no errors