---
id: declarative-room-and-dungeon
level: initiative
title: "Declarative room and dungeon layout for mapgen"
short_code: "MIMIR-I-0060"
created_at: 2026-03-11T23:40:03.187330+00:00
updated_at: 2026-03-13T02:38:53.449193+00:00
parent: MIMIR-V-0001
blocked_by: []
archived: false

tags:
  - "#initiative"
  - "#phase/completed"


exit_criteria_met: false
estimated_complexity: L
initiative_id: declarative-room-and-dungeon
---

# Declarative room and dungeon layout for mapgen Initiative

## Context

The mapgen system currently generates outdoor terrain only — forests, grasslands, caves with terrain textures, trees, clutter, roads, rivers, water, and elevation contours. The Dungeondraft format supports walls, portals (doors/windows), and multi-level structures, but the generation pipeline never populates them.

Users (via the creative director agent) need to be able to define dungeon and interior layouts declaratively: rooms placed on the grid with explicit boundaries, connections, doors, and windows. No procedural room generation — the layout is fully specified, and the system renders it into Dungeondraft's wall/portal format.

### Current State
- `MapWall` and `MapPortal` in `entities.rs` are `serde_json::Value` wrappers — not fully typed
- `Level` struct has `walls: Vec<MapWall>` and `portals: Vec<MapPortal>` but they're never populated
- The pipeline is 10 stages, all outdoor-focused (noise → terrain → objects → paths → water → elevation → lighting → assembly)

## Goals & Non-Goals

**Goals:**
- Declarative room definitions in YAML config — position, dimensions, wall segments on the grid
- Door and window placement as portal types on wall segments
- Corridor/connection definitions between rooms
- Terrain control per room (e.g., stone floor in a dungeon, dirt in a cellar)
- Fully typed `MapWall` and `MapPortal` structs matching Dungeondraft's format
- Integration with existing outdoor features (a dungeon entrance in a forest map)
- Updated mapgen skill so the creative director agent can translate "guard room connected to a throne room via a locked door" into room declarations

**Non-Goals:**
- Procedural room layout generation (BSP, cellular automata, etc.) — rooms are user/agent-specified, not random
- Furniture or object placement inside rooms (that's Dungeondraft's job)
- Multi-level dungeons (Dungeondraft supports levels but this is a stretch goal, not v1)
- Interior lighting per room (global lighting config applies; per-room lighting is future work)

## Detailed Design

### Room Schema (YAML)

```yaml
rooms:
  - id: "guard_room"
    x: 4            # grid position (top-left corner)
    y: 6
    width: 5        # grid squares
    height: 4
    terrain_slot: 3  # which terrain texture fills the floor (0-3)
    walls:           # optional overrides — by default all 4 sides are walled
      north: true
      south: true
      east: true
      west: false    # open side (e.g., corridor entrance)
    portals:
      - wall: "north"
        position: 2       # grid offset along the wall
        type: "door"       # door, window, archway, secret_door
        width: 1           # portal width in grid squares
      - wall: "east"
        position: 1
        type: "window"
        width: 1

corridors:
  - from: "guard_room"
    from_wall: "west"
    to: "throne_room"
    to_wall: "east"
    width: 2          # corridor width in grid squares
    terrain_slot: 3
```

### Wall Generation
- Each room's boundaries become `MapWall` entries — line segments in pixel coordinates
- Walls are generated for all sides unless explicitly set to `false`
- Portal positions create gaps in wall segments (wall splits around the portal)
- Corridors generate walls along their length connecting two rooms

### Portal Generation
- Portals become `MapPortal` entries at the specified wall position
- Types map to Dungeondraft portal types (door, window, archway, secret_door)
- Portal width determines the gap in the wall

### Coordinate System
- Room positions and sizes are in **grid squares** (matching the map's width/height)
- Internally converted to pixel coordinates (grid × pixels_per_cell, typically 64px)
- This keeps declarations readable and aligned to the Dungeondraft grid

### Pipeline Integration

Rooms are **first-class layout primitives** — they go into the pipeline early so everything else respects them.

**Revised pipeline order:**
1. **Room layout** (NEW) — resolve rooms, corridors, walls, portals into spatial data
2. **Noise generation** — unchanged, but room interiors are masked out of noise
3. **Terrain** — noise-based terrain applied outside rooms; room interiors get their declared terrain applied as a second pass
4. **Roads** — pathfinding routes around room boundaries (rooms act as obstacles)
5. **Rivers** — flow paths bend around rooms (rooms are impassable to river routing)
6. **Object placement** (trees, clutter, clumps) — room interiors and corridors are exclusion zones
7. **Corridor clearing** — corridor paths clear objects, similar to road corridors
8. **Water** — water polygons respect room boundaries
9. **Elevation** — contours stop at room walls
10. **Lighting** — unchanged (global)
11. **Assembly** — walls and portals written into the Level struct

The key insight: rooms define **exclusion zones** that the outdoor generation pipeline treats as obstacles. Roads and rivers pathfind around them. Objects don't spawn inside them. Terrain inside rooms is applied declaratively, not from noise.

## Alternatives Considered

**Procedural generation (BSP/graph-based):** Rejected per user direction. The creative director agent should specify layouts deliberately, not generate them randomly. Procedural generation could be a future initiative layered on top of the declarative system.

**Pixel-coordinate room definitions:** Rejected in favor of grid-square coordinates. Grid squares are more intuitive, align with Dungeondraft's grid, and match how users think about room sizes ("5x4 room" not "320x256 pixels").

## Implementation Plan

1. **Fully type MapWall and MapPortal** — Replace `serde_json::Value` wrappers with proper structs matching Dungeondraft's format
2. **Room and corridor schema** — Add room/corridor definitions to MapConfig YAML schema
3. **Wall generation** — Convert room boundaries to wall segments, handle portal gaps
4. **Portal generation** — Create door/window/archway entries from portal declarations  
5. **Corridor generation** — Connect rooms with walled corridors
6. **Pipeline integration** — New stage for room layout, terrain override, object exclusion zones
7. **Config validation** — Room overlap detection, corridor endpoint validation, portal position bounds
8. **Tests** — Unit tests for wall/portal generation, integration tests for room+outdoor maps
9. **Skill and plugin updates** — Teach the creative director agent to declare rooms
10. **CLI support** — Ensure CLI handles configs with room declarations