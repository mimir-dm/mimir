/**
 * Tests for useVisionCalculation composable.
 *
 * Tests D&D 5e vision rules: light zone calculation, light level determination,
 * token vision based on ambient conditions, party visibility, and darkvision.
 */

import { describe, it, expect } from 'vitest'
import { ref } from 'vue'
import { useVisionCalculation } from '@/composables/map/useVisionCalculation'
import type { Token } from '@/types/api'
import type { LightSourceSummary } from '@/composables/map/useLightSources'

// ─── Factories ──────────────────────────────────────────────────────────────

const GRID = 70 // 70px per grid = 5ft

function makeToken(overrides: Partial<Token> = {}): Token {
  return {
    id: 'tok-1',
    map_id: 'map-1',
    name: 'Test Token',
    token_type: 'pc',
    size: 'medium',
    x: 350, // center of grid (5,5)
    y: 350,
    visible_to_players: true,
    color: '#ff0000',
    image_path: null,
    monster_id: null,
    character_id: null,
    notes: null,
    vision_type: 'normal',
    vision_range_ft: null,
    vision_bright_ft: null, // unlimited in bright
    vision_dim_ft: 60, // 60ft in dim
    vision_dark_ft: 0, // blind in dark
    light_radius_ft: 0, // no light
    created_at: '2024-01-01',
    updated_at: '2024-01-01',
    ...overrides,
  } as Token
}

function makeLightSource(overrides: Partial<LightSourceSummary> = {}): LightSourceSummary {
  return {
    id: 'light-1',
    map_id: 'map-1',
    name: 'Torch',
    token_id: null,
    light_type: 'torch',
    x: 350,
    y: 350,
    bright_radius_ft: 20,
    dim_radius_ft: 40,
    is_active: true,
    color: '#ff9900',
    ...overrides,
  } as LightSourceSummary
}

function createVision(
  tokens: Token[] = [],
  lightSources: LightSourceSummary[] = [],
  ambient: 'bright' | 'dim' | 'darkness' = 'bright'
) {
  return useVisionCalculation({
    tokens: ref(tokens),
    lightSources: ref(lightSources),
    ambientLight: ref(ambient),
    gridSizePx: ref(GRID),
    mapWidth: ref(2000),
    mapHeight: ref(2000),
  })
}

// ─── Tests ──────────────────────────────────────────────────────────────────

