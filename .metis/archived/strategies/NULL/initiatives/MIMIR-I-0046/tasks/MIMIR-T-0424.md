---
id: usevisibilitypolygon-raycasting
level: task
title: "useVisibilityPolygon raycasting port"
short_code: "MIMIR-T-0424"
created_at: 2026-01-25T02:44:31.287477+00:00
updated_at: 2026-01-25T16:04:44.424049+00:00
parent: MIMIR-I-0046
blocked_by: []
archived: true

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: MIMIR-I-0046
---

# useVisibilityPolygon raycasting port

## Parent Initiative

[[MIMIR-I-0046]] - Map & Token VTT System

## Objective

Port the v1 useVisibilityPolygon composable that calculates line-of-sight visibility polygons using raycasting against UVTT wall data.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `computeVisibilityPolygon(position, walls, visionRange)` calculates visible area
- [ ] Raycasting from token center to wall endpoints
- [ ] Returns SVG path string for clipping
- [ ] Handles portal (door) state - closed doors block, open doors don't
- [ ] Performance acceptable with many walls (< 16ms per frame)

## Implementation Notes

### Algorithm Overview

1. Collect all wall endpoints within vision range
2. For each endpoint, cast ray from token position
3. Find intersections with all walls
4. Build polygon from closest intersections sorted by angle
5. Return as SVG path

### Core Functions

```typescript
interface Point { x: number; y: number }
interface Wall { start: Point; end: Point }

function computeVisibilityPolygon(
  position: Point,
  walls: Wall[],
  maxDistance: number
): string {
  // Get all endpoints
  const endpoints = getEndpoints(walls)
  
  // Cast rays with slight offsets for corner handling
  const rays: Point[] = []
  for (const endpoint of endpoints) {
    const angle = Math.atan2(endpoint.y - position.y, endpoint.x - position.x)
    
    // Cast 3 rays per endpoint (exact + slight offsets)
    for (const offset of [-0.0001, 0, 0.0001]) {
      const ray = castRay(position, angle + offset, walls, maxDistance)
      if (ray) rays.push(ray)
    }
  }
  
  // Sort by angle and build polygon
  rays.sort((a, b) => {
    const angleA = Math.atan2(a.y - position.y, a.x - position.x)
    const angleB = Math.atan2(b.y - position.y, b.x - position.x)
    return angleA - angleB
  })
  
  return buildSvgPath(rays)
}

function castRay(
  origin: Point,
  angle: number,
  walls: Wall[],
  maxDistance: number
): Point | null {
  // Find closest intersection
}

function raySegmentIntersection(
  rayOrigin: Point,
  rayDir: Point,
  segStart: Point,
  segEnd: Point
): Point | null {
  // Standard ray-segment intersection math
}
```

### Integration with UVTT

```typescript
const { walls, portals } = useUvttMap()

const blockingWalls = computed(() => {
  const closedPortalWalls = portals.value
    .filter(p => p.closed)
    .map(p => p.wall)
  return [...walls.value, ...closedPortalWalls]
})
```

### Reference

Port from: `backup/mimir-dm-bu/mimir-dm/frontend/src/composables/useVisibilityPolygon.ts`

### Files to Create/Modify

- `crates/mimir/frontend/src/composables/useVisibilityPolygon.ts`

### Dependencies

- useUvttMap composable (for walls and portals)

## Status Updates

*To be added during implementation*