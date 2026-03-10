/**
 * Tests for UVTT data conversion functions from useVisibilityPolygon.
 *
 * Tests wall/portal/light coordinate conversion from grid units to pixels,
 * ambient light level parsing from ARGB hex, and edge cases.
 */

import { describe, it, expect } from 'vitest'
import {
  uvttWallsToPixels,
  uvttPortalsToPixels,
  uvttLightsToPixels,
  uvttAmbientToLevel,
  type UvttData,
} from '../useVisibilityPolygon'

function makeUvttData(overrides: Partial<UvttData> = {}): UvttData {
  return {
    format: 1,
    resolution: {
      map_size: { x: 30, y: 20 },
      pixels_per_grid: 70,
    },
    ...overrides,
  }
}

describe('uvttAmbientToLevel', () => {
  it('returns bright for undefined input', () => {
    expect(uvttAmbientToLevel(undefined)).toBe('bright')
  })

  it('returns bright for white ARGB', () => {
    expect(uvttAmbientToLevel('ffffffff')).toBe('bright')
  })

  it('returns darkness for black ARGB', () => {
    expect(uvttAmbientToLevel('ff000000')).toBe('darkness')
  })

  it('returns dim for mid-gray ARGB', () => {
    // RGB (128,128,128) -> brightness = 128 -> between 85 and 170 = dim
    expect(uvttAmbientToLevel('ff808080')).toBe('dim')
  })

  it('returns bright for short hex strings', () => {
    expect(uvttAmbientToLevel('fff')).toBe('bright')
  })

  it('handles hash prefix', () => {
    expect(uvttAmbientToLevel('#ff000000')).toBe('darkness')
  })
})

describe('uvttWallsToPixels', () => {
  it('returns empty array when no line_of_sight', () => {
    const uvtt = makeUvttData()
    expect(uvttWallsToPixels(uvtt)).toEqual([])
  })

  it('converts a single wall segment from grid to pixels', () => {
    const uvtt = makeUvttData({
      line_of_sight: [
        [{ x: 1, y: 2 }, { x: 3, y: 4 }],
      ],
    })

    const walls = uvttWallsToPixels(uvtt)
    expect(walls).toHaveLength(1)
    expect(walls[0].p1).toEqual({ x: 70, y: 140 })
    expect(walls[0].p2).toEqual({ x: 210, y: 280 })
  })

  it('converts polyline to multiple wall segments', () => {
    const uvtt = makeUvttData({
      line_of_sight: [
        [{ x: 0, y: 0 }, { x: 1, y: 0 }, { x: 1, y: 1 }],
      ],
    })

    const walls = uvttWallsToPixels(uvtt)
    expect(walls).toHaveLength(2)
    expect(walls[0].p1).toEqual({ x: 0, y: 0 })
    expect(walls[0].p2).toEqual({ x: 70, y: 0 })
    expect(walls[1].p1).toEqual({ x: 70, y: 0 })
    expect(walls[1].p2).toEqual({ x: 70, y: 70 })
  })

  it('handles multiple wall segments', () => {
    const uvtt = makeUvttData({
      line_of_sight: [
        [{ x: 0, y: 0 }, { x: 1, y: 0 }],
        [{ x: 2, y: 2 }, { x: 3, y: 3 }],
      ],
    })

    const walls = uvttWallsToPixels(uvtt)
    expect(walls).toHaveLength(2)
  })

  it('uses default pixels_per_grid of 70', () => {
    const uvtt: UvttData = {
      resolution: {
        map_size: { x: 10, y: 10 },
        pixels_per_grid: 0, // falsy => default
      },
      line_of_sight: [
        [{ x: 1, y: 0 }, { x: 2, y: 0 }],
      ],
    }

    const walls = uvttWallsToPixels(uvtt)
    expect(walls[0].p1.x).toBe(70)
    expect(walls[0].p2.x).toBe(140)
  })
})

describe('uvttPortalsToPixels', () => {
  it('returns empty array when no portals', () => {
    const uvtt = makeUvttData()
    expect(uvttPortalsToPixels(uvtt)).toEqual([])
  })

  it('converts portal bounds to pixel coordinates', () => {
    const uvtt = makeUvttData({
      portals: [{
        position: { x: 5, y: 5 },
        bounds: [{ x: 4, y: 5 }, { x: 6, y: 5 }],
        rotation: 0,
        closed: true,
        freestanding: false,
      }],
    })

    const portals = uvttPortalsToPixels(uvtt)
    expect(portals).toHaveLength(1)
    expect(portals[0].id).toBe('portal-0')
    expect(portals[0].closed).toBe(true)
    expect(portals[0].wall.p1).toEqual({ x: 280, y: 350 })
    expect(portals[0].wall.p2).toEqual({ x: 420, y: 350 })
  })

  it('preserves closed state from UVTT data', () => {
    const uvtt = makeUvttData({
      portals: [
        {
          position: { x: 0, y: 0 },
          bounds: [{ x: 0, y: 0 }, { x: 1, y: 0 }],
          rotation: 0,
          closed: false,
          freestanding: false,
        },
        {
          position: { x: 0, y: 0 },
          bounds: [{ x: 0, y: 0 }, { x: 1, y: 0 }],
          rotation: 0,
          closed: true,
          freestanding: false,
        },
      ],
    })

    const portals = uvttPortalsToPixels(uvtt)
    expect(portals[0].closed).toBe(false)
    expect(portals[1].closed).toBe(true)
  })
})

describe('uvttLightsToPixels', () => {
  it('returns empty array when no lights', () => {
    const uvtt = makeUvttData()
    expect(uvttLightsToPixels(uvtt)).toEqual([])
  })

  it('converts light position and range to pixels', () => {
    const uvtt = makeUvttData({
      lights: [{
        position: { x: 5, y: 3 },
        range: 4,       // 4 grid cells
        intensity: 1,
        color: 'ffeccd8b',
        shadows: true,
      }],
    })

    const lights = uvttLightsToPixels(uvtt)
    expect(lights).toHaveLength(1)
    expect(lights[0].id).toBe('light-0')
    expect(lights[0].position).toEqual({ x: 350, y: 210 })
    expect(lights[0].range).toBe(280) // 4 * 70
    expect(lights[0].intensity).toBe(1)
    expect(lights[0].shadows).toBe(true)
  })

  it('converts ARGB color to CSS rgba', () => {
    const uvtt = makeUvttData({
      lights: [{
        position: { x: 0, y: 0 },
        range: 1,
        intensity: 1,
        color: 'ffff0000', // A:ff R:ff G:00 B:00
        shadows: false,
      }],
    })

    const lights = uvttLightsToPixels(uvtt)
    expect(lights[0].color).toBe('rgba(255, 0, 0, 1)')
  })

  it('handles multiple lights', () => {
    const uvtt = makeUvttData({
      lights: [
        { position: { x: 1, y: 1 }, range: 2, intensity: 1, color: 'ffffffff', shadows: true },
        { position: { x: 5, y: 5 }, range: 3, intensity: 0.5, color: 'ff000000', shadows: false },
      ],
    })

    const lights = uvttLightsToPixels(uvtt)
    expect(lights).toHaveLength(2)
    expect(lights[0].id).toBe('light-0')
    expect(lights[1].id).toBe('light-1')
  })
})
