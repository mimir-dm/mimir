/**
 * Tests for usePlayerDisplayEvents composable.
 *
 * Tests event handler registration, payload types, and cleanup behavior.
 * Since this composable uses onMounted/onUnmounted, we test it by mounting
 * a dummy component that uses it.
 */

import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest'
import { mount, flushPromises } from '@vue/test-utils'
import { defineComponent, ref, nextTick } from 'vue'

// Mock @tauri-apps/api/event before importing the composable
const mockListeners = new Map<string, Function>()
const mockUnlistenFns: Function[] = []

vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn(async (eventName: string, handler: Function) => {
    mockListeners.set(eventName, handler)
    const unlisten = vi.fn(() => {
      mockListeners.delete(eventName)
    })
    mockUnlistenFns.push(unlisten)
    return unlisten
  }),
  emit: vi.fn(),
}))

import type {
  MapUpdatePayload,
  TokensUpdatePayload,
  FogUpdatePayload,
  LightSourcesUpdatePayload,
  MarkersUpdatePayload,
  PlayerDisplayEventHandlers,
} from '@/composables/map/usePlayerDisplayEvents'
import { usePlayerDisplayEvents } from '@/composables/map/usePlayerDisplayEvents'

function createHandlerSpies(): PlayerDisplayEventHandlers {
  return {
    onMapUpdate: vi.fn(),
    onBlackout: vi.fn(),
    onTokensUpdate: vi.fn(),
    onFogUpdate: vi.fn(),
    onLightSourcesUpdate: vi.fn(),
    onMarkersUpdate: vi.fn(),
  }
}

function mountWithHandlers(handlers: PlayerDisplayEventHandlers) {
  return mount(defineComponent({
    setup() {
      usePlayerDisplayEvents(handlers)
      return () => null
    },
  }))
}

