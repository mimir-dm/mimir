---
id: generated-dungeondraft-map-files
level: task
title: "Generated .dungeondraft_map files won't open in Dungeondraft"
short_code: "MIMIR-T-0595"
created_at: 2026-03-12T01:40:57.973869+00:00
updated_at: 2026-03-12T01:40:57.973869+00:00
parent: 
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/backlog"
  - "#bug"


exit_criteria_met: false
initiative_id: NULL
---

# Generated .dungeondraft_map files won't open in Dungeondraft

## Objective

Fix generated `.dungeondraft_map` files so they open in Dungeondraft. Currently all generated maps fail to load silently — no error, just refuses to open.

## Reproduction

1. `./target/release/mimir-mapgen generate test-dungeon.yaml -o dungeon-test.dungeondraft_map`
2. Open `dungeon-test.dungeondraft_map` in Dungeondraft → fails silently

## Bugs Found (via incremental build-up against known-good file)

### Bug 1: Cave bitmap size formula is wrong (CONFIRMED BLOCKER)
- **File**: `crates/mimir-mapgen/src/format/world.rs` — `Level::new_ground()`
- **Was**: `PoolByteArray::from_vec(vec![0; width * height])` (cell_count = w*h)
- **Should be**: `w*h*2 + floor(1.5*(w+h)) + 2`
- **Source**: Ruby reference script (`randombiome_v1.00.rb` line 66)
- **Confirmed**: Fixing this alone (step5b) made the file openable
- **Affects both**: `cave.bitmap` and `cave.entrance_bitmap`

### Bug 2: Missing `mod` section at root level (FIXED, untested alone)
- **File**: `crates/mimir-mapgen/src/format/mod.rs` — `DungeondraftMap`
- **Was**: Only `header` + `world` at root
- **Should be**: `header` + `world` + `mod: {".node_table": {}}`
- **Fix applied**: Added `mod_data` field with `ModData` struct

### Bug 3: Missing Cave fields (FIXED, untested alone)
- **File**: `crates/mimir-mapgen/src/format/world.rs` — `Cave` struct
- **Was**: Only `bitmap` field
- **Missing**: `ground_color`, `wall_color`, `entrance_bitmap`, `texture`
- **Fix applied**: Added all fields with defaults from known-good file

### Bug 4: Missing `texts_vis` on Level (FIXED, untested alone)
- **File**: `crates/mimir-mapgen/src/format/world.rs` — `Level` struct
- **Should be**: `texts_vis: true`

### Bug 5: Missing `lookup` on Tiles (FIXED, untested alone)
- **File**: `crates/mimir-mapgen/src/format/world.rs` — `Tiles` struct
- **Should be**: `lookup: {}` (empty BTreeMap)

### Bug 6: Spurious `water.tree` in empty maps (FIXED, untested alone)
- **File**: `crates/mimir-mapgen/src/format/world.rs` — `Level::new_ground()`
- **Was**: Constructed a dummy `WaterTree` with empty polygon
- **Should be**: `tree: None` (omitted via `skip_serializing_if`)

### Bug 8: Portal direction vector is wall segment direction, not outward normal (CONFIRMED)
- **File**: `crates/mimir-mapgen/src/rooms.rs` — `make_portal()`
- **Was**: `direction` set to outward normal of the wall (e.g., `(0, -1)` for north wall)
- **Should be**: `direction` set to the wall segment direction (e.g., `(1, 0)` for segment going east)
- **Confirmed**: Fixing direction from `(0,-1)` to `(1,0)` for north wall portal fixed the "cockeyed" wall rendering
- **Evidence**: DD-placed portal on south wall (seg 2, going west) had `direction: (-1, 0)` = segment direction
- **Rule**: For segment from point A to point B, direction = normalize(B - A)
- **Rotation**: Also matches segment angle — seg going east = rotation 0, seg going west = rotation π

### Potential Bug 7: Node IDs should be hex (NEEDS INVESTIGATION)
- **File**: `crates/mimir-mapgen/src/format/mod.rs` — `NodeIdAllocator`
- Good file uses hex node_ids: `b`, `c`, `d`, `12`, `13`, `14`
- Our file uses decimal: `1`, `4`, `6`, `7`, `8`, `9`
- Ruby script confirms: `$next_node_id.to_s(16)` (hex)
- Low IDs overlap (0-9 are same in hex/decimal) so may not cause load failure, but will break for node_id >= 10

## Testing Approach

Building up from known-good `test.dungeondraft_map` incrementally:
- step1b: good file resized to 20x16 → **OPENED** (truncated visuals expected)
- step2: rebuilt from scratch with correct keys → **FAILED**
- step3: step1b stripped of walls/objects → **OPENED**
- step4b: step3 with resized bitmaps/terrain (wrong cave size) → **FAILED**
- step5a: step3 with only tiles resized → **OPENED**
- step5b: step3 with correct cave bitmap size + resized tiles/terrain → **OPENED**
- step6: step5b + walls (no portals) + shapes → **OPENED**
- step7c: step6 + one portal (direction as outward normal) → **BROKEN** (cockeyed walls)
- step8: step7c saved by DD, our portal removed → **OPENED**
- step7c: portal direction changed to segment direction → **OPENED**
- step8: two known-good door portals → **OPENED**
- step9: + window on east wall → **OPENED**
- step10: + archway on throne room (radius 256 then 128) → **OPENED** (archway invisible, portal always 1 grid = radius 128)
- step11: + freestanding corridor door → **OPENED**
- step12: + terrain + lighting → **OPENED** (full dungeon-test equivalent)

### Bug 9: Portal radius is always 128 (1 grid square)
- Portal width is always 1 grid square regardless of radius value
- `radius: 128` = half a grid square = portal spans 1 grid square
- Multi-square portals not supported natively — would need multiple adjacent portals
- Our code was using `width * 128.0` for radius, should always be `128`

## Acceptance Criteria

- [ ] Cave bitmap uses correct size formula
- [ ] Node IDs are hex strings
- [ ] All required fields present (mod, cave fields, texts_vis, tiles.lookup)
- [ ] Generated dungeon-test map opens in Dungeondraft
- [ ] Generated forest-ruin-test map opens in Dungeondraft
- [ ] Generated forest-preset-test map opens in Dungeondraft

## Status Updates

### 2026-03-12: Initial investigation
- Compared generated output against known-good `test.dungeondraft_map`
- Found cave bitmap size formula from Ruby reference script
- Confirmed cave bitmap size is the primary blocker
- Applied fixes for bugs 2-6 in Rust code, tests pass
- Currently building up step-by-step to identify any remaining issues