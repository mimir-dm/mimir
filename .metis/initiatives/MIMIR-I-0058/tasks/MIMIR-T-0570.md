---
id: dungeondraft-map-format-parser-and
level: task
title: "Dungeondraft map format parser and writer"
short_code: "MIMIR-T-0570"
created_at: 2026-03-11T21:23:28.224926+00:00
updated_at: 2026-03-11T22:45:06.384102+00:00
parent: MIMIR-I-0058
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: MIMIR-I-0058
---

# Dungeondraft map format parser and writer

## Objective

Implement the `format` module: type-safe Rust structs for the `.dungeondraft_map` JSON format with serde serialization/deserialization. This is the I/O layer that all generation feeds into.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `DungeondraftMap` struct with `header: Header` and `world: World` fields
- [ ] Godot type wrappers: `Vector2`, `PoolByteArray`, `PoolVector2Array` with custom serde (de)serialization matching DD's string format (e.g., `"Vector2( 100, 200 )"`)
- [ ] `Header` struct: `creation_build`, `creation_date`, `uses_default_assets`, `asset_manifest: Vec<AssetPackRef>`, `editor_state`
- [ ] `World` struct: `format`, `width`, `height`, `next_node_id`, `grid`, `levels: HashMap<String, Level>`
- [ ] `Level` struct with all entity arrays: `terrain`, `objects`, `paths`, `water`, `lights`, `patterns`, `walls`, `roofs`, etc.
- [ ] Entity structs: `MapObject`, `MapPath`, `MapLight`, `MapPattern`, `WaterTree`, `Terrain`
- [ ] `Terrain` struct with splat map encode/decode (byte array ↔ 4-channel texture weights)
- [ ] Round-trip test: parse the reference `baseline.dungeondraft_map` → serialize → parse again → assert equal
- [ ] Builder API: `DungeondraftMap::new(width, height)` creates a valid empty map with sensible defaults
- [ ] `NodeIdAllocator` for auto-incrementing `ref` IDs across all entity types

## Implementation Notes

- See **MIMIR-S-0001** for the full format specification
- Custom serde for Godot types is critical — DD is strict about whitespace (`Vector2( x, y )` not `Vector2(x,y)`)
- The `baseline.dungeondraft_map` from the reference repo should be used as a test fixture
- `PoolByteArray` can be very large for terrain splat maps — consider lazy/streaming serialization
- Color format is ARGB hex string (e.g., `"ff3aa19a"`)

### Dependencies
Depends on: MIMIR-T-0569 (crate scaffold)

## Status Updates

### 2026-03-11
- Built `format/` module with 4 files: `godot_types.rs`, `header.rs`, `world.rs`, `entities.rs`, `mod.rs`
- Godot types with custom serde: `Vector2`, `PoolByteArray`, `PoolVector2Array`, `PoolIntArray`, `NullableVector2`
- Full struct hierarchy: `DungeondraftMap` → `Header` + `World` → `Level` → all entity types
- Entity types: `MapObject`, `MapPath`, `MapLight`, `MapText`, `MapWall`, `MapPortal`, `MapPattern`
- `Terrain` with 4-texture splat map, `WaterTree` recursive structure, `Roofs`
- Builder APIs: `DungeondraftMap::new()`, `Level::new_ground()`, `Terrain::new_uniform()`, entity builders
- `NodeIdAllocator` for unique ref IDs
- Copied `baseline.dungeondraft_map` as test fixture
- 15 tests passing including real DD file round-trip parse → serialize → reparse