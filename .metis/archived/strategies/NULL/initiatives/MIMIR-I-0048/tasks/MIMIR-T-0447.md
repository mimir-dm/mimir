---
id: frontend-spells-and-features-steps
level: task
title: "Frontend - Spells and Features Steps"
short_code: "MIMIR-T-0447"
created_at: 2026-01-27T21:15:12.846073+00:00
updated_at: 2026-01-28T01:39:30.338700+00:00
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

# Frontend - Spells and Features Steps

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[MIMIR-I-0048]]

## Objective

Implement the Spells step, Feature Choices step, Features Display step, and Review step of the level-up wizard.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

### SpellsStep
- [ ] Shows spell selection for Spells Known casters
- [ ] Shows "Add 2 spells to spellbook" for Wizard
- [ ] Supports spell swap (current spell → new spell) for Known casters
- [ ] Shows cantrip selection when gaining cantrips
- [ ] Filters spells by max castable level for the class
- [ ] Searchable spell list from catalog
- [ ] Shows spell school, level, casting time in list

### FeatureChoicesStep
- [ ] Dynamic content based on available choices at this level
- [ ] Fighting Style selector (Fighter 1, Paladin 2, Ranger 2)
- [ ] Expertise skill selector (Rogue 1/6, Bard 3/10) - pick 2 from proficient skills
- [ ] Metamagic selector (Sorcerer 3/10/17)
- [ ] Maneuvers selector with swap (Battle Master)
- [ ] Invocations selector with swap and prereq display (Warlock)
- [ ] Pact Boon selector (Warlock 3)

### FeaturesDisplayStep
- [ ] Read-only display of all features gained at this level
- [ ] Fetches from class features catalog
- [ ] Shows feature name and description
- [ ] Expandable for long descriptions

### ReviewStep
- [ ] Summary of all choices made
- [ ] New level display (e.g., "Fighter 4 → Fighter 5")
- [ ] HP change (e.g., "+8 HP (6 + 2 CON)")
- [ ] Ability score changes if ASI taken
- [ ] Feat name if feat taken
- [ ] New spells/cantrips list
- [ ] New features list
- [ ] "Confirm Level Up" button

## Implementation Notes

### Spell Selection UI
Reuse or extend existing SpellSelector component if available. Key features:
- Filter by class spell list
- Filter by max level
- Show concentration, ritual tags
- Multi-select for new spells

### Invocation Prerequisites Display
```vue
<div class="invocation" :class="{ disabled: !meetsPrereqs }">
  <span class="name">Thirsting Blade</span>
  <span class="prereqs">
    Requires: Level 5, Pact of the Blade
    <span v-if="!hasPactBlade" class="missing">(Missing: Pact of the Blade)</span>
  </span>
</div>
```

### Dependencies
- MIMIR-T-0445 (Wizard Framework)
- MIMIR-T-0446 (Core Steps)
- MIMIR-T-0444 (Catalog API)

## Status Updates

### 2026-01-28: Implementation Complete

**SpellsStep.vue** - Full catalog integration:
- Loads cantrips and leveled spells via `get_spells_by_class`
- Calculates max spell level based on caster type (full, half, third, pact)
- Searchable grids with level filtering
- Spell swap section for Known casters (Sorcerer, Bard, Ranger, Warlock)
- Shows spell school, level, concentration (C), ritual (R) tags

**FeatureChoicesStep.vue** - Full catalog integration:
- Fighting styles from `list_fighting_styles()` with class filtering
- Metamagic from `list_metamagic()` with sorcery point costs
- Maneuvers from `list_maneuvers()` with search
- Invocations from `list_invocations()` with prereq checking (level, pact boon)
- Pact Boon selector with 4 options (Chain, Blade, Tome, Talisman)
- Expertise from character's proficient skills

**FeaturesDisplayStep.vue** - New component created:
- Loads class features via `list_class_features()`
- Loads subclass features via `get_subclass_by_name()`
- Shows features gained at current level
- Level summary section with computed values:
  - Proficiency bonus changes
  - Cantrips/spells known
  - Max spell level
  - Extra Attack progression
  - Class-specific resources (rage, sneak attack, ki, sorcery points, etc.)

**Integration updates:**
- Added `featuresDisplay` step to `useLevelUp.ts`
- Registered FeaturesDisplayStep in `LevelUpDialog.vue`

All type checks pass.