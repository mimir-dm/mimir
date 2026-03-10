---
id: homebrew-rendering-tests-homebrew
level: task
title: "Homebrew rendering tests — homebrew items, monsters, and spells display correctly"
short_code: "MIMIR-T-0559"
created_at: 2026-03-10T01:31:54.887276+00:00
updated_at: 2026-03-10T01:31:54.887276+00:00
parent: MIMIR-I-0056
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/todo"


exit_criteria_met: false
initiative_id: MIMIR-I-0056
---

# Homebrew rendering tests — homebrew items, monsters, and spells display correctly

**Phase 6** — Homebrew & Advanced Features

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[MIMIR-I-0056]]

## Objective

Write Vitest component tests for rendering homebrew content — verifying that homebrew items, monsters, and spells display correctly when viewed in the catalog browser, module monster list, character inventory, and stat block views. This tests the consumer side (rendering) rather than the producer side (CRUD, covered in MIMIR-T-0558).

## Acceptance Criteria

- [ ] Homebrew item displays with "HB" source badge in catalog/inventory views
- [ ] Homebrew monster stat block renders correctly from custom JSON data
- [ ] Homebrew spell renders with correct fields from custom JSON
- [ ] Homebrew items in character inventory show correct type icon and properties
- [ ] Homebrew monsters in module monster list show correct CR and source
- [ ] Clone-from-catalog homebrew items retain original data with modifications
- [ ] Missing/malformed homebrew JSON data renders gracefully (no crash)
- [ ] All tests pass in CI

## Key Components

- Item/monster/spell detail renderers (same as catalog, but with HB data)
- Character inventory section (homebrew item display)
- Module monster list (homebrew monster display)
- Source badge / "HB" indicator

## Implementation Notes

Reuse the catalog rendering test patterns from MIMIR-T-0548/0549/0550 but with homebrew-shaped fixture data. The key difference is that homebrew data comes from `campaign_homebrew_*` tables and may have slightly different JSON structure than catalog data. Test that the renderers handle both formats gracefully.

## Status Updates

*To be added during implementation*