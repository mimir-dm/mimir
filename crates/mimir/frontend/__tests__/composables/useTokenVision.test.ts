/**
 * Tests for useTokenVision composable
 *
 * Tests vision presets, settings extraction, description generation,
 * custom settings detection, and preset matching.
 */

import { describe, it, expect, beforeEach, afterEach } from 'vitest'
import {
  setupInvokeMock,
  resetInvokeMock,
  mockCommand,
  expectCommandCalledWith,
} from '@tests/helpers/mockInvoke'
import { useTokenVision, VISION_PRESETS } from '@/composables/map/useTokenVision'

// ─── Factories ──────────────────────────────────────────────────────────────

function makeToken(overrides: Record<string, unknown> = {}) {
  return {
    id: 'tok-1',
    map_id: 'map-1',
    name: 'Goblin',
    token_type: 'monster',
    size: 'medium',
    color: '#ff0000',
    x: 100,
    y: 200,
    visible_to_players: true,
    vision_bright_ft: null,
    vision_dim_ft: null,
    vision_dark_ft: 0,
    light_radius_ft: 0,
    ...overrides,
  } as any
}

// ─── Tests ──────────────────────────────────────────────────────────────────

describe('useTokenVision', () => {
  beforeEach(() => {
    setupInvokeMock()
  })

  afterEach(() => {
    resetInvokeMock()
  })

  describe('VISION_PRESETS', () => {
    it('has human preset with normal vision', () => {
      expect(VISION_PRESETS.human.vision_dark_ft).toBe(0)
      expect(VISION_PRESETS.human.light_radius_ft).toBe(0)
    })

    it('has darkvision presets', () => {
      expect(VISION_PRESETS.darkvision60.vision_dark_ft).toBe(60)
      expect(VISION_PRESETS.darkvision120.vision_dark_ft).toBe(120)
    })

    it('has blindsight presets', () => {
      expect(VISION_PRESETS.blindsight30.vision_bright_ft).toBe(30)
      expect(VISION_PRESETS.blindsight60.vision_bright_ft).toBe(60)
    })

    it('has torch preset with light radius', () => {
      expect(VISION_PRESETS.humanWithTorch.light_radius_ft).toBe(40)
    })

    it('has lantern preset with light radius', () => {
      expect(VISION_PRESETS.humanWithLantern.light_radius_ft).toBe(60)
    })
  })

  describe('getVisionSettings', () => {
    it('extracts vision settings from token', () => {
      const { getVisionSettings } = useTokenVision()
      const token = makeToken({ vision_dark_ft: 60, light_radius_ft: 40 })

      const settings = getVisionSettings(token)
      expect(settings.vision_dark_ft).toBe(60)
      expect(settings.light_radius_ft).toBe(40)
      expect(settings.vision_bright_ft).toBeNull()
      expect(settings.vision_dim_ft).toBeNull()
    })
  })

  describe('hasCustomSettings', () => {
    it('returns false for default human vision', () => {
      const { hasCustomSettings } = useTokenVision()
      const token = makeToken()
      expect(hasCustomSettings(token)).toBe(false)
    })

    it('returns true for darkvision', () => {
      const { hasCustomSettings } = useTokenVision()
      const token = makeToken({ vision_dark_ft: 60 })
      expect(hasCustomSettings(token)).toBe(true)
    })

    it('returns true for light radius', () => {
      const { hasCustomSettings } = useTokenVision()
      const token = makeToken({ light_radius_ft: 40 })
      expect(hasCustomSettings(token)).toBe(true)
    })

    it('returns true for custom bright vision', () => {
      const { hasCustomSettings } = useTokenVision()
      const token = makeToken({ vision_bright_ft: 30 })
      expect(hasCustomSettings(token)).toBe(true)
    })
  })

  describe('getVisionDescription', () => {
    it('returns Normal vision for defaults', () => {
      const { getVisionDescription } = useTokenVision()
      expect(getVisionDescription(makeToken())).toBe('Normal vision')
    })

    it('describes darkvision', () => {
      const { getVisionDescription } = useTokenVision()
      const token = makeToken({ vision_dark_ft: 60 })
      expect(getVisionDescription(token)).toBe('Darkvision 60 ft.')
    })

    it('describes light sources', () => {
      const { getVisionDescription } = useTokenVision()
      const token = makeToken({ light_radius_ft: 40 })
      expect(getVisionDescription(token)).toBe('Light 20/40 ft.')
    })

    it('combines darkvision and light', () => {
      const { getVisionDescription } = useTokenVision()
      const token = makeToken({ vision_dark_ft: 60, light_radius_ft: 40 })
      expect(getVisionDescription(token)).toBe('Darkvision 60 ft., Light 20/40 ft.')
    })
  })

  describe('findMatchingPreset', () => {
    it('matches human preset', () => {
      const { findMatchingPreset } = useTokenVision()
      expect(findMatchingPreset(makeToken())).toBe('human')
    })

    it('matches darkvision60 preset', () => {
      const { findMatchingPreset } = useTokenVision()
      const token = makeToken({ vision_dark_ft: 60 })
      expect(findMatchingPreset(token)).toBe('darkvision60')
    })

    it('matches humanWithTorch preset', () => {
      const { findMatchingPreset } = useTokenVision()
      const token = makeToken({ light_radius_ft: 40 })
      expect(findMatchingPreset(token)).toBe('humanWithTorch')
    })

    it('returns null for custom settings', () => {
      const { findMatchingPreset } = useTokenVision()
      const token = makeToken({ vision_dark_ft: 45, light_radius_ft: 30 })
      expect(findMatchingPreset(token)).toBeNull()
    })
  })

  describe('updateVisionSettings', () => {
    it('calls update_token_vision with correct args', async () => {
      const { updateVisionSettings } = useTokenVision()
      mockCommand('update_token_vision', makeToken({ vision_dark_ft: 60 }))

      const result = await updateVisionSettings('tok-1', {
        vision_bright_ft: null,
        vision_dim_ft: null,
        vision_dark_ft: 60,
        light_radius_ft: 0,
      })

      expect(result).not.toBeNull()
      expectCommandCalledWith('update_token_vision', {
        id: 'tok-1',
        visionBrightFt: null,
        visionDimFt: null,
        visionDarkFt: 60,
        lightRadiusFt: 0,
      })
    })

    it('returns null on error', async () => {
      const { updateVisionSettings } = useTokenVision()
      // No mock registered, will get default error response
      const result = await updateVisionSettings('tok-1', {
        vision_bright_ft: null,
        vision_dim_ft: null,
        vision_dark_ft: 0,
        light_radius_ft: 0,
      })
      expect(result).toBeNull()
    })
  })

  describe('applyPreset', () => {
    it('applies darkvision60 preset', async () => {
      const { applyPreset } = useTokenVision()
      mockCommand('update_token_vision', makeToken({ vision_dark_ft: 60 }))

      const result = await applyPreset('tok-1', 'darkvision60')
      expect(result).not.toBeNull()
      expectCommandCalledWith('update_token_vision', {
        id: 'tok-1',
        visionDarkFt: 60,
        lightRadiusFt: 0,
      })
    })
  })
})
