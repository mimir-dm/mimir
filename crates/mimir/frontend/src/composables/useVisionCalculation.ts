/**
 * Composable for calculating token vision based on D&D 5e rules.
 *
 * Uses the new vision fields:
 * - vision_bright_ft: Vision range in bright light (null = unlimited)
 * - vision_dim_ft: Vision range in dim light (null = unlimited)
 * - vision_dark_ft: Vision range in darkness (0 = blind, 60 = darkvision)
 * - light_radius_ft: Token's light source radius (0 = no light)
 */
import { computed, type Ref } from 'vue'
import type { Token } from '@/types/api'
import type { LightSourceSummary } from './useLightSources'

/** Ambient light levels matching D&D 5e */
export type AmbientLight = 'bright' | 'dim' | 'darkness'

/** Light level at a specific point */
export type LightLevel = 'bright' | 'dim' | 'darkness'

/** Light zone on the map (from map light sources or token lights) */
export interface LightZone {
  sourceId: string
  x: number
  y: number
  brightRadiusPx: number
  dimRadiusPx: number
}

/** Vision result for a single token */
export interface TokenVision {
  tokenId: string
  tokenColor: string | null
  x: number
  y: number
  /** Effective vision radius in pixels based on current conditions */
  visionRadiusPx: number
  /** Whether vision is treated as dim (disadvantage) */
  isDimVision: boolean
  /** Token's own light radius in pixels */
  lightRadiusPx: number
}

/** Configuration for vision calculation */
export interface VisionCalculationConfig {
  /** All tokens on the map */
  tokens: Ref<Token[]>
  /** Map light sources (from database) */
  lightSources: Ref<LightSourceSummary[]>
  /** Current ambient light level (DM controlled) */
  ambientLight: Ref<AmbientLight>
  /** Grid size in pixels (1 grid = 5 feet) */
  gridSizePx: Ref<number>
  /** Map dimensions for bounds */
  mapWidth: Ref<number>
  mapHeight: Ref<number>
}

/** Convert feet to pixels (1 grid square = 5 feet) */
function feetToPixels(feet: number, gridSizePx: number): number {
  return (feet / 5) * gridSizePx
}

/** Large radius representing "unlimited" vision */
const UNLIMITED_RADIUS = 100000

/**
 * Calculate light zones from map light sources and token lights.
 * Token lights use the consistent bright=half, dim=full convention.
 */
function calculateLightZones(
  mapLightSources: LightSourceSummary[],
  tokens: Token[],
  gridSizePx: number
): LightZone[] {
  const zones: LightZone[] = []

  // Add map light sources
  for (const light of mapLightSources) {
    if (!light.is_active) continue

    zones.push({
      sourceId: `map-${light.id}`,
      x: light.x,
      y: light.y,
      brightRadiusPx: feetToPixels(light.bright_radius_ft, gridSizePx),
      dimRadiusPx: feetToPixels(light.dim_radius_ft, gridSizePx)
    })
  }

  // Add token light sources (light_radius_ft is dim radius, bright = half)
  for (const token of tokens) {
    if (token.light_radius_ft > 0) {
      const dimRadiusFt = token.light_radius_ft
      const brightRadiusFt = dimRadiusFt / 2

      zones.push({
        sourceId: `token-${token.id}`,
        x: token.x,
        y: token.y,
        brightRadiusPx: feetToPixels(brightRadiusFt, gridSizePx),
        dimRadiusPx: feetToPixels(dimRadiusFt, gridSizePx)
      })
    }
  }

  return zones
}

/**
 * Get the effective light level at a point, considering ambient light and light sources.
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
      return 'bright' // Can't get better than bright
    } else if (distance <= zone.dimRadiusPx && bestLight === 'darkness') {
      bestLight = 'dim'
    }
  }

  return bestLight
}

/**
 * Calculate vision for a single token based on the light level at their position.
 *
 * Key rule: A token must be INSIDE a light's radius to benefit from it.
 * We check the light level at the token's position, not at distant points.
 */
