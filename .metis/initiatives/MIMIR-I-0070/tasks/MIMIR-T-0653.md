---
id: mapstateservice-skeleton-fog
level: task
title: "MapStateService skeleton + fog subsystem"
short_code: "MIMIR-T-0653"
created_at: 2026-07-08T11:59:11.687993+00:00
updated_at: 2026-07-08T15:02:38.058274+00:00
parent: MIMIR-I-0070
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/active"


exit_criteria_met: false
initiative_id: MIMIR-I-0070
---

# MapStateService skeleton + fog subsystem

## Parent Initiative

[[MIMIR-I-0070]] — MapStateService: fog, lights, traps, POIs behind one seam

## Objective **[REQUIRED]**

Create `MapStateService` in `mimir-core/src/services/map_state.rs` and hoist the
fog subsystem — the smallest surface, carrying the design decisions for the
other three tasks.

## Acceptance Criteria

## Acceptance Criteria **[REQUIRED]**

- [ ] `MapStateService::new(conn)` (no app_data_dir — table-state touches no assets), registered in services/mod.rs
- [ ] Fog ops hoisted from `crates/mimir/src/commands/map/fog.rs` (251 lines): `fog_state` (map flag + revealed areas), `enable_fog`, `disable_fog`, `toggle_fog` (read-then-flip inside the service), `reveal_rect`, `reveal_circle`, `reveal_all`, `reset_fog`, `delete_revealed_area`
- [ ] `fog.rs` commands become thin wrappers (connect → service call → to_api_response); `FogState` response type stays command-side, service returns models
- [ ] Service unit tests on in-memory DB: toggle round-trip, reveal shapes persist, reset clears
- [ ] Tauri command names, arguments, and response shapes unchanged (Vue `useFog` composable and its tests must not need edits)
- [ ] Full workspace suite green

## Implementation Notes

- Mirror the SourceService pattern (fc5a3d5) — same file layout, same test style.
- The dal layer (dal/campaign/fog.rs) is complete; this task moves logic, not queries.
- Watch the id-generation + fetch-back pattern in reveal_*: service should own uuid generation and return the created model.

## Status Updates **[REQUIRED]**

*To be added during implementation*