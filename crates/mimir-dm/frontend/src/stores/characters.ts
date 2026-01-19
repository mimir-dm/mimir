import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { dataEvents } from '@/shared/utils/dataEvents'
import type {
  Character,
  CharacterData,
  CharacterVersion,
  CharacterWithData,
  CreateCharacterRequest,
  LevelUpRequest,
  CurrencyUpdate
} from '../types/character'

export const useCharacterStore = defineStore('characters', () => {
  // State
  const characters = ref<Character[]>([])
  const currentCharacter = ref<CharacterWithData | null>(null)
  const characterVersions = ref<CharacterVersion[]>([])
  const loading = ref(false)
  const error = ref<string | null>(null)

  // Computed
  const characterCount = computed(() => characters.value.length)

  const getCharacterById = computed(() => {
    return (id: number) => characters.value.find(c => c.id === id)
  })

  const currentCharacterLevel = computed(() => {
    return currentCharacter.value?.data.level || 0
  })

  const currentCharacterProficiencyBonus = computed(() => {
    const level = currentCharacterLevel.value
    return Math.ceil(level / 4) + 1
  })

  // Actions

  /**
   * Fetch all characters (including unassigned)
   */
  const fetchAllCharacters = async () => {
    loading.value = true
    error.value = null

    try {
      const result = await invoke<Character[]>('list_all_characters')
      characters.value = result
      return result
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to fetch characters'
      console.error('Error fetching characters:', e)
      throw e
    } finally {
      loading.value = false
    }
  }

  /**
   * Fetch all characters for a campaign
   */
  const fetchCharactersForCampaign = async (campaignId: number) => {
    loading.value = true
    error.value = null

    try {
      const result = await invoke<Character[]>('list_characters_for_campaign', {
        campaignId
      })
      characters.value = result
      return result
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to fetch characters'
      console.error('Error fetching characters:', e)
      throw e
    } finally {
      loading.value = false
    }
  }

  /**
   * Get a specific character with full data
   */
  const getCharacter = async (characterId: number, emitUpdate = true) => {
    loading.value = true
    error.value = null

    try {
      // Backend returns tuple [Character, CharacterData], convert to object
      const [character, data] = await invoke<[Character, CharacterData]>('get_character', {
        characterId
      })
      const result: CharacterWithData = { character, data }
      currentCharacter.value = result

      // Update in characters list if present
      const index = characters.value.findIndex(c => c.id === characterId)
      if (index !== -1) {
        characters.value[index] = result.character
      } else {
        characters.value.push(result.character)
      }

      // Emit update event for other listeners (e.g., after mutations call getCharacter)
      if (emitUpdate) {
        dataEvents.emit('character:updated', { characterId })
      }

      return result
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to fetch character'
      console.error('Error fetching character:', e)
      throw e
    } finally {
      loading.value = false
    }
  }

  /**
   * Create a new character
   */
  const createCharacter = async (request: CreateCharacterRequest, campaignId?: number) => {
    loading.value = true
    error.value = null

    try {
      const result = await invoke<CharacterData>('create_character', {
        request
      })

      // Emit event for listeners - need characterId from result
      // The result is CharacterData which should have character_id or id
      const characterId = (result as any).character_id || (result as any).id
      if (campaignId && characterId) {
        dataEvents.emit('character:created', { campaignId, characterId })
      }

      return result
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to create character'
      console.error('Error creating character:', e)
      throw e
    } finally {
      loading.value = false
    }
  }

  /**
   * Update character HP
   */
  const updateCharacterHp = async (
    characterId: number,
    newHp: number,
    reason: string
  ) => {
    loading.value = true
    error.value = null

    try {
      const result = await invoke<CharacterVersion>('update_character_hp', {
        characterId,
        newHp,
        reason
      })

      // Refresh character data
      await getCharacter(characterId)

      return result
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to update HP'
      console.error('Error updating HP:', e)
      throw e
    } finally {
      loading.value = false
    }
  }

  /**
   * Level up a character
   */
  const levelUpCharacter = async (
    characterId: number,
    request: LevelUpRequest
  ) => {
    loading.value = true
    error.value = null

    try {
      const result = await invoke<CharacterVersion>('level_up_character', {
        characterId,
        request
      })

      // Refresh character data
      await getCharacter(characterId)

      return result
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to level up character'
      console.error('Error leveling up character:', e)
      throw e
    } finally {
      loading.value = false
    }
  }

  /**
   * Add a spell to character's known spells
   */
  const addSpellToKnown = async (
    characterId: number,
    spellName: string,
    spellSource: string,
    isCantrip: boolean
  ) => {
    loading.value = true
    error.value = null

    try {
      const result = await invoke<CharacterVersion>('add_spell_to_known', {
        characterId,
        spellName,
        spellSource,
        isCantrip
      })

      // Refresh character data
      await getCharacter(characterId)

      return result
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to add spell'
      console.error('Error adding spell:', e)
      throw e
    } finally {
      loading.value = false
    }
  }

  /**
   * Prepare spells for a character
   */
  const prepareSpells = async (
    characterId: number,
    spellKeys: string[],
    spellcastingAbility: string
  ) => {
    loading.value = true
    error.value = null

    try {
      const result = await invoke<CharacterVersion>('prepare_spells', {
        characterId,
        spellKeys,
        spellcastingAbility
      })

      // Refresh character data
      await getCharacter(characterId)

      return result
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to prepare spells'
      console.error('Error preparing spells:', e)
      throw e
    } finally {
      loading.value = false
    }
  }

  /**
   * Cast a spell
   */
  const castSpell = async (
    characterId: number,
    spellName: string,
    spellLevel: number,
    isRitual: boolean = false
  ) => {
    loading.value = true
    error.value = null

    try {
      const result = await invoke<CharacterVersion>('cast_spell', {
        characterId,
        spellName,
        spellLevel,
        isRitual
      })

      // Refresh character data
      await getCharacter(characterId)

      return result
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to cast spell'
      console.error('Error casting spell:', e)
      throw e
    } finally {
      loading.value = false
    }
  }

  /**
   * Rest (short or long)
   */
  const rest = async (
    characterId: number,
    restType: 'short' | 'long'
  ) => {
    loading.value = true
    error.value = null

    try {
      const result = await invoke<CharacterVersion>('rest_character', {
        characterId,
        restType
      })

      // Refresh character data
      await getCharacter(characterId)

      return result
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to complete rest'
      console.error('Error resting:', e)
      throw e
    } finally {
      loading.value = false
    }
  }

  /**
   * Add an item to inventory
   */
  const addItem = async (
    characterId: number,
    itemName: string,
    itemSource: string,
    quantity: number,
    notes?: string
  ) => {
    loading.value = true
    error.value = null

    try {
      const result = await invoke<CharacterVersion>('add_item_to_inventory', {
        characterId,
        itemName,
        itemSource,
        quantity,
        notes: notes || null
      })

      // Refresh character data
      await getCharacter(characterId)

      return result
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to add item'
      console.error('Error adding item:', e)
      throw e
    } finally {
      loading.value = false
    }
  }

  /**
   * Remove an item from inventory
   */
  const removeItem = async (
    characterId: number,
    itemName: string,
    quantity: number
  ) => {
    loading.value = true
    error.value = null

    try {
      const result = await invoke<CharacterVersion>('remove_item_from_inventory', {
        characterId,
        itemName,
        quantity
      })

      // Refresh character data
      await getCharacter(characterId)

      return result
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to remove item'
      console.error('Error removing item:', e)
      throw e
    } finally {
      loading.value = false
    }
  }

  /**
   * Update character currency
   */
  const updateCurrency = async (
    characterId: number,
    update: CurrencyUpdate
  ) => {
    loading.value = true
    error.value = null

    try {
      const result = await invoke<CharacterVersion>('update_character_currency', {
        characterId,
        currency: update
      })

      // Refresh character data
      await getCharacter(characterId)

      return result
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to update currency'
      console.error('Error updating currency:', e)
      throw e
    } finally {
      loading.value = false
    }
  }

  /**
   * Update character equipped items
   */
  const updateEquipped = async (
    characterId: number,
    armor: string | null,
    shield: string | null,
    mainHand: string | null,
    offHand: string | null
  ) => {
    loading.value = true
    error.value = null

    try {
      const result = await invoke<CharacterVersion>('update_character_equipped', {
        characterId,
        armor,
        shield,
        mainHand,
        offHand
      })

      // Refresh character data
      await getCharacter(characterId)

      return result
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to update equipped items'
      console.error('Error updating equipped items:', e)
      throw e
    } finally {
      loading.value = false
    }
  }

  /**
   * Delete a character
   */
  const deleteCharacter = async (characterId: number, campaignId?: number) => {
    loading.value = true
    error.value = null

    try {
      await invoke<void>('delete_character', { characterId })

      // Remove from characters list
      characters.value = characters.value.filter(c => c.id !== characterId)

      // Clear current character if it was deleted
      if (currentCharacter.value?.character.id === characterId) {
        currentCharacter.value = null
      }

      // Emit event for listeners
      if (campaignId) {
        dataEvents.emit('character:deleted', { campaignId, characterId })
      }

      return true
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to delete character'
      console.error('Error deleting character:', e)
      throw e
    } finally {
      loading.value = false
    }
  }

  /**
   * Get character version history
   */
  const getCharacterVersions = async (characterId: number) => {
    loading.value = true
    error.value = null

    try {
      const result = await invoke<CharacterVersion[]>('get_character_versions', {
        characterId
      })
      characterVersions.value = result
      return result
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to fetch versions'
      console.error('Error fetching versions:', e)
      throw e
    } finally {
      loading.value = false
    }
  }

  /**
   * Get a specific character version
   */
  const getCharacterVersion = async (
    characterId: number,
    versionNumber: number
  ) => {
    loading.value = true
    error.value = null

    try {
      const result = await invoke<CharacterData>('get_character_version', {
        characterId,
        versionNumber
      })
      return result
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to fetch version'
      console.error('Error fetching version:', e)
      throw e
    } finally {
      loading.value = false
    }
  }

  /**
   * Set the current character
   */
  const setCurrentCharacter = (character: CharacterWithData | null) => {
    currentCharacter.value = character
  }

  /**
   * Clear all state
   */
  const reset = () => {
    characters.value = []
    currentCharacter.value = null
    characterVersions.value = []
    loading.value = false
    error.value = null
  }

  return {
    // State
    characters,
    currentCharacter,
    characterVersions,
    loading,
    error,

    // Computed
    characterCount,
    getCharacterById,
    currentCharacterLevel,
    currentCharacterProficiencyBonus,

    // Actions
    fetchAllCharacters,
    fetchCharactersForCampaign,
    getCharacter,
    createCharacter,
    updateCharacterHp,
    levelUpCharacter,
    addSpellToKnown,
    prepareSpells,
    castSpell,
    rest,
    addItem,
    removeItem,
    updateCurrency,
    updateEquipped,
    deleteCharacter,
    getCharacterVersions,
    getCharacterVersion,
    setCurrentCharacter,
    reset
  }
})
