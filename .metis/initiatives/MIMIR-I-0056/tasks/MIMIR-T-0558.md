---
id: homebrew-crud-tests-item-monster
level: task
title: "Homebrew CRUD tests — item, monster, and spell create/update/delete flows"
short_code: "MIMIR-T-0558"
created_at: 2026-03-10T01:31:53.546061+00:00
updated_at: 2026-03-10T01:31:53.546061+00:00
parent: MIMIR-I-0056
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/todo"


exit_criteria_met: false
initiative_id: MIMIR-I-0056
---

# Homebrew CRUD tests — item, monster, and spell create/update/delete flows

**Phase 6** — Homebrew & Advanced Features

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[MIMIR-I-0056]]

## Objective

Write Vitest component tests for homebrew item, monster, and spell CRUD flows in the HomebrewTab. Test the create/edit forms, validation, data submission via invoke, delete confirmations, and list updates after mutations.

## Acceptance Criteria

- [ ] Homebrew item create form submits correct JSON data including 5etools type code
- [ ] Homebrew item edit form pre-populates existing data and saves changes
- [ ] Homebrew item delete shows confirmation and removes from list
- [ ] Homebrew monster create form submits correct stat block JSON
- [ ] Homebrew monster edit form pre-populates and saves
- [ ] Homebrew monster delete shows confirmation and removes from list
- [ ] Homebrew spell create form submits correct spell JSON
- [ ] Homebrew spell edit/delete work correctly
- [ ] Form validation prevents submission of invalid data (missing name, etc.)
- [ ] `formToDataJson()` produces correct 5etools-compatible JSON for all item types
- [ ] Lists refresh after create/update/delete
- [ ] All tests pass in CI

## Key Components

- `HomebrewTab.vue` — main homebrew management panel
- Item/monster/spell create/edit modals
- `formToDataJson()` utility for item type code mapping

## Implementation Notes

The `formToDataJson()` function in HomebrewTab.vue is critical — it maps frontend form values to 5etools JSON format including type codes. Test this function directly in addition to testing the form submission flow. Use the invoke mock to verify the correct invoke commands are called with properly structured payloads.

## Status Updates

*To be added during implementation*