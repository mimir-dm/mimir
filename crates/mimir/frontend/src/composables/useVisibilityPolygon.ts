/**
 * Visibility polygon calculation using 2D raycasting.
 * Computes the visible area from a point given a set of walls.
 *
 * Based on the algorithm described at:
 * https://www.redblobgames.com/articles/visibility/
 */

import { computed, type Ref } from 'vue'

/** A 2D point */
export interface Point {
  x: number
  y: number
}

/** A wall segment (line from p1 to p2) */
export interface Wall {
  p1: Point
  p2: Point
}

/** A portal (door) with state */
export interface Portal {
  id: string
  wall: Wall
  closed: boolean
}

/** UVTT format point (in grid units) */
export interface UvttPoint {
  x: number
  y: number
}

/** UVTT portal structure */
export interface UvttPortal {
  position: UvttPoint
  bounds: [UvttPoint, UvttPoint]
  rotation: number
  closed: boolean
  freestanding: boolean
}

/** UVTT light structure */
export interface UvttLight {
  position: UvttPoint
  range: number
  intensity: number
  color: string  // ARGB hex format (e.g., "ffeccd8b")
  shadows: boolean
}

/** UVTT environment settings */
export interface UvttEnvironment {
  baked_lighting: boolean
  ambient_light: string  // ARGB hex (e.g., "ffffffff" = bright, "ff000000" = dark)
}

/** Light source in pixel coordinates */
export interface Light {
  id: string
  position: Point
  range: number      // Range in pixels
  intensity: number  // 1 = normal, higher = brighter
  color: string      // CSS color format
  shadows: boolean   // Whether this light casts shadows
}

/** UVTT file format */
export interface UvttData {
  format: number
  resolution: {
    map_origin: UvttPoint
    map_size: UvttPoint
    pixels_per_grid: number
  }
  line_of_sight: UvttPoint[][]  // Array of wall segments, each segment is array of points
  portals: UvttPortal[]
  lights: UvttLight[]
  environment?: UvttEnvironment
}

/** Ambient light type matching D&D 5e */
export type AmbientLightLevel = 'bright' | 'dim' | 'darkness'

/**
 * Convert UVTT ambient light (ARGB hex) to AmbientLightLevel.
 * Uses brightness of the color to determine light level.
 */
