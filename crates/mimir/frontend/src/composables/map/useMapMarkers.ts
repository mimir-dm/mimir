import { ref, type Ref, type ComputedRef } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { emit } from '@tauri-apps/api/event'

/**
 * Map trap data structure
 */
export interface MapTrap {
  id: string
  map_id: string
  grid_x: number
  grid_y: number
  name: string
  description: string | null
  trigger_description: string | null
  effect_description: string | null
  dc: number | null
  visible: number
  created_at: string
  updated_at: string
}

/**
 * Map POI (Point of Interest) data structure
 */
export interface MapPoi {
  id: string
  map_id: string
  grid_x: number
  grid_y: number
  name: string
  description: string | null
  icon: string
  color: string | null
  visible: number
  created_at: string
  updated_at: string
}

interface UseMapMarkersOptions {
  mapId: ComputedRef<string | null>
  gridSizePx: ComputedRef<number>
  isDisplayOpen: Ref<boolean>
}

/**
 * Composable for managing map markers (traps and POIs)
 */
export function useMapMarkers(options: UseMapMarkersOptions) {
  const { mapId, gridSizePx, isDisplayOpen } = options

  // State
  const mapTraps = ref<MapTrap[]>([])
  const mapPois = ref<MapPoi[]>([])
  const selectedTrapId = ref<string | null>(null)
  const selectedPoiId = ref<string | null>(null)

  // POI context menu state
  const poiContextMenu = ref<{
    visible: boolean
    x: number
    y: number
    poi: MapPoi | null
  }>({
    visible: false,
    x: 0,
    y: 0,
    poi: null
  })

  // POI edit modal state
  const showPoiEditModal = ref(false)
  const poiToEdit = ref<MapPoi | null>(null)

  /**
   * Load traps for the map
   */
  async function loadMapTraps(id?: string) {
    const targetMapId = id ?? mapId.value
    if (!targetMapId) return

    try {
      const response = await invoke<{ success: boolean; data?: MapTrap[] }>('list_map_traps', { mapId: targetMapId })
      if (response.success && response.data) {
        mapTraps.value = response.data
      }
    } catch (e) {
      console.error('Failed to load map traps:', e)
      mapTraps.value = []
    }
  }

  /**
   * Load POIs for the map
   */
  async function loadMapPois(id?: string) {
    const targetMapId = id ?? mapId.value
    if (!targetMapId) return

    try {
      const response = await invoke<{ success: boolean; data?: MapPoi[] }>('list_map_pois', { mapId: targetMapId })
      if (response.success && response.data) {
        mapPois.value = response.data
      }
    } catch (e) {
      console.error('Failed to load map POIs:', e)
      mapPois.value = []
    }
  }

  /**
   * Toggle trap visibility (right-click)
   */
  async function toggleTrapVisibility(trap: MapTrap) {
    try {
      await invoke('toggle_map_trap_visibility', { id: trap.id })
      await loadMapTraps()
      sendMarkersToDisplay()
    } catch (e) {
      console.error('Failed to toggle trap visibility:', e)
    }
  }

  /**
   * Toggle POI visibility (right-click)
   */
  async function togglePoiVisibility(poi: MapPoi) {
    try {
      await invoke('toggle_map_poi_visibility', { id: poi.id })
      await loadMapPois()
      sendMarkersToDisplay()
    } catch (e) {
      console.error('Failed to toggle POI visibility:', e)
    }
  }

  /**
   * Show POI context menu
   */
  function showPoiContextMenuAt(event: MouseEvent, poi: MapPoi) {
    selectedPoiId.value = poi.id
    poiContextMenu.value = {
      visible: true,
      x: event.clientX,
      y: event.clientY,
      poi
    }
  }

  /**
   * Open POI edit modal from context menu
   */
  function openPoiEditModal() {
    if (poiContextMenu.value.poi) {
      poiToEdit.value = poiContextMenu.value.poi
      showPoiEditModal.value = true
    }
    poiContextMenu.value.visible = false
  }

  /**
   * Close POI edit modal
   */
  function closePoiEditModal() {
    showPoiEditModal.value = false
    poiToEdit.value = null
  }

  /**
   * Handle POI saved from edit modal
   */
  function handlePoiSaved(updatedPoi: MapPoi) {
    // Update the POI in our local list
    const index = mapPois.value.findIndex(p => p.id === updatedPoi.id)
    if (index !== -1) {
      mapPois.value[index] = updatedPoi
    }
    // Send updated markers to player display
    sendMarkersToDisplay()
    closePoiEditModal()
  }

  /**
   * Toggle POI visibility from context menu
   */
  async function togglePoiVisibilityFromContext() {
    if (poiContextMenu.value.poi) {
      await togglePoiVisibility(poiContextMenu.value.poi)
    }
    poiContextMenu.value.visible = false
  }

  /**
   * Delete POI from context menu
   */
  async function deletePoiFromContext() {
    const poi = poiContextMenu.value.poi
    if (!poi) return

    if (confirm(`Delete POI "${poi.name}"?`)) {
      try {
        await invoke('delete_map_poi', { id: poi.id })
        await loadMapPois()
        sendMarkersToDisplay()
      } catch (e) {
        console.error('Failed to delete POI:', e)
      }
    }
    poiContextMenu.value.visible = false
  }

  /**
   * Send visible markers (traps & POIs) to player display
   */
  async function sendMarkersToDisplay() {
    if (!isDisplayOpen.value || !mapId.value) return

    const visibleTraps = mapTraps.value.filter(t => t.visible === 1)
    const visiblePois = mapPois.value.filter(p => p.visible === 1)

    try {
      await emit('player-display:markers-update', {
        mapId: mapId.value,
        traps: visibleTraps,
        pois: visiblePois,
        gridSizePx: gridSizePx.value
      })
    } catch (e) {
      console.error('Failed to send markers to display:', e)
    }
  }

  /**
   * Close context menu
   */
  function closePoiContextMenu() {
    poiContextMenu.value.visible = false
  }

  /**
   * Clear all marker state (when map changes)
   */
  function clearMarkers() {
    mapTraps.value = []
    mapPois.value = []
    selectedTrapId.value = null
    selectedPoiId.value = null
  }

  /**
   * Get icon character for POI type
   */
  function getPoiIcon(icon: string): string {
    const iconMap: Record<string, string> = {
      'pin': '📍',
      'star': '⭐',
      'skull': '💀',
      'chest': '📦',
      'door': '🚪',
      'secret': '🔮',
      'question': '❓',
      'exclamation': '❗'
    }
    return iconMap[icon] || '📍'
  }

  return {
    // State
    mapTraps,
    mapPois,
    selectedTrapId,
    selectedPoiId,
    poiContextMenu,
    showPoiEditModal,
    poiToEdit,

    // Loading
    loadMapTraps,
    loadMapPois,

    // Actions
    toggleTrapVisibility,
    togglePoiVisibility,
    showPoiContextMenuAt,
    openPoiEditModal,
    closePoiEditModal,
    handlePoiSaved,
    togglePoiVisibilityFromContext,
    deletePoiFromContext,
    sendMarkersToDisplay,
    closePoiContextMenu,
    clearMarkers,

    // Utilities
    getPoiIcon
  }
}
