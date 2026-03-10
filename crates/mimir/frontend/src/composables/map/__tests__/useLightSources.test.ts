/**
 * Tests for useLightSources composable.
 *
 * Tests light source state management, CRUD operations, computed properties,
 * and unit conversion utilities. All invoke calls are mocked.
 */

import { describe, it, expect, beforeEach, vi } from 'vitest'
import { invoke } from '@tauri-apps/api/core'
import { useLightSources, LIGHT_PRESETS, type LightSourceSummary, type LightSource } from '../useLightSources'

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}))

const mockInvoke = vi.mocked(invoke)

function makeSummary(overrides: Partial<LightSourceSummary> = {}): LightSourceSummary {
  return {
    id: 'ls-1',
    map_id: 'map-1',
    token_id: null,
    token_name: null,
    name: 'Torch',
    light_type: 'torch',
    x: 350,
    y: 350,
    bright_radius_ft: 20,
    dim_radius_ft: 40,
    color: '#ff9933',
    is_active: true,
    ...overrides,
  }
}

function makeLightSource(overrides: Partial<LightSource> = {}): LightSource {
  return {
    id: 'ls-1',
    map_id: 'map-1',
    token_id: null,
    name: 'Torch',
    light_type: 'torch',
    x: 350,
    y: 350,
    bright_radius_ft: 20,
    dim_radius_ft: 40,
    color: '#ff9933',
    is_active: true,
    created_at: '2024-01-01',
    updated_at: '2024-01-01',
    ...overrides,
  }
}

