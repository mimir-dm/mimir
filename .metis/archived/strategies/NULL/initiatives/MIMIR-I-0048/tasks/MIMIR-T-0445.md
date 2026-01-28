---
id: frontend-level-up-wizard-framework
level: task
title: "Frontend - Level Up Wizard Framework"
short_code: "MIMIR-T-0445"
created_at: 2026-01-27T21:15:11.798141+00:00
updated_at: 2026-01-27T22:46:17.911687+00:00
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

# Frontend - Level Up Wizard Framework

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[MIMIR-I-0048]]

## Objective

Implement the dynamic multi-step wizard framework for the LevelUpDialog that conditionally shows/hides steps based on class and level.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [x] Wizard supports up to 8 steps with dynamic visibility
- [x] Progress indicator shows current step and total applicable steps
- [x] Back/Next navigation between steps
- [x] Step validation prevents advancing with incomplete choices
- [x] State persists across step navigation
- [x] Final "Confirm Level Up" button submits to backend
- [x] Loading state during backend submission
- [x] Error handling and display

## Implementation Notes

### Step Visibility Logic

```typescript
interface WizardStep {
  id: string
  component: Component
  title: string
  isVisible: (context: LevelUpContext) => boolean
  isComplete: (context: LevelUpContext) => boolean
}

const steps: WizardStep[] = [
  { id: 'class', component: ClassSelectionStep, isVisible: () => true, ... },
  { id: 'subclass', component: SubclassStep, isVisible: (ctx) => isAtSubclassLevel(ctx), ... },
  { id: 'hp', component: HitPointsStep, isVisible: () => true, ... },
  { id: 'asi', component: AbilityScoreStep, isVisible: (ctx) => isAtAsiLevel(ctx), ... },
  { id: 'spells', component: SpellsStep, isVisible: (ctx) => gainsSpellsAtLevel(ctx), ... },
  { id: 'features', component: FeatureChoicesStep, isVisible: (ctx) => hasFeatureChoices(ctx), ... },
  { id: 'display', component: FeaturesDisplayStep, isVisible: () => true, ... },
  { id: 'review', component: ReviewStep, isVisible: () => true, ... },
]
```

### LevelUpContext

```typescript
interface LevelUpContext {
  character: Character
  selectedClass: CharacterClass | null
  newLevel: number
  classInfo: ClassInfo | null  // From catalog
  
  // Collected choices
  hpMethod: HpGainMethod
  hpGain: number
  subclass: SubclassChoice | null
  asiOrFeat: AsiOrFeat | null
  spellChanges: SpellChanges | null
  featureChoices: FeatureChoices | null
}
```

### File Structure
```
src/features/characters/components/levelup/
├── LevelUpDialog.vue          # Main wizard container
├── LevelUpWizard.vue          # Step navigation logic
├── steps/
│   ├── ClassSelectionStep.vue
│   ├── SubclassStep.vue
│   ├── HitPointsStep.vue
│   ├── AbilityScoreStep.vue
│   ├── SpellsStep.vue
│   ├── FeatureChoicesStep.vue
│   ├── FeaturesDisplayStep.vue
│   └── ReviewStep.vue
└── composables/
    └── useLevelUp.ts          # Shared state and logic
```

### Dependencies
- MIMIR-T-0444 (Catalog API for class info)

## Status Updates

### 2026-01-27 - Implementation Complete

Created complete level up wizard framework with dynamic step visibility.

**Files Created:**
- `src/features/characters/composables/useLevelUp.ts` - Composable with full wizard state management
- `src/features/characters/components/levelup/LevelUpDialog.vue` - Main wizard container with progress indicator
- `src/features/characters/components/levelup/steps/ClassSelectionStep.vue` - Existing class or multiclass selection
- `src/features/characters/components/levelup/steps/SubclassStep.vue` - Subclass selection (placeholder with manual entry)
- `src/features/characters/components/levelup/steps/HitPointsStep.vue` - Average/Roll/Manual HP selection
- `src/features/characters/components/levelup/steps/AbilityScoreStep.vue` - ASI controls (+/- for 2 points), feat placeholder
- `src/features/characters/components/levelup/steps/SpellsStep.vue` - New cantrips/spells, spell swap (placeholder with manual entry)
- `src/features/characters/components/levelup/steps/FeatureChoicesStep.vue` - Fighting style, metamagic, maneuvers, invocations, etc.
- `src/features/characters/components/levelup/steps/ReviewStep.vue` - Summary of all choices before submission

**Level Up Types Added to `types/character.ts`:**
- HpGainMethod, AsiOrFeat, SubclassChoice, SpellReference, SpellChanges
- FeatureReference, ManeuverChoices, InvocationChoices, FeatureChoices
- LevelUpRequest, LevelUpResult

**Composable Features (useLevelUp.ts):**
- Step visibility functions: isAtSubclassLevel(), isAtAsiLevel(), gainsSpellsAtLevel(), hasFeatureChoices()
- Dynamic step list based on class/level
- Navigation: goToNextStep(), goToPreviousStep(), goToStep()
- Class selection with isNewClass flag for multiclass
- Catalog lookup integration via classInfo
- Build and submit LevelUpRequest to backend

**Step Visibility Rules (2014 PHB):**
- Subclass: Fighter 3, Wizard 2, Cleric 1, etc.
- ASI: Levels 4, 8, 12, 16, 19 (plus Fighter/Rogue extras)
- Spells: Spellcasting classes based on progression
- Features: Fighting style, metamagic, maneuvers, invocations, pact boon, expertise

Build passes: `npm run type-check` and `npm run build` both successful.