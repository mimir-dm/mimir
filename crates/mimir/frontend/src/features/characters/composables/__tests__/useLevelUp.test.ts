/**
 * Tests for useLevelUp composable.
 *
 * Tests wizard step visibility, navigation, class selection, ASI/subclass level
 * detection, request building, submission, and reset.
 */

import { describe, it, expect, beforeEach, vi } from 'vitest'
import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import {
  useLevelUp,
  isAtSubclassLevel,
  isAtAsiLevel,
  gainsSpellsAtLevel,
  hasFeatureChoices,
  type ClassInfo,
  type LevelUpContext,
} from '../useLevelUp'
import type { Character, CharacterClass } from '@/types/character'

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}))

const mockInvoke = vi.mocked(invoke)

// --- Factories ---

function makeCharacterClass(overrides: Partial<CharacterClass> = {}): CharacterClass {
  return {
    id: 'cc-1',
    character_id: 'char-1',
    class_name: 'Fighter',
    class_source: 'PHB',
    level: 3,
    subclass_name: null,
    subclass_source: null,
    starting_class: 1,
    ...overrides,
  }
}

function makeCharacter(overrides: Partial<Character> = {}): Character {
  return {
    id: 'char-1',
    campaign_id: 'camp-1',
    name: 'Test Hero',
    is_npc: 0,
    player_name: 'Player',
    race_name: 'Human',
    race_source: 'PHB',
    background_name: 'Soldier',
    background_source: 'PHB',
    strength: 16,
    dexterity: 14,
    constitution: 14,
    intelligence: 10,
    wisdom: 12,
    charisma: 8,
    cp: 0, sp: 0, ep: 0, gp: 50, pp: 0,
    traits: null, ideals: null, bonds: null, flaws: null,
    role: null, location: null, faction: null,
    created_at: '2024-01-01',
    updated_at: '2024-01-01',
    classes: [makeCharacterClass()],
    proficiencies: [],
    ...overrides,
  } as Character
}

function makeClassInfo(overrides: Partial<ClassInfo> = {}): ClassInfo {
  return {
    name: 'Fighter',
    source: 'PHB',
    hit_die: 10,
    subclass_level: 3,
    asi_levels: [4, 6, 8, 12, 14, 16, 19],
    multiclass_prereqs: 'Strength 13',
    spellcasting_ability: null,
    caster_type: null,
    ...overrides,
  }
}

function makeContext(overrides: Partial<LevelUpContext> = {}): LevelUpContext {
  return {
    character: makeCharacter(),
    selectedClass: makeCharacterClass(),
    isNewClass: false,
    newClassLevel: 4,
    newTotalLevel: 4,
    classInfo: makeClassInfo(),
    hpMethod: null,
    subclass: null,
    asiOrFeat: null,
    spellChanges: null,
    featureChoices: null,
    ...overrides,
  }
}

// --- Tests ---

