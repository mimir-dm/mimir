---
id: migrate-character-tool-family
level: task
title: "Migrate character tool family (typed args)"
short_code: "MIMIR-T-0650"
created_at: 2026-07-08T11:12:12.881025+00:00
updated_at: 2026-07-08T11:27:33.768139+00:00
parent: MIMIR-I-0069
blocked_by: [MIMIR-T-0648]
archived: false

tags:
  - "#task"
  - "#phase/active"


exit_criteria_met: false
initiative_id: MIMIR-I-0069
---

# Migrate character tool family (typed args)

## Parent Initiative

[[MIMIR-I-0069]] — MCP tool registry with typed arguments

## Objective **[REQUIRED]**

Migrate the character family — the biggest win: 13 tools in 1,129 lines of
`tools/character.rs`, most of it hand-rolled `args.get(...)` parsing. This is
where typed argument structs pay the most.

## Acceptance Criteria

## Acceptance Criteria **[REQUIRED]**

- [ ] All 13 tools (list/get/create/edit/delete character, inventory ×4, spells ×3, level_up) on the registry with typed args
- [ ] `level_up_character`'s flat-args → `LevelUpRequest` assembly becomes a typed struct with a `TryFrom`/builder into the service request (asi/hp/subclass conditionals preserved exactly)
- [ ] `add_item_to_character`'s homebrew auto-source detection kept as handler pre-logic (documented as presentation, not moved to the service)
- [ ] Wire compatibility: character functional tests (inventory lifecycle, spells lifecycle incl. case-insensitive removal, level-up) pass unchanged
- [ ] `tools/character.rs` line count drops materially (expect roughly half)

## Implementation Notes

- Watch optional-vs-required drift: the current hand parsers are the de-facto contract — derive the structs from what the PARSERS accept today, not from what the schemas claim, then fix any schema that disagrees (that's the drift this initiative exists to kill; note each fix in status updates).

## Status Updates **[REQUIRED]**

*To be added during implementation*