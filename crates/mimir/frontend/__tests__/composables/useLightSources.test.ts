/**
 * Tests for useLightSources composable
 *
 * Tests light source presets, CRUD operations, unit conversion,
 * and computed properties.
 */

import { describe, it, expect, beforeEach, afterEach } from 'vitest'
import {
  setupInvokeMock,
  resetInvokeMock,
  mockCommand,
  expectCommandCalled,
  expectCommandCalledWith,
} from '@tests/helpers/mockInvoke'
import { useLightSources, LIGHT_PRESETS } from '@/composables/map/useLightSources'

// ─── Factories ──────────────────────────────────────────────────────────────

function makeLightSource(overrides: Record<string, unknown> = {}) {
  return {
    id: 'light-1',
    map_id: 'map-1',
    token_id: null,
    token_name: null,
    name: 'Torch',
    light_type: 'torch',
    x: 100,
    y: 200,
    bright_radius_ft: 20,
    dim_radius_ft: 40,
    color: '#ff9933',
    is_active: true,
    ...overrides,
  }
}

// ─── Tests ──────────────────────────────────────────────────────────────────

describe('useLightSources', () => {
  beforeEach(() => {
    setupInvokeMock()
  })

  afterEach(() => {
    resetInvokeMock()
  })

  describe('LIGHT_PRESETS', () => {
    it('has torch preset with correct radii', () => {
      expect(LIGHT_PRESETS.torch.bright_ft).toBe(20)
      expect(LIGHT_PRESETS.torch.dim_ft).toBe(40)
      expect(LIGHT_PRESETS.torch.color).toBe('#ff9933')
    })

    it('has lantern preset with correct radii', () => {
      expect(LIGHT_PRESETS.lantern.bright_ft).toBe(30)
      expect(LIGHT_PRESETS.lantern.dim_ft).toBe(60)
    })

    it('has candle preset with small radii', () => {
      expect(LIGHT_PRESETS.candle.bright_ft).toBe(5)
      expect(LIGHT_PRESETS.candle.dim_ft).toBe(10)
    })

    it('has spell preset with no default color', () => {
      expect(LIGHT_PRESETS.spell.color).toBeNull()
    })
  })

  describe('unit conversions', () => {
    it('converts feet to pixels (1 square = 5ft)', () => {
      const { feetToPixels } = useLightSources('map-1')
      // 70px grid, 5ft per square
      expect(feetToPixels(5, 70)).toBe(70)
      expect(feetToPixels(10, 70)).toBe(140)
      expect(feetToPixels(30, 70)).toBe(420)
    })

    it('converts pixels to feet', () => {
      const { pixelsToFeet } = useLightSources('map-1')
      expect(pixelsToFeet(70, 70)).toBe(5)
      expect(pixelsToFeet(140, 70)).toBe(10)
      expect(pixelsToFeet(420, 70)).toBe(30)
    })

    it('handles non-standard grid sizes', () => {
      const { feetToPixels, pixelsToFeet } = useLightSources('map-1')
      // 50px grid
      expect(feetToPixels(10, 50)).toBe(100)
      expect(pixelsToFeet(100, 50)).toBe(10)
    })
  })

  describe('loadLightSources', () => {
    it('loads light sources from backend', async () => {
      const lights = [makeLightSource(), makeLightSource({ id: 'light-2', name: 'Lantern' })]
      mockCommand('list_light_sources', lights)

      const { loadLightSources, lightSources } = useLightSources('map-1')
      await loadLightSources()

      expect(lightSources.value).toHaveLength(2)
      expectCommandCalledWith('list_light_sources', { mapId: 'map-1' })
    })
  })

  describe('computed properties', () => {
    it('hasLightSources reflects state', async () => {
      mockCommand('list_light_sources', [makeLightSource()])

      const { loadLightSources, hasLightSources } = useLightSources('map-1')
      expect(hasLightSources.value).toBe(false)
      await loadLightSources()
      expect(hasLightSources.value).toBe(true)
    })

    it('activeLightSources filters active lights', async () => {
      mockCommand('list_light_sources', [
        makeLightSource({ id: 'l1', is_active: true }),
        makeLightSource({ id: 'l2', is_active: false }),
      ])

      const { loadLightSources, activeLightSources, activeLightCount } = useLightSources('map-1')
      await loadLightSources()

      expect(activeLightSources.value).toHaveLength(1)
      expect(activeLightCount.value).toBe(1)
    })
  })

  describe('createTorch', () => {
    it('creates torch at position', async () => {
      mockCommand('create_torch', makeLightSource())
      mockCommand('list_light_sources', [makeLightSource()])

      const { createTorch } = useLightSources('map-1')
      const result = await createTorch(100, 200)

      expect(result).not.toBeNull()
      expectCommandCalledWith('create_torch', { mapId: 'map-1', x: 100, y: 200 })
    })
  })

  describe('toggleLightSource', () => {
    it('toggles light on/off', async () => {
      mockCommand('list_light_sources', [makeLightSource({ is_active: true })])
      mockCommand('toggle_light_source', makeLightSource({ is_active: false }))

      const { loadLightSources, toggleLightSource, lightSources } = useLightSources('map-1')
      await loadLightSources()

      const result = await toggleLightSource('light-1')
      expect(result).not.toBeNull()
      expect(lightSources.value[0].is_active).toBe(false)
    })
  })

  describe('deleteLightSource', () => {
    it('removes light from local state on success', async () => {
      mockCommand('list_light_sources', [
        makeLightSource({ id: 'l1' }),
        makeLightSource({ id: 'l2' }),
      ])
      mockCommand('delete_light_source', null)

      const { loadLightSources, deleteLightSource, lightSources } = useLightSources('map-1')
      await loadLightSources()
      expect(lightSources.value).toHaveLength(2)

      const success = await deleteLightSource('l1')
      expect(success).toBe(true)
      expect(lightSources.value).toHaveLength(1)
      expect(lightSources.value[0].id).toBe('l2')
    })
  })

  describe('deleteAllLightSources', () => {
    it('clears all lights', async () => {
      mockCommand('list_light_sources', [makeLightSource()])
      mockCommand('delete_all_light_sources', 1)

      const { loadLightSources, deleteAllLightSources, lightSources } = useLightSources('map-1')
      await loadLightSources()

      const success = await deleteAllLightSources()
      expect(success).toBe(true)
      expect(lightSources.value).toHaveLength(0)
    })
  })

  describe('getLightSource', () => {
    it('finds light by ID', async () => {
      mockCommand('list_light_sources', [
        makeLightSource({ id: 'l1', name: 'Torch' }),
        makeLightSource({ id: 'l2', name: 'Lantern' }),
      ])

      const { loadLightSources, getLightSource } = useLightSources('map-1')
      await loadLightSources()

      const light = getLightSource('l2')
      expect(light?.name).toBe('Lantern')
    })

    it('returns undefined for unknown ID', async () => {
      mockCommand('list_light_sources', [])

      const { loadLightSources, getLightSource } = useLightSources('map-1')
      await loadLightSources()

      expect(getLightSource('nonexistent')).toBeUndefined()
    })
  })
})
