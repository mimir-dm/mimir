import { onMounted, onUnmounted } from 'vue'
import { listen, emit, type UnlistenFn } from '@tauri-apps/api/event'
import type { Token } from '@/types/api'
import type { LightSourceSummary } from '@/composables/map/useLightSources'
import type { Light, Wall, Portal } from '@/composables/map/useVisibilityPolygon'
import type { AmbientLight } from '@/composables/map/useVisionCalculation'

/**
 * Map update payload from DM window
 */
export interface MapUpdatePayload {
  mapId: string
  gridType: string
  gridSizePx: number | null
  gridOffsetX: number
  gridOffsetY: number
  ambientLight?: string
  mapWidth?: number
  mapHeight?: number
}

/**
 * Tokens update payload from DM window
 */
export interface TokensUpdatePayload {
  mapId: string
  tokens: Token[]
  deadTokenIds?: string[]
}

/**
 * Fog/LOS update payload from DM window
 */
export interface FogUpdatePayload {
  mapId: string
  revealMap: boolean
  tokenOnlyLos: boolean
  visionCircles: { tokenId: string; x: number; y: number; radiusPx: number }[]
  useLosBlocking?: boolean
  visibilityPaths?: { tokenId: string; path: string; polygon?: { x: number; y: number }[] }[]
  blockingWalls?: Wall[]
  uvttLights?: Light[]
  portals?: Portal[]
  ambientLight?: AmbientLight
}

/**
 * Light sources update payload from DM window
 */
export interface LightSourcesUpdatePayload {
  mapId: string
  lightSources: LightSourceSummary[]
}

/**
 * Markers update payload from DM window
 */
export interface MarkersUpdatePayload {
  mapId: string
  traps: { id: string; grid_x: number; grid_y: number; name: string }[]
  pois: { id: string; grid_x: number; grid_y: number; name: string; icon: string; color: string | null }[]
  gridSizePx: number
}

/**
 * Event handlers for player display IPC events
 */
export interface PlayerDisplayEventHandlers {
  onMapUpdate: (payload: MapUpdatePayload) => void | Promise<void>
  onBlackout: (isBlackout: boolean) => void
  onTokensUpdate: (payload: TokensUpdatePayload) => void | Promise<void>
  onFogUpdate: (payload: FogUpdatePayload) => void
  onLightSourcesUpdate: (payload: LightSourcesUpdatePayload) => void
  onMarkersUpdate: (payload: MarkersUpdatePayload) => void
}

/**
 * Composable for managing player display IPC event listeners.
 *
 * Consolidates 6 IPC event listeners into a single setup/cleanup pattern.
 * Use in conjunction with onMounted to register handlers and auto-cleanup on unmount.
 *
 * @example
 * ```ts
 * usePlayerDisplayEvents({
 *   onMapUpdate: async (payload) => {
 *     mapState.value.mapId = payload.mapId
 *     await loadMapImage(payload.mapId)
 *   },
 *   onBlackout: (isBlackout) => {
 *     mapState.value.isBlackout = isBlackout
 *   },
 *   // ... other handlers
 * })
 * ```
 */
export function usePlayerDisplayEvents(handlers: PlayerDisplayEventHandlers): void {
  const unlisteners: UnlistenFn[] = []

  onMounted(async () => {
    console.log('usePlayerDisplayEvents: Setting up event listeners')

    // Map update listener
    unlisteners.push(
      await listen<MapUpdatePayload>('player-display:map-update', async (event) => {
        console.log('usePlayerDisplayEvents: Received map-update', event.payload.mapId)
        await handlers.onMapUpdate(event.payload)
        // Request current state from DM window
        await emit('player-display:request-state', { mapId: event.payload.mapId })
      })
    )

    // Blackout listener
    unlisteners.push(
      await listen<{ isBlackout: boolean }>('player-display:blackout', (event) => {
        handlers.onBlackout(event.payload.isBlackout)
      })
    )

    // Tokens update listener
    unlisteners.push(
      await listen<TokensUpdatePayload>('player-display:tokens-update', async (event) => {
        console.log('usePlayerDisplayEvents: Received tokens-update', event.payload.tokens.length, 'tokens')
        await handlers.onTokensUpdate(event.payload)
      })
    )

    // Fog/LOS update listener
    unlisteners.push(
      await listen<FogUpdatePayload>('player-display:fog-update', (event) => {
        handlers.onFogUpdate(event.payload)
      })
    )

    // Light sources update listener
    unlisteners.push(
      await listen<LightSourcesUpdatePayload>('player-display:light-sources-update', (event) => {
        handlers.onLightSourcesUpdate(event.payload)
      })
    )

    // Markers update listener
    unlisteners.push(
      await listen<MarkersUpdatePayload>('player-display:markers-update', (event) => {
        handlers.onMarkersUpdate(event.payload)
      })
    )
  })

  onUnmounted(() => {
    console.log('usePlayerDisplayEvents: Cleaning up', unlisteners.length, 'listeners')
    unlisteners.forEach(unlisten => unlisten())
  })
}
