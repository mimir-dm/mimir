---
id: point-light-placement-scatter
level: task
title: "Point light placement: scatter, along-path, and with-objects modes"
short_code: "MIMIR-T-0629"
created_at: 2026-03-15T00:42:41.349107+00:00
updated_at: 2026-03-15T00:42:41.349107+00:00
parent: MIMIR-I-0062
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/todo"


exit_criteria_met: false
initiative_id: MIMIR-I-0062
---

# Point light placement: scatter, along-path, and with-objects modes

**Depends on:** MIMIR-T-0627 (format structs), MIMIR-T-0628 (named IDs)

## Objective

Implement point light generation as a new pipeline stage. Three placement modes: scatter (Poisson disc + noise gating), along-path (sample points along a named feature), with-objects (one light per placed object in a named group). All produce `MapLight` entries pushed to `Level.lights`.

## Backlog Item Details **[CONDITIONAL: Backlog Item]**

{Delete this section when task is assigned to an initiative}

### Type
- [ ] Bug - Production issue that needs fixing
- [ ] Feature - New functionality or enhancement  
- [ ] Tech Debt - Code improvement or refactoring
- [ ] Chore - Maintenance or setup work

### Priority
- [ ] P0 - Critical (blocks users/revenue)
- [ ] P1 - High (important for user experience)
- [ ] P2 - Medium (nice to have)
- [ ] P3 - Low (when time permits)

### Impact Assessment **[CONDITIONAL: Bug]**
- **Affected Users**: {Number/percentage of users affected}
- **Reproduction Steps**: 
  1. {Step 1}
  2. {Step 2}
  3. {Step 3}
- **Expected vs Actual**: {What should happen vs what happens}

### Business Justification **[CONDITIONAL: Feature]**
- **User Value**: {Why users need this}
- **Business Value**: {Impact on metrics/revenue}
- **Effort Estimate**: {Rough size - S/M/L/XL}

### Technical Debt Impact **[CONDITIONAL: Tech Debt]**
- **Current Problems**: {What's difficult/slow/buggy now}
- **Benefits of Fixing**: {What improves after refactoring}
- **Risk Assessment**: {Risks of not addressing this}

## Acceptance Criteria

- [ ] `LightConfig` struct in `pipeline.rs` with placement enum (Scatter, AlongPath, WithObjects) and shared fields (color, intensity, range, shadows, layer, margin)
- [ ] `Scatter` mode: Poisson disc placement gated by noise_lower/upper + probability. Reuses `distribution.rs`
- [ ] `AlongPath` mode: accepts a named feature reference, samples points at `density` intervals along the path
- [ ] `WithObjects` mode: accepts a named object group reference, creates one light per placed object position
- [ ] New `src/lights.rs` module with `generate_lights(config, noise_map, features) -> Vec<MapLight>`
- [ ] Pipeline wires lights stage after objects, pushes to `Level.lights`
- [ ] Config section: `lights: Vec<LightConfig>` in `MapConfig`
- [ ] Lights respect margin (avoid map edges)
- [ ] Generated map with lights opens correctly in Dungeondraft
- [ ] Unit tests for each placement mode
- [ ] `cargo test -p mimir-mapgen` passes

## Implementation Notes

### New file
- `crates/mimir-mapgen/src/lights.rs`

### Reuse
- `distribution.rs` — Poisson disc for scatter mode
- `GeneratedFeatures` registry — path/object lookups for along_path and with_objects

### Gull Rock reference values (from `/tmp/dd-procedural-forest/config.json`)
- Primary ambient: density 0.21, intensity 0.5, color "96eaefca", radius 15
- Road lights: density 0.2, intensity 0.5, color "baeaefca", radius 5
- Tree-top lights: intensity 0.6, color "ffeaefca", radius 1.5

## Status Updates

*To be added during implementation*