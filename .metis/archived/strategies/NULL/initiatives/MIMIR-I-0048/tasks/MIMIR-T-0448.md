---
id: level-up-testing-and-validation
level: task
title: "Level Up Testing and Validation"
short_code: "MIMIR-T-0448"
created_at: 2026-01-27T21:15:13.344103+00:00
updated_at: 2026-01-28T03:50:45.846363+00:00
parent: MIMIR-I-0048
blocked_by: []
archived: true

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: MIMIR-I-0048
---

# Level Up Testing and Validation

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[MIMIR-I-0048]]

## Objective

Comprehensive testing of the level-up workflow covering all D&D 5e rules and edge cases.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] All test scenarios pass
- [ ] No regressions in existing character functionality
- [ ] Error handling works correctly for invalid inputs

## Test Scenarios

### Single-Class Progression
- [ ] Fighter 1→2: HP gain only (no choices)
- [ ] Fighter 3→4: HP + ASI (standard ASI level)
- [ ] Fighter 5→6: HP + ASI (Fighter extra ASI)
- [ ] Rogue 9→10: HP + ASI (Rogue extra ASI)
- [ ] Fighter 2→3: HP + Subclass selection (Battle Master)

### Multiclass Scenarios
- [ ] Fighter 5 + Wizard 1: Validates INT 13 prereq, adds new class entry
- [ ] Wizard 5 + Fighter 1: Validates INT 13 (leaving) + STR/DEX 13 (entering)
- [ ] Monk 1 attempting Paladin: Should fail (needs STR 13 AND CHA 13)

### HP Calculation
- [ ] Average HP: Fighter (d10) = 6 + CON mod
- [ ] Rolled HP: Validates roll between 1 and hit_die
- [ ] Manual HP: Accepts any positive value + CON mod
- [ ] Negative CON: -2 CON mod still results in minimum 1 HP

### ASI/Feat
- [ ] ASI +2 single: STR 16 → 18
- [ ] ASI +1/+1 split: STR 15 → 16, CON 15 → 16
- [ ] ASI cap at 20: STR 19 + 2 = 20 (not 21)
- [ ] Feat selection: Stores feat in character_feats

### Subclass Selection
- [ ] Cleric level 1: Shows subclass (Domain) selection
- [ ] Wizard level 2: Shows subclass (Tradition) selection
- [ ] Fighter level 3: Shows subclass (Archetype) selection
- [ ] Fighter level 4: No subclass step (already selected)

### Spellcasting
- [ ] Bard 2→3: New spells + spell swap option
- [ ] Wizard 1→2: Add 2 spells to spellbook
- [ ] Sorcerer 3→4: New spell + cantrip (if applicable)
- [ ] Fighter (non-caster): No spell step shown

### Class Feature Choices
- [ ] Fighter 1: Fighting Style selection
- [ ] Rogue 1: Expertise (2 skills)
- [ ] Sorcerer 3: Metamagic (2 options)
- [ ] Battle Master 3: Maneuvers (3 options)
- [ ] Warlock 2: Invocations (2 options)
- [ ] Warlock 3: Pact Boon selection

### Warlock Special Cases
- [ ] Warlock 5: New invocation + invocation swap
- [ ] Invocation with level prereq: Thirsting Blade requires level 5
- [ ] Invocation with pact prereq: Can't select Blade invocations without Pact of the Blade

### Edge Cases
- [ ] Character with no classes (shouldn't happen but handle gracefully)
- [ ] Level 20 character (max level, no level up available)
- [ ] Multiclass 3+ classes: All prereqs validated

## Dependencies
- All other tasks in this initiative must be complete

## Status Updates

*To be added during implementation*