function calculateTokenVision(
  token: Token,
  lightZones: LightZone[],
  ambientLight: AmbientLight,
  gridSizePx: number
): TokenVision {
  // Get light level at the token's position
  const lightAtToken = getLightLevelAtPoint(token.x, token.y, lightZones, ambientLight)

  // Determine vision radius based on light level
  let visionFt: number | null
  let isDimVision = false

  switch (lightAtToken) {
    case 'bright':
      visionFt = token.vision_bright_ft
      break
    case 'dim':
      visionFt = token.vision_dim_ft
      isDimVision = true
      break
    case 'darkness':
      // In darkness, use dark vision OR own light radius (whichever is greater)
      const darkVisionFt = token.vision_dark_ft
      const ownLightFt = token.light_radius_ft
      visionFt = Math.max(darkVisionFt, ownLightFt)
      isDimVision = darkVisionFt > 0 // Darkvision sees as dim, not bright
      break
  }

  // Convert to pixels (null = unlimited)
  const visionRadiusPx = visionFt === null
    ? UNLIMITED_RADIUS
    : feetToPixels(visionFt, gridSizePx)

  return {
    tokenId: token.id,
    tokenColor: token.color,
    x: token.x,
    y: token.y,
    visionRadiusPx,
    isDimVision,
    lightRadiusPx: feetToPixels(token.light_radius_ft, gridSizePx)
  }
}

/**
 * Composable for calculating token vision.
 */
export function useVisionCalculation(config: VisionCalculationConfig) {
  const { tokens, lightSources, ambientLight, gridSizePx, mapWidth, mapHeight } = config

  /** All light zones (map lights + token lights) */
  const lightZones = computed<LightZone[]>(() => {
    return calculateLightZones(
      lightSources.value,
      tokens.value,
      gridSizePx.value
    )
  })

  /** Vision calculations for PC tokens only */
  const pcVision = computed<TokenVision[]>(() => {
    return tokens.value
      .filter(t => t.token_type === 'pc' && t.visible_to_players)
      .map(token => calculateTokenVision(
        token,
        lightZones.value,
        ambientLight.value,
        gridSizePx.value
      ))
  })

  /** Vision calculations for ALL tokens (for DM view boundaries) */
  const allTokenVision = computed<TokenVision[]>(() => {
    return tokens.value.map(token => calculateTokenVision(
      token,
      lightZones.value,
      ambientLight.value,
      gridSizePx.value
    ))
  })

  /**
   * Check if a point is within a token's vision radius.
   */
  function isPointInVision(
    pointX: number,
    pointY: number,
    vision: TokenVision
  ): boolean {
    const dx = pointX - vision.x
    const dy = pointY - vision.y
    const distance = Math.sqrt(dx * dx + dy * dy)
    return distance <= vision.visionRadiusPx
  }

  /**
   * Check if any PC can see a specific point.
   */
  function canPartySeePoint(x: number, y: number): { canSee: boolean; isDim: boolean } {
    let canSee = false
    let allDim = true

    for (const vision of pcVision.value) {
      if (isPointInVision(x, y, vision)) {
        canSee = true
        if (!vision.isDimVision) {
          allDim = false
        }
      }
    }

    return { canSee, isDim: canSee && allDim }
  }

  /**
   * Get light level at a specific point.
   */
  function getLightLevel(x: number, y: number): LightLevel {
    return getLightLevelAtPoint(x, y, lightZones.value, ambientLight.value)
  }

  /**
   * Check if we need vision overlay rendering.
   * In bright ambient light with unlimited vision, no overlay needed.
   */
  const needsVisionOverlay = computed(() => {
    if (ambientLight.value === 'bright') {
      // Check if any PC has limited bright vision
      return pcVision.value.some(v => v.visionRadiusPx < UNLIMITED_RADIUS)
    }
    return true // Always need overlay in dim/dark
  })

  /**
   * Get max vision radius for bounds calculation.
   */
  const maxVisionRadius = computed(() => {
    const maxMap = Math.max(mapWidth.value, mapHeight.value) * 2
    return Math.min(
      Math.max(...pcVision.value.map(v => v.visionRadiusPx), 0),
      maxMap
    )
  })

  return {
    // Computed state
    lightZones,
    pcVision,
    allTokenVision,
    needsVisionOverlay,
    maxVisionRadius,
    // Methods
    isPointInVision,
    canPartySeePoint,
    getLightLevel,
    // Utilities
    feetToPixels: (feet: number) => feetToPixels(feet, gridSizePx.value)
  }
}
