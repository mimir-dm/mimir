/**
 * Composable for managing fog of war on maps.
 * Provides fog state management and reveal operations.
 */
import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'

interface ApiResponse<T> {
  success: boolean
  data?: T
  error?: string
}

export interface FogRevealedArea {
  id: number
  map_id: number
  x: number
  y: number
  width: number
  height: number
}

export interface FogState {
  map_id: number
  fog_enabled: boolean
  revealed_areas: FogRevealedArea[]
}

export interface RevealRectRequest {
  map_id: number
  x: number
  y: number
  width: number
  height: number
}

export interface RevealCircleRequest {
  map_id: number
  center_x: number
  center_y: number
  radius: number
}

export function useFog(mapId: number) {
  const fogEnabled = ref(false)
  const revealedAreas = ref<FogRevealedArea[]>([])
  const loading = ref(false)
  const error = ref<string | null>(null)

  // Computed
  const hasRevealedAreas = computed(() => revealedAreas.value.length > 0)

  // Load fog state for the map
  async function loadFogState(): Promise<void> {
    loading.value = true
    error.value = null
    try {
      const response = await invoke<ApiResponse<FogState>>('get_fog_state', { mapId })
      if (response.success && response.data) {
        fogEnabled.value = response.data.fog_enabled
        revealedAreas.value = response.data.revealed_areas
      } else {
        error.value = response.error || 'Failed to load fog state'
      }
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to load fog state'
      console.error('Failed to load fog state:', e)
    } finally {
      loading.value = false
    }
  }

  // Toggle fog on/off
  async function toggleFog(): Promise<boolean> {
    loading.value = true
    error.value = null
    try {
      const response = await invoke<ApiResponse<boolean>>('toggle_fog', { mapId })
      if (response.success && response.data !== undefined) {
        fogEnabled.value = response.data
        return response.data
      } else {
        error.value = response.error || 'Failed to toggle fog'
        return fogEnabled.value
      }
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to toggle fog'
      console.error('Failed to toggle fog:', e)
      return fogEnabled.value
    } finally {
      loading.value = false
    }
  }

  // Enable fog
  async function enableFog(): Promise<boolean> {
    try {
      const response = await invoke<ApiResponse<void>>('enable_fog', { mapId })
      if (response.success) {
        fogEnabled.value = true
        return true
      }
      return false
    } catch (e) {
      console.error('Failed to enable fog:', e)
      return false
    }
  }

  // Disable fog
  async function disableFog(): Promise<boolean> {
    try {
      const response = await invoke<ApiResponse<void>>('disable_fog', { mapId })
      if (response.success) {
        fogEnabled.value = false
        return true
      }
      return false
    } catch (e) {
      console.error('Failed to disable fog:', e)
      return false
    }
  }

  // Reveal a rectangular area
  async function revealRect(x: number, y: number, width: number, height: number): Promise<FogRevealedArea | null> {
    try {
      const request: RevealRectRequest = { map_id: mapId, x, y, width, height }
      const response = await invoke<ApiResponse<FogRevealedArea>>('reveal_rect', { request })
      if (response.success && response.data) {
        revealedAreas.value.push(response.data)
        return response.data
      }
      return null
    } catch (e) {
      console.error('Failed to reveal rect:', e)
      return null
    }
  }

  // Reveal a circular area
  async function revealCircle(centerX: number, centerY: number, radius: number): Promise<FogRevealedArea | null> {
    try {
      const request: RevealCircleRequest = { map_id: mapId, center_x: centerX, center_y: centerY, radius }
      const response = await invoke<ApiResponse<FogRevealedArea>>('reveal_circle', { request })
      if (response.success && response.data) {
        revealedAreas.value.push(response.data)
        return response.data
      }
      return null
    } catch (e) {
      console.error('Failed to reveal circle:', e)
      return null
    }
  }

  // Reveal entire map
  async function revealAll(mapWidth: number, mapHeight: number): Promise<FogRevealedArea | null> {
    try {
      const request = { map_id: mapId, map_width: mapWidth, map_height: mapHeight }
      const response = await invoke<ApiResponse<FogRevealedArea>>('reveal_all', { request })
      if (response.success && response.data) {
        revealedAreas.value.push(response.data)
        return response.data
      }
      return null
    } catch (e) {
      console.error('Failed to reveal all:', e)
      return null
    }
  }

  // Delete a revealed area (re-fog)
  async function deleteRevealedArea(id: number): Promise<boolean> {
    try {
      const response = await invoke<ApiResponse<void>>('delete_revealed_area', { id })
      if (response.success) {
        revealedAreas.value = revealedAreas.value.filter(a => a.id !== id)
        return true
      }
      return false
    } catch (e) {
      console.error('Failed to delete revealed area:', e)
      return false
    }
  }

  // Reset fog (clear all revealed areas)
  async function resetFog(): Promise<boolean> {
    try {
      const response = await invoke<ApiResponse<number>>('reset_fog', { mapId })
      if (response.success) {
        revealedAreas.value = []
        return true
      }
      return false
    } catch (e) {
      console.error('Failed to reset fog:', e)
      return false
    }
  }

  // Check if a point is in a revealed area
  function isPointRevealed(px: number, py: number): boolean {
    return revealedAreas.value.some(area =>
      px >= area.x && px <= area.x + area.width &&
      py >= area.y && py <= area.y + area.height
    )
  }

  return {
    // State
    fogEnabled,
    revealedAreas,
    loading,
    error,
    // Computed
    hasRevealedAreas,
    // Methods
    loadFogState,
    toggleFog,
    enableFog,
    disableFog,
    revealRect,
    revealCircle,
    revealAll,
    deleteRevealedArea,
    resetFog,
    isPointRevealed
  }
}
