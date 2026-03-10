/**
 * Tests for useSpellManagement composable.
 *
 * Tests spell slot calculation (single/multiclass), spell grouping,
 * helper functions (school names, level display, components, etc.),
 * toggle behavior, and Warlock pact magic.
 */

import { describe, it, expect, beforeEach, vi } from 'vitest'
import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useSpellManagement, type SpellInfo } from '../useSpellManagement'
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
    class_name: 'Wizard',
    class_source: 'PHB',
    level: 5,
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
    name: 'Test Wizard',
    is_npc: 0,
    player_name: 'Player',
    race_name: 'Elf',
    race_source: 'PHB',
    background_name: 'Sage',
    background_source: 'PHB',
    strength: 8,
    dexterity: 14,
    constitution: 12,
    intelligence: 18,
    wisdom: 12,
    charisma: 10,
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

function makeSpell(overrides: Partial<SpellInfo> = {}): SpellInfo {
  return {
    name: 'Magic Missile',
    source: 'PHB',
    level: 1,
    school: 'V',
    ritual: false,
    concentration: false,
    data: {},
    ...overrides,
  }
}

function createSpellManagement(charOverrides: Partial<Character> = {}) {
  const character = ref<Character | null>(makeCharacter(charOverrides))
  const characterId = computed(() => character.value?.id ?? '')
  return { sm: useSpellManagement(character, characterId), character }
}

// --- Tests ---

