/**
 * Composable for managing UVTT map data including walls, portals, and lights.
 * Handles fetching UVTT data and runtime portal (door) state.
 */

import { ref, computed, watch, type Ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import {
  type UvttData,
  type Wall,
  type Portal,
  type Light,
  type AmbientLightLevel,
  uvttWallsToPixels,
  uvttPortalsToPixels,
  uvttLightsToPixels,
  uvttAmbientToLevel
} from './useVisibilityPolygon'

interface ApiResponse<T> {
  success: boolean
  data?: T
  error?: string
}

export interface UvttMapState {
  /** Raw UVTT data from file */
  uvttData: UvttData | null
  /** Wall segments in pixel coordinates */
  walls: Wall[]
  /** Portals with runtime closed/open state */
  portals: Portal[]
  /** Light sources in pixel coordinates */
  lights: Light[]
  /** Ambient light level from UVTT environment */
  ambientLight: AmbientLightLevel
  /** Pixels per grid cell */
  pixelsPerGrid: number
  /** Grid columns */
  gridCols: number
  /** Grid rows */
  gridRows: number
  /** Whether UVTT data is loaded */
  isLoaded: boolean
  /** Loading state */
  isLoading: boolean
  /** Error message if any */
  error: string | null
}

/**
 * Composable for managing UVTT map data.
 *
 * @param mapId - The map ID (used for API calls)
 * @param campaignId - The campaign ID (used for watch dependencies)
 * @param moduleId - Optional module ID (used for watch dependencies)
 * @param mapFilePath - The UVTT file path (used for watch dependencies)
 */
export function useUvttMap(
  mapId: Ref<string | null>,
  campaignId: Ref<string | null>,
  moduleId: Ref<string | null>,
  mapFilePath: Ref<string | null>
) {
  const uvttData = ref<UvttData | null>(null)
  const walls = ref<Wall[]>([])
  const portals = ref<Portal[]>([])
  const lights = ref<Light[]>([])
  const ambientLight = ref<AmbientLightLevel>('bright')
  const pixelsPerGrid = ref(70)
  const gridCols = ref(0)
  const gridRows = ref(0)
  const isLoaded = ref(false)
  const isLoading = ref(false)
  const error = ref<string | null>(null)

  /** Fetch UVTT data from the backend */
  async function loadUvttData() {
    if (!mapId.value) {
      isLoaded.value = false
      return
    }

    isLoading.value = true
    error.value = null

    try {
      const response = await invoke<ApiResponse<UvttData>>('get_uvtt_map', {
        id: mapId.value
      })

      if (response.success && response.data) {
        uvttData.value = response.data

        // Convert to pixel coordinates
        walls.value = uvttWallsToPixels(response.data)
        portals.value = uvttPortalsToPixels(response.data)
        lights.value = uvttLightsToPixels(response.data)

        // Set ambient light from environment
        ambientLight.value = uvttAmbientToLevel(response.data.environment?.ambient_light)

        console.log('useUvttMap: Loaded UVTT data')
        console.log('  - walls:', walls.value.length)
        console.log('  - portals:', portals.value.length)
        console.log('  - lights:', lights.value.length)
        console.log('  - ambient:', ambientLight.value, '(from:', response.data.environment?.ambient_light, ')')

        // Store grid info
        pixelsPerGrid.value = response.data.resolution.pixels_per_grid
        gridCols.value = Math.round(response.data.resolution.map_size.x)
        gridRows.value = Math.round(response.data.resolution.map_size.y)

        isLoaded.value = true
      } else {
        error.value = response.error || 'Failed to load UVTT data'
        isLoaded.value = false
      }
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to load UVTT data'
      isLoaded.value = false
      console.error('Failed to load UVTT data:', e)
    } finally {
      isLoading.value = false
    }
  }

  /** Toggle a portal's open/closed state */
  function togglePortal(portalId: string) {
    const portal = portals.value.find(p => p.id === portalId)
    if (portal) {
      portal.closed = !portal.closed
    }
  }

  /** Set a portal's state */
  function setPortalState(portalId: string, closed: boolean) {
    const portal = portals.value.find(p => p.id === portalId)
    if (portal) {
      portal.closed = closed
    }
  }

  /** Open all portals */
  function openAllPortals() {
    portals.value.forEach(p => p.closed = false)
  }

  /** Close all portals */
  function closeAllPortals() {
    portals.value.forEach(p => p.closed = true)
  }

  /** Reset portal states to their original UVTT values */
  function resetPortalStates() {
    if (uvttData.value) {
      portals.value = uvttPortalsToPixels(uvttData.value)
    }
  }

  // Computed: walls that are currently blocking (walls + closed portals)
  const blockingWalls = computed(() => {
    const closedPortalWalls = portals.value
      .filter(p => p.closed)
      .map(p => p.wall)
    return [...walls.value, ...closedPortalWalls]
  })

  // Computed: map dimensions in pixels
  const mapWidthPx = computed(() => gridCols.value * pixelsPerGrid.value)
  const mapHeightPx = computed(() => gridRows.value * pixelsPerGrid.value)

  // Watch for map changes and reload
  watch([mapId], () => {
    if (mapId.value) {
      loadUvttData()
    } else {
      // Clear state when no map
      uvttData.value = null
      walls.value = []
      portals.value = []
      lights.value = []
      ambientLight.value = 'bright'
      isLoaded.value = false
    }
  }, { immediate: true })

  return {
    // State
    uvttData,
    walls,
    portals,
    lights,
    ambientLight,
    pixelsPerGrid,
    gridCols,
    gridRows,
    isLoaded,
    isLoading,
    error,

    // Computed
    blockingWalls,
    mapWidthPx,
    mapHeightPx,

    // Methods
    loadUvttData,
    togglePortal,
    setPortalState,
    openAllPortals,
    closeAllPortals,
    resetPortalStates
  }
}

/**
 * Shared UVTT map state for use across components.
 * Call this once in the parent component and pass down the state.
 */
export type UvttMapComposable = ReturnType<typeof useUvttMap>
