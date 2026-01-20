import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { Player } from '../types/character'

export const usePlayerStore = defineStore('players', () => {
  // State
  const players = ref<Player[]>([])
  const currentPlayer = ref<Player | null>(null)
  const loading = ref(false)
  const error = ref<string | null>(null)

  // Computed
  const playerCount = computed(() => players.value.length)

  const getPlayerById = computed(() => {
    return (id: number) => players.value.find(p => p.id === id)
  })

  // Actions

  /**
   * Fetch all players
   */
  const fetchPlayers = async () => {
    loading.value = true
    error.value = null

    try {
      const result = await invoke<Player[]>('list_players')
      players.value = result
      return result
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to fetch players'
      console.error('Error fetching players:', e)
      throw e
    } finally {
      loading.value = false
    }
  }

  /**
   * Get a specific player by ID
   */
  const getPlayer = async (playerId: number) => {
    loading.value = true
    error.value = null

    try {
      const result = await invoke<Player>('get_player', { playerId })
      currentPlayer.value = result

      // Update in players list if present
      const index = players.value.findIndex(p => p.id === playerId)
      if (index !== -1) {
        players.value[index] = result
      } else {
        players.value.push(result)
      }

      return result
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to fetch player'
      console.error('Error fetching player:', e)
      throw e
    } finally {
      loading.value = false
    }
  }

  /**
   * Create a new player
   */
  const createPlayer = async (
    name: string,
    email?: string,
    notes?: string
  ) => {
    loading.value = true
    error.value = null

    try {
      const result = await invoke<Player>('create_player', {
        name,
        email: email || null,
        notes: notes || null
      })

      players.value.push(result)
      currentPlayer.value = result

      return result
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to create player'
      console.error('Error creating player:', e)
      throw e
    } finally {
      loading.value = false
    }
  }

  /**
   * Update an existing player
   */
  const updatePlayer = async (
    playerId: number,
    updates: {
      name?: string
      email?: string | null
      notes?: string | null
    }
  ) => {
    loading.value = true
    error.value = null

    try {
      const result = await invoke<Player>('update_player', {
        playerId,
        name: updates.name,
        email: updates.email !== undefined ? updates.email : undefined,
        notes: updates.notes !== undefined ? updates.notes : undefined
      })

      // Update in players list
      const index = players.value.findIndex(p => p.id === playerId)
      if (index !== -1) {
        players.value[index] = result
      }

      // Update current player if it's the one being updated
      if (currentPlayer.value?.id === playerId) {
        currentPlayer.value = result
      }

      return result
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to update player'
      console.error('Error updating player:', e)
      throw e
    } finally {
      loading.value = false
    }
  }

  /**
   * Delete a player
   */
  const deletePlayer = async (playerId: number) => {
    loading.value = true
    error.value = null

    try {
      await invoke<void>('delete_player', { playerId })

      // Remove from players list
      players.value = players.value.filter(p => p.id !== playerId)

      // Clear current player if it was deleted
      if (currentPlayer.value?.id === playerId) {
        currentPlayer.value = null
      }

      return true
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to delete player'
      console.error('Error deleting player:', e)
      throw e
    } finally {
      loading.value = false
    }
  }

  /**
   * Set the current player
   */
  const setCurrentPlayer = (player: Player | null) => {
    currentPlayer.value = player
  }

  /**
   * Clear all state
   */
  const reset = () => {
    players.value = []
    currentPlayer.value = null
    loading.value = false
    error.value = null
  }

  return {
    // State
    players,
    currentPlayer,
    loading,
    error,

    // Computed
    playerCount,
    getPlayerById,

    // Actions
    fetchPlayers,
    getPlayer,
    createPlayer,
    updatePlayer,
    deletePlayer,
    setCurrentPlayer,
    reset
  }
})
