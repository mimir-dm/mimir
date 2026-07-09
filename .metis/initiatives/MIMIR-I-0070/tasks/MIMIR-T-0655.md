---
id: traps-pois-into-mapstateservice
level: task
title: "Traps + POIs into MapStateService"
short_code: "MIMIR-T-0655"
created_at: 2026-07-08T11:59:13.999115+00:00
updated_at: 2026-07-09T02:53:14.808789+00:00
parent: MIMIR-I-0070
blocked_by: [MIMIR-T-0653]
archived: false

tags:
  - "#task"
  - "#phase/active"


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

## Acceptance Criteria

## Acceptance Criteria **[REQUIRED]**

- [ ] Trap methods: `list_traps`, `get_trap`, `create_trap`, `update_trap`, `move_trap`, `toggle_trap_visibility`, `trigger_trap`, `reset_trap`, `delete_trap` (+ visible/armed list variants used by the UI)
- [ ] POI methods: `list_pois`, `get_poi`, `create_poi`, `update_poi`, `move_poi`, `toggle_poi_visibility`, `delete_poi`
- [ ] Both command files become thin wrappers; read-then-flip logic (visibility, trigger state) lives in the service
- [ ] Service unit tests: trap lifecycle (create → trigger → reset), visibility toggles, POI CRUD
- [ ] Tauri command names/args/response shapes unchanged
- [ ] Full workspace suite green

## Status Updates **[REQUIRED]**

- 2026-07-09: COMPLETE on `feat/map-state-service`. All trap ops (incl.
  visible/armed list variants) and POI ops in MapStateService with typed input
  structs (CreateTrapInput/UpdateTrapInput/CreatePoiInput/UpdatePoiInput).
  Trigger/reset lifecycle and both read-then-flip visibility toggles behind the
  seam with first-ever unit tests (4 new: lifecycle + armed-list, trap
  visibility/update/move/delete, POI CRUD + visibility, missing-id NotFound).
  traps.rs 259→~180, pois.rs 211→~150 — pure wrappers. Wire unchanged.
  Workspace suite 1,508 green. All four map/ command files now hold zero
  business logic; the audit's 1,034-line finding is fully resolved. T-0656
  (MCP authoring tools) is unblocked.