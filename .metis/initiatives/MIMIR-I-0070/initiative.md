---
id: mapstateservice-fog-lights-traps
level: initiative
title: "MapStateService: fog, lights, traps, POIs behind one seam"
short_code: "MIMIR-I-0070"
created_at: 2026-07-08T11:07:58.553348+00:00
updated_at: 2026-07-08T15:01:21.492174+00:00
parent: 
blocked_by: []
archived: false

tags:
  - "#initiative"
  - "#phase/active"


exit_criteria_met: false
estimated_complexity: M
initiative_id: mapstateservice-fog-lights-traps
---

# MapStateService: fog, lights, traps, POIs behind one seam Initiative

## Context **[REQUIRED]**

Candidate #2 (Strong) from the 2026-07-07 architecture review.

Fog of war, light sources, traps, and POIs are the last surviving instance of the
anti-pattern that caused the 2026-07-07 production incident (MIMIR-T-0643):
business logic living in one frontend's adapter, written directly against `dal::`.

- `crates/mimir/src/commands/map/fog.rs` — 251 lines (toggle reads map state then writes the inverse)
- `crates/mimir/src/commands/map/light.rs` — 313 lines (torch/lantern presets inline)
- `crates/mimir/src/commands/map/traps.rs` — 259 lines (trigger/reset lifecycle)
- `crates/mimir/src/commands/map/pois.rs` — 211 lines

= 1,034 lines, ~30 Tauri commands, zero unit tests possible (only testable by
driving a Tauri command end-to-end), and invisible to the MCP server — an agent
can manage a whole campaign but cannot reveal fog or place a light. Same
capability-gap shape as the missing `update_module_monster` that made the
duplicate-row cleanup painful.

Deletion test passes emphatically: delete the four command files and the
fog/light/trap rules have nowhere to go but a service.

## Goals & Non-Goals **[REQUIRED]**

**Goals:**
- `MapStateService` in `mimir-core/src/services/` owning fog ops, light ops (incl. presets), trap lifecycle, POI CRUD
- Tauri commands become 6-line wrappers (same shape as fc5a3d5 did for module monsters)
- Service-level unit tests on in-memory DB for all four subsystems
- MCP exposure becomes possible (actual MCP tools may land here or as a fast follow, especially if MIMIR-I-0069's registry exists first)

**Non-Goals:**
- Changing frontend Vue composables (useFog, useLightSources, etc.) — the Tauri command signatures stay wire-compatible
- Vision/visibility calculation (lives in the frontend composables today; separate question)
- map/crud.rs and uvtt.rs — already routed through MapService

## Detailed Design **[REQUIRED]**

Rough shape: one service (or one with four sub-impls) mirroring the SourceService
pattern: `MapStateService::new(conn)` with `fog_state`, `toggle_fog`, `reveal_rect`,
`reveal_circle`, `reveal_all`, `reset_fog`, `delete_revealed_area`; `list_lights`,
`add_light`, `add_torch`, `add_lantern`, `update_light`, `move_light`, `toggle_light`,
`delete_light`; trap lifecycle (`create/list/update/move/toggle/trigger/reset/delete`);
POI CRUD. Presets (torch/lantern radii) move from light.rs into the service as
constants/constructors.

## Alternatives Considered **[REQUIRED]**

- **Fold into existing MapService**: possible, but MapService is about map records/UVTT assets; table-state (fog/lights/traps/POIs) is a distinct concern with its own churn. Keep the seam explicit; revisit at design.
- **Leave as Tauri-only**: rejected — it is the proven incident-generating pattern and blocks agent-driven session prep.

## Implementation Plan **[REQUIRED]**

1. Service skeleton + fog (smallest surface, carries design decisions), rewire fog.rs, tests
2. Lights (presets move into service), rewire light.rs
3. Traps + POIs (identical shapes, one mechanical task), rewire
4. MCP placement tools for traps, POIs, and lights via the registry

**Design ruling (Dylan, 2026-07-08, refined): agents build worlds, people run
them.** The MCP interface may place and edit ANY token/object (traps, POIs,
lights — full authoring power). What it must never do is edit the viewable
layer during live play: fog reveal/toggle, trap trigger/reset,
player-visibility toggles, flipping lights on/off mid-session. Those remain
Tauri-UI-only. Fog is excluded from MCP entirely (pure viewable-layer state).
Recorded in memory (feedback_agent_prep_not_play). This supersedes the "MCP
tools for fog" idea in the original goals.

Design decisions: single `MapStateService::new(conn)` in mimir-core (no
app_data_dir — table-state touches no assets); presentation types (FogState,
LightSourceResponse) stay command-side; service returns models.

Related: registry from MIMIR-I-0069 makes task 4 one registration per tool.