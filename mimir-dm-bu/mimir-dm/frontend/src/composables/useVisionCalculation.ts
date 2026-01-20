/**
 * Composable for calculating composite party vision.
 *
 * Determines what players can collectively see based on their tokens'
 * positions, vision types, vision ranges, and current lighting conditions.
 */
import { computed, type Ref } from 'vue'
import type { Token, VisionType } from '@/types/api'
import type { LightSourceSummary } from './useLightSources'

/** Ambient light levels matching D&D 5e */
export type AmbientLight = 'bright' | 'dim' | 'darkness'

/** Light level at a specific point */
export type LightLevel = 'bright' | 'dim' | 'darkness'

/** Visible area circle for a token */
export interface VisibleArea {
  tokenId: number
  x: number
  y: number
  /** Radius in pixels where token has full clear vision */
  brightVisionRadiusPx: number
  /** Radius in pixels where token has dim vision (disadvantage on Perception) */
  dimVisionRadiusPx: number
  /** Whether this is special vision that ignores light (blindsight, tremorsense) */
  ignoresLight: boolean
}

/** Light zone on the map */
export interface LightZone {
  lightSourceId: number
  x: number
  y: number
  brightRadiusPx: number
  dimRadiusPx: number
}

/** Configuration for vision calculation */
export interface VisionCalculationConfig {
  /** Tokens to calculate vision for (typically visible player tokens) */
  tokens: Ref<Token[]>
  /** Light sources on the map */
  lightSources: Ref<LightSourceSummary[]>
  /** Map ambient light level */
  ambientLight: Ref<AmbientLight>
  /** Grid size in pixels (1 grid = 5 feet in D&D) */
  gridSizePx: Ref<number>
  /** Map dimensions for bounds checking */
  mapWidth: Ref<number>
  mapHeight: Ref<number>
}

/** Convert feet to pixels based on grid size (1 grid square = 5 feet) */
function feetToPixels(feet: number, gridSizePx: number): number {
  return (feet / 5) * gridSizePx
}

/**
 * Calculate visibility radii for a token based on its vision type
 * and the ambient light level.
 */
function calculateTokenVision(
  token: Token,
  ambientLight: AmbientLight,
  lightZones: LightZone[],
  gridSizePx: number
): VisibleArea {
  const visionType = (token.vision_type || 'normal') as VisionType
  const visionRangeFt = token.vision_range_ft || 0
  const visionRangePx = feetToPixels(visionRangeFt, gridSizePx)

  // Default: no special vision
  let brightVisionRadiusPx = 0
  let dimVisionRadiusPx = 0
  let ignoresLight = false

  // Calculate based on vision type
  switch (visionType) {
    case 'normal':
      // Normal vision depends entirely on light sources and ambient light
      // In bright light: can see normally
      // In dim light: can see but with disadvantage
      // In darkness: can't see
      if (ambientLight === 'bright') {
        // Can see everything (limited only by map bounds)
        brightVisionRadiusPx = Infinity
      } else if (ambientLight === 'dim') {
        // Can see but with disadvantage
        dimVisionRadiusPx = Infinity
      }
      // In darkness, normal vision sees nothing without light
      break

    case 'darkvision':
      // Darkvision: treat dim light as bright, darkness as dim (within range)
      if (ambientLight === 'bright') {
        brightVisionRadiusPx = Infinity
      } else if (ambientLight === 'dim') {
        // Treat dim as bright within darkvision range
        brightVisionRadiusPx = visionRangePx
        dimVisionRadiusPx = Infinity // Beyond range, still dim vision
      } else {
        // Darkness: treat as dim within range
        dimVisionRadiusPx = visionRangePx
      }
      break

    case 'blindsight':
    case 'tremorsense':
      // These ignore light completely within range
      ignoresLight = true
      brightVisionRadiusPx = visionRangePx
      // Beyond blindsight range, fall back to normal vision
      if (ambientLight === 'bright') {
        dimVisionRadiusPx = Infinity
      } else if (ambientLight === 'dim') {
        dimVisionRadiusPx = Infinity
      }
      break

    case 'truesight':
      // Truesight: sees through magical darkness, illusions, etc.
      // Treat as full vision within range
      ignoresLight = true
      brightVisionRadiusPx = visionRangePx
      // Beyond truesight range, normal darkvision-like behavior
      if (ambientLight !== 'darkness') {
        dimVisionRadiusPx = Infinity
      }
      break

    case 'devils_sight':
      // Devil's Sight: can see normally in darkness (magical or nonmagical) within range
      // Effectively darkvision that works in magical darkness too
      if (ambientLight === 'bright') {
        brightVisionRadiusPx = Infinity
      } else {
        // In dim or darkness, see normally within range
        brightVisionRadiusPx = visionRangePx
        if (ambientLight === 'dim') {
          dimVisionRadiusPx = Infinity
        }
      }
      break
  }

  return {
    tokenId: token.id,
    x: token.x,
    y: token.y,
    brightVisionRadiusPx,
    dimVisionRadiusPx,
    ignoresLight
  }
}