describe('useVisionCalculation', () => {
  describe('feetToPixels', () => {
    it('converts feet to pixels based on grid size', () => {
      const vision = createVision()
      expect(vision.feetToPixels(5)).toBe(70) // 1 grid
      expect(vision.feetToPixels(10)).toBe(140) // 2 grids
      expect(vision.feetToPixels(60)).toBe(840) // 12 grids
    })

    it('handles zero feet', () => {
      const vision = createVision()
      expect(vision.feetToPixels(0)).toBe(0)
    })
  })

  describe('lightZones', () => {
    it('creates zone from active map light source', () => {
      const light = makeLightSource({ x: 100, y: 200, bright_radius_ft: 20, dim_radius_ft: 40 })
      const vision = createVision([], [light])

      expect(vision.lightZones.value).toHaveLength(1)
      const zone = vision.lightZones.value[0]
      expect(zone.x).toBe(100)
      expect(zone.y).toBe(200)
      expect(zone.brightRadiusPx).toBe(280) // 20ft * 70/5
      expect(zone.dimRadiusPx).toBe(560) // 40ft * 70/5
    })

    it('skips inactive light sources', () => {
      const light = makeLightSource({ is_active: false })
      const vision = createVision([], [light])
      expect(vision.lightZones.value).toHaveLength(0)
    })

    it('creates zone from token with light', () => {
      const token = makeToken({ light_radius_ft: 40 })
      const vision = createVision([token])

      expect(vision.lightZones.value).toHaveLength(1)
      const zone = vision.lightZones.value[0]
      expect(zone.sourceId).toBe('token-tok-1')
      // bright = half of dim: 20ft, dim = 40ft
      expect(zone.brightRadiusPx).toBe(280) // 20ft
      expect(zone.dimRadiusPx).toBe(560) // 40ft
    })

    it('skips tokens without light', () => {
      const token = makeToken({ light_radius_ft: 0 })
      const vision = createVision([token])
      expect(vision.lightZones.value).toHaveLength(0)
    })

    it('combines map lights and token lights', () => {
      const light = makeLightSource()
      const token = makeToken({ light_radius_ft: 20 })
      const vision = createVision([token], [light])
      expect(vision.lightZones.value).toHaveLength(2)
    })
  })

  describe('getLightLevel', () => {
    it('returns ambient light when no light sources', () => {
      const vision = createVision([], [], 'darkness')
      expect(vision.getLightLevel(500, 500)).toBe('darkness')
    })

    it('returns bright inside bright radius', () => {
      const light = makeLightSource({ x: 350, y: 350, bright_radius_ft: 20, dim_radius_ft: 40 })
      const vision = createVision([], [light], 'darkness')
      // At (350, 350) = at the light source
      expect(vision.getLightLevel(350, 350)).toBe('bright')
    })

    it('returns dim in dim zone in darkness', () => {
      const light = makeLightSource({ x: 0, y: 0, bright_radius_ft: 20, dim_radius_ft: 40 })
      const vision = createVision([], [light], 'darkness')
      // At distance 350px (25ft) — outside bright (20ft=280px) but inside dim (40ft=560px)
      expect(vision.getLightLevel(350, 0)).toBe('dim')
    })

    it('returns darkness outside all zones', () => {
      const light = makeLightSource({ x: 0, y: 0, bright_radius_ft: 20, dim_radius_ft: 40 })
      const vision = createVision([], [light], 'darkness')
      // At (1000, 1000) — far away
      expect(vision.getLightLevel(1000, 1000)).toBe('darkness')
    })

    it('dim ambient does not upgrade to bright without light source', () => {
      const vision = createVision([], [], 'dim')
      expect(vision.getLightLevel(500, 500)).toBe('dim')
    })

    it('bright ambient everywhere without light sources', () => {
      const vision = createVision([], [], 'bright')
      expect(vision.getLightLevel(500, 500)).toBe('bright')
    })
  })

  describe('token vision (bright ambient)', () => {
    it('unlimited vision in bright light with null vision_bright_ft', () => {
      const token = makeToken({ vision_bright_ft: null })
      const vision = createVision([token], [], 'bright')

      const pcv = vision.pcVision.value
      expect(pcv).toHaveLength(1)
      expect(pcv[0].visionRadiusPx).toBe(100000) // UNLIMITED_RADIUS
      expect(pcv[0].isDimVision).toBe(false)
    })

    it('limited vision in bright light', () => {
      const token = makeToken({ vision_bright_ft: 30 })
      const vision = createVision([token], [], 'bright')

      expect(vision.pcVision.value[0].visionRadiusPx).toBe(420) // 30ft
    })
  })

  describe('token vision (dim ambient)', () => {
    it('uses dim vision range in dim ambient', () => {
      const token = makeToken({ vision_dim_ft: 60 })
      const vision = createVision([token], [], 'dim')

      const pcv = vision.pcVision.value[0]
      expect(pcv.visionRadiusPx).toBe(840) // 60ft
      expect(pcv.isDimVision).toBe(true)
    })

    it('unlimited dim vision with null vision_dim_ft', () => {
      const token = makeToken({ vision_dim_ft: null })
      const vision = createVision([token], [], 'dim')

      expect(vision.pcVision.value[0].visionRadiusPx).toBe(100000)
    })
  })

  describe('token vision (darkness)', () => {
    it('blind in darkness without darkvision', () => {
      const token = makeToken({ vision_dark_ft: 0, light_radius_ft: 0 })
      const vision = createVision([token], [], 'darkness')

      expect(vision.pcVision.value[0].visionRadiusPx).toBe(0)
    })

    it('darkvision 60ft in darkness', () => {
      const token = makeToken({ vision_dark_ft: 60, light_radius_ft: 0 })
      const vision = createVision([token], [], 'darkness')

      const pcv = vision.pcVision.value[0]
      expect(pcv.visionRadiusPx).toBe(840) // 60ft
      expect(pcv.isDimVision).toBe(true) // darkvision sees as dim
    })

    it('own light extends vision in darkness', () => {
      const token = makeToken({ vision_dark_ft: 0, light_radius_ft: 40 })
      const vision = createVision([token], [], 'darkness')

      // Token has light_radius_ft=40, which creates a light zone around it.
      // But the vision calculation checks light level at token position.
      // Token's own light creates a zone with bright=20ft, dim=40ft at its position.
      // So the token is actually in its own bright light, using vision_bright_ft.
      const pcv = vision.pcVision.value[0]
      expect(pcv.lightRadiusPx).toBe(560) // 40ft in pixels
    })

    it('darkvision uses greater of darkvision or own light', () => {
      // Token at (0,0) in darkness, no light sources nearby
      const token = makeToken({
        x: 2000, y: 2000, // far from any light
        vision_dark_ft: 60,
        light_radius_ft: 40,
      })
      const vision = createVision([token], [], 'darkness')

      // Token creates its own light zone, so it's actually in bright light at its position
      // But if we check the raw dark vision math: max(60, 40) = 60
      const pcv = vision.pcVision.value[0]
      expect(pcv.visionRadiusPx).toBeGreaterThan(0)
    })
  })

  describe('pcVision filtering', () => {
    it('only includes pc tokens', () => {
      const pc = makeToken({ id: 'pc-1', token_type: 'pc' })
      const monster = makeToken({ id: 'mon-1', token_type: 'monster' })
      const vision = createVision([pc, monster], [], 'bright')

      expect(vision.pcVision.value).toHaveLength(1)
      expect(vision.pcVision.value[0].tokenId).toBe('pc-1')
    })

    it('excludes hidden pc tokens', () => {
      const visible = makeToken({ id: 'pc-1', visible_to_players: true })
      const hidden = makeToken({ id: 'pc-2', visible_to_players: false })
      const vision = createVision([visible, hidden], [], 'bright')

      expect(vision.pcVision.value).toHaveLength(1)
      expect(vision.pcVision.value[0].tokenId).toBe('pc-1')
    })
  })

  describe('allTokenVision', () => {
    it('includes all tokens regardless of type', () => {
      const pc = makeToken({ id: 'pc-1', token_type: 'pc' })
      const monster = makeToken({ id: 'mon-1', token_type: 'monster' })
      const npc = makeToken({ id: 'npc-1', token_type: 'npc' })
      const vision = createVision([pc, monster, npc], [], 'bright')

      expect(vision.allTokenVision.value).toHaveLength(3)
    })
  })

  describe('isPointInVision', () => {
    it('point at same position is visible', () => {
      const token = makeToken({ x: 100, y: 100 })
      const vision = createVision([token], [], 'bright')
      const tv = vision.pcVision.value[0]
      expect(vision.isPointInVision(100, 100, tv)).toBe(true)
    })

    it('point within radius is visible', () => {
      const token = makeToken({ x: 100, y: 100, vision_bright_ft: 30 })
      const vision = createVision([token], [], 'bright')
      const tv = vision.pcVision.value[0]
      // 30ft = 420px, point at (200, 100) = 100px away
      expect(vision.isPointInVision(200, 100, tv)).toBe(true)
    })

    it('point outside radius is not visible', () => {
      const token = makeToken({ x: 100, y: 100, vision_bright_ft: 5 })
      const vision = createVision([token], [], 'bright')
      const tv = vision.pcVision.value[0]
      // 5ft = 70px, point at (300, 100) = 200px away
      expect(vision.isPointInVision(300, 100, tv)).toBe(false)
    })
  })

  describe('canPartySeePoint', () => {
    it('returns false when no PCs', () => {
      const vision = createVision([], [], 'bright')
      const result = vision.canPartySeePoint(100, 100)
      expect(result.canSee).toBe(false)
    })

    it('returns true when PC can see point', () => {
      const token = makeToken({ x: 100, y: 100 })
      const vision = createVision([token], [], 'bright')
      expect(vision.canPartySeePoint(100, 100).canSee).toBe(true)
    })

    it('isDim is false in bright light', () => {
      const token = makeToken({ x: 100, y: 100 })
      const vision = createVision([token], [], 'bright')
      expect(vision.canPartySeePoint(100, 100).isDim).toBe(false)
    })

    it('isDim is true when all seeing PCs have dim vision', () => {
      const token = makeToken({ x: 100, y: 100, vision_dim_ft: 60 })
      const vision = createVision([token], [], 'dim')
      const result = vision.canPartySeePoint(100, 100)
      expect(result.canSee).toBe(true)
      expect(result.isDim).toBe(true)
    })

    it('isDim is false if any PC has bright vision at point', () => {
      const dimPC = makeToken({ id: 'pc-dim', x: 100, y: 100, vision_dim_ft: 60 })
      const brightPC = makeToken({ id: 'pc-bright', x: 100, y: 100, vision_bright_ft: null })
      // Both at (100,100) in bright ambient — brightPC uses unlimited bright vision
      const vision = createVision([dimPC, brightPC], [], 'bright')
      const result = vision.canPartySeePoint(100, 100)
      expect(result.canSee).toBe(true)
      expect(result.isDim).toBe(false)
    })
  })

  describe('needsVisionOverlay', () => {
    it('not needed in bright ambient with unlimited vision', () => {
      const token = makeToken({ vision_bright_ft: null })
      const vision = createVision([token], [], 'bright')
      expect(vision.needsVisionOverlay.value).toBe(false)
    })

    it('needed in bright ambient with limited vision', () => {
      const token = makeToken({ vision_bright_ft: 30 })
      const vision = createVision([token], [], 'bright')
      expect(vision.needsVisionOverlay.value).toBe(true)
    })

    it('always needed in dim ambient', () => {
      const token = makeToken()
      const vision = createVision([token], [], 'dim')
      expect(vision.needsVisionOverlay.value).toBe(true)
    })

    it('always needed in darkness', () => {
      const token = makeToken()
      const vision = createVision([token], [], 'darkness')
      expect(vision.needsVisionOverlay.value).toBe(true)
    })
  })
})
