---
id: pipeline-integration-rooms-as
level: task
title: "Pipeline integration: rooms as exclusion zones"
short_code: "MIMIR-T-0592"
created_at: 2026-03-11T23:56:30.940614+00:00
updated_at: 2026-03-12T00:59:37.365201+00:00
parent: MIMIR-I-0060
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: MIMIR-I-0060
---

# Pipeline integration: rooms as exclusion zones

## Parent Initiative

[[MIMIR-I-0060]]

## Objective

Integrate rooms into the generation pipeline as first-class exclusion zones. Room layout runs first, then all subsequent stages (noise, terrain, roads, rivers, objects, water, elevation) respect room boundaries — routing around them, excluding objects from interiors, and applying room terrain as a second pass after noise-based terrain.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] Room layout runs as the first pipeline stage in `generate()` (before noise)
- [ ] `RoomExclusionZones` struct: collection of axis-aligned rectangles (rooms + corridors) in pixel coordinates
- [ ] Noise generation: rooms masked from noise map (optional — noise still generates but room areas are overridden)
- [ ] Terrain: noise-based terrain applied normally, then room interiors overwritten with their declared terrain_slot in the splat map
- [ ] Roads: greedy pathfinder treats room rectangles as impassable obstacles, routing around them
- [ ] Rivers: same obstacle avoidance as roads
- [ ] Object placement (trees, clutter, clumps): skip placement inside exclusion zones
- [ ] Water: water polygon generation respects room boundaries
- [ ] Elevation contours: stop at room walls
- [ ] Walls and portals written into `Level.walls` and `Level.portals`
- [ ] Integration test: config with rooms AND outdoor features (trees, road) — road routes around rooms, no objects inside rooms
- [ ] `GenerateStats` gains `walls_generated` and `portals_generated` counters
- [ ] Existing outdoor-only configs produce identical output (no regression)

## Implementation Notes

- File: `crates/mimir-mapgen/src/pipeline.rs` — modify `generate()` function
- The exclusion zone check is a simple AABB point-in-rect test — fast enough for Poisson Disc sampling
- Road/river pathfinding currently uses greedy noise-following (paths.rs). Adding obstacle avoidance means checking candidate positions against exclusion zones during the greedy walk
- Terrain override: modify the splat map bytes directly — set the 4 bytes per cell to weight the target terrain_slot to 255 and others to 0
- Depends on MIMIR-T-0590 (room layout) and MIMIR-T-0591 (corridors)

### Assembly Details (from T-0587)
- Room walls go into `Level.walls`
- Room portals are embedded in their parent wall (NOT in `Level.portals`)
- Must also populate `Shapes.walls` (node_ids as decimal ints) and `Shapes.polygons` (room outlines as PoolVector2Array)
- `Shapes.polygons` entries define the floor fill regions that Dungeondraft renders with tile colors

## Status Updates

### Completed
- Room layout runs as first pipeline stage (stage 0, before noise generation)
- `ExclusionZone` struct with `contains()` method for AABB point-in-rect testing
- `build_exclusion_zones()` — creates zones from rooms and corridors (handles straight + L-shaped)
- `is_excluded()` — checks if a point falls in any zone
- Terrain overrides: applied after noise-based terrain, room floors override splat map bytes
- Object exclusion: objects filtered from room/corridor zones after placement + corridor clearing
- Walls and portals assembled into `Level.walls`, `Level.portals`, and `Level.shapes`
- `GenerateStats` gains `walls_generated` and `portals_generated` counters
- Integration test: config with 2 rooms, corridor, trees + terrain — verifies wall/portal counts, no objects inside rooms, terrain override applied, shapes populated, JSON round-trip
- Regression test: outdoor-only config produces 0 walls/portals (unchanged behavior)
- All 111 tests pass, 0 warnings