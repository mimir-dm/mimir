---
id: cross-reference-rendering-tests
level: task
title: "Cross-reference rendering tests — modal formatters for all ref types (@spell, @item, @creature, etc.)"
short_code: "MIMIR-T-0552"
created_at: 2026-03-10T01:31:41.825576+00:00
updated_at: 2026-03-10T01:31:41.825576+00:00
parent: MIMIR-I-0056
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/todo"


exit_criteria_met: false
initiative_id: MIMIR-I-0056
---

# Cross-reference rendering tests — modal formatters for all ref types (@spell, @item, @creature, etc.)

**Phase 4** — Catalog & Search Coverage

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[MIMIR-I-0056]]

## Objective

Write Vitest tests for the cross-reference rendering system — the modal formatters that handle 5etools inline references like `{@spell fireball}`, `{@creature goblin}`, `{@item longsword}`, `{@class wizard}`, `{@race elf}`, `{@background soldier}`, `{@condition frightened}`, `{@dice 2d6}`, etc. These are used throughout the app in document rendering, stat blocks, and descriptions.

## Acceptance Criteria

- [ ] `{@spell name}` references resolve to spell data and render a clickable link
- [ ] `{@creature name}` references resolve to monster stat block
- [ ] `{@item name}` references resolve to item detail
- [ ] `{@class name}` references link to class info
- [ ] `{@race name}` references link to race info
- [ ] `{@condition name}` references show condition tooltip/modal
- [ ] `{@dice XdY+Z}` references render with dice notation
- [ ] `{@damage XdY}` references render damage dice
- [ ] `{@dc X}` references render DC value
- [ ] References with display text override (`{@spell fireball|PHB|a fireball}`) use the override text
- [ ] References with source hints (`{@creature goblin|MM}`) resolve correctly
- [ ] Unknown/malformed references render gracefully (don't crash)
- [ ] All tests pass in CI

## Key Components

- `textFormatting.ts` — main tag parser
- Individual formatter files in `features/sources/formatters/`
- Reference modal/tooltip components

## Implementation Notes

The `textFormatting.ts` utility is the core of this — it's a pure function that can be unit tested directly. Test both the parsing (extracting tag type, name, source, display text) and the rendering (producing correct HTML/component output). The formatters are mostly pure functions too. Focus on edge cases: nested references, malformed tags, missing sources.

## Status Updates

*To be added during implementation*