describe('usePlayerDisplayEvents', () => {
  beforeEach(() => {
    mockListeners.clear()
    mockUnlistenFns.length = 0
    vi.clearAllMocks()
  })

  afterEach(() => {
    vi.restoreAllMocks()
  })

  describe('event registration', () => {
    it('registers all 6 event listeners on mount', async () => {
      const handlers = createHandlerSpies()
      mountWithHandlers(handlers)
      await flushPromises()

      expect(mockListeners.has('player-display:map-update')).toBe(true)
      expect(mockListeners.has('player-display:blackout')).toBe(true)
      expect(mockListeners.has('player-display:tokens-update')).toBe(true)
      expect(mockListeners.has('player-display:fog-update')).toBe(true)
      expect(mockListeners.has('player-display:light-sources-update')).toBe(true)
      expect(mockListeners.has('player-display:markers-update')).toBe(true)
    })

    it('creates 6 unlisten functions', async () => {
      const handlers = createHandlerSpies()
      mountWithHandlers(handlers)
      await flushPromises()

      expect(mockUnlistenFns.length).toBe(6)
    })
  })

  describe('event dispatching', () => {
    it('dispatches map update to handler', async () => {
      const handlers = createHandlerSpies()
      mountWithHandlers(handlers)
      await flushPromises()

      const payload: MapUpdatePayload = {
        mapId: 'map-1',
        gridType: 'square',
        gridSizePx: 70,
        gridOffsetX: 0,
        gridOffsetY: 0,
      }
      const listener = mockListeners.get('player-display:map-update')!
      await listener({ payload })

      expect(handlers.onMapUpdate).toHaveBeenCalledWith(payload)
    })

    it('dispatches blackout to handler', async () => {
      const handlers = createHandlerSpies()
      mountWithHandlers(handlers)
      await flushPromises()

      const listener = mockListeners.get('player-display:blackout')!
      listener({ payload: { isBlackout: true } })

      expect(handlers.onBlackout).toHaveBeenCalledWith(true)
    })

    it('dispatches tokens update to handler', async () => {
      const handlers = createHandlerSpies()
      mountWithHandlers(handlers)
      await flushPromises()

      const payload: TokensUpdatePayload = {
        mapId: 'map-1',
        tokens: [
          {
            id: 'tok-1',
            map_id: 'map-1',
            name: 'Goblin',
            token_type: 'monster',
            size: 'medium',
            color: '#ff0000',
            grid_x: 5,
            grid_y: 3,
            visible_to_players: 1,
          } as any,
        ],
      }
      const listener = mockListeners.get('player-display:tokens-update')!
      await listener({ payload })

      expect(handlers.onTokensUpdate).toHaveBeenCalledWith(payload)
    })

    it('dispatches fog update to handler', async () => {
      const handlers = createHandlerSpies()
      mountWithHandlers(handlers)
      await flushPromises()

      const payload: FogUpdatePayload = {
        mapId: 'map-1',
        revealMap: false,
        tokenOnlyLos: false,
        visionCircles: [],
      }
      const listener = mockListeners.get('player-display:fog-update')!
      listener({ payload })

      expect(handlers.onFogUpdate).toHaveBeenCalledWith(payload)
    })

    it('dispatches light sources update to handler', async () => {
      const handlers = createHandlerSpies()
      mountWithHandlers(handlers)
      await flushPromises()

      const payload: LightSourcesUpdatePayload = {
        mapId: 'map-1',
        lightSources: [],
      }
      const listener = mockListeners.get('player-display:light-sources-update')!
      listener({ payload })

      expect(handlers.onLightSourcesUpdate).toHaveBeenCalledWith(payload)
    })

    it('dispatches markers update to handler', async () => {
      const handlers = createHandlerSpies()
      mountWithHandlers(handlers)
      await flushPromises()

      const payload: MarkersUpdatePayload = {
        mapId: 'map-1',
        traps: [{ id: 'trap-1', grid_x: 3, grid_y: 5, name: 'Pit Trap' }],
        pois: [{ id: 'poi-1', grid_x: 7, grid_y: 2, name: 'Chest', icon: '📦', color: '#gold' }],
        gridSizePx: 70,
      }
      const listener = mockListeners.get('player-display:markers-update')!
      listener({ payload })

      expect(handlers.onMarkersUpdate).toHaveBeenCalledWith(payload)
    })
  })

  describe('cleanup', () => {
    it('calls all unlisten functions on unmount', async () => {
      const handlers = createHandlerSpies()
      const wrapper = mountWithHandlers(handlers)
      await flushPromises()

      expect(mockUnlistenFns.length).toBe(6)

      wrapper.unmount()

      for (const fn of mockUnlistenFns) {
        expect(fn).toHaveBeenCalled()
      }
    })
  })

  describe('payload types', () => {
    it('MapUpdatePayload supports optional fields', () => {
      const payload: MapUpdatePayload = {
        mapId: 'map-1',
        gridType: 'hex',
        gridSizePx: null,
        gridOffsetX: 10,
        gridOffsetY: 20,
        ambientLight: 'dim',
        mapWidth: 3000,
        mapHeight: 2000,
      }
      expect(payload.ambientLight).toBe('dim')
      expect(payload.mapWidth).toBe(3000)
    })

    it('FogUpdatePayload supports vision circles and LOS blocking', () => {
      const payload: FogUpdatePayload = {
        mapId: 'map-1',
        revealMap: true,
        tokenOnlyLos: true,
        visionCircles: [
          { tokenId: 'tok-1', x: 100, y: 200, radiusPx: 420 },
        ],
        useLosBlocking: true,
        visibilityPaths: [
          { tokenId: 'tok-1', path: 'M 0,0 L 100,100 Z' },
        ],
      }
      expect(payload.visionCircles).toHaveLength(1)
      expect(payload.useLosBlocking).toBe(true)
    })

    it('TokensUpdatePayload supports dead token IDs', () => {
      const payload: TokensUpdatePayload = {
        mapId: 'map-1',
        tokens: [],
        deadTokenIds: ['tok-3', 'tok-7'],
      }
      expect(payload.deadTokenIds).toHaveLength(2)
    })
  })
})
