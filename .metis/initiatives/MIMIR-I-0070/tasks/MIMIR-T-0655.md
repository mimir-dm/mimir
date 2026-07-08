---
id: traps-pois-into-mapstateservice
level: task
title: "Traps + POIs into MapStateService"
short_code: "MIMIR-T-0655"
created_at: 2026-07-08T11:59:13.999115+00:00
updated_at: 2026-07-08T11:59:13.999115+00:00
parent: MIMIR-I-0070
blocked_by: ["MIMIR-T-0653"]
archived: false

tags:
  - "#task"
  - "#phase/todo"


exit_criteria_met: false
initiative_id: MIMIR-I-0070
---

# Traps + POIs into MapStateService

## Parent Initiative

[[MIMIR-I-0070]] — MapStateService: fog, lights, traps, POIs behind one seam

## Objective **[REQUIRED]**

Hoist traps (`traps.rs`, 259 lines) and POIs (`pois.rs`, 211 lines) into
`MapStateService`. The two subsystems have identical shapes (CRUD + move +
visibility toggle, plus the trap trigger/reset lifecycle) — one mechanical task
once the pilot pattern exists.

## Acceptance Criteria **[REQUIRED]**

- [ ] Trap methods: `list_traps`, `get_trap`, `create_trap`, `update_trap`, `move_trap`, `toggle_trap_visibility`, `trigger_trap`, `reset_trap`, `delete_trap` (+ visible/armed list variants used by the UI)
- [ ] POI methods: `list_pois`, `get_poi`, `create_poi`, `update_poi`, `move_poi`, `toggle_poi_visibility`, `delete_poi`
- [ ] Both command files become thin wrappers; read-then-flip logic (visibility, trigger state) lives in the service
- [ ] Service unit tests: trap lifecycle (create → trigger → reset), visibility toggles, POI CRUD
- [ ] Tauri command names/args/response shapes unchanged
- [ ] Full workspace suite green

## Status Updates **[REQUIRED]**

*To be added during implementation*
