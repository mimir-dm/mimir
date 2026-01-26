/**
 * Composable for managing token vision settings.
 *
 * Vision settings are stored in the backend (token_placements table).
 * This composable provides:
 * - Preset definitions for common vision types
 * - Helper to update vision via backend API
 * - Utility functions for vision calculations
 */
import { invoke } from '@tauri-apps/api/core'
import type { Token } from '@/types/api'

/** Vision settings structure (matches backend fields) */
export interface TokenVisionSettings {
  vision_bright_ft: number | null  // null = unlimited
  vision_dim_ft: number | null     // null = unlimited
  vision_dark_ft: number           // 0 = blind
  light_radius_ft: number          // 0 = no light
}

/** Common presets for quick setup */
export const VISION_PRESETS = {
  human: {
    label: 'Human (Normal)',
    vision_bright_ft: null,
    vision_dim_ft: null,
    vision_dark_ft: 0,
    light_radius_ft: 0
  },
  humanWithTorch: {
    label: 'Human + Torch',
    vision_bright_ft: null,
    vision_dim_ft: null,
    vision_dark_ft: 0,
    light_radius_ft: 40
  },
  humanWithLantern: {
    label: 'Human + Lantern',
    vision_bright_ft: null,
    vision_dim_ft: null,
    vision_dark_ft: 0,
    light_radius_ft: 60
  },
  darkvision60: {
    label: 'Darkvision 60 ft.',
    vision_bright_ft: null,
    vision_dim_ft: null,
    vision_dark_ft: 60,
    light_radius_ft: 0
  },
  darkvision120: {
    label: 'Darkvision 120 ft.',
    vision_bright_ft: null,
    vision_dim_ft: null,
    vision_dark_ft: 120,
    light_radius_ft: 0
  },
  blindsight30: {
    label: 'Blindsight 30 ft.',
    vision_bright_ft: 30,
    vision_dim_ft: 30,
    vision_dark_ft: 30,
    light_radius_ft: 0
  },
  blindsight60: {
    label: 'Blindsight 60 ft.',
    vision_bright_ft: 60,
    vision_dim_ft: 60,
    vision_dark_ft: 60,
    light_radius_ft: 0
  },
  devilsSight: {
    label: "Devil's Sight 120 ft.",
    vision_bright_ft: null,
    vision_dim_ft: null,
    vision_dark_ft: 120,
    light_radius_ft: 0
  }
} as const

export type VisionPresetKey = keyof typeof VISION_PRESETS

interface ApiResponse<T> {
  success: boolean
  data?: T
  error?: string
}

/**
 * Composable for managing token vision settings.
 */
export function useTokenVision() {
  /**
   * Get vision settings from a token object.
   */
  function getVisionSettings(token: Token): TokenVisionSettings {
    return {
      vision_bright_ft: token.vision_bright_ft,
      vision_dim_ft: token.vision_dim_ft,
      vision_dark_ft: token.vision_dark_ft,
      light_radius_ft: token.light_radius_ft
    }
  }

  /**
   * Update vision settings for a token via backend API.
   * Returns the updated token on success, null on failure.
   */
  async function updateVisionSettings(
    tokenId: string,
    settings: TokenVisionSettings
  ): Promise<Token | null> {
    try {
      const response = await invoke<ApiResponse<Token>>('update_token_vision', {
        id: tokenId,
        visionBrightFt: settings.vision_bright_ft,
        visionDimFt: settings.vision_dim_ft,
        visionDarkFt: settings.vision_dark_ft,
        lightRadiusFt: settings.light_radius_ft
      })
      if (response.success && response.data) {
        return response.data
      }
      console.error('Failed to update token vision:', response.error)
      return null
    } catch (e) {
      console.error('Failed to update token vision:', e)
      return null
    }
  }

  /**
   * Apply a preset to a token.
   * Returns the updated token on success, null on failure.
   */
  async function applyPreset(
    tokenId: string,
    presetKey: VisionPresetKey
  ): Promise<Token | null> {
    const preset = VISION_PRESETS[presetKey]
    return updateVisionSettings(tokenId, {
      vision_bright_ft: preset.vision_bright_ft,
      vision_dim_ft: preset.vision_dim_ft,
      vision_dark_ft: preset.vision_dark_ft,
      light_radius_ft: preset.light_radius_ft
    })
  }

  /**
   * Reset vision to default (human normal).
   */
  async function resetToDefault(tokenId: string): Promise<Token | null> {
    return applyPreset(tokenId, 'human')
  }

  /**
   * Check if a token has non-default vision settings.
   */
  function hasCustomSettings(token: Token): boolean {
    return (
      token.vision_bright_ft !== null ||
      token.vision_dim_ft !== null ||
      token.vision_dark_ft !== 0 ||
      token.light_radius_ft !== 0
    )
  }

  /**
   * Get description of token's current vision.
   */
  function getVisionDescription(token: Token): string {
    const parts: string[] = []

    if (token.vision_dark_ft > 0) {
      parts.push(`Darkvision ${token.vision_dark_ft} ft.`)
    }

    if (token.light_radius_ft > 0) {
      const brightRadius = token.light_radius_ft / 2
      parts.push(`Light ${brightRadius}/${token.light_radius_ft} ft.`)
    }

    if (parts.length === 0) {
      return 'Normal vision'
    }

    return parts.join(', ')
  }

  /**
   * Find the preset that matches a token's settings (if any).
   */
  function findMatchingPreset(token: Token): VisionPresetKey | null {
    for (const [key, preset] of Object.entries(VISION_PRESETS)) {
      if (
        token.vision_bright_ft === preset.vision_bright_ft &&
        token.vision_dim_ft === preset.vision_dim_ft &&
        token.vision_dark_ft === preset.vision_dark_ft &&
        token.light_radius_ft === preset.light_radius_ft
      ) {
        return key as VisionPresetKey
      }
    }
    return null
  }

  return {
    // Presets
    VISION_PRESETS,
    // Methods
    getVisionSettings,
    updateVisionSettings,
    applyPreset,
    resetToDefault,
    hasCustomSettings,
    getVisionDescription,
    findMatchingPreset
  }
}
