---
id: type-format-structs-mappattern
level: task
title: "Type format structs: MapPattern, MaterialEntry, and update MapPath/MapObject"
short_code: "MIMIR-T-0627"
created_at: 2026-03-15T00:42:35.930708+00:00
updated_at: 2026-03-15T00:42:35.930708+00:00
parent: MIMIR-I-0062
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/todo"


exit_criteria_met: false
initiative_id: MIMIR-I-0062
---

# Type format structs: MapPattern, MaterialEntry, and update MapPath/MapObject

**Blocks:** MIMIR-T-0629, MIMIR-T-0630, MIMIR-T-0631, MIMIR-T-0632

## Objective

Replace opaque `serde_json::Value` types in the DD format layer with properly typed Rust structs based on the spike findings (MIMIR-S-0001). Update existing structs that are missing fields. This unblocks all four feature tasks.

## Acceptance Criteria

- [ ] `MapPattern` in `format/entities.rs` has typed fields: position, shape_rotation, scale, points, layer, color, outline, texture, rotation, node_id (replacing `#[serde(flatten)] data: serde_json::Value`)
- [ ] New `MaterialEntry` struct with fields: bitmap (PoolByteArray), texture (String), smooth (bool)
- [ ] `Level.materials` in `format/world.rs` changed from `BTreeMap<String, serde_json::Value>` to `BTreeMap<String, Vec<MaterialEntry>>`
- [ ] `MapPath` updated with verified fields: position, rotation, scale, edit_points, smoothness, texture, width, layer, fade_in, fade_out, grow, shrink, block_light, loop, node_id
- [ ] `MapObject` updated with missing fields: shadow, block_light, custom_color (optional), prefab_id (optional)
- [ ] `MapLight` verified against existing struct — add any missing fields
- [ ] All existing tests pass (`cargo test -p mimir-mapgen`)
- [ ] Generated example maps still open correctly in Dungeondraft

## Implementation Notes

### Files to modify
- `crates/mimir-mapgen/src/format/entities.rs` — MapPattern, MapPath, MapObject, MapLight
- `crates/mimir-mapgen/src/format/world.rs` — Level.materials type, MaterialEntry struct
- `crates/mimir-mapgen/src/format/mod.rs` — re-exports if needed
- `crates/mimir-mapgen/src/paths.rs` — update MapPath construction to match new fields
- `crates/mimir-mapgen/src/objects.rs` — update MapObject construction for new fields

### Reference
- MIMIR-S-0001 (Dungeondraft Map Format Specification) — verified field definitions
- `Untitled.dungeondraft_map` in repo root — spike map with real DD output
- [DD Modding API](https://megasploot.github.io/DungeondraftModdingAPI/) — authoritative class definitions

## Status Updates

*To be added during implementation*