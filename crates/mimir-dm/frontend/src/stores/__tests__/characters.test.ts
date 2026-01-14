import { describe, it, expect, beforeEach, vi } from 'vitest'
import { setActivePinia, createPinia } from 'pinia'
import { useCharacterStore } from '../characters'
import { invoke } from '@tauri-apps/api/core'
import type { Character, CharacterData, CharacterWithData, CharacterVersion } from '../../types/character'

const mockInvoke = vi.mocked(invoke)

// Helper factories
const createMockCharacter = (overrides: Partial<Character> = {}): Character => ({
  id: 1,
  campaign_id: 1,
  player_id: null,
  character_type: 'PC',
  created_at: '2024-01-01T00:00:00Z',
  updated_at: '2024-01-01T00:00:00Z',
  ...overrides
})

const createMockCharacterData = (overrides: Partial<CharacterData> = {}): CharacterData => ({
  character_name: 'Test Character',
  level: 5,
  class_name: 'Fighter',
  race: 'Human',
  background: 'Soldier',
  alignment: 'Lawful Good',
  experience_points: 6500,
  hit_points: { current: 45, maximum: 45, temporary: 0 },
  ability_scores: {
    strength: 16,
    dexterity: 14,
    constitution: 14,
    intelligence: 10,
    wisdom: 12,
    charisma: 8
  },
  proficiencies: { skills: [], tools: [], languages: [], armor: [], weapons: [] },
  inventory: [],
  currency: { copper: 0, silver: 0, electrum: 0, gold: 50, platinum: 0 },
  equipped: { armor: null, shield: null, main_hand: null, off_hand: null },
  spells: { cantrips_known: [], spells_known: [], prepared_spells: [], spell_slots: {} },
  features: [],
  notes: '',
  ...overrides
})

const createMockCharacterVersion = (overrides: Partial<CharacterVersion> = {}): CharacterVersion => ({
  id: 1,
  character_id: 1,
  version_number: 1,
  reason: 'Initial creation',
  data: createMockCharacterData(),
  created_at: '2024-01-01T00:00:00Z',
  ...overrides
})

describe('useCharacterStore', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.clearAllMocks()
  })

  describe('initial state', () => {
    it('has empty characters array', () => {
      const store = useCharacterStore()
      expect(store.characters).toEqual([])
    })

    it('has null currentCharacter', () => {
      const store = useCharacterStore()
      expect(store.currentCharacter).toBeNull()
    })

    it('has empty characterVersions array', () => {
      const store = useCharacterStore()
      expect(store.characterVersions).toEqual([])
    })

    it('is not loading initially', () => {
      const store = useCharacterStore()
      expect(store.loading).toBe(false)
    })

    it('has no error initially', () => {
      const store = useCharacterStore()
      expect(store.error).toBeNull()
    })
  })

  describe('computed properties', () => {
    describe('characterCount', () => {
      it('returns 0 for empty list', () => {
        const store = useCharacterStore()
        expect(store.characterCount).toBe(0)
      })

      it('returns correct count', () => {
        const store = useCharacterStore()
        store.characters = [
          createMockCharacter({ id: 1 }),
          createMockCharacter({ id: 2 }),
          createMockCharacter({ id: 3 })
        ]
        expect(store.characterCount).toBe(3)
      })
    })

    describe('getCharacterById', () => {
      it('returns character when found', () => {
        const store = useCharacterStore()
        const character = createMockCharacter({ id: 42 })
        store.characters = [character]

        expect(store.getCharacterById(42)).toEqual(character)
      })

      it('returns undefined when not found', () => {
        const store = useCharacterStore()
        store.characters = [createMockCharacter({ id: 1 })]

        expect(store.getCharacterById(999)).toBeUndefined()
      })
    })

    describe('currentCharacterLevel', () => {
      it('returns 0 when no current character', () => {
        const store = useCharacterStore()
        expect(store.currentCharacterLevel).toBe(0)
      })

      it('returns character level', () => {
        const store = useCharacterStore()
        store.currentCharacter = {
          character: createMockCharacter(),
          data: createMockCharacterData({ level: 10 })
        }
        expect(store.currentCharacterLevel).toBe(10)
      })
    })

    describe('currentCharacterProficiencyBonus', () => {
      it.each([
        [1, 2], [4, 2],
        [5, 3], [8, 3],
        [9, 4], [12, 4],
        [13, 5], [16, 5],
        [17, 6], [20, 6]
      ])('returns correct bonus for level %i', (level, expectedBonus) => {
        const store = useCharacterStore()
        store.currentCharacter = {
          character: createMockCharacter(),
          data: createMockCharacterData({ level })
        }
        expect(store.currentCharacterProficiencyBonus).toBe(expectedBonus)
      })
    })
  })

  describe('fetchAllCharacters', () => {
    it('fetches all characters', async () => {
      const mockCharacters = [
        createMockCharacter({ id: 1 }),
        createMockCharacter({ id: 2 })
      ]
      mockInvoke.mockResolvedValueOnce(mockCharacters)

      const store = useCharacterStore()
      const result = await store.fetchAllCharacters()

      expect(mockInvoke).toHaveBeenCalledWith('list_all_characters')
      expect(result).toEqual(mockCharacters)
      expect(store.characters).toEqual(mockCharacters)
    })

    it('sets loading state during fetch', async () => {
      mockInvoke.mockImplementation(() => new Promise(() => {}))

      const store = useCharacterStore()
      const promise = store.fetchAllCharacters()

      expect(store.loading).toBe(true)
    })

    it('handles error', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Database error'))

      const store = useCharacterStore()

      await expect(store.fetchAllCharacters()).rejects.toThrow('Database error')
      expect(store.error).toBe('Database error')
      expect(store.loading).toBe(false)
    })
  })

  describe('fetchCharactersForCampaign', () => {
    it('fetches characters for specific campaign', async () => {
      const mockCharacters = [createMockCharacter({ campaign_id: 5 })]
      mockInvoke.mockResolvedValueOnce(mockCharacters)

      const store = useCharacterStore()
      const result = await store.fetchCharactersForCampaign(5)

      expect(mockInvoke).toHaveBeenCalledWith('list_characters_for_campaign', { campaignId: 5 })
      expect(result).toEqual(mockCharacters)
    })
  })

  describe('getCharacter', () => {
    it('fetches character with data', async () => {
      const character = createMockCharacter({ id: 1 })
      const data = createMockCharacterData()
      mockInvoke.mockResolvedValueOnce([character, data])

      const store = useCharacterStore()
      const result = await store.getCharacter(1)

      expect(mockInvoke).toHaveBeenCalledWith('get_character', { characterId: 1 })
      expect(result).toEqual({ character, data })
      expect(store.currentCharacter).toEqual({ character, data })
    })

    it('updates character in list if present', async () => {
      const oldCharacter = createMockCharacter({ id: 1 })
      const updatedCharacter = createMockCharacter({ id: 1, updated_at: '2024-02-01T00:00:00Z' })
      const data = createMockCharacterData()

      const store = useCharacterStore()
      store.characters = [oldCharacter]

      mockInvoke.mockResolvedValueOnce([updatedCharacter, data])
      await store.getCharacter(1)

      expect(store.characters[0]).toEqual(updatedCharacter)
    })

    it('adds character to list if not present', async () => {
      const character = createMockCharacter({ id: 99 })
      const data = createMockCharacterData()

      const store = useCharacterStore()
      store.characters = []

      mockInvoke.mockResolvedValueOnce([character, data])
      await store.getCharacter(99)

      expect(store.characters).toContainEqual(character)
    })
  })

  describe('createCharacter', () => {
    it('creates character with request', async () => {
      const newData = createMockCharacterData({ character_name: 'New Hero' })
      mockInvoke.mockResolvedValueOnce(newData)

      const store = useCharacterStore()
      const result = await store.createCharacter({
        character_name: 'New Hero',
        class_name: 'Wizard',
        race: 'Elf',
        level: 1
      })

      expect(mockInvoke).toHaveBeenCalledWith('create_character', {
        request: expect.objectContaining({ character_name: 'New Hero' })
      })
      expect(result).toEqual(newData)
    })

    it('handles creation error', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Validation failed'))

      const store = useCharacterStore()
      await expect(store.createCharacter({
        character_name: '',
        class_name: '',
        race: '',
        level: 0
      })).rejects.toThrow()

      expect(store.error).toBe('Validation failed')
    })
  })

  describe('updateCharacterHp', () => {
    it('updates HP and refreshes character', async () => {
      const version = createMockCharacterVersion()
      const character = createMockCharacter({ id: 1 })
      const data = createMockCharacterData({ hit_points: { current: 30, maximum: 45, temporary: 0 } })

      mockInvoke
        .mockResolvedValueOnce(version) // update_character_hp
        .mockResolvedValueOnce([character, data]) // get_character refresh

      const store = useCharacterStore()
      const result = await store.updateCharacterHp(1, 30, 'Took damage')

      expect(mockInvoke).toHaveBeenCalledWith('update_character_hp', {
        characterId: 1,
        newHp: 30,
        reason: 'Took damage'
      })
      expect(result).toEqual(version)
    })
  })

  describe('deleteCharacter', () => {
    it('removes character from list', async () => {
      mockInvoke.mockResolvedValueOnce(undefined)

      const store = useCharacterStore()
      store.characters = [
        createMockCharacter({ id: 1 }),
        createMockCharacter({ id: 2 })
      ]

      const result = await store.deleteCharacter(1)

      expect(mockInvoke).toHaveBeenCalledWith('delete_character', { characterId: 1 })
      expect(result).toBe(true)
      expect(store.characters).toHaveLength(1)
      expect(store.characters[0].id).toBe(2)
    })

    it('clears currentCharacter if deleted', async () => {
      mockInvoke.mockResolvedValueOnce(undefined)

      const store = useCharacterStore()
      const character = createMockCharacter({ id: 1 })
      store.currentCharacter = { character, data: createMockCharacterData() }
      store.characters = [character]

      await store.deleteCharacter(1)

      expect(store.currentCharacter).toBeNull()
    })
  })

  describe('getCharacterVersions', () => {
    it('fetches version history', async () => {
      const versions = [
        createMockCharacterVersion({ version_number: 1 }),
        createMockCharacterVersion({ version_number: 2 })
      ]
      mockInvoke.mockResolvedValueOnce(versions)

      const store = useCharacterStore()
      const result = await store.getCharacterVersions(1)

      expect(mockInvoke).toHaveBeenCalledWith('get_character_versions', { characterId: 1 })
      expect(result).toEqual(versions)
      expect(store.characterVersions).toEqual(versions)
    })
  })

  describe('getCharacterVersion', () => {
    it('fetches specific version data', async () => {
      const data = createMockCharacterData({ level: 3 })
      mockInvoke.mockResolvedValueOnce(data)

      const store = useCharacterStore()
      const result = await store.getCharacterVersion(1, 2)

      expect(mockInvoke).toHaveBeenCalledWith('get_character_version', {
        characterId: 1,
        versionNumber: 2
      })
      expect(result).toEqual(data)
    })
  })

  describe('setCurrentCharacter', () => {
    it('sets current character', () => {
      const store = useCharacterStore()
      const characterWithData: CharacterWithData = {
        character: createMockCharacter(),
        data: createMockCharacterData()
      }

      store.setCurrentCharacter(characterWithData)

      expect(store.currentCharacter).toEqual(characterWithData)
    })

    it('clears current character with null', () => {
      const store = useCharacterStore()
      store.currentCharacter = {
        character: createMockCharacter(),
        data: createMockCharacterData()
      }

      store.setCurrentCharacter(null)

      expect(store.currentCharacter).toBeNull()
    })
  })

  describe('reset', () => {
    it('resets all state', () => {
      const store = useCharacterStore()
      store.characters = [createMockCharacter()]
      store.currentCharacter = { character: createMockCharacter(), data: createMockCharacterData() }
      store.characterVersions = [createMockCharacterVersion()]
      store.loading = true
      store.error = 'Some error'

      store.reset()

      expect(store.characters).toEqual([])
      expect(store.currentCharacter).toBeNull()
      expect(store.characterVersions).toEqual([])
      expect(store.loading).toBe(false)
      expect(store.error).toBeNull()
    })
  })

  describe('spell operations', () => {
    describe('addSpellToKnown', () => {
      it('adds spell to character', async () => {
        const version = createMockCharacterVersion()
        const character = createMockCharacter()
        const data = createMockCharacterData()

        mockInvoke
          .mockResolvedValueOnce(version)
          .mockResolvedValueOnce([character, data])

        const store = useCharacterStore()
        const result = await store.addSpellToKnown(1, 'Fireball', 'PHB', false)

        expect(mockInvoke).toHaveBeenCalledWith('add_spell_to_known', {
          characterId: 1,
          spellName: 'Fireball',
          spellSource: 'PHB',
          isCantrip: false
        })
        expect(result).toEqual(version)
      })
    })

    describe('prepareSpells', () => {
      it('prepares spells for character', async () => {
        const version = createMockCharacterVersion()
        const character = createMockCharacter()
        const data = createMockCharacterData()

        mockInvoke
          .mockResolvedValueOnce(version)
          .mockResolvedValueOnce([character, data])

        const store = useCharacterStore()
        await store.prepareSpells(1, ['Fireball|PHB', 'Shield|PHB'], 'intelligence')

        expect(mockInvoke).toHaveBeenCalledWith('prepare_spells', {
          characterId: 1,
          spellKeys: ['Fireball|PHB', 'Shield|PHB'],
          spellcastingAbility: 'intelligence'
        })
      })
    })

    describe('castSpell', () => {
      it('casts spell for character', async () => {
        const version = createMockCharacterVersion()
        const character = createMockCharacter()
        const data = createMockCharacterData()

        mockInvoke
          .mockResolvedValueOnce(version)
          .mockResolvedValueOnce([character, data])

        const store = useCharacterStore()
        await store.castSpell(1, 'Fireball', 3, false)

        expect(mockInvoke).toHaveBeenCalledWith('cast_spell', {
          characterId: 1,
          spellName: 'Fireball',
          spellLevel: 3,
          isRitual: false
        })
      })
    })
  })

  describe('rest', () => {
    it('performs short rest', async () => {
      const version = createMockCharacterVersion()
      const character = createMockCharacter()
      const data = createMockCharacterData()

      mockInvoke
        .mockResolvedValueOnce(version)
        .mockResolvedValueOnce([character, data])

      const store = useCharacterStore()
      await store.rest(1, 'short')

      expect(mockInvoke).toHaveBeenCalledWith('rest_character', {
        characterId: 1,
        restType: 'short'
      })
    })

    it('performs long rest', async () => {
      const version = createMockCharacterVersion()
      const character = createMockCharacter()
      const data = createMockCharacterData()

      mockInvoke
        .mockResolvedValueOnce(version)
        .mockResolvedValueOnce([character, data])

      const store = useCharacterStore()
      await store.rest(1, 'long')

      expect(mockInvoke).toHaveBeenCalledWith('rest_character', {
        characterId: 1,
        restType: 'long'
      })
    })
  })

  describe('inventory operations', () => {
    describe('addItem', () => {
      it('adds item to inventory', async () => {
        const version = createMockCharacterVersion()
        const character = createMockCharacter()
        const data = createMockCharacterData()

        mockInvoke
          .mockResolvedValueOnce(version)
          .mockResolvedValueOnce([character, data])

        const store = useCharacterStore()
        await store.addItem(1, 'Longsword', 'PHB', 1, 'Found in dungeon')

        expect(mockInvoke).toHaveBeenCalledWith('add_item_to_inventory', {
          characterId: 1,
          itemName: 'Longsword',
          itemSource: 'PHB',
          quantity: 1,
          notes: 'Found in dungeon'
        })
      })

      it('passes null for missing notes', async () => {
        const version = createMockCharacterVersion()
        const character = createMockCharacter()
        const data = createMockCharacterData()

        mockInvoke
          .mockResolvedValueOnce(version)
          .mockResolvedValueOnce([character, data])

        const store = useCharacterStore()
        await store.addItem(1, 'Longsword', 'PHB', 1)

        expect(mockInvoke).toHaveBeenCalledWith('add_item_to_inventory', {
          characterId: 1,
          itemName: 'Longsword',
          itemSource: 'PHB',
          quantity: 1,
          notes: null
        })
      })
    })

    describe('removeItem', () => {
      it('removes item from inventory', async () => {
        const version = createMockCharacterVersion()
        const character = createMockCharacter()
        const data = createMockCharacterData()

        mockInvoke
          .mockResolvedValueOnce(version)
          .mockResolvedValueOnce([character, data])

        const store = useCharacterStore()
        await store.removeItem(1, 'Longsword', 1)

        expect(mockInvoke).toHaveBeenCalledWith('remove_item_from_inventory', {
          characterId: 1,
          itemName: 'Longsword',
          quantity: 1
        })
      })
    })

    describe('updateCurrency', () => {
      it('updates character currency', async () => {
        const version = createMockCharacterVersion()
        const character = createMockCharacter()
        const data = createMockCharacterData()

        mockInvoke
          .mockResolvedValueOnce(version)
          .mockResolvedValueOnce([character, data])

        const store = useCharacterStore()
        await store.updateCurrency(1, { gold: 100 })

        expect(mockInvoke).toHaveBeenCalledWith('update_character_currency', {
          characterId: 1,
          currency: { gold: 100 }
        })
      })
    })

    describe('updateEquipped', () => {
      it('updates equipped items', async () => {
        const version = createMockCharacterVersion()
        const character = createMockCharacter()
        const data = createMockCharacterData()

        mockInvoke
          .mockResolvedValueOnce(version)
          .mockResolvedValueOnce([character, data])

        const store = useCharacterStore()
        await store.updateEquipped(1, 'Chain Mail', 'Shield', 'Longsword', null)

        expect(mockInvoke).toHaveBeenCalledWith('update_character_equipped', {
          characterId: 1,
          armor: 'Chain Mail',
          shield: 'Shield',
          mainHand: 'Longsword',
          offHand: null
        })
      })
    })
  })
})
