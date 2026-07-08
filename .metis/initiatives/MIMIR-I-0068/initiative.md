---
id: consolidate-frontend-business-logic
level: initiative
title: "Consolidate frontend business logic into mimir-core services"
short_code: "MIMIR-I-0068"
created_at: 2026-07-07T22:00:00.000000+00:00
updated_at: 2026-07-07T22:00:00.000000+00:00
parent:
blocked_by: []
archived: false

tags:
  - "#initiative"
  - "#phase/discovery"

exit_criteria_met: false
complexity: M
---

# Consolidate frontend business logic into mimir-core services

## Context

Mimir has two frontends — Tauri commands (`crates/mimir/src/commands/`) and MCP tools
(`crates/mimir-mcp/src/tools/`) — over a shared `mimir-core` (models → dal → services).
The T-0511 refactor moved business logic into services, but incompletely. Where a service
method is missing, each frontend hand-rolls the operation against raw `dal::`, and the
two copies have drifted.

**This caused real production damage on 2026-07-07** (see MIMIR-T-0643): the MCP
`add_monster_to_module` has no duplicate detection while the Tauri command dedups and
increments quantity — client retries via Claude Desktop created duplicate roster rows
that required manual DB surgery.

## Audit (2026-07-07)

Full audit performed across both frontends. Summary:

**Cleanly shared (no action):** campaign core CRUD, documents, homebrew (items/monsters/
spells), maps core CRUD, tokens, archive export/import, level-up, character CRUD +
inventory. All route through CampaignService / DocumentService / HomebrewService /
MapService / TokenService / ArchiveService / CharacterService.

**Every true divergence maps to a missing service surface:**

| # | Operation | Divergence | Risk |
|---|-----------|-----------|------|
| 1 | module_monsters.add | Tauri (`commands/module.rs:258`) dedups + increments quantity, requires name+source; MCP (`tools/module.rs:333`) blind-inserts, defaults source="MM" | HIGH — duplicate rows from retries (observed in prod); dedup can never match across frontends |
| 2 | character_spells.remove | Tauri (`commands/character.rs:678`) case-insensitive name match; MCP (`tools/character.rs:1093`) case-sensitive | MED — "fireball" removes in UI, silently fails via MCP |
| 3 | character_spells.add | Both hand-roll identical dedup via `find_character_spell_by_name` — consistent today, copy-paste fragile | MED — next rule change lands on one side only |
| 4 | module_monsters.update | Tauri-only (`commands/module.rs:361`); MCP cannot adjust quantity/notes | MED — quantity state only maintainable from UI |
| 5 | campaign/character sources set | Tauri-only, non-transactional delete-all + reinsert loop (`commands/campaign.rs:257`, `commands/character.rs:554`) | MED — mid-loop failure wipes sources |
| 6 | module_monsters.remove / list | Parallel raw-dal implementations, same behavior today | LOW |
| 7 | catalog.search | Independent read paths; MCP merges homebrew into results, Tauri does not | LOW (reads) |

Root cause pattern: `ModuleService` has zero module-monster methods, `CharacterService`
has zero spell methods, and no source service exists. Those three gaps contain all the
real duplication.

## Goal

One implementation per business operation, living in `mimir-core` services. Frontends
(Tauri commands, MCP tools) only translate transport formats — no validation, dedup,
defaults, or multi-step writes outside services.

## Proposed decomposition (pending approval)

1. **ModuleService monster methods** — `add_monster` (Tauri semantics: XOR
   catalog/homebrew ref, homebrew-existence check, dedup → quantity increment),
   `update_monster`, `remove_monster`, `list_monsters`. Rewire both frontends. Expose
   `update_module_monster` as a new MCP tool (closes gap #4). Covers #1, #4, #6.
2. **CharacterService spell methods** — `add_spell` (dedup), `remove_spell`
   (case-insensitive match, adopting Tauri semantics), `toggle_prepared`,
   `list_spells`. Rewire both frontends. Covers #2, #3.
3. **Source management service** — transactional `set_campaign_sources` /
   `set_character_sources` (single transaction, no partial wipes) + add/remove/list.
   Rewire Tauri; optionally expose to MCP. Covers #5.
4. **Parity regression tests** — behavioral tests in mimir-core for each new service
   method; MCP functional tests updated to pin the shared semantics (e.g., identical
   add → quantity increment, not duplicate row).

Out of scope: catalog read-path unification (#7, low risk), Tauri-only map subsystems
(fog/light/traps/pois — single-frontend, no drift), MCP presentation logic (homebrew
auto-source detection, flat-arg assembly) which stays in tools by design.

## Exit criteria

- [x] No `dal::` write calls with business rules remain in either frontend for module
      monsters, character spells, or sources
- [x] Both frontends produce identical DB state for the same logical operation
- [x] Retrying an identical add_monster_to_module call does not create duplicate rows
- [x] Full workspace test suite green

## Status updates

- 2026-07-07: Created from drift audit following the Claude Desktop incident
  (MIMIR-T-0643). Decomposition proposed, awaiting user approval.

- 2026-07-07: All four tasks implemented on branch `feat/service-layer-consolidation`:
  ModuleService monster methods (add w/ dedup+increment, update, remove, list) — both
  frontends rewired, new `update_module_monster` MCP tool; CharacterService spell methods
  (case-insensitive removal adopted everywhere); SourceService with transactional set for
  campaign+character sources — Tauri rewired. Parity tests added (increment-not-duplicate,
  case-insensitive removal, atomic source replace). Full workspace suite green (~1,490 tests).