/**
 * Calculate light zones from light sources.
 */
function calculateLightZones(
  lightSources: LightSourceSummary[],
  tokens: Token[],
  gridSizePx: number
): LightZone[] {
  return lightSources
    .filter(light => light.is_active)
    .map(light => {
      // Get position (use token position if attached)
      let x = light.x
      let y = light.y
      if (light.token_id) {
        const token = tokens.find(t => t.id === light.token_id)
        if (token) {
          x = token.x
          y = token.y
        }
      }

      return {
        lightSourceId: light.id,
        x,
        y,
        brightRadiusPx: feetToPixels(light.bright_radius_ft, gridSizePx),
        dimRadiusPx: feetToPixels(light.dim_radius_ft, gridSizePx)
      }
    })
}

/**
 * Check if a point is within a light zone and return the light level.
 */
function getLightLevelAtPoint(
  x: number,
  y: number,
  lightZones: LightZone[],
  ambientLight: AmbientLight
): LightLevel {
  let bestLight: LightLevel = ambientLight

  for (const zone of lightZones) {
    const dx = x - zone.x
    const dy = y - zone.y
    const distance = Math.sqrt(dx * dx + dy * dy)

    if (distance <= zone.brightRadiusPx) {
      // In bright light zone
      return 'bright'
    } else if (distance <= zone.dimRadiusPx) {
      // In dim light zone - upgrade from darkness or ambient dim
      if (bestLight === 'darkness') {
        bestLight = 'dim'
      }
    }
  }

  return bestLight
}

/**
 * Composable for calculating composite party vision.
 */
export function useVisionCalculation(config: VisionCalculationConfig) {
  const { tokens, lightSources, ambientLight, gridSizePx, mapWidth, mapHeight } = config

  /** Light zones calculated from light sources */
  const lightZones = computed<LightZone[]>(() => {
    return calculateLightZones(
      lightSources.value,
      tokens.value,
      gridSizePx.value
    )
  })

  /** Visible areas for each token (only PC tokens create player vision) */
  const tokenVisibleAreas = computed<VisibleArea[]>(() => {
    return tokens.value
      .filter(t => t.visible_to_players && t.token_type === 'pc')
      .map(token => calculateTokenVision(
        token,
        ambientLight.value,
        lightZones.value,
        gridSizePx.value
      ))
  })

  /**
   * Calculate the effective vision radius at a point for a specific token.
   * This accounts for light sources that might extend vision.
   */
  function getEffectiveVisionAtPoint(
    tokenArea: VisibleArea,
    pointX: number,
    pointY: number
  ): { canSee: boolean; isDim: boolean } {
    const dx = pointX - tokenArea.x
    const dy = pointY - tokenArea.y
    const distance = Math.sqrt(dx * dx + dy * dy)

    // Check if token ignores light (blindsight, etc.)
    if (tokenArea.ignoresLight) {
      if (distance <= tokenArea.brightVisionRadiusPx) {
        return { canSee: true, isDim: false }
      }
    }

    // Check if within bright vision radius
    if (tokenArea.brightVisionRadiusPx === Infinity || distance <= tokenArea.brightVisionRadiusPx) {
      // Check light level at the point
      const lightLevel = getLightLevelAtPoint(pointX, pointY, lightZones.value, ambientLight.value)
      if (lightLevel === 'bright') {
        return { canSee: true, isDim: false }
      }
      if (lightLevel === 'dim') {
        return { canSee: true, isDim: true }
      }
    }

    // Check if within dim vision radius
    if (tokenArea.dimVisionRadiusPx === Infinity || distance <= tokenArea.dimVisionRadiusPx) {
      const lightLevel = getLightLevelAtPoint(pointX, pointY, lightZones.value, ambientLight.value)
      if (lightLevel !== 'darkness') {
        return { canSee: true, isDim: true }
      }
      // Darkvision can see in darkness as dim
      if (tokenArea.dimVisionRadiusPx > 0 && distance <= tokenArea.dimVisionRadiusPx) {
        return { canSee: true, isDim: true }
      }
    }

    return { canSee: false, isDim: false }
  }

  /**
   * Check if any party member can see a specific point.
   */
  function canPartySeePoint(x: number, y: number): { canSee: boolean; isDim: boolean } {
    let canSee = false
    let allDim = true

    for (const area of tokenVisibleAreas.value) {
      const result = getEffectiveVisionAtPoint(area, x, y)
      if (result.canSee) {
        canSee = true
        if (!result.isDim) {
          allDim = false
        }
      }
    }

    return { canSee, isDim: canSee && allDim }
  }

  /**
   * Generate visibility data for rendering.
   * Returns circles representing each token's vision range.
   */
  const visibilityCircles = computed(() => {
    return tokenVisibleAreas.value.map(area => ({
      tokenId: area.tokenId,
      x: area.x,
      y: area.y,
      // Use the larger of bright/dim radius, capped at reasonable max
      radiusPx: Math.min(
        Math.max(area.brightVisionRadiusPx, area.dimVisionRadiusPx),
        Math.max(mapWidth.value, mapHeight.value) * 2
      ),
      brightRadiusPx: area.brightVisionRadiusPx === Infinity
        ? Math.max(mapWidth.value, mapHeight.value) * 2
        : area.brightVisionRadiusPx,
      dimRadiusPx: area.dimVisionRadiusPx === Infinity
        ? Math.max(mapWidth.value, mapHeight.value) * 2
        : area.dimVisionRadiusPx,
      ignoresLight: area.ignoresLight
    }))
  })

  /**
   * Check if we're in a "lights out" scenario where vision matters.
   * In bright ambient light with no special requirements, we don't need
   * to render vision overlays.
   */
  const needsVisionOverlay = computed(() => {
    // If ambient light is bright, no overlay needed unless there are
    // darkness zones or special vision requirements
    if (ambientLight.value === 'bright' && lightZones.value.length === 0) {
      return false
    }
    // If any tokens have limited vision, we need overlay
    return tokenVisibleAreas.value.some(
      area => area.brightVisionRadiusPx !== Infinity || area.dimVisionRadiusPx !== Infinity
    )
  })

  return {
    // Computed
    lightZones,
    tokenVisibleAreas,
    visibilityCircles,
    needsVisionOverlay,
    // Methods
    getEffectiveVisionAtPoint,
    canPartySeePoint,
    getLightLevelAtPoint: (x: number, y: number) =>
      getLightLevelAtPoint(x, y, lightZones.value, ambientLight.value),
    // Utilities
    feetToPixels: (feet: number) => feetToPixels(feet, gridSizePx.value)
  }
}