describe('step visibility functions', () => {
  describe('isAtSubclassLevel', () => {
    it('returns true when at subclass level without subclass', () => {
      const ctx = makeContext({
        newClassLevel: 3,
        classInfo: makeClassInfo({ subclass_level: 3 }),
        selectedClass: makeCharacterClass({ subclass_name: null }),
      })
      expect(isAtSubclassLevel(ctx)).toBe(true)
    })

    it('returns false when already has subclass', () => {
      const ctx = makeContext({
        newClassLevel: 3,
        classInfo: makeClassInfo({ subclass_level: 3 }),
        selectedClass: makeCharacterClass({ subclass_name: 'Champion' }),
      })
      expect(isAtSubclassLevel(ctx)).toBe(false)
    })

    it('returns false when not at subclass level', () => {
      const ctx = makeContext({
        newClassLevel: 4,
        classInfo: makeClassInfo({ subclass_level: 3 }),
      })
      expect(isAtSubclassLevel(ctx)).toBe(false)
    })

    it('returns false without classInfo', () => {
      const ctx = makeContext({ classInfo: null })
      expect(isAtSubclassLevel(ctx)).toBe(false)
    })

    it('handles Wizard subclass at level 2', () => {
      const ctx = makeContext({
        newClassLevel: 2,
        classInfo: makeClassInfo({ name: 'Wizard', subclass_level: 2 }),
        selectedClass: makeCharacterClass({ class_name: 'Wizard', subclass_name: null }),
      })
      expect(isAtSubclassLevel(ctx)).toBe(true)
    })
  })

  describe('isAtAsiLevel', () => {
    it('returns true at standard ASI levels', () => {
      const info = makeClassInfo({ asi_levels: [4, 8, 12, 16, 19] })
      for (const level of [4, 8, 12, 16, 19]) {
        const ctx = makeContext({ newClassLevel: level, classInfo: info })
        expect(isAtAsiLevel(ctx)).toBe(true)
      }
    })

    it('returns false at non-ASI levels', () => {
      const info = makeClassInfo({ asi_levels: [4, 8, 12, 16, 19] })
      for (const level of [1, 2, 3, 5, 6, 7]) {
        const ctx = makeContext({ newClassLevel: level, classInfo: info })
        expect(isAtAsiLevel(ctx)).toBe(false)
      }
    })

    it('handles Fighter extra ASI levels (6, 14)', () => {
      const info = makeClassInfo({ asi_levels: [4, 6, 8, 12, 14, 16, 19] })
      expect(isAtAsiLevel(makeContext({ newClassLevel: 6, classInfo: info }))).toBe(true)
      expect(isAtAsiLevel(makeContext({ newClassLevel: 14, classInfo: info }))).toBe(true)
    })

    it('returns false without classInfo', () => {
      expect(isAtAsiLevel(makeContext({ classInfo: null }))).toBe(false)
    })
  })

  describe('gainsSpellsAtLevel', () => {
    it('returns true for spellcasting classes', () => {
      const ctx = makeContext({
        classInfo: makeClassInfo({ spellcasting_ability: 'Intelligence' }),
      })
      expect(gainsSpellsAtLevel(ctx)).toBe(true)
    })

    it('returns false for non-casters', () => {
      const ctx = makeContext({
        classInfo: makeClassInfo({ spellcasting_ability: null }),
      })
      expect(gainsSpellsAtLevel(ctx)).toBe(false)
    })

    it('returns false without classInfo', () => {
      expect(gainsSpellsAtLevel(makeContext({ classInfo: null }))).toBe(false)
    })
  })

  describe('hasFeatureChoices', () => {
    it('Fighter 1 gets Fighting Style', () => {
      const ctx = makeContext({
        newClassLevel: 1,
        selectedClass: makeCharacterClass({ class_name: 'Fighter' }),
        classInfo: makeClassInfo(),
      })
      expect(hasFeatureChoices(ctx)).toBe(true)
    })

    it('Paladin 2 gets Fighting Style', () => {
      const ctx = makeContext({
        newClassLevel: 2,
        selectedClass: makeCharacterClass({ class_name: 'Paladin' }),
        classInfo: makeClassInfo({ name: 'Paladin' }),
      })
      expect(hasFeatureChoices(ctx)).toBe(true)
    })

    it('Sorcerer 3 gets Metamagic', () => {
      const ctx = makeContext({
        newClassLevel: 3,
        selectedClass: makeCharacterClass({ class_name: 'Sorcerer' }),
        classInfo: makeClassInfo({ name: 'Sorcerer' }),
      })
      expect(hasFeatureChoices(ctx)).toBe(true)
    })

    it('Warlock 2 gets Invocations', () => {
      const ctx = makeContext({
        newClassLevel: 2,
        selectedClass: makeCharacterClass({ class_name: 'Warlock' }),
        classInfo: makeClassInfo({ name: 'Warlock' }),
      })
      expect(hasFeatureChoices(ctx)).toBe(true)
    })

    it('Rogue 1 gets Expertise', () => {
      const ctx = makeContext({
        newClassLevel: 1,
        selectedClass: makeCharacterClass({ class_name: 'Rogue' }),
        classInfo: makeClassInfo({ name: 'Rogue' }),
      })
      expect(hasFeatureChoices(ctx)).toBe(true)
    })

    it('Bard 3 gets Expertise', () => {
      const ctx = makeContext({
        newClassLevel: 3,
        selectedClass: makeCharacterClass({ class_name: 'Bard' }),
        classInfo: makeClassInfo({ name: 'Bard' }),
      })
      expect(hasFeatureChoices(ctx)).toBe(true)
    })

    it('Battle Master 3 gets Maneuvers', () => {
      const ctx = makeContext({
        newClassLevel: 3,
        selectedClass: makeCharacterClass({
          class_name: 'Fighter',
          subclass_name: 'Battle Master',
        }),
        classInfo: makeClassInfo(),
      })
      expect(hasFeatureChoices(ctx)).toBe(true)
    })

    it('Fighter 2 has no feature choices', () => {
      const ctx = makeContext({
        newClassLevel: 2,
        selectedClass: makeCharacterClass({ class_name: 'Fighter' }),
        classInfo: makeClassInfo(),
      })
      expect(hasFeatureChoices(ctx)).toBe(false)
    })

    it('Wizard never has feature choices', () => {
      const ctx = makeContext({
        newClassLevel: 5,
        selectedClass: makeCharacterClass({ class_name: 'Wizard' }),
        classInfo: makeClassInfo({ name: 'Wizard' }),
      })
      expect(hasFeatureChoices(ctx)).toBe(false)
    })
  })
})

