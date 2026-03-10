/**
 * Tests for useFog composable.
 *
 * Tests fog state management, reveal operations, and isPointRevealed logic.
 * All invoke calls are mocked.
 */

import { describe, it, expect, beforeEach, vi } from 'vitest'
import { invoke } from '@tauri-apps/api/core'
import { useFog, type FogRevealedArea } from '../useFog'

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}))

const mockInvoke = vi.mocked(invoke)

function makeArea(overrides: Partial<FogRevealedArea> = {}): FogRevealedArea {
  return {
    id: 'area-1',
    map_id: 'map-1',
    x: 100,
    y: 100,
    width: 200,
    height: 200,
    ...overrides,
  }
}

describe('useFog', () => {
  beforeEach(() => {
    vi.clearAllMocks()
  })

  describe('initial state', () => {
    it('starts with fog disabled', () => {
      const fog = useFog('map-1')
      expect(fog.fogEnabled.value).toBe(false)
    })

    it('starts with no revealed areas', () => {
      const fog = useFog('map-1')
      expect(fog.revealedAreas.value).toEqual([])
      expect(fog.hasRevealedAreas.value).toBe(false)
    })

    it('starts with no error and not loading', () => {
      const fog = useFog('map-1')
      expect(fog.error.value).toBeNull()
      expect(fog.loading.value).toBe(false)
    })
  })

  describe('loadFogState', () => {
    it('loads fog state from backend', async () => {
      const areas = [makeArea()]
      mockInvoke.mockResolvedValueOnce({
        success: true,
        data: { fog_enabled: true, revealed_areas: areas },
      })

      const fog = useFog('map-1')
      await fog.loadFogState()

      expect(fog.fogEnabled.value).toBe(true)
      expect(fog.revealedAreas.value).toEqual(areas)
      expect(fog.hasRevealedAreas.value).toBe(true)
    })

    it('sets error on failure', async () => {
      mockInvoke.mockResolvedValueOnce({
        success: false,
        error: 'Database error',
      })

      const fog = useFog('map-1')
      await fog.loadFogState()

      expect(fog.error.value).toBe('Database error')
    })

    it('handles thrown errors', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Network error'))

      const fog = useFog('map-1')
      await fog.loadFogState()

      expect(fog.error.value).toBe('Network error')
    })
  })

  describe('toggleFog', () => {
    it('toggles fog state', async () => {
      mockInvoke.mockResolvedValueOnce({ success: true, data: true })

      const fog = useFog('map-1')
      const result = await fog.toggleFog()

      expect(result).toBe(true)
      expect(fog.fogEnabled.value).toBe(true)
    })

    it('returns current state on error', async () => {
      mockInvoke.mockResolvedValueOnce({ success: false, error: 'Failed' })

      const fog = useFog('map-1')
      const result = await fog.toggleFog()

      expect(result).toBe(false)
      expect(fog.error.value).toBe('Failed')
    })
  })

  describe('enableFog / disableFog', () => {
    it('enableFog sets fogEnabled to true', async () => {
      mockInvoke.mockResolvedValueOnce({ success: true })

      const fog = useFog('map-1')
      const result = await fog.enableFog()

      expect(result).toBe(true)
      expect(fog.fogEnabled.value).toBe(true)
    })

    it('disableFog sets fogEnabled to false', async () => {
      mockInvoke.mockResolvedValueOnce({ success: true })

      const fog = useFog('map-1')
      fog.fogEnabled.value = true
      const result = await fog.disableFog()

      expect(result).toBe(true)
      expect(fog.fogEnabled.value).toBe(false)
    })
  })

  describe('revealRect', () => {
    it('adds revealed area on success', async () => {
      const area = makeArea()
      mockInvoke.mockResolvedValueOnce({ success: true, data: area })

      const fog = useFog('map-1')
      const result = await fog.revealRect(100, 100, 200, 200)

      expect(result).toEqual(area)
      expect(fog.revealedAreas.value).toHaveLength(1)
    })

    it('returns null on failure', async () => {
      mockInvoke.mockResolvedValueOnce({ success: false })

      const fog = useFog('map-1')
      const result = await fog.revealRect(100, 100, 200, 200)

      expect(result).toBeNull()
    })
  })

  describe('revealCircle', () => {
    it('adds revealed area on success', async () => {
      const area = makeArea({ x: 50, y: 50, width: 100, height: 100 })
      mockInvoke.mockResolvedValueOnce({ success: true, data: area })

      const fog = useFog('map-1')
      const result = await fog.revealCircle(100, 100, 50)

      expect(result).toEqual(area)
      expect(fog.revealedAreas.value).toHaveLength(1)
    })
  })

  describe('deleteRevealedArea', () => {
    it('removes area from local state', async () => {
      mockInvoke.mockResolvedValueOnce({ success: true })

      const fog = useFog('map-1')
      fog.revealedAreas.value = [makeArea({ id: 'a1' }), makeArea({ id: 'a2' })]

      const result = await fog.deleteRevealedArea('a1')

      expect(result).toBe(true)
      expect(fog.revealedAreas.value).toHaveLength(1)
      expect(fog.revealedAreas.value[0].id).toBe('a2')
    })
  })

  describe('resetFog', () => {
    it('clears all revealed areas', async () => {
      mockInvoke.mockResolvedValueOnce({ success: true })

      const fog = useFog('map-1')
      fog.revealedAreas.value = [makeArea(), makeArea({ id: 'a2' })]

      const result = await fog.resetFog()

      expect(result).toBe(true)
      expect(fog.revealedAreas.value).toEqual([])
    })
  })

  describe('isPointRevealed', () => {
    it('returns true for point inside revealed area', () => {
      const fog = useFog('map-1')
      fog.revealedAreas.value = [makeArea({ x: 100, y: 100, width: 200, height: 200 })]

      expect(fog.isPointRevealed(150, 150)).toBe(true)
      expect(fog.isPointRevealed(100, 100)).toBe(true) // top-left edge
      expect(fog.isPointRevealed(300, 300)).toBe(true) // bottom-right edge
    })

    it('returns false for point outside revealed area', () => {
      const fog = useFog('map-1')
      fog.revealedAreas.value = [makeArea({ x: 100, y: 100, width: 200, height: 200 })]

      expect(fog.isPointRevealed(50, 50)).toBe(false)
      expect(fog.isPointRevealed(400, 400)).toBe(false)
    })

    it('returns false when no areas revealed', () => {
      const fog = useFog('map-1')
      expect(fog.isPointRevealed(150, 150)).toBe(false)
    })

    it('checks multiple revealed areas', () => {
      const fog = useFog('map-1')
      fog.revealedAreas.value = [
        makeArea({ id: 'a1', x: 0, y: 0, width: 100, height: 100 }),
        makeArea({ id: 'a2', x: 500, y: 500, width: 100, height: 100 }),
      ]

      expect(fog.isPointRevealed(50, 50)).toBe(true)
      expect(fog.isPointRevealed(550, 550)).toBe(true)
      expect(fog.isPointRevealed(300, 300)).toBe(false)
    })
  })
})
