---
id: light-sources-into-mapstateservice
level: task
title: "Light sources into MapStateService (presets included)"
short_code: "MIMIR-T-0654"
created_at: 2026-07-08T11:59:13.055907+00:00
updated_at: 2026-07-09T01:30:55.785509+00:00
parent: MIMIR-I-0070
blocked_by: [MIMIR-T-0653]
archived: false

tags:
  - "#task"
  - "#phase/active"


exit_criteria_met: false
initiative_id: MIMIR-I-0070
---

# Light sources into MapStateService (presets included)

## Parent Initiative

[[MIMIR-I-0070]] — MapStateService: fog, lights, traps, POIs behind one seam

## Objective **[REQUIRED]**

Hoist the light-source subsystem from `crates/mimir/src/commands/map/light.rs`
(313 lines, 10 commands) into `MapStateService`, including the torch/lantern
presets.

## Acceptance Criteria

## Acceptance Criteria **[REQUIRED]**

- [ ] Service methods: `list_lights`, `create_light`, `create_torch`, `create_lantern`, `update_light`, `move_light`, `toggle_light` (read-then-flip in service), `delete_light`, `delete_all_lights`
- [ ] Torch/lantern preset values (radii etc.) move from light.rs into the service as constants/constructors
- [ ] `light.rs` commands become thin wrappers; `LightSourceResponse` conversion stays command-side
- [ ] Service unit tests: preset values pinned, toggle round-trip, delete_all count
- [ ] Tauri command names/args/response shapes unchanged (`useLightSources` composable tests untouched)
- [ ] Full workspace suite green

## Implementation Notes

- Per the refined design ruling (2026-07-08), light *placement* IS agent-exposable
  (any token/object placement is world-building) — MCP tools for it land in
  [[MIMIR-T-0656]]. What stays UI-only is flipping lights on/off during live play
  (`toggle_light` gets no MCP tool).

## Status Updates **[REQUIRED]**

*To be added during implementation*