describe('useLevelUp', () => {
  beforeEach(() => {
    vi.clearAllMocks()
  })

  function createLevelUp(charOverrides: Partial<Character> = {}) {
    const character = ref(makeCharacter(charOverrides))
    return { levelUp: useLevelUp(character), character }
  }

  describe('initial state', () => {
    it('starts at step 0 with no selections', () => {
      const { levelUp } = createLevelUp()
      expect(levelUp.currentStepIndex.value).toBe(0)
      expect(levelUp.selectedClass.value).toBeNull()
      expect(levelUp.isNewClass.value).toBe(false)
      expect(levelUp.hpMethod.value).toBeNull()
      expect(levelUp.subclass.value).toBeNull()
      expect(levelUp.asiOrFeat.value).toBeNull()
      expect(levelUp.error.value).toBeNull()
    })

    it('calculates new total level', () => {
      const { levelUp } = createLevelUp({
        classes: [makeCharacterClass({ level: 5 })],
      })
      expect(levelUp.newTotalLevel.value).toBe(6)
    })

    it('calculates multiclass total level', () => {
      const { levelUp } = createLevelUp({
        classes: [
          makeCharacterClass({ class_name: 'Fighter', level: 5 }),
          makeCharacterClass({ id: 'cc-2', class_name: 'Rogue', level: 3 }),
        ],
      })
      expect(levelUp.newTotalLevel.value).toBe(9)
    })
  })

  describe('visible steps', () => {
    it('always shows class, hp, summary, and review', () => {
      const { levelUp } = createLevelUp()
      const stepIds = levelUp.visibleSteps.value.map(s => s.id)
      expect(stepIds).toContain('class')
      expect(stepIds).toContain('hp')
      expect(stepIds).toContain('featuresDisplay')
      expect(stepIds).toContain('review')
    })

    it('does not show subclass/asi/spells/features by default', () => {
      const { levelUp } = createLevelUp()
      // No class selected, so context-dependent steps are hidden
      const stepIds = levelUp.visibleSteps.value.map(s => s.id)
      expect(stepIds).not.toContain('subclass')
      expect(stepIds).not.toContain('asi')
      expect(stepIds).not.toContain('spells')
      expect(stepIds).not.toContain('features')
    })
  })

  describe('class selection', () => {
    it('selects existing class and fetches class info', async () => {
      mockInvoke.mockResolvedValueOnce({
        success: true,
        data: makeClassInfo(),
      })

      const { levelUp } = createLevelUp()
      const charClass = makeCharacterClass({ level: 3 })
      await levelUp.selectClass(charClass, false)

      expect(levelUp.selectedClass.value).toEqual(charClass)
      expect(levelUp.isNewClass.value).toBe(false)
      expect(levelUp.classInfo.value).not.toBeNull()
      expect(levelUp.newClassLevel.value).toBe(4) // level 3 + 1
    })

    it('selects new multiclass at level 1', async () => {
      mockInvoke.mockResolvedValueOnce({
        success: true,
        data: makeClassInfo({ name: 'Wizard' }),
      })

      const { levelUp } = createLevelUp()
      const charClass = makeCharacterClass({ class_name: 'Wizard', level: 0 })
      await levelUp.selectClass(charClass, true)

      expect(levelUp.isNewClass.value).toBe(true)
      expect(levelUp.newClassLevel.value).toBe(1)
    })

    it('uses defaults when class info fetch fails', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('network error'))

      const { levelUp } = createLevelUp()
      await levelUp.selectClass(makeCharacterClass(), false)

      expect(levelUp.classInfo.value).not.toBeNull()
      expect(levelUp.classInfo.value!.hit_die).toBe(8)
      expect(levelUp.classInfo.value!.asi_levels).toEqual([4, 8, 12, 16, 19])
    })
  })

  describe('navigation', () => {
    it('canGoBack is false at first step', () => {
      const { levelUp } = createLevelUp()
      expect(levelUp.canGoBack.value).toBe(false)
    })

    it('canGoForward requires current step to be complete', () => {
      const { levelUp } = createLevelUp()
      // Step 0 is "class" which requires selectedClass !== null
      expect(levelUp.canGoForward.value).toBe(false)
    })

    it('goToNextStep advances index', async () => {
      mockInvoke.mockResolvedValueOnce({
        success: true,
        data: makeClassInfo(),
      })

      const { levelUp } = createLevelUp()
      await levelUp.selectClass(makeCharacterClass(), false)

      // Class step is now complete, can go forward
      expect(levelUp.canGoForward.value).toBe(true)
      levelUp.goToNextStep()
      expect(levelUp.currentStepIndex.value).toBe(1)
      expect(levelUp.canGoBack.value).toBe(true)
    })

    it('goToPreviousStep decrements index', async () => {
      mockInvoke.mockResolvedValueOnce({
        success: true,
        data: makeClassInfo(),
      })

      const { levelUp } = createLevelUp()
      await levelUp.selectClass(makeCharacterClass(), false)
      levelUp.goToNextStep()
      expect(levelUp.currentStepIndex.value).toBe(1)

      levelUp.goToPreviousStep()
      expect(levelUp.currentStepIndex.value).toBe(0)
    })

    it('goToStep navigates to specific step', () => {
      const { levelUp } = createLevelUp()
      levelUp.goToStep(2)
      expect(levelUp.currentStepIndex.value).toBe(2)
    })

    it('goToStep ignores out-of-bounds index', () => {
      const { levelUp } = createLevelUp()
      levelUp.goToStep(100)
      expect(levelUp.currentStepIndex.value).toBe(0)
    })
  })

  describe('buildRequest / submit', () => {
    it('returns null when required fields missing', async () => {
      const { levelUp } = createLevelUp()
      // submit without selectClass or hpMethod
      const result = await levelUp.submit()
      expect(result).toBeNull()
      expect(levelUp.error.value).toBe('Please complete all required steps')
    })

    it('builds minimal request with class and hp', async () => {
      mockInvoke
        .mockResolvedValueOnce({ success: true, data: makeClassInfo() }) // selectClass
        .mockResolvedValueOnce({ success: true, data: { character: makeCharacter(), class: makeCharacterClass(), hp_gained: 6, new_total_level: 4, is_multiclass: false } }) // submit

      const { levelUp } = createLevelUp()
      await levelUp.selectClass(makeCharacterClass(), false)
      levelUp.hpMethod.value = { type: 'Average' }

      const result = await levelUp.submit()
      expect(result).not.toBeNull()
      expect(result!.hp_gained).toBe(6)

      // Verify invoke was called with correct request shape
      expect(mockInvoke).toHaveBeenCalledWith('level_up_character', expect.objectContaining({
        characterId: 'char-1',
        request: expect.objectContaining({
          class_name: 'Fighter',
          class_source: 'PHB',
          hit_points_method: { type: 'Average' },
        }),
      }))
    })

    it('includes optional fields when set', async () => {
      mockInvoke
        .mockResolvedValueOnce({ success: true, data: makeClassInfo() })
        .mockResolvedValueOnce({ success: true, data: { character: makeCharacter(), class: makeCharacterClass(), hp_gained: 8, new_total_level: 4, is_multiclass: false } })

      const { levelUp } = createLevelUp()
      await levelUp.selectClass(makeCharacterClass(), false)
      levelUp.hpMethod.value = { type: 'Roll', value: 8 }
      levelUp.subclass.value = { name: 'Champion', source: 'PHB' }
      levelUp.asiOrFeat.value = {
        type: 'AbilityScoreImprovement',
        ability1: 'Strength',
        increase1: 1,
        ability2: 'Constitution',
        increase2: 1,
      }

      await levelUp.submit()

      expect(mockInvoke).toHaveBeenCalledWith('level_up_character', expect.objectContaining({
        request: expect.objectContaining({
          subclass: { name: 'Champion', source: 'PHB' },
          asi_or_feat: expect.objectContaining({ type: 'AbilityScoreImprovement' }),
        }),
      }))
    })

    it('includes feat selection', async () => {
      mockInvoke
        .mockResolvedValueOnce({ success: true, data: makeClassInfo() })
        .mockResolvedValueOnce({ success: true, data: { character: makeCharacter(), class: makeCharacterClass(), hp_gained: 6, new_total_level: 4, is_multiclass: false } })

      const { levelUp } = createLevelUp()
      await levelUp.selectClass(makeCharacterClass(), false)
      levelUp.hpMethod.value = { type: 'Average' }
      levelUp.asiOrFeat.value = {
        type: 'Feat',
        name: 'Great Weapon Master',
        source: 'PHB',
      }

      await levelUp.submit()

      expect(mockInvoke).toHaveBeenCalledWith('level_up_character', expect.objectContaining({
        request: expect.objectContaining({
          asi_or_feat: { type: 'Feat', name: 'Great Weapon Master', source: 'PHB' },
        }),
      }))
    })

    it('handles submit error from backend', async () => {
      mockInvoke
        .mockResolvedValueOnce({ success: true, data: makeClassInfo() })
        .mockResolvedValueOnce({ success: false, error: 'Multiclass prerequisites not met' })

      const { levelUp } = createLevelUp()
      await levelUp.selectClass(makeCharacterClass(), false)
      levelUp.hpMethod.value = { type: 'Average' }

      const result = await levelUp.submit()
      expect(result).toBeNull()
      expect(levelUp.error.value).toBe('Multiclass prerequisites not met')
    })

    it('handles thrown error on submit', async () => {
      mockInvoke
        .mockResolvedValueOnce({ success: true, data: makeClassInfo() })
        .mockRejectedValueOnce(new Error('Network error'))

      const { levelUp } = createLevelUp()
      await levelUp.selectClass(makeCharacterClass(), false)
      levelUp.hpMethod.value = { type: 'Average' }

      const result = await levelUp.submit()
      expect(result).toBeNull()
      expect(levelUp.error.value).toBe('Network error')
    })
  })

  describe('reset', () => {
    it('clears all state', async () => {
      mockInvoke.mockResolvedValueOnce({ success: true, data: makeClassInfo() })

      const { levelUp } = createLevelUp()
      await levelUp.selectClass(makeCharacterClass(), false)
      levelUp.hpMethod.value = { type: 'Average' }
      levelUp.goToNextStep()

      levelUp.reset()

      expect(levelUp.currentStepIndex.value).toBe(0)
      expect(levelUp.selectedClass.value).toBeNull()
      expect(levelUp.isNewClass.value).toBe(false)
      expect(levelUp.classInfo.value).toBeNull()
      expect(levelUp.hpMethod.value).toBeNull()
      expect(levelUp.subclass.value).toBeNull()
      expect(levelUp.asiOrFeat.value).toBeNull()
      expect(levelUp.spellChanges.value).toBeNull()
      expect(levelUp.featureChoices.value).toBeNull()
      expect(levelUp.error.value).toBeNull()
    })
  })

  describe('HP method types', () => {
    it('accepts Average method', () => {
      const { levelUp } = createLevelUp()
      levelUp.hpMethod.value = { type: 'Average' }
      expect(levelUp.hpMethod.value.type).toBe('Average')
    })

    it('accepts Roll method with value', () => {
      const { levelUp } = createLevelUp()
      levelUp.hpMethod.value = { type: 'Roll', value: 7 }
      expect(levelUp.hpMethod.value).toEqual({ type: 'Roll', value: 7 })
    })

    it('accepts Manual method with value', () => {
      const { levelUp } = createLevelUp()
      levelUp.hpMethod.value = { type: 'Manual', value: 10 }
      expect(levelUp.hpMethod.value).toEqual({ type: 'Manual', value: 10 })
    })
  })
})