describe('useSpellManagement', () => {
  beforeEach(() => {
    vi.clearAllMocks()
  })

  describe('initial state', () => {
    it('starts with empty spell list', () => {
      const { sm } = createSpellManagement()
      expect(sm.classSpells.value).toEqual([])
      expect(sm.loadingSpells.value).toBe(false)
    })

    it('all spell levels start collapsed', () => {
      const { sm } = createSpellManagement()
      for (let i = 0; i <= 9; i++) {
        expect(sm.isSpellLevelCollapsed(i)).toBe(true)
      }
    })
  })

  describe('spellcasting detection', () => {
    it('detects Wizard as spellcaster', () => {
      const { sm } = createSpellManagement()
      expect(sm.characterIsSpellcaster.value).toBe(true)
    })

    it('detects Fighter as non-spellcaster', () => {
      const { sm } = createSpellManagement({
        classes: [makeCharacterClass({ class_name: 'Fighter' })],
      })
      expect(sm.characterIsSpellcaster.value).toBe(false)
    })

    it('detects null character as non-spellcaster', () => {
      const character = ref<Character | null>(null)
      const characterId = computed(() => '')
      const sm = useSpellManagement(character, characterId)
      expect(sm.characterIsSpellcaster.value).toBe(false)
    })

    it('returns Intelligence as Wizard spellcasting ability', () => {
      const { sm } = createSpellManagement()
      expect(sm.spellcastingAbility.value).toBe('intelligence')
    })
  })

  describe('spell slots (single class)', () => {
    it('calculates slots for Wizard 5', () => {
      const { sm } = createSpellManagement({
        classes: [makeCharacterClass({ class_name: 'Wizard', level: 5 })],
      })
      const slots = sm.spellSlots.value
      expect(slots).not.toBeNull()
      expect(slots![1]).toBe(4)  // 4 first-level slots
      expect(slots![2]).toBe(3)  // 3 second-level slots
      expect(slots![3]).toBe(2)  // 2 third-level slots
    })

    it('returns null for non-casters', () => {
      const { sm } = createSpellManagement({
        classes: [makeCharacterClass({ class_name: 'Fighter', level: 5 })],
      })
      expect(sm.spellSlots.value).toBeNull()
    })
  })

  describe('spell slots (Warlock pact magic)', () => {
    it('calculates Warlock pact slots', () => {
      const { sm } = createSpellManagement({
        classes: [makeCharacterClass({ class_name: 'Warlock', level: 5 })],
      })
      const slots = sm.spellSlots.value
      expect(slots).not.toBeNull()
      // Warlock 5: 2 slots at level 3
      expect(slots![3]).toBe(2)
    })
  })

  describe('spell slots (multiclass)', () => {
    it('combines multiclass caster levels', () => {
      const { sm } = createSpellManagement({
        classes: [
          makeCharacterClass({ id: 'cc-1', class_name: 'Wizard', level: 3 }),
          makeCharacterClass({ id: 'cc-2', class_name: 'Cleric', level: 2 }),
        ],
      })
      // Wizard 3 + Cleric 2 = caster level 5
      const slots = sm.spellSlots.value
      expect(slots).not.toBeNull()
      expect(slots![3]).toBe(2) // 5th-level caster gets 3rd-level slots
    })

    it('multiclass half caster adds half levels', () => {
      const { sm } = createSpellManagement({
        classes: [
          makeCharacterClass({ id: 'cc-1', class_name: 'Wizard', level: 3 }),
          makeCharacterClass({ id: 'cc-2', class_name: 'Paladin', level: 4 }),
        ],
      })
      // Wizard 3 (full) + Paladin 4 (half = 2) = caster level 5
      const slots = sm.spellSlots.value
      expect(slots).not.toBeNull()
      expect(slots![1]).toBeGreaterThan(0)
    })
  })

  describe('multiclass spellcaster detection', () => {
    it('detects single class as non-multiclass', () => {
      const { sm } = createSpellManagement({
        classes: [makeCharacterClass({ class_name: 'Wizard', level: 5 })],
      })
      expect(sm.isMulticlassSpellcaster.value).toBe(false)
    })

    it('detects two spellcasting classes as multiclass', () => {
      const { sm } = createSpellManagement({
        classes: [
          makeCharacterClass({ id: 'cc-1', class_name: 'Wizard', level: 3 }),
          makeCharacterClass({ id: 'cc-2', class_name: 'Cleric', level: 2 }),
        ],
      })
      expect(sm.isMulticlassSpellcaster.value).toBe(true)
    })

    it('Fighter/Wizard is not multiclass spellcaster (only one caster)', () => {
      const { sm } = createSpellManagement({
        classes: [
          makeCharacterClass({ id: 'cc-1', class_name: 'Fighter', level: 5 }),
          makeCharacterClass({ id: 'cc-2', class_name: 'Wizard', level: 3 }),
        ],
      })
      expect(sm.isMulticlassSpellcaster.value).toBe(false)
    })
  })

  describe('spellsByLevel grouping', () => {
    it('groups spells by level', () => {
      const { sm } = createSpellManagement()
      sm.classSpells.value = [
        makeSpell({ name: 'Fire Bolt', level: 0 }),
        makeSpell({ name: 'Magic Missile', level: 1 }),
        makeSpell({ name: 'Shield', level: 1 }),
        makeSpell({ name: 'Fireball', level: 3 }),
      ]

      const grouped = sm.spellsByLevel.value
      expect(grouped[0]).toHaveLength(1)
      expect(grouped[1]).toHaveLength(2)
      expect(grouped[3]).toHaveLength(1)
      expect(grouped[2]).toBeUndefined()
    })
  })

  describe('helper functions', () => {
    it('getSchoolName maps codes to names', () => {
      const { sm } = createSpellManagement()
      expect(sm.getSchoolName('V')).toBe('Evocation')
      expect(sm.getSchoolName('A')).toBe('Abjuration')
      expect(sm.getSchoolName('N')).toBe('Necromancy')
      expect(sm.getSchoolName(null)).toBe('Unknown')
    })

    it('getLevelDisplay formats spell levels', () => {
      const { sm } = createSpellManagement()
      expect(sm.getLevelDisplay(0)).toBe('Cantrip')
      expect(sm.getLevelDisplay(1)).toBe('1st Level')
      expect(sm.getLevelDisplay(2)).toBe('2nd Level')
      expect(sm.getLevelDisplay(3)).toBe('3rd Level')
      expect(sm.getLevelDisplay(4)).toBe('4th Level')
      expect(sm.getLevelDisplay(9)).toBe('9th Level')
    })

    it('getSpellCastingTime extracts from data', () => {
      const { sm } = createSpellManagement()
      const spell = makeSpell({
        data: { time: [{ number: 1, unit: 'action' }] },
      })
      expect(sm.getSpellCastingTime(spell)).toBe('1 action')
    })

    it('getSpellCastingTime returns Unknown for missing data', () => {
      const { sm } = createSpellManagement()
      expect(sm.getSpellCastingTime(makeSpell())).toBe('Unknown')
    })

    it('getSpellRange handles point/self/touch', () => {
      const { sm } = createSpellManagement()
      expect(sm.getSpellRange(makeSpell({
        data: { range: { type: 'point', distance: { type: 'self' } } },
      }))).toBe('Self')

      expect(sm.getSpellRange(makeSpell({
        data: { range: { type: 'point', distance: { type: 'touch' } } },
      }))).toBe('Touch')

      expect(sm.getSpellRange(makeSpell({
        data: { range: { type: 'point', distance: { type: 'feet', amount: 120 } } },
      }))).toBe('120 feet')
    })

    it('getSpellComponents formats V, S, M', () => {
      const { sm } = createSpellManagement()
      expect(sm.getSpellComponents(makeSpell({
        data: { components: { v: true, s: true, m: 'a bit of fleece' } },
      }))).toBe('V, S, M')

      expect(sm.getSpellComponents(makeSpell({
        data: { components: { v: true } },
      }))).toBe('V')
    })

    it('getSpellDuration handles instant and concentration', () => {
      const { sm } = createSpellManagement()
      expect(sm.getSpellDuration(makeSpell({
        data: { duration: [{ type: 'instant' }] },
      }))).toBe('Instantaneous')

      expect(sm.getSpellDuration(makeSpell({
        data: { duration: [{ type: 'timed', concentration: true, duration: { type: 'minute', amount: 1 } }] },
      }))).toBe('Concentration, 1 minute')
    })
  })

  describe('toggle behavior', () => {
    it('toggleSpellDetails adds and removes spell key', () => {
      const { sm } = createSpellManagement()
      expect(sm.isSpellExpanded('Magic Missile', 'PHB')).toBe(false)

      sm.toggleSpellDetails('Magic Missile', 'PHB')
      expect(sm.isSpellExpanded('Magic Missile', 'PHB')).toBe(true)

      sm.toggleSpellDetails('Magic Missile', 'PHB')
      expect(sm.isSpellExpanded('Magic Missile', 'PHB')).toBe(false)
    })

    it('toggleSpellLevel collapses and expands', () => {
      const { sm } = createSpellManagement()
      // All start collapsed
      expect(sm.isSpellLevelCollapsed(1)).toBe(true)

      sm.toggleSpellLevel(1)
      expect(sm.isSpellLevelCollapsed(1)).toBe(false)

      sm.toggleSpellLevel(1)
      expect(sm.isSpellLevelCollapsed(1)).toBe(true)
    })
  })

  describe('loadClassSpells', () => {
    it('fetches spells for spellcasting classes', async () => {
      mockInvoke
        .mockResolvedValueOnce({ success: true, data: ['PHB'] })  // list_character_sources
        .mockResolvedValueOnce({  // get_spells_by_class
          success: true,
          data: [
            { name: 'Fire Bolt', source: 'PHB', level: 0, school: 'V', ritual: 0, concentration: 0 },
            { name: 'Magic Missile', source: 'PHB', level: 1, school: 'V', ritual: 0, concentration: 0 },
          ],
        })

      const { sm } = createSpellManagement()
      await sm.loadClassSpells()

      expect(sm.classSpells.value).toHaveLength(2)
      expect(sm.classSpells.value[0].name).toBe('Fire Bolt')
      expect(sm.classSpells.value[1].name).toBe('Magic Missile')
    })

    it('skips non-spellcasting classes', async () => {
      const { sm } = createSpellManagement({
        classes: [makeCharacterClass({ class_name: 'Fighter' })],
      })
      await sm.loadClassSpells()
      expect(mockInvoke).not.toHaveBeenCalled()
    })

    it('handles fetch errors gracefully', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Network error'))

      const { sm } = createSpellManagement()
      await sm.loadClassSpells()

      expect(sm.classSpells.value).toEqual([])
      expect(sm.loadingSpells.value).toBe(false)
    })

    it('filters spells by source when sources configured', async () => {
      mockInvoke
        .mockResolvedValueOnce({ success: true, data: ['PHB'] })  // allowed sources
        .mockResolvedValueOnce({
          success: true,
          data: [
            { name: 'Magic Missile', source: 'PHB', level: 1, school: 'V', ritual: 0, concentration: 0 },
            { name: 'Ice Knife', source: 'XGE', level: 1, school: 'C', ritual: 0, concentration: 0 },
          ],
        })

      const { sm } = createSpellManagement()
      await sm.loadClassSpells()

      // XGE spell should be filtered out
      expect(sm.classSpells.value).toHaveLength(1)
      expect(sm.classSpells.value[0].name).toBe('Magic Missile')
    })

    it('deduplicates spells from multiple classes', async () => {
      const char = makeCharacter({
        classes: [
          makeCharacterClass({ id: 'cc-1', class_name: 'Wizard', level: 3 }),
          makeCharacterClass({ id: 'cc-2', class_name: 'Cleric', level: 2 }),
        ],
      })

      mockInvoke
        .mockResolvedValueOnce({ success: true, data: [] })  // sources (empty = show all)
        .mockResolvedValueOnce({  // Wizard spells
          success: true,
          data: [
            { name: 'Detect Magic', source: 'PHB', level: 1, school: 'D', ritual: 1, concentration: 1 },
          ],
        })
        .mockResolvedValueOnce({  // Cleric spells
          success: true,
          data: [
            { name: 'Detect Magic', source: 'PHB', level: 1, school: 'D', ritual: 1, concentration: 1 },
            { name: 'Cure Wounds', source: 'PHB', level: 1, school: 'V', ritual: 0, concentration: 0 },
          ],
        })

      const character = ref<Character | null>(char)
      const characterId = computed(() => character.value?.id ?? '')
      const sm = useSpellManagement(character, characterId)
      await sm.loadClassSpells()

      // Detect Magic appears in both lists but should only show once
      expect(sm.classSpells.value).toHaveLength(2)
    })
  })
})