export function uvttAmbientToLevel(argb: string | undefined): AmbientLightLevel {
  if (!argb) return 'bright'

  // Parse ARGB hex (e.g., "ffffffff" or "ff000000")
  const hex = argb.replace(/^#/, '')
  if (hex.length < 6) return 'bright'

  // Extract RGB (skip alpha)
  const r = parseInt(hex.slice(2, 4), 16) || 0
  const g = parseInt(hex.slice(4, 6), 16) || 0
  const b = parseInt(hex.slice(6, 8), 16) || 0

  // Calculate perceived brightness (0-255)
  const brightness = (r * 299 + g * 587 + b * 114) / 1000

  if (brightness > 170) return 'bright'
  if (brightness > 85) return 'dim'
  return 'darkness'
}

/** Convert UVTT walls (grid units) to pixel coordinates */
export function uvttWallsToPixels(uvtt: UvttData): Wall[] {
  const ppg = uvtt.resolution.pixels_per_grid
  const walls: Wall[] = []

  for (const segment of uvtt.line_of_sight) {
    // Each segment is a polyline - convert to wall segments
    for (let i = 0; i < segment.length - 1; i++) {
      walls.push({
        p1: { x: segment[i].x * ppg, y: segment[i].y * ppg },
        p2: { x: segment[i + 1].x * ppg, y: segment[i + 1].y * ppg }
      })
    }
  }

  return walls
}

/** Convert UVTT portals to Portal objects */
export function uvttPortalsToPixels(uvtt: UvttData): Portal[] {
  const ppg = uvtt.resolution.pixels_per_grid

  return uvtt.portals.map((p, idx) => ({
    id: `portal-${idx}`,
    wall: {
      p1: { x: p.bounds[0].x * ppg, y: p.bounds[0].y * ppg },
      p2: { x: p.bounds[1].x * ppg, y: p.bounds[1].y * ppg }
    },
    closed: p.closed
  }))
}

/** Convert ARGB hex color to CSS rgba */
function argbToRgba(argb: string): string {
  // UVTT uses ARGB format: "ffeccd8b" = A:ff R:ec G:cd B:8b
  const hex = argb.replace(/^#/, '')
  if (hex.length === 8) {
    const a = parseInt(hex.slice(0, 2), 16) / 255
    const r = parseInt(hex.slice(2, 4), 16)
    const g = parseInt(hex.slice(4, 6), 16)
    const b = parseInt(hex.slice(6, 8), 16)
    return `rgba(${r}, ${g}, ${b}, ${a})`
  }
  // Fallback for RGB
  if (hex.length === 6) {
    const r = parseInt(hex.slice(0, 2), 16)
    const g = parseInt(hex.slice(2, 4), 16)
    const b = parseInt(hex.slice(4, 6), 16)
    return `rgb(${r}, ${g}, ${b})`
  }
  return '#ffcc66' // Warm default
}

/** Convert UVTT lights to Light objects with pixel coordinates */
export function uvttLightsToPixels(uvtt: UvttData): Light[] {
  const ppg = uvtt.resolution.pixels_per_grid

  return uvtt.lights.map((light, idx) => ({
    id: `light-${idx}`,
    position: {
      x: light.position.x * ppg,
      y: light.position.y * ppg
    },
    range: light.range * ppg,
    intensity: light.intensity,
    color: argbToRgba(light.color),
    shadows: light.shadows
  }))
}

/** Ray casting result */
interface RayHit {
  point: Point
  angle: number
  dist: number
}

/** Calculate distance between two points */
function distance(p1: Point, p2: Point): number {
  const dx = p2.x - p1.x
  const dy = p2.y - p1.y
  return Math.sqrt(dx * dx + dy * dy)
}

/** Wall margin - negative value extends visibility past wall lines to reveal wall artwork */
const WALL_MARGIN = -5 // pixels - negative pushes visibility into walls to show wall graphics

/** Find intersection point of ray with line segment */
function raySegmentIntersection(
  origin: Point,
  angle: number,
  seg: Wall,
  pullBack: boolean = false
): Point | null {
  const dx = Math.cos(angle)
  const dy = Math.sin(angle)

  const x1 = seg.p1.x
  const y1 = seg.p1.y
  const x2 = seg.p2.x
  const y2 = seg.p2.y

  const x3 = origin.x
  const y3 = origin.y
  const x4 = origin.x + dx
  const y4 = origin.y + dy

  const denom = (x1 - x2) * (y3 - y4) - (y1 - y2) * (x3 - x4)

  if (Math.abs(denom) < 1e-10) {
    return null // Parallel
  }

  const t = ((x1 - x3) * (y3 - y4) - (y1 - y3) * (x3 - x4)) / denom
  const u = -((x1 - x2) * (y1 - y3) - (y1 - y2) * (x1 - x3)) / denom

  if (t >= 0 && t <= 1 && u > 0) {
    let hitX = x1 + t * (x2 - x1)
    let hitY = y1 + t * (y2 - y1)

    // Pull back the hit point slightly toward origin to prevent bleed-through
    if (pullBack) {
      const dist = Math.sqrt((hitX - origin.x) ** 2 + (hitY - origin.y) ** 2)
      if (dist > WALL_MARGIN) {
        const pullBackRatio = (dist - WALL_MARGIN) / dist
        hitX = origin.x + (hitX - origin.x) * pullBackRatio
        hitY = origin.y + (hitY - origin.y) * pullBackRatio
      }
    }

    return { x: hitX, y: hitY }
  }

  return null
}

/**
 * Calculate the visibility polygon from a point.
 *
 * @param origin - The viewing point
 * @param walls - Array of wall segments that block vision
 * @param maxRadius - Maximum vision radius in pixels
 * @param mapWidth - Map width for boundary walls
 * @param mapHeight - Map height for boundary walls
 * @returns Array of points forming the visibility polygon (CCW order)
 */
export function calculateVisibilityPolygon(
  origin: Point,
  walls: Wall[],
  maxRadius: number,
  mapWidth: number,
  mapHeight: number
): Point[] {
  // Collect all unique endpoints from walls
  const endpoints = new Set<string>()
  const points: Point[] = []

  // Add wall endpoints
  for (const wall of walls) {
    const key1 = `${wall.p1.x},${wall.p1.y}`
    const key2 = `${wall.p2.x},${wall.p2.y}`
    if (!endpoints.has(key1)) {
      endpoints.add(key1)
      points.push(wall.p1)
    }
    if (!endpoints.has(key2)) {
      endpoints.add(key2)
      points.push(wall.p2)
    }
  }

  // Add map corner points (bounded by maxRadius)
  const corners = [
    { x: Math.max(0, origin.x - maxRadius), y: Math.max(0, origin.y - maxRadius) },
    { x: Math.min(mapWidth, origin.x + maxRadius), y: Math.max(0, origin.y - maxRadius) },
    { x: Math.min(mapWidth, origin.x + maxRadius), y: Math.min(mapHeight, origin.y + maxRadius) },
    { x: Math.max(0, origin.x - maxRadius), y: Math.min(mapHeight, origin.y + maxRadius) }
  ]

  for (const corner of corners) {
    const key = `${corner.x},${corner.y}`
    if (!endpoints.has(key)) {
      endpoints.add(key)
      points.push(corner)
    }
  }

  // Add boundary walls
  const boundaryWalls: Wall[] = [
    { p1: corners[0], p2: corners[1] },  // top
    { p1: corners[1], p2: corners[2] },  // right
    { p1: corners[2], p2: corners[3] },  // bottom
    { p1: corners[3], p2: corners[0] }   // left
  ]

  // Cast rays to each endpoint (plus small offsets for edge detection)
  const hits: RayHit[] = []
  const epsilon = 0.0001

  for (const point of points) {
    const baseAngle = Math.atan2(point.y - origin.y, point.x - origin.x)

    // Cast 3 rays: one directly at the point, and two offset slightly
    for (const offset of [-epsilon, 0, epsilon]) {
      const angle = baseAngle + offset

      // Find closest intersection, checking real walls first (with pullBack)
      let closestHit: Point | null = null
      let closestDist = Infinity
      let hitRealWall = false

      // Check real walls first (with pullBack to prevent bleed-through)
      for (const wall of walls) {
        const intersection = raySegmentIntersection(origin, angle, wall, true)
        if (intersection) {
          const dist = distance(origin, intersection)
          if (dist < closestDist && dist <= maxRadius) {
            closestDist = dist
            closestHit = intersection
            hitRealWall = true
          }
        }
      }

      // Check boundary walls (no pullBack needed)
      for (const wall of boundaryWalls) {
        const intersection = raySegmentIntersection(origin, angle, wall, false)
        if (intersection) {
          const dist = distance(origin, intersection)
          if (dist < closestDist && dist <= maxRadius) {
            closestDist = dist
            closestHit = intersection
            hitRealWall = false
          }
        }
      }

      // If no wall hit, use max radius intersection with boundary
      if (!closestHit) {
        closestHit = {
          x: origin.x + Math.cos(angle) * maxRadius,
          y: origin.y + Math.sin(angle) * maxRadius
        }
        // Clamp to map bounds
        closestHit.x = Math.max(0, Math.min(mapWidth, closestHit.x))
        closestHit.y = Math.max(0, Math.min(mapHeight, closestHit.y))
        closestDist = distance(origin, closestHit)
      }

      if (closestHit) {
        hits.push({
          point: closestHit,
          angle,
          dist: closestDist
        })
      }
    }
  }

  // Sort hits by angle
  hits.sort((a, b) => a.angle - b.angle)

  // Remove duplicate points (very close together)
  const uniqueHits: RayHit[] = []
  for (const hit of hits) {
    const isDuplicate = uniqueHits.some(h =>
      Math.abs(h.point.x - hit.point.x) < 0.5 &&
      Math.abs(h.point.y - hit.point.y) < 0.5
    )
    if (!isDuplicate) {
      uniqueHits.push(hit)
    }
  }

  return uniqueHits.map(h => h.point)
}

/**
 * Convert visibility polygon points to SVG path data.
 * Returns a path string suitable for use in an SVG path element.
 */
export function polygonToSvgPath(points: Point[]): string {
  if (points.length < 3) return ''

  const path = points.map((p, i) =>
    i === 0 ? `M ${p.x} ${p.y}` : `L ${p.x} ${p.y}`
  ).join(' ')

  return path + ' Z'
}

/**
 * Composable for reactive visibility polygon calculation.
 */
export function useVisibilityPolygon(
  origin: Ref<Point | null>,
  walls: Ref<Wall[]>,
  portals: Ref<Portal[]>,
  maxRadius: Ref<number>,
  mapWidth: Ref<number>,
  mapHeight: Ref<number>
) {
  // Compute effective walls (walls + closed portals)
  const effectiveWalls = computed(() => {
    const closed = portals.value
      .filter(p => p.closed)
      .map(p => p.wall)
    return [...walls.value, ...closed]
  })

  // Compute visibility polygon
  const visibilityPolygon = computed(() => {
    if (!origin.value || mapWidth.value === 0 || mapHeight.value === 0) {
      return []
    }

    return calculateVisibilityPolygon(
      origin.value,
      effectiveWalls.value,
      maxRadius.value,
      mapWidth.value,
      mapHeight.value
    )
  })

  // SVG path for the visibility polygon
  const visibilityPath = computed(() => {
    return polygonToSvgPath(visibilityPolygon.value)
  })

  return {
    effectiveWalls,
    visibilityPolygon,
    visibilityPath
  }
}

/**
 * Calculate visibility polygons for multiple tokens.
 */
export function useMultiTokenVisibility(
  tokens: Ref<Array<{ id: string; x: number; y: number; visionRadius: number }>>,
  walls: Ref<Wall[]>,
  portals: Ref<Portal[]>,
  mapWidth: Ref<number>,
  mapHeight: Ref<number>
) {
  // Compute effective walls (walls + closed portals)
  const effectiveWalls = computed(() => {
    const closed = portals.value
      .filter(p => p.closed)
      .map(p => p.wall)
    return [...walls.value, ...closed]
  })

  // Compute visibility polygons for all tokens
  const visibilityPolygons = computed(() => {
    if (mapWidth.value === 0 || mapHeight.value === 0) {
      return []
    }

    return tokens.value.map(token => ({
      tokenId: token.id,
      polygon: calculateVisibilityPolygon(
        { x: token.x, y: token.y },
        effectiveWalls.value,
        token.visionRadius,
        mapWidth.value,
        mapHeight.value
      ),
      path: '' // Will be computed below
    })).map(v => ({
      ...v,
      path: polygonToSvgPath(v.polygon)
    }))
  })

  // Combined visibility path (union of all token visibility)
  const combinedVisibilityPath = computed(() => {
    // For SVG, we can use multiple paths in a single mask
    // Each path cuts out a visible area
    return visibilityPolygons.value.map(v => v.path).join(' ')
  })

  return {
    effectiveWalls,
    visibilityPolygons,
    combinedVisibilityPath
  }
}