describe('useLightSources', () => {
  beforeEach(() => {
    vi.clearAllMocks()
  })

  describe('initial state', () => {
    it('starts with empty light sources', () => {
      const ls = useLightSources('map-1')
      expect(ls.lightSources.value).toEqual([])
      expect(ls.hasLightSources.value).toBe(false)
      expect(ls.lightSourceCount.value).toBe(0)
      expect(ls.activeLightCount.value).toBe(0)
    })
  })

  describe('loadLightSources', () => {
    it('loads light sources from backend', async () => {
      const sources = [makeSummary(), makeSummary({ id: 'ls-2', name: 'Lantern' })]
      mockInvoke.mockResolvedValueOnce({ success: true, data: sources })

      const ls = useLightSources('map-1')
      await ls.loadLightSources()

      expect(ls.lightSources.value).toHaveLength(2)
      expect(ls.hasLightSources.value).toBe(true)
      expect(ls.lightSourceCount.value).toBe(2)
    })

    it('sets error on failure', async () => {
      mockInvoke.mockResolvedValueOnce({ success: false, error: 'DB error' })

      const ls = useLightSources('map-1')
      await ls.loadLightSources()

      expect(ls.error.value).toBe('DB error')
    })
  })

  describe('computed properties', () => {
    it('activeLightSources filters by is_active', () => {
      const ls = useLightSources('map-1')
      ls.lightSources.value = [
        makeSummary({ id: 'ls-1', is_active: true }),
        makeSummary({ id: 'ls-2', is_active: false }),
        makeSummary({ id: 'ls-3', is_active: true }),
      ]

      expect(ls.activeLightSources.value).toHaveLength(2)
      expect(ls.activeLightCount.value).toBe(2)
    })
  })

  describe('createLightSource', () => {
    it('creates and reloads list', async () => {
      const created = makeLightSource()
      mockInvoke
        .mockResolvedValueOnce({ success: true, data: created })  // create
        .mockResolvedValueOnce({ success: true, data: [makeSummary()] }) // reload

      const ls = useLightSources('map-1')
      const result = await ls.createLightSource({
        map_id: 'map-1',
        name: 'Torch',
        light_type: 'torch',
        x: 350,
        y: 350,
        bright_radius_ft: 20,
        dim_radius_ft: 40,
      })

      expect(result).toEqual(created)
      expect(ls.lightSources.value).toHaveLength(1) // reloaded
    })

    it('returns null on failure', async () => {
      mockInvoke.mockResolvedValueOnce({ success: false, error: 'Failed' })

      const ls = useLightSources('map-1')
      const result = await ls.createLightSource({
        map_id: 'map-1',
        name: 'Torch',
        light_type: 'torch',
        x: 0,
        y: 0,
        bright_radius_ft: 20,
        dim_radius_ft: 40,
      })

      expect(result).toBeNull()
      expect(ls.error.value).toBe('Failed')
    })
  })

  describe('toggleLightSource', () => {
    it('updates is_active in local state', async () => {
      const toggled = makeLightSource({ is_active: false })
      mockInvoke.mockResolvedValueOnce({ success: true, data: toggled })

      const ls = useLightSources('map-1')
      ls.lightSources.value = [makeSummary({ id: 'ls-1', is_active: true })]

      await ls.toggleLightSource('ls-1')

      expect(ls.lightSources.value[0].is_active).toBe(false)
    })
  })

  describe('moveLightSource', () => {
    it('updates position in local state', async () => {
      const moved = makeLightSource({ x: 500, y: 600 })
      mockInvoke.mockResolvedValueOnce({ success: true, data: moved })

      const ls = useLightSources('map-1')
      ls.lightSources.value = [makeSummary({ id: 'ls-1', x: 350, y: 350 })]

      await ls.moveLightSource('ls-1', 500, 600)

      expect(ls.lightSources.value[0].x).toBe(500)
      expect(ls.lightSources.value[0].y).toBe(600)
    })
  })

  describe('deleteLightSource', () => {
    it('removes from local state', async () => {
      mockInvoke.mockResolvedValueOnce({ success: true })

      const ls = useLightSources('map-1')
      ls.lightSources.value = [
        makeSummary({ id: 'ls-1' }),
        makeSummary({ id: 'ls-2' }),
      ]

      const result = await ls.deleteLightSource('ls-1')

      expect(result).toBe(true)
      expect(ls.lightSources.value).toHaveLength(1)
      expect(ls.lightSources.value[0].id).toBe('ls-2')
    })
  })

  describe('deleteAllLightSources', () => {
    it('clears all light sources', async () => {
      mockInvoke.mockResolvedValueOnce({ success: true })

      const ls = useLightSources('map-1')
      ls.lightSources.value = [makeSummary(), makeSummary({ id: 'ls-2' })]

      const result = await ls.deleteAllLightSources()

      expect(result).toBe(true)
      expect(ls.lightSources.value).toEqual([])
    })
  })

  describe('getLightSource', () => {
    it('finds light source by ID', () => {
      const ls = useLightSources('map-1')
      ls.lightSources.value = [
        makeSummary({ id: 'ls-1', name: 'Torch' }),
        makeSummary({ id: 'ls-2', name: 'Lantern' }),
      ]

      expect(ls.getLightSource('ls-2')?.name).toBe('Lantern')
    })

    it('returns undefined for unknown ID', () => {
      const ls = useLightSources('map-1')
      expect(ls.getLightSource('unknown')).toBeUndefined()
    })
  })

  describe('unit conversion', () => {
    it('feetToPixels converts correctly', () => {
      const ls = useLightSources('map-1')
      // 1 grid = 5ft, so 20ft = 4 grids
      expect(ls.feetToPixels(20, 70)).toBe(280) // 4 * 70
      expect(ls.feetToPixels(5, 70)).toBe(70)   // 1 grid
      expect(ls.feetToPixels(0, 70)).toBe(0)
    })

    it('pixelsToFeet converts correctly', () => {
      const ls = useLightSources('map-1')
      expect(ls.pixelsToFeet(280, 70)).toBe(20) // 4 grids * 5ft
      expect(ls.pixelsToFeet(70, 70)).toBe(5)   // 1 grid
      expect(ls.pixelsToFeet(0, 70)).toBe(0)
    })
  })

  describe('LIGHT_PRESETS', () => {
    it('torch has correct values', () => {
      expect(LIGHT_PRESETS.torch).toEqual({ bright_ft: 20, dim_ft: 40, color: '#ff9933' })
    })

    it('lantern is brighter than torch', () => {
      expect(LIGHT_PRESETS.lantern.bright_ft).toBeGreaterThan(LIGHT_PRESETS.torch.bright_ft)
    })

    it('candle is dimmer than torch', () => {
      expect(LIGHT_PRESETS.candle.bright_ft).toBeLessThan(LIGHT_PRESETS.torch.bright_ft)
    })

    it('all preset types exist', () => {
      expect(Object.keys(LIGHT_PRESETS)).toEqual(['torch', 'lantern', 'candle', 'spell', 'custom'])
    })
  })
})
