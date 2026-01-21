/**
 * Composable for managing light sources on maps.
 * Provides light source state management and CRUD operations.
 */
import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'

interface ApiResponse<T> {
  success: boolean
  data?: T
  error?: string
}

/** Light source types matching D&D 5e light sources */
export type LightType = 'torch' | 'lantern' | 'candle' | 'spell' | 'custom'

/** Light source data from the database */
export interface LightSource {
  id: number
  map_id: number
  token_id: number | null
  name: string
  light_type: LightType
  x: number
  y: number
  bright_radius_ft: number
  dim_radius_ft: number
  color: string | null
  is_active: boolean
  created_at: string
  updated_at: string
}

/** Light source summary with token info */
export interface LightSourceSummary {
  id: number
  map_id: number
  token_id: number | null
  token_name: string | null
  name: string
  light_type: LightType
  x: number
  y: number
  bright_radius_ft: number
  dim_radius_ft: number
  color: string | null
  is_active: boolean
}

/** Request to create a new light source */
export interface CreateLightSourceRequest {
  map_id: number
  token_id?: number | null
  name: string
  light_type: LightType
  x: number
  y: number
  bright_radius_ft: number
  dim_radius_ft: number
  color?: string | null
}

/** Request to update a light source */
export interface UpdateLightSourceRequest {
  name?: string
  light_type?: LightType
  x?: number
  y?: number
  bright_radius_ft?: number
  dim_radius_ft?: number
  color?: string | null
  is_active?: boolean
}

/** Default light source configurations */
export const LIGHT_PRESETS: Record<LightType, { bright_ft: number; dim_ft: number; color: string | null }> = {
  torch: { bright_ft: 20, dim_ft: 40, color: '#ff9933' },
  lantern: { bright_ft: 30, dim_ft: 60, color: '#ffcc66' },
  candle: { bright_ft: 5, dim_ft: 10, color: '#ffaa44' },
  spell: { bright_ft: 20, dim_ft: 40, color: null },
  custom: { bright_ft: 20, dim_ft: 40, color: null }
}

export function useLightSources(mapId: number) {
  const lightSources = ref<LightSourceSummary[]>([])
  const loading = ref(false)
  const error = ref<string | null>(null)

  // Computed
  const hasLightSources = computed(() => lightSources.value.length > 0)
  const activeLightSources = computed(() => lightSources.value.filter(l => l.is_active))
  const lightSourceCount = computed(() => lightSources.value.length)
  const activeLightCount = computed(() => activeLightSources.value.length)

  // Load all light sources for the map
  async function loadLightSources(): Promise<void> {
    loading.value = true
    error.value = null
    try {
      const response = await invoke<ApiResponse<LightSourceSummary[]>>('list_light_sources', { mapId })
      if (response.success && response.data) {
        lightSources.value = response.data
      } else {
        error.value = response.error || 'Failed to load light sources'
      }
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to load light sources'
      console.error('Failed to load light sources:', e)
    } finally {
      loading.value = false
    }
  }

  // Create a new light source
  async function createLightSource(request: CreateLightSourceRequest): Promise<LightSource | null> {
    try {
      const response = await invoke<ApiResponse<LightSource>>('create_light_source', { request })
      if (response.success && response.data) {
        // Reload to get updated summaries with token names
        await loadLightSources()
        return response.data
      }
      error.value = response.error || 'Failed to create light source'
      return null
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to create light source'
      console.error('Failed to create light source:', e)
      return null
    }
  }

  // Create a torch at a position
  async function createTorch(x: number, y: number): Promise<LightSource | null> {
    try {
      const response = await invoke<ApiResponse<LightSource>>('create_torch', { mapId, x, y })
      if (response.success && response.data) {
        await loadLightSources()
        return response.data
      }
      return null
    } catch (e) {
      console.error('Failed to create torch:', e)
      return null
    }
  }

  // Create a lantern at a position
  async function createLantern(x: number, y: number): Promise<LightSource | null> {
    try {
      const response = await invoke<ApiResponse<LightSource>>('create_lantern', { mapId, x, y })
      if (response.success && response.data) {
        await loadLightSources()
        return response.data
      }
      return null
    } catch (e) {
      console.error('Failed to create lantern:', e)
      return null
    }
  }

  // Update a light source
  async function updateLightSource(id: number, request: UpdateLightSourceRequest): Promise<LightSource | null> {
    try {
      const response = await invoke<ApiResponse<LightSource>>('update_light_source', { id, request })
      if (response.success && response.data) {
        // Update local state
        const index = lightSources.value.findIndex(l => l.id === id)
        if (index !== -1) {
          lightSources.value[index] = {
            ...lightSources.value[index],
            ...response.data
          }
        }
        return response.data
      }
      return null
    } catch (e) {
      console.error('Failed to update light source:', e)
      return null
    }
  }

  // Move a light source
  async function moveLightSource(id: number, x: number, y: number): Promise<LightSource | null> {
    try {
      const response = await invoke<ApiResponse<LightSource>>('move_light_source', { id, x, y })
      if (response.success && response.data) {
        const index = lightSources.value.findIndex(l => l.id === id)
        if (index !== -1) {
          lightSources.value[index].x = x
          lightSources.value[index].y = y
        }
        return response.data
      }
      return null
    } catch (e) {
      console.error('Failed to move light source:', e)
      return null
    }
  }

  // Toggle a light source on/off
  async function toggleLightSource(id: number): Promise<LightSource | null> {
    try {
      const response = await invoke<ApiResponse<LightSource>>('toggle_light_source', { id })
      if (response.success && response.data) {
        const index = lightSources.value.findIndex(l => l.id === id)
        if (index !== -1) {
          lightSources.value[index].is_active = response.data.is_active
        }
        return response.data
      }
      return null
    } catch (e) {
      console.error('Failed to toggle light source:', e)
      return null
    }
  }

  // Delete a light source
  async function deleteLightSource(id: number): Promise<boolean> {
    try {
      const response = await invoke<ApiResponse<void>>('delete_light_source', { id })
      if (response.success) {
        lightSources.value = lightSources.value.filter(l => l.id !== id)
        return true
      }
      return false
    } catch (e) {
      console.error('Failed to delete light source:', e)
      return false
    }
  }

  // Delete all light sources on the map
  async function deleteAllLightSources(): Promise<boolean> {
    try {
      const response = await invoke<ApiResponse<number>>('delete_all_light_sources', { mapId })
      if (response.success) {
        lightSources.value = []
        return true
      }
      return false
    } catch (e) {
      console.error('Failed to delete all light sources:', e)
      return false
    }
  }

  // Get a light source by ID
  function getLightSource(id: number): LightSourceSummary | undefined {
    return lightSources.value.find(l => l.id === id)
  }

  // Convert feet to pixels based on grid size
  function feetToPixels(feet: number, gridSizePx: number): number {
    // 1 grid square = 5 feet in D&D
    return (feet / 5) * gridSizePx
  }

  // Convert pixels to feet based on grid size
  function pixelsToFeet(pixels: number, gridSizePx: number): number {
    return (pixels / gridSizePx) * 5
  }

  return {
    // State
    lightSources,
    loading,
    error,
    // Computed
    hasLightSources,
    activeLightSources,
    lightSourceCount,
    activeLightCount,
    // Methods
    loadLightSources,
    createLightSource,
    createTorch,
    createLantern,
    updateLightSource,
    moveLightSource,
    toggleLightSource,
    deleteLightSource,
    deleteAllLightSources,
    getLightSource,
    // Utilities
    feetToPixels,
    pixelsToFeet,
    // Constants
    LIGHT_PRESETS
  }
}
