---
id: 001-mapgen-crate-architecture-and
level: adr
title: "Mapgen Crate Architecture and Dependency Strategy"
number: 1
short_code: "MIMIR-A-0008"
created_at: 2026-03-11T21:11:43.685740+00:00
updated_at: 2026-03-11T21:11:43.685740+00:00
decision_date: 
decision_maker: 
parent: 
archived: false

tags:
  - "#adr"
  - "#phase/draft"


exit_criteria_met: false
initiative_id: NULL
---

# ADR-8: Mapgen Crate Architecture and Dependency Strategy

Related initiative: MIMIR-I-0058 (Declarative Dungeondraft Map Generation)

## Context

We're building a procedural map generator for Dungeondraft maps. The reference implementation is ~4,500 lines of Ruby implementing Perlin noise, Poisson Disc sampling, Marching Squares, and Bezier curves from scratch. We need to decide:

1. **Crate structure**: standalone `mimir-mapgen` crate vs integrated into existing crates
2. **Algorithm dependencies**: use existing Rust crates vs reimplement algorithms
3. **Config format**: JSON (matching reference impl) vs YAML vs TOML
4. **Asset strategy**: hardcode default paths vs discover from Dungeondraft installation vs user-specified

## Decision

### Crate Structure
Create a new `crates/mimir-mapgen` crate that is independent of `mimir-core` (no database, no Diesel). It should be usable as a standalone library and CLI tool, with integration into Mimir via Tauri commands and MCP tools in the app crate.

### Algorithm Dependencies
Prefer existing well-tested Rust crates over reimplementation:

| Algorithm | Crate | Rationale |
|-----------|-------|-----------|
| Perlin noise | `noise` (crate) | Mature, supports fractal/multi-octave, 2D/3D |
| Poisson Disc | `poisson-disk-sampling` or custom | Evaluate — if crate exists with noise-gating, use it; otherwise small enough to implement |
| Marching Squares | Custom | DD-specific contour needs (path generation, polygon merging) make a generic crate unlikely to fit |
| Bezier curves | `lyon` or custom | `lyon` is heavyweight; if we only need cubic Bezier evaluation, implement directly |
| Random | `rand` + `rand_chacha` | Reproducible seeded generation via ChaCha8 PRNG |
| JSON serialization | `serde_json` | Already in workspace, handles DD's JSON format |

### Config Format
YAML via `serde_yaml` for human-authored configs (more readable for complex nested structures like biome definitions). JSON accepted as well via serde's format-agnostic deserialization.

### Asset Strategy
Three-tier approach:
1. **Embedded catalog**: Ship known default asset paths as const data in the crate (from MIMIR-S-0002). Biome presets reference these by semantic name (e.g., `trees::oak_big`) rather than raw paths.
2. **User override**: Config files can specify exact `res://` paths, overriding defaults.
3. **Future: pack discovery**: Optionally parse `.dungeondraft_pack` files to enumerate available third-party assets. Deferred to a later iteration.

## Rationale

- **Standalone crate**: Keeps mapgen decoupled from the database/ORM layer. Enables CLI usage, easier testing, and potential future use as a library by other tools. The Mimir app layer handles integration (saving generated maps to campaigns).
- **Use existing crates where possible**: Perlin noise and random number generation are well-solved problems. Reimplementing adds maintenance burden and bug risk. Marching Squares and Bezier are small enough and DD-specific enough to warrant custom code.
- **YAML config**: The reference impl uses JSON, but its configs are deeply nested and verbose. YAML is more ergonomic for human editing while serde makes format switching trivial.
- **Embedded asset catalog**: Avoids requiring a Dungeondraft installation to generate maps. Default-asset-only maps work out of the box. Third-party pack assets require explicit config (unavoidable since pack IDs are random).

## Consequences

### Positive
- `mimir-mapgen` can be developed and tested independently of the main app
- Reproducible builds via seeded PRNG — same seed + config = same map
- Biome presets "just work" with default assets, no user configuration needed
- YAML configs are readable and version-controllable

### Negative
- New crate adds to workspace build time
- Asset catalog will be incomplete initially — needs community contribution to discover more paths
- Marching Squares and Bezier implementations need thorough testing (no crate safety net)

### Neutral
- The crate produces `.dungeondraft_map` JSON files that can be opened in Dungeondraft OR imported into Mimir — dual-use output