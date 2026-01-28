---
id: frontend-core-wizard-steps
level: task
title: "Frontend - Core Wizard Steps"
short_code: "MIMIR-T-0446"
created_at: 2026-01-27T21:15:12.251159+00:00
updated_at: 2026-01-28T01:29:45.501289+00:00
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

# Frontend - Core Wizard Steps

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[MIMIR-I-0048]]

## Objective

Implement the core wizard steps: Class Selection, Subclass, Hit Points, and Ability Score/Feat selection.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

### ClassSelectionStep
- [x] Shows current class(es) with "Level X → Level X+1" display
- [x] Shows available multiclass options from catalog
- [x] Displays prerequisites for entering new class (multiclass prereqs)
- [x] Disables multiclass options that don't meet prerequisites
- [x] Fetches class info from catalog on selection

### SubclassStep
- [x] Only shown when at subclass selection level for the class
- [x] Lists subclasses for the selected class from catalog
- [x] Shows subclass descriptions and features at level
- [x] Stores selection in context
- [x] Manual entry fallback when catalog unavailable

### HitPointsStep
- [x] Shows hit die for selected class (e.g., "d10 for Fighter")
- [x] Three method options: Average, Roll, Manual
- [x] Average shows calculated value: `(die/2)+1 + CON mod`
- [x] Roll has input for die result + CON mod
- [x] Manual has number input
- [x] Displays CON modifier clearly

### AbilityScoreStep
- [x] Toggle between ASI and Feat modes
- [x] ASI mode: +2 to one ability OR +1 to two abilities
- [x] Shows current scores and new scores preview
- [x] Prevents increases above 20
- [x] Feat mode: Searchable grid from catalog
- [x] Shows feat prerequisites with met/unmet status
- [x] Disables feats that don't meet prerequisites

## Implementation Notes

### Multiclass Prerequisite Display

```vue
<div class="multiclass-option" :class="{ disabled: !meetsPrereqs }">
  <span class="class-name">Fighter</span>
  <span class="prereqs">
    Requires: STR 13 or DEX 13
    <span v-if="!meetsPrereqs" class="missing">(You have STR 10, DEX 12)</span>
  </span>
</div>
```

### HP Calculation Display

```vue
<div class="hp-total">
  <span class="base">{{ baseHp }}</span>
  <span class="modifier">{{ conMod >= 0 ? '+' : '' }}{{ conMod }}</span>
  <span class="equals">=</span>
  <span class="total">{{ Math.max(1, baseHp + conMod) }}</span>
  <span v-if="baseHp + conMod < 1" class="minimum-note">(minimum 1)</span>
</div>
```

### Dependencies
- MIMIR-T-0445 (Wizard Framework)
- MIMIR-T-0444 (Catalog API)

## Status Updates

### 2026-01-27 - Implementation Complete

Enhanced all core wizard steps with catalog integration.

**ClassSelectionStep enhancements:**
- Loads PHB classes from catalog via `search_classes`
- Parses multiclass prerequisites from class data JSON
- Checks character ability scores against requirements
- Displays "Requires: STR 13" with "(You have STR 10)" when unmet
- Disables multiclass cards for classes that don't meet prereqs
- Shows hit die for each class option

**SubclassStep enhancements:**
- Loads subclasses from catalog via `list_subclasses_by_class`
- Parses subclass features to show what's gained at current level
- Displays subclass grid with name and source
- Includes details panel showing selected subclass info
- Manual entry fallback when catalog unavailable

**HitPointsStep:**
- Already fully implemented in T-0445
- Shows hit die, CON modifier, total calculation
- Average/Roll/Manual methods all working

**AbilityScoreStep enhancements:**
- Loads feats from catalog via `list_feats_with_prereqs`
- Searchable feat grid with filtering
- Parses prerequisites and checks against character:
  - Ability score requirements
  - Spellcasting requirements (checks for caster classes)
  - Armor proficiency requirements (simplified)
- Shows prerequisites with met/unmet styling
- Disables feats that don't meet prereqs
- Manual entry fallback

Build passes: `npm run type-check` successful