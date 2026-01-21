import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { dataEvents } from '@/shared/utils/dataEvents'
import type { ApiResponse } from '@/types/api'
import type {
  Character,
  CharacterInventory,
  CreatePcRequest,
  CreateNpcRequest,
  UpdateCharacterRequest,
  AddInventoryRequest,
  UpdateInventoryRequest
} from '@/types/character'

export const useCharacterStore = defineStore('characters', () => {
  // State
  const characters = ref<Character[]>([])
  const currentCharacter = ref<Character | null>(null)
  const currentInventory = ref<CharacterInventory[]>([])
  const loading = ref(false)
  const error = ref<string | null>(null)

  // Computed
  const characterCount = computed(() => characters.value.length)

  const getCharacterById = computed(() => {
    return (id: string) => characters.value.find(c => c.id === id)
  })

  const pcs = computed(() => characters.value.filter(c => c.is_npc === 0))
  const npcs = computed(() => characters.value.filter(c => c.is_npc !== 0))

  // ==========================================================================
  // List Commands
  // ==========================================================================

  /**
   * Fetch all characters for a campaign
   */
  const fetchCharacters = async (campaignId: string) => {
    loading.value = true
    error.value = null

    try {
      const response = await invoke<ApiResponse<Character[]>>('list_characters', {
        campaignId
      })
      if (response.success && response.data) {
        characters.value = response.data
        return response.data
      } else {
        error.value = response.error || 'Failed to fetch characters'
        return []
      }
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to fetch characters'
      console.error('Error fetching characters:', e)
      return []
    } finally {
      loading.value = false
    }
  }

  /**
   * Fetch only player characters for a campaign
   */
  const fetchPcs = async (campaignId: string) => {
    loading.value = true
    error.value = null

    try {
      const response = await invoke<ApiResponse<Character[]>>('list_pcs', {
        campaignId
      })
      if (response.success && response.data) {
        // Update the characters list with PCs
        const pcIds = new Set(response.data.map(c => c.id))
        characters.value = [
          ...characters.value.filter(c => !pcIds.has(c.id)),
          ...response.data
        ]
        return response.data
      } else {
        error.value = response.error || 'Failed to fetch player characters'
        return []
      }
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to fetch player characters'
      console.error('Error fetching PCs:', e)
      return []
    } finally {
      loading.value = false
    }
  }

  /**
   * Fetch only NPCs for a campaign
   */
  const fetchNpcs = async (campaignId: string) => {
    loading.value = true
    error.value = null

    try {
      const response = await invoke<ApiResponse<Character[]>>('list_npcs', {
        campaignId
      })
      if (response.success && response.data) {
        // Update the characters list with NPCs
        const npcIds = new Set(response.data.map(c => c.id))
        characters.value = [
          ...characters.value.filter(c => !npcIds.has(c.id)),
          ...response.data
        ]
        return response.data
      } else {
        error.value = response.error || 'Failed to fetch NPCs'
        return []
      }
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to fetch NPCs'
      console.error('Error fetching NPCs:', e)
      return []
    } finally {
      loading.value = false
    }
  }

  // ==========================================================================
  // CRUD Commands
  // ==========================================================================

  /**
   * Get a specific character
   */
  const getCharacter = async (id: string) => {
    loading.value = true
    error.value = null

    try {
      const response = await invoke<ApiResponse<Character>>('get_character', { id })
      if (response.success && response.data) {
        currentCharacter.value = response.data

        // Update in characters list if present
        const index = characters.value.findIndex(c => c.id === id)
        if (index !== -1) {
          characters.value[index] = response.data
        } else {
          characters.value.push(response.data)
        }

        return response.data
      } else {
        error.value = response.error || 'Failed to fetch character'
        return null
      }
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to fetch character'
      console.error('Error fetching character:', e)
      return null
    } finally {
      loading.value = false
    }
  }

  /**
   * Create a new player character
   */
  const createPc = async (request: CreatePcRequest) => {
    loading.value = true
    error.value = null

    try {
      const response = await invoke<ApiResponse<Character>>('create_pc', { request })
      if (response.success && response.data) {
        characters.value.push(response.data)
        dataEvents.emit('character:created', {
          campaignId: response.data.campaign_id,
          characterId: response.data.id
        })
        return response.data
      } else {
        error.value = response.error || 'Failed to create character'
        return null
      }
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to create character'
      console.error('Error creating PC:', e)
      return null
    } finally {
      loading.value = false
    }
  }

  /**
   * Create a new NPC
   */
  const createNpc = async (request: CreateNpcRequest) => {
    loading.value = true
    error.value = null

    try {
      const response = await invoke<ApiResponse<Character>>('create_npc', { request })
      if (response.success && response.data) {
        characters.value.push(response.data)
        dataEvents.emit('character:created', {
          campaignId: response.data.campaign_id,
          characterId: response.data.id
        })
        return response.data
      } else {
        error.value = response.error || 'Failed to create NPC'
        return null
      }
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to create NPC'
      console.error('Error creating NPC:', e)
      return null
    } finally {
      loading.value = false
    }
  }

  /**
   * Update a character
   */
  const updateCharacter = async (id: string, request: UpdateCharacterRequest) => {
    loading.value = true
    error.value = null

    try {
      const response = await invoke<ApiResponse<Character>>('update_character', { id, request })
      if (response.success && response.data) {
        // Update in characters list
        const index = characters.value.findIndex(c => c.id === id)
        if (index !== -1) {
          characters.value[index] = response.data
        }
        if (currentCharacter.value?.id === id) {
          currentCharacter.value = response.data
        }
        dataEvents.emit('character:updated', { characterId: id })
        return response.data
      } else {
        error.value = response.error || 'Failed to update character'
        return null
      }
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to update character'
      console.error('Error updating character:', e)
      return null
    } finally {
      loading.value = false
    }
  }

  /**
   * Delete a character
   */
  const deleteCharacter = async (id: string) => {
    loading.value = true
    error.value = null

    try {
      const character = characters.value.find(c => c.id === id)
      const response = await invoke<ApiResponse<void>>('delete_character', { id })
      if (response.success) {
        characters.value = characters.value.filter(c => c.id !== id)
        if (currentCharacter.value?.id === id) {
          currentCharacter.value = null
        }
        if (character) {
          dataEvents.emit('character:deleted', {
            campaignId: character.campaign_id,
            characterId: id
          })
        }
        return true
      } else {
        error.value = response.error || 'Failed to delete character'
        return false
      }
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to delete character'
      console.error('Error deleting character:', e)
      return false
    } finally {
      loading.value = false
    }
  }

  // ==========================================================================
  // Inventory Commands
  // ==========================================================================

  /**
   * Fetch inventory for a character
   */
  const fetchInventory = async (characterId: string) => {
    loading.value = true
    error.value = null

    try {
      const response = await invoke<ApiResponse<CharacterInventory[]>>('get_character_inventory', {
        characterId
      })
      if (response.success && response.data) {
        currentInventory.value = response.data
        return response.data
      } else {
        error.value = response.error || 'Failed to fetch inventory'
        return []
      }
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to fetch inventory'
      console.error('Error fetching inventory:', e)
      return []
    } finally {
      loading.value = false
    }
  }

  /**
   * Fetch equipped items for a character
   */
  const fetchEquippedItems = async (characterId: string) => {
    loading.value = true
    error.value = null

    try {
      const response = await invoke<ApiResponse<CharacterInventory[]>>('get_equipped_items', {
        characterId
      })
      if (response.success && response.data) {
        return response.data
      } else {
        error.value = response.error || 'Failed to fetch equipped items'
        return []
      }
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to fetch equipped items'
      console.error('Error fetching equipped items:', e)
      return []
    } finally {
      loading.value = false
    }
  }

  /**
   * Fetch attuned items for a character
   */
  const fetchAttunedItems = async (characterId: string) => {
    loading.value = true
    error.value = null

    try {
      const response = await invoke<ApiResponse<CharacterInventory[]>>('get_attuned_items', {
        characterId
      })
      if (response.success && response.data) {
        return response.data
      } else {
        error.value = response.error || 'Failed to fetch attuned items'
        return []
      }
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to fetch attuned items'
      console.error('Error fetching attuned items:', e)
      return []
    } finally {
      loading.value = false
    }
  }

  /**
   * Add an item to inventory
   */
  const addInventoryItem = async (characterId: string, request: AddInventoryRequest) => {
    loading.value = true
    error.value = null

    try {
      const response = await invoke<ApiResponse<CharacterInventory>>('add_inventory_item', {
        characterId,
        request
      })
      if (response.success && response.data) {
        currentInventory.value.push(response.data)
        return response.data
      } else {
        error.value = response.error || 'Failed to add item'
        return null
      }
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to add item'
      console.error('Error adding inventory item:', e)
      return null
    } finally {
      loading.value = false
    }
  }

  /**
   * Remove an item from inventory
   */
  const removeInventoryItem = async (inventoryId: string) => {
    loading.value = true
    error.value = null

    try {
      const response = await invoke<ApiResponse<void>>('remove_inventory_item', { inventoryId })
      if (response.success) {
        currentInventory.value = currentInventory.value.filter(i => i.id !== inventoryId)
        return true
      } else {
        error.value = response.error || 'Failed to remove item'
        return false
      }
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to remove item'
      console.error('Error removing inventory item:', e)
      return false
    } finally {
      loading.value = false
    }
  }

  /**
   * Update an inventory item
   */
  const updateInventoryItem = async (inventoryId: string, request: UpdateInventoryRequest) => {
    loading.value = true
    error.value = null

    try {
      const response = await invoke<ApiResponse<CharacterInventory>>('update_inventory_item', {
        inventoryId,
        request
      })
      if (response.success && response.data) {
        const index = currentInventory.value.findIndex(i => i.id === inventoryId)
        if (index !== -1) {
          currentInventory.value[index] = response.data
        }
        return response.data
      } else {
        error.value = response.error || 'Failed to update item'
        return null
      }
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to update item'
      console.error('Error updating inventory item:', e)
      return null
    } finally {
      loading.value = false
    }
  }

  // ==========================================================================
  // Local State Management
  // ==========================================================================

  /**
   * Set the current character
   */
  const setCurrentCharacter = (character: Character | null) => {
    currentCharacter.value = character
  }

  /**
   * Clear all state
   */
  const reset = () => {
    characters.value = []
    currentCharacter.value = null
    currentInventory.value = []
    loading.value = false
    error.value = null
  }

  /**
   * Clear error
   */
  const clearError = () => {
    error.value = null
  }

  return {
    // State
    characters,
    currentCharacter,
    currentInventory,
    loading,
    error,

    // Computed
    characterCount,
    getCharacterById,
    pcs,
    npcs,

    // List actions
    fetchCharacters,
    fetchPcs,
    fetchNpcs,

    // CRUD actions
    getCharacter,
    createPc,
    createNpc,
    updateCharacter,
    deleteCharacter,

    // Inventory actions
    fetchInventory,
    fetchEquippedItems,
    fetchAttunedItems,
    addInventoryItem,
    removeInventoryItem,
    updateInventoryItem,

    // Local state
    setCurrentCharacter,
    reset,
    clearError
  }
})
