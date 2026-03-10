---
id: regression-tests-for-all-fixed
level: task
title: "Regression tests for all fixed character sheet bugs"
short_code: "MIMIR-T-0540"
created_at: 2026-03-09T14:25:16.758402+00:00
updated_at: 2026-03-10T17:37:38.482070+00:00
parent: MIMIR-I-0056
blocked_by: []
archived: true

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: MIMIR-I-0056
---

# Regression tests for all fixed character sheet bugs

## Parent Initiative

[[MIMIR-I-0056]]

## Objective

Write comprehensive regression tests for `src/utils/characterUtils.ts` — the core character sheet calculation module. This file contains all D&D 5e ability modifiers, proficiency bonuses, skill/save calculations, armor AC, weapon damage, spellcasting stats, multiclass caster level, hit dice, and display formatting functions. Several of these were targets of past bug fixes (multiclass spellcasting stats, multiclass caster level, spell dedup).

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [x] Tests for getModifier — standard scores, edge cases (1, 10, 20, 30)
- [x] Tests for formatModifier — positive, zero, negative
- [x] Tests for getProficiencyBonus — levels 0-20, edge at boundaries
- [x] Tests for getTotalLevel — single class, multiclass, no classes
- [x] Tests for proficiency helpers — isProficientInSkill, hasSkillExpertise, isProficientInSave
- [x] Tests for getSkillBonus — bare modifier, proficient, expertise
- [x] Tests for getSaveBonus — proficient and non-proficient
- [x] Tests for getPassivePerception — base, proficient, expertise
- [x] Tests for getArmorAC — light/medium/heavy armor, magic bonuses
- [x] Tests for getWeaponDamage — all weapon categories
- [x] Tests for isFinesse and isRanged — weapon property detection
- [x] Tests for spellcasting detection and ability mapping
- [x] Tests for getSpellSaveDC and getSpellAttackBonus
- [x] Tests for getAllSpellcastingStats — single and multiclass (regression: d3bdbbd)
- [x] Tests for getMulticlassCasterLevel — full/half/third casters, Warlock exclusion (regression: d3bdbbd)
- [x] Tests for getHitDiceString — single and multiclass
- [x] Tests for formatClassString — with/without subclass
- [x] All tests pass via `angreal test unit --ui`

## Status Updates

### Session 1 (2026-03-10)
- Identified regression targets from git history: multiclass spellcasting stats (d3bdbbd), multiclass caster level (d3bdbbd), spell dedup (29ac88f)
- Read full characterUtils.ts (504 lines, 30+ exported functions/constants)
- Writing comprehensive test file at `src/utils/__tests__/characterUtils.test.ts`
- Created `src/utils/__tests__/characterUtils.test.ts` with 80+ tests covering all exported functions
- Key regression areas tested: `getAllSpellcastingStats` multiclass (d3bdbbd), `getMulticlassCasterLevel` with Warlock exclusion and Artificer rounding (d3bdbbd)
- All 1459 tests pass across 59 test files