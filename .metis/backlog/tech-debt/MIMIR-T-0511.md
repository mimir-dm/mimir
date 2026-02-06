---
id: extract-shared-use-case-layer-to
level: task
title: "Extract shared use-case layer to eliminate Tauri/MCP logic duplication"
short_code: "MIMIR-T-0511"
created_at: 2026-02-02T01:25:04.104041+00:00
updated_at: 2026-02-04T14:07:47.301780+00:00
parent: 
blocked_by: []
archived: false

tags:
  - "#task"
  - "#tech-debt"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: NULL
---

# Extract shared use-case layer to eliminate Tauri/MCP logic duplication

## Objective

Extract shared "use-case" functions in mimir-core that both Tauri commands and MCP tools can call, eliminating duplicated business logic between the two layers. Currently both layers independently implement input validation, module type parsing, error handling, and response construction for the same operations.

## Backlog Item Details

### Type
- [x] Tech Debt - Code improvement or refactoring

### Priority
- [x] P2 - Medium (nice to have)

### Technical Debt Impact
- **Current Problems**: Tauri commands and MCP tools duplicate identical logic — module type parsing (`parse_module_type` in both `commands/module.rs:58-66` and `tools/module.rs:187-194`), timestamp formatting (Tauri uses `%Y-%m-%d %H:%M:%S`, MCP uses `to_rfc3339()`), and input construction. Adding a new field means updating both layers.
- **Benefits of Fixing**: Single source of truth for all business operations. Both Tauri and MCP become thin adapters (deserialize input format → call use-case → serialize response). Adding a third consumer (CLI, REST) becomes trivial.
- **Risk Assessment**: Medium — requires careful design of use-case input/output types that work for both sync (Tauri) and async (MCP) callers.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] Shared use-case functions in mimir-core accept typed input structs and return `ServiceResult<T>`
- [ ] Tauri commands are thin wrappers: parse request → call use-case → wrap in `ApiResponse`
- [ ] MCP tools are thin wrappers: parse JSON args → call use-case → wrap in JSON response
- [ ] Module type parsing, timestamp formatting, and input validation happen in exactly one place
- [ ] All tests pass, no behavioral regressions

## Implementation Notes

### Technical Approach
Extend the existing service layer pattern. For entities that already have services (Campaign, Character, Module, Document), ensure all logic lives there. For entities that don't (homebrew — see MIMIR-T-0509), create services. The key principle: if logic appears in both a Tauri command and an MCP tool, it belongs in a service.

### Dependencies
- MIMIR-T-0509 (HomebrewService) is a subset of this work and should be done first

## Status Updates

### 2026-02-04: Decomposed into subtasks

Analysis complete. Task decomposed into 5 focused subtasks:

| Code | Task | Size |
|------|------|------|
| MIMIR-T-0517 | Extract ModuleType parsing to enum impl | Small |
| MIMIR-T-0518 | Create centralized timestamp utility | Small |
| MIMIR-T-0519 | Unify catalog query limit constants | Small |
| MIMIR-T-0520 | Extract character enrichment to CharacterService | Medium |
| MIMIR-T-0521 | Create TokenService and MCP token tools | Large |

This parent task marked complete - work continues in subtasks.