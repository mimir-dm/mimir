---
id: module-view-tests-module-list
level: task
title: "Module view tests — module list, document list, monster list, NPC list, map list"
short_code: "MIMIR-T-0544"
created_at: 2026-03-10T01:31:29.089569+00:00
updated_at: 2026-03-10T01:31:29.089569+00:00
parent: MIMIR-I-0056
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/todo"


exit_criteria_met: false
initiative_id: MIMIR-I-0056
---

# Module view tests — module list, document list, monster list, NPC list, map list

**Phase 3** — Campaign & Module Coverage

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[MIMIR-I-0056]]

## Objective

Write Vitest component tests for the module view, covering list rendering for all entity types within a module: documents, monsters, NPCs/characters, and maps. Test the ModulePlayView and its sub-panels.

## Acceptance Criteria

- [ ] Module list view renders modules with names, descriptions, and entity counts
- [ ] Module document list renders documents in correct order with titles
- [ ] Module monster list renders monsters with names, CR, source, quantity, and display name overrides
- [ ] Module NPC list renders characters assigned to the module
- [ ] Module map list renders maps with names and dimensions
- [ ] Selecting a module loads its details via `get_module_details`
- [ ] Empty states for modules with no entities of a given type
- [ ] All tests pass in CI

## Key Components

- `ModulePlayView.vue` — main module view with tabs/panels
- `ModuleMonsters.vue` — monster list panel
- Document list panel
- Map list panel
- NPC/character panel within module

## Implementation Notes

Mock invoke calls for `list_modules`, `get_module_details`, `list_module_monsters`, `list_module_documents`, `list_module_maps`. Use SRD fixture data for monster entries.

## Status Updates

*To be added during implementation*