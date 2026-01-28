/**
 * Level Up Composable
 *
 * Shared state and logic for the level up wizard.
 */

import { ref, computed, readonly, type Ref, type ComputedRef } from 'vue'
import type {
  Character,
  CharacterClass,
  HpGainMethod,
  SubclassChoice,
  AsiOrFeat,
  SpellChanges,
  FeatureChoices,
  LevelUpRequest,
  LevelUpResult
} from '@/types/character'
import { totalLevel } from '@/types/character'
import { invoke } from '@tauri-apps/api/core'

// =============================================================================
// Types
// =============================================================================

/**
 * Class info from catalog for level up decisions.
 */
export interface ClassInfo {
  name: string
  source: string
  hit_die: number
  subclass_level: number
  asi_levels: number[]
  multiclass_prereqs: string | null
  spellcasting_ability: string | null
  caster_type: string | null
}

/**
 * Context for level up wizard steps.
 */
export interface LevelUpContext {
  character: Character
  selectedClass: CharacterClass | null
  isNewClass: boolean
  newClassLevel: number
  newTotalLevel: number
  classInfo: ClassInfo | null

  // Collected choices
  hpMethod: HpGainMethod | null
  subclass: SubclassChoice | null
  asiOrFeat: AsiOrFeat | null
  spellChanges: SpellChanges | null
  featureChoices: FeatureChoices | null
}

/**
 * Wizard step definition.
 */
export interface WizardStep {
  id: string
  title: string
  isVisible: (context: LevelUpContext) => boolean
  isComplete: (context: LevelUpContext) => boolean
}

// =============================================================================
// Step Visibility Logic
// =============================================================================

/**
 * Check if this level is a subclass selection level.
 */
export function isAtSubclassLevel(context: LevelUpContext): boolean {
  if (!context.classInfo || !context.selectedClass) return false

  // Check if already has subclass
  if (context.selectedClass.subclass_name) return false

  // Check if at subclass level
  return context.newClassLevel === context.classInfo.subclass_level
}

/**
 * Check if this level grants an ASI/Feat.
 */
export function isAtAsiLevel(context: LevelUpContext): boolean {
  if (!context.classInfo) return false
  return context.classInfo.asi_levels.includes(context.newClassLevel)
}

/**
 * Check if this class gains spells at this level.
 */
export function gainsSpellsAtLevel(context: LevelUpContext): boolean {
  if (!context.classInfo) return false
  // Has spellcasting ability = is a caster
  return context.classInfo.spellcasting_ability !== null
}

/**
 * Check if this level has feature choices (fighting style, metamagic, etc).
 */
export function hasFeatureChoices(context: LevelUpContext): boolean {
  if (!context.classInfo || !context.selectedClass) return false

  const className = context.selectedClass.class_name.toLowerCase()
  const level = context.newClassLevel

  // Fighting Style - Fighter 1, Paladin 2, Ranger 2
  if (className === 'fighter' && level === 1) return true
  if (className === 'paladin' && level === 2) return true
  if (className === 'ranger' && level === 2) return true

  // Metamagic - Sorcerer 3, 10, 17
  if (className === 'sorcerer' && [3, 10, 17].includes(level)) return true

  // Battle Master Maneuvers - Fighter with Battle Master subclass at 3, 7, 10, 15
  if (
    className === 'fighter' &&
    context.selectedClass.subclass_name?.toLowerCase().includes('battle master') &&
    [3, 7, 10, 15].includes(level)
  )
    return true

  // Warlock Invocations - 2, 5, 7, 9, 12, 15, 18
  if (className === 'warlock' && [2, 5, 7, 9, 12, 15, 18].includes(level)) return true

  // Warlock Pact Boon - 3
  if (className === 'warlock' && level === 3) return true

  // Expertise - Rogue 1, 6 and Bard 3, 10
  if (className === 'rogue' && [1, 6].includes(level)) return true
  if (className === 'bard' && [3, 10].includes(level)) return true

  return false
}

// =============================================================================
// Composable
// =============================================================================

/**
 * Create a level up wizard instance.
 */
export function useLevelUp(character: Ref<Character>) {
  // State
  const currentStepIndex = ref(0)
  const isSubmitting = ref(false)
  const error = ref<string | null>(null)

  // Selected class for level up
  const selectedClass = ref<CharacterClass | null>(null)
  const isNewClass = ref(false)
  const classInfo = ref<ClassInfo | null>(null)

  // Collected choices
  const hpMethod = ref<HpGainMethod | null>(null)
  const subclass = ref<SubclassChoice | null>(null)
  const asiOrFeat = ref<AsiOrFeat | null>(null)
  const spellChanges = ref<SpellChanges | null>(null)
  const featureChoices = ref<FeatureChoices | null>(null)

  // Computed values
  const newClassLevel = computed(() => {
    if (!selectedClass.value) return 1
    return isNewClass.value ? 1 : selectedClass.value.level + 1
  })

  const newTotalLevel = computed(() => {
    return totalLevel(character.value) + 1
  })

  // Context for step visibility/completion checks
  const context = computed<LevelUpContext>(() => ({
    character: character.value,
    selectedClass: selectedClass.value,
    isNewClass: isNewClass.value,
    newClassLevel: newClassLevel.value,
    newTotalLevel: newTotalLevel.value,
    classInfo: classInfo.value,
    hpMethod: hpMethod.value,
    subclass: subclass.value,
    asiOrFeat: asiOrFeat.value,
    spellChanges: spellChanges.value,
    featureChoices: featureChoices.value
  }))

  // Step definitions
  const allSteps: WizardStep[] = [
    {
      id: 'class',
      title: 'Select Class',
      isVisible: () => true,
      isComplete: (ctx) => ctx.selectedClass !== null
    },
    {
      id: 'subclass',
      title: 'Choose Subclass',
      isVisible: isAtSubclassLevel,
      isComplete: (ctx) => ctx.subclass !== null
    },
    {
      id: 'hp',
      title: 'Hit Points',
      isVisible: () => true,
      isComplete: (ctx) => ctx.hpMethod !== null
    },
    {
      id: 'asi',
      title: 'Ability Score',
      isVisible: isAtAsiLevel,
      isComplete: (ctx) => ctx.asiOrFeat !== null
    },
    {
      id: 'spells',
      title: 'Spells',
      isVisible: gainsSpellsAtLevel,
      isComplete: () => true // Optional step
    },
    {
      id: 'features',
      title: 'Features',
      isVisible: hasFeatureChoices,
      isComplete: () => true // Validation handled per feature type
    },
    {
      id: 'featuresDisplay',
      title: 'Summary',
      isVisible: () => true,
      isComplete: () => true // Informational step
    },
    {
      id: 'review',
      title: 'Review',
      isVisible: () => true,
      isComplete: () => true
    }
  ]

  // Visible steps (filtered based on context)
  const visibleSteps = computed(() => {
    return allSteps.filter((step) => step.isVisible(context.value))
  })

  // Current step
  const currentStep = computed(() => {
    return visibleSteps.value[currentStepIndex.value] || null
  })

  // Can navigate
  const canGoBack = computed(() => currentStepIndex.value > 0)
  const canGoForward = computed(() => {
    if (!currentStep.value) return false
    return currentStep.value.isComplete(context.value)
  })
  const isLastStep = computed(() => {
    return currentStepIndex.value === visibleSteps.value.length - 1
  })

  // Navigation
  function goToNextStep() {
    if (canGoForward.value && !isLastStep.value) {
      currentStepIndex.value++
    }
  }

  function goToPreviousStep() {
    if (canGoBack.value) {
      currentStepIndex.value--
    }
  }

  function goToStep(index: number) {
    if (index >= 0 && index < visibleSteps.value.length) {
      currentStepIndex.value = index
    }
  }

  // Class selection
  async function selectClass(charClass: CharacterClass, isNew: boolean) {
    selectedClass.value = charClass
    isNewClass.value = isNew
    error.value = null

    // Fetch class info from catalog
    try {
      const info = await invoke<{ success: boolean; data: ClassInfo; error?: string }>(
        'get_class_info',
        {
          className: charClass.class_name,
          classSource: charClass.class_source
        }
      )
      if (info.success) {
        classInfo.value = info.data
      } else {
        console.warn('Could not load class info:', info.error)
        // Set defaults
        classInfo.value = {
          name: charClass.class_name,
          source: charClass.class_source,
          hit_die: 8,
          subclass_level: 3,
          asi_levels: [4, 8, 12, 16, 19],
          multiclass_prereqs: null,
          spellcasting_ability: null,
          caster_type: null
        }
      }
    } catch (e) {
      console.error('Error fetching class info:', e)
      // Set defaults on error
      classInfo.value = {
        name: charClass.class_name,
        source: charClass.class_source,
        hit_die: 8,
        subclass_level: 3,
        asi_levels: [4, 8, 12, 16, 19],
        multiclass_prereqs: null,
        spellcasting_ability: null,
        caster_type: null
      }
    }
  }

  // Build level up request
  function buildRequest(): LevelUpRequest | null {
    if (!selectedClass.value || !hpMethod.value) {
      return null
    }

    const request: LevelUpRequest = {
      class_name: selectedClass.value.class_name,
      class_source: selectedClass.value.class_source,
      hit_points_method: hpMethod.value
    }

    if (subclass.value) {
      request.subclass = subclass.value
    }

    if (asiOrFeat.value) {
      request.asi_or_feat = asiOrFeat.value
    }

    if (spellChanges.value) {
      request.spell_changes = spellChanges.value
    }

    if (featureChoices.value) {
      request.feature_choices = featureChoices.value
    }

    return request
  }

  // Submit level up
  async function submit(): Promise<LevelUpResult | null> {
    const request = buildRequest()
    if (!request) {
      error.value = 'Please complete all required steps'
      return null
    }

    isSubmitting.value = true
    error.value = null

    try {
      const result = await invoke<{ success: boolean; data: LevelUpResult; error?: string }>(
        'level_up_character',
        {
          characterId: character.value.id,
          request
        }
      )

      if (result.success) {
        return result.data
      } else {
        error.value = result.error || 'Failed to level up character'
        return null
      }
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to level up character'
      return null
    } finally {
      isSubmitting.value = false
    }
  }

  // Reset state
  function reset() {
    currentStepIndex.value = 0
    selectedClass.value = null
    isNewClass.value = false
    classInfo.value = null
    hpMethod.value = null
    subclass.value = null
    asiOrFeat.value = null
    spellChanges.value = null
    featureChoices.value = null
    error.value = null
    isSubmitting.value = false
  }

  return {
    // State (readonly where appropriate)
    currentStepIndex: readonly(currentStepIndex),
    isSubmitting: readonly(isSubmitting),
    error: readonly(error),
    selectedClass: readonly(selectedClass),
    isNewClass: readonly(isNewClass),
    classInfo: readonly(classInfo),

    // Choices (writable refs for steps to modify)
    hpMethod,
    subclass,
    asiOrFeat,
    spellChanges,
    featureChoices,

    // Computed
    context,
    newClassLevel,
    newTotalLevel,
    visibleSteps,
    currentStep,
    canGoBack,
    canGoForward,
    isLastStep,

    // Methods
    goToNextStep,
    goToPreviousStep,
    goToStep,
    selectClass,
    submit,
    reset
  }
}

export type LevelUpComposable = ReturnType<typeof useLevelUp>
