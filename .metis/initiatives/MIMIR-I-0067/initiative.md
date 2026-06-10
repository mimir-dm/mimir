---
id: play-mode-combat-tools
level: initiative
title: "Play Mode Combat Tools"
short_code: "MIMIR-I-0067"
created_at: 2026-05-08T13:12:30.887506+00:00
updated_at: 2026-05-08T13:12:30.887506+00:00
parent: MIMIR-V-0001
blocked_by: []
archived: false

tags:
  - "#initiative"
  - "#phase/discovery"


exit_criteria_met: false
estimated_complexity: L
initiative_id: play-mode-combat-tools
---

# Play Mode Combat Tools Initiative

## Context

`ModulePlayView.vue` is the most polished single screen in the app:

- Yellow `border-bottom: 2px solid var(--color-warning)` on the play header signals "you are live" — strongest single design signal in Mimir.
- Auto-saving notes panel (`usePlayNotes` composable) with collapse, "Saving…" / "Saved" indicators.
- Collapsible left sidebar with Monsters and Maps lists.
- Cross-reference tooltips for monster stat block links.
- Player display window controls (open/close, blackout) integrated.
- Map viewer with grid, lighting modes, token rendering.

But it's missing the central DM tool: **nothing tracks initiative order, manages HP, surfaces concentration, or tracks status effects.** The actual loop a DM runs at the table happens in their head + paper. The polished surface around an empty center is the most disorienting thing in the app — users feel "this is almost there" without being able to articulate the gap.

## Goals & Non-Goals

**Goals:**
- Initiative tracker that orders party members + active monsters + custom additions, with turn advancement.
- HP management for monsters (and optionally PCs/NPCs) with damage/heal increments and per-creature damage history.
- Concentration tracking with prompted save when a concentration-bearing creature takes damage.
- Status effect tracking (basic 5e SRD conditions) per creature with optional duration counters.
- Selected creature's HP / initiative state surfaces in the existing Monster Stats Panel.
- Optional player-display readout of initiative order (without HP, to preserve surprise).

**Non-Goals:**
- Automated dice rolling (stay opt-in or link out to existing tools).
- Combat encounter builder (handled at module-prep time, not during play).
- Spell-slot tracking for casters (later initiative).
- Cross-session combat persistence (single-encounter scope first; persistence is v2).

## Architecture

### Overview

Combat session is a per-encounter, in-memory or per-encounter-persisted concept tied to the active module/play view. Backend extends `mimir-core` with a CombatSession model. Frontend adds a single `InitiativeTracker.vue` component plus a small store/composable to drive it.

### Data Model

- `CombatSession` — id, module_id, started_at, current_round, current_turn_index, status (active/ended).
- `InitiativeEntry` — id, session_id, source_kind (`monster_module` | `character` | `custom`), source_id (nullable for custom), display_name, init_roll, current_hp, max_hp, conditions (Vec<Condition>), is_concentrating, concentration_dc (nullable).
- `Condition` — id, name (5e SRD enum), source (description), expires_round (nullable).

Persistence decision (per-encounter vs in-memory) deferred to design phase; default toward per-encounter so a DM can recover from a crash without losing combat state.

### Map Token Sync

Tokens already on the map have HP from monster stat blocks. Initiative entries reference tokens by ID where applicable; HP changes propagate both directions (token tracker → initiative tracker → monster stats panel).

## Detailed Design

- **`InitiativeTracker.vue`** — new component, lives as a right-edge drawer (preferred) or top stripe under the play-header. Decision deferred to design phase, with mockups.
- **State** — `useCombatSession(moduleId)` composable mirrors the `usePlayNotes` pattern. Loads session on mount, autosaves changes, exposes ordered entries + currentTurn helpers.
- **Add entries** — quick-add UI: pick from module's Monsters list (auto-populates HP/AC), pick from PCs (manual init roll), or "Custom" (free-text name + init only).
- **Damage/heal** — number input + +/- buttons + recent-damage history (last 3 hits) per entry.
- **Conditions** — small icon pills next to each entry. Tap to add/remove. Source 5e SRD conditions (charmed, frightened, paralyzed, poisoned, prone, restrained, stunned, unconscious, etc.).
- **Concentration** — when a creature with `is_concentrating: true` takes damage, surface inline prompt: "Save vs DC X for concentration?" Half-damage rule auto-applied; manual override.
- **Turn advancement** — single button advances to next entry; wraps to round 2 with a visible "Round 2" badge. Entries can be re-rolled or edited mid-round.

## UI/UX Design

Two layout candidates, AskUserQuestion at design phase:

- **Right-edge drawer.** Vertical list of ~10 entries with init number, name, HP bar, condition pill row, current-turn highlight. Default open in play mode; collapsible.
- **Top stripe.** Horizontal scrolling chips under the play-header. Less screen-eating but harder to track HP at a glance.

Selected entry expands inline (or focuses the existing Monster Stats Panel — preferred to preserve the existing flow).

## Alternatives Considered

- **Bolt on an external combat tracker** (improved-initiative.com or similar). Rejected. Mimir already has all the data; staying in-app preserves cross-references and player-display sync.
- **Combat tracker as a separate window** (like player-display). Rejected — the DM is already managing the play view, monster stats panel, and notes. Adding another floating window is a regression.
- **No separate tracker; embed turn order in the existing Monsters sidebar.** Rejected — the existing sidebar is for *picking* a monster to inspect, not for running combat. Conflating both surfaces makes both worse.

## Implementation Plan

- **Phase 1 — Spec the data model** (CombatSession + InitiativeEntry + Condition). Persistence decision. UI placement decision (drawer vs stripe). One spec doc + AskUserQuestion check-ins.
- **Phase 2 — Backend** — `mimir-core` types, DAL, service layer, Tauri commands. ADR if persistence decisions are non-obvious.
- **Phase 3 — Initiative tracker MVP** (frontend) — add/remove entries, sort by init, advance turn, manual init rolls. No HP yet.
- **Phase 4 — HP tracking** — damage/heal UI, recent-damage history, max HP from monster source.
- **Phase 5 — Conditions** — pills, add/remove, 5e SRD enum.
- **Phase 6 — Concentration prompts** — save-on-damage flow.
- **Phase 7 — Player-display readout** — optional initiative list rendered to the secondary window.
- **Phase 8 — Map token sync** — HP changes flow between token state and initiative entry.