---
id: split-characterservice-along-its
level: initiative
title: "Split CharacterService along its internal seams"
short_code: "MIMIR-I-0071"
created_at: 2026-07-08T11:07:59.493146+00:00
updated_at: 2026-07-08T11:07:59.493146+00:00
parent: 
blocked_by: []
archived: false

tags:
  - "#initiative"
  - "#phase/discovery"


exit_criteria_met: false
estimated_complexity: S
initiative_id: split-characterservice-along-its
---

# Split CharacterService along its internal seams Initiative

## Context **[REQUIRED]**

Candidate #3 (Worth exploring) from the 2026-07-07 architecture review.

`crates/mimir-core/src/services/character.rs` is 2,249 lines — a deep
implementation with poor navigability. Four real modules are trapped in one file:

- The level-up engine (LevelUpRequest: subclass choices, ASI-vs-feat, HP methods,
  spell changes) — the most intricate domain logic in the codebase
- CRUD + enrichment (create/update/delete, CharacterResponse assembly)
- Inventory (add/update/remove/equipped/attuned)
- Spells (added 2026-07-07 in fc5a3d5)

Understanding "how does leveling work" costs the whole 2,249-line file — for a
human or an AI agent. The internal seams are real and unmarked; inventory and
spells share nothing with level-up but a connection handle.

## Goals & Non-Goals **[REQUIRED]**

**Goals:**
- `services/character/` directory: `core.rs` (CRUD + enrichment), `levelup.rs` (engine + request/result types), `inventory.rs`, `spells.rs`
- `CharacterService`'s public interface unchanged — this reveals internal module boundaries, it does not move the seam callers see
- The level-up engine gains a dedicated table-driven test module (multiclass, subclass timing, ASI edge cases)

**Non-Goals:**
- Any behavior change whatsoever (pure structural refactor; full suite must pass untouched)
- Splitting the public service into multiple services (one interface, better internals)

## Detailed Design **[REQUIRED]**

Mechanical: convert `character.rs` into `character/mod.rs` re-exporting the same
names; move impl blocks by concern (`impl CharacterService` can be split across
files within the module). Level-up types (LevelUpRequest, HpGainMethod, AsiOrFeat,
SubclassChoice, etc.) move with the engine. Existing tests move next to their
concern; add table-driven level-up cases.

## Alternatives Considered **[REQUIRED]**

- **Leave it**: viable (the module is deep and correct today) — rejected because navigability is a stated project value (AI-operated codebase) and the file grows with every consolidation initiative.
- **Split the public interface into LevelUpService/InventoryService/etc.**: rejected — widens the seam both frontends must know, for no depth gain.

## Implementation Plan **[REQUIRED]**

1. Directory conversion + move CRUD/enrichment (no logic edits), suite green
2. Move level-up engine + inventory + spells, suite green
3. Add table-driven level-up tests

Independent of the other initiatives; lowest risk of the set.
