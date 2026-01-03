import { ref, computed, type Ref, type ComputedRef } from 'vue'
import { invoke } from '@tauri-apps/api/core'

// Types
export interface MapSummary {
  id: number
  campaign_id: number
  module_id: number | null
  name: string
  grid_type: string
  grid_size_px: number | null
  grid_offset_x: number
  grid_offset_y: number
  module_name: string | null
  width_px: number
  height_px: number
  ambient_light: string
  image_path: string
}

interface UseModuleMapsOptions {
  moduleId: Ref<number>
  campaignId: ComputedRef<number | null | undefined>
  isDisplayOpen: Ref<boolean>
}

/**
 * Composable for managing module maps
 * Handles loading maps and sending them to player display
 */
export function useModuleMaps({ moduleId, campaignId, isDisplayOpen }: UseModuleMapsOptions) {
  // State
  const allMaps = ref<MapSummary[]>([])
  const mapsLoading = ref(false)
  const activeMapId = ref<number | null>(null)

  // Get the active map details for the DmMapViewer
  const activeMap = computed(() => {
    if (!activeMapId.value) return null
    return allMaps.value.find(m => m.id === activeMapId.value) || null
  })

  // Load maps for this module (campaign-level maps + this module's maps only)
  async function loadMaps() {
    if (!campaignId.value) return

    mapsLoading.value = true
    try {
      const response = await invoke<{ success: boolean; data?: MapSummary[] }>('list_map_summaries', {
        campaignId: campaignId.value
      })

      if (response.success && response.data) {
        // Filter to show only:
        // 1. Campaign-level maps (module_id is null)
        // 2. Maps for the current module
        allMaps.value = response.data.filter(map =>
          map.module_id === null || map.module_id === moduleId.value
        )
      }
    } catch (e) {
      console.error('Failed to load maps:', e)
    } finally {
      mapsLoading.value = false
    }
  }

  // Send a map to the player display
  async function sendMapToPlayerDisplay(map: MapSummary) {
    // Always set the active map (caller handles view mode switch)
    activeMapId.value = map.id

    // If display is open, send the map to it
    if (isDisplayOpen.value) {
      try {
        await invoke('send_map_to_display', {
          mapId: map.id,
          gridType: map.grid_type,
          gridSizePx: map.grid_size_px,
          gridOffsetX: map.grid_offset_x,
          gridOffsetY: map.grid_offset_y,
          ambientLight: map.ambient_light,
          mapWidth: map.width_px,
          mapHeight: map.height_px
        })
      } catch (err) {
        console.error('Failed to send map to display:', err)
      }
    }
  }

  // Set active map without sending to display
  function setActiveMap(mapId: number | null) {
    activeMapId.value = mapId
  }

  return {
    // State
    allMaps,
    mapsLoading,
    activeMapId,
    // Computed
    activeMap,
    // Actions
    loadMaps,
    sendMapToPlayerDisplay,
    setActiveMap
  }
}
