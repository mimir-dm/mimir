---
id: scaffold-mimir-mapgen-crate-with
level: task
title: "Scaffold mimir-mapgen crate with Cargo workspace integration"
short_code: "MIMIR-T-0569"
created_at: 2026-03-11T21:23:26.922320+00:00
updated_at: 2026-03-11T21:56:13.945459+00:00
parent: MIMIR-I-0058
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: MIMIR-I-0058
---

# Scaffold mimir-mapgen crate with Cargo workspace integration

## Objective

Create the `crates/mimir-mapgen` crate, add it to the Cargo workspace, and set up the module structure and initial dependencies. This is the foundation all other tasks build on.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `crates/mimir-mapgen/Cargo.toml` exists with version `0.6.1`, correct dependencies (`serde`, `serde_json`, `serde_yaml`, `noise`, `rand`, `rand_chacha`)
- [ ] Crate added to root `Cargo.toml` workspace members
- [ ] Module structure stubbed: `lib.rs` with `pub mod format`, `pub mod noise`, `pub mod distribution`, `pub mod contour`, `pub mod curves`, `pub mod terrain`, `pub mod objects`, `pub mod paths`, `pub mod water`, `pub mod elevation`, `pub mod biomes`, `pub mod pipeline`, `pub mod assets`
- [ ] `cargo check -p mimir-mapgen` passes
- [ ] Basic crate-level doc comment and README intent

## Implementation Notes

- Independent of `mimir-core` — no Diesel, no database
- Evaluate `noise` crate vs `libnoise` for Perlin; `poisson2d` or similar for Poisson Disc
- Include `rand = "0.8"` and `rand_chacha = "0.3"` for reproducible seeded generation
- Include `serde_yaml` for config parsing

### Dependencies
None — this is the first task.

## Status Updates

### 2026-03-11
- Created `crates/mimir-mapgen/` with Cargo.toml (v0.6.1), bin + lib targets
- Added to workspace members in root Cargo.toml
- Added `noise = "0.9"`, `rand = "0.8"`, `rand_chacha = "0.3"` to workspace deps
- Module structure: format, noise_gen, distribution, contour, curves, terrain, objects, paths, water, elevation, biomes, pipeline, assets
- CLI scaffolded with clap: `generate`, `validate`, `list-presets` subcommands (stubs)
- `cargo check -p mimir-mapgen` passes clean