---
id: split-large-command-files-by-sub
level: task
title: "Split large command files by sub-domain"
short_code: "MIMIR-T-0515"
created_at: 2026-02-02T01:25:07.986206+00:00
updated_at: 2026-02-02T01:25:07.986206+00:00
parent: 
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/backlog"
  - "#tech-debt"


exit_criteria_met: false
strategy_id: NULL
initiative_id: NULL
---

# Split large command files by sub-domain

## Objective

Split oversized Tauri command files into smaller, sub-domain-focused modules for better navigability and testability.

## Backlog Item Details

### Type
- [x] Tech Debt - Code improvement or refactoring

### Priority
- [x] P3 - Low (when time permits)

### Technical Debt Impact
- **Current Problems**: `catalog.rs` has 50+ functions, `character.rs` has 40+, `map.rs` has 20+. These are hard to navigate, and finding a specific command requires scrolling through hundreds of lines.
- **Benefits of Fixing**: Easier navigation, better code ownership, more focused test files, clearer git blame history.
- **Risk Assessment**: Very low — purely structural refactoring. No logic changes. Tauri command registration in `main.rs` just needs updated import paths.

## Acceptance Criteria

- [ ] `catalog.rs` split into `catalog_monsters.rs`, `catalog_spells.rs`, `catalog_items.rs`, `catalog_other.rs` (or similar grouping)
- [ ] `character.rs` split into `character.rs` (core CRUD), `character_inventory.rs`, `character_leveling.rs`
- [ ] `map.rs` split into `map.rs` (core CRUD), `map_tokens.rs`, `map_features.rs` (POIs, traps, light sources, fog)
- [ ] `main.rs` invoke_handler updated with new import paths
- [ ] All commands still registered and functional
- [ ] All tests pass

## Implementation Notes

### Technical Approach
Create sub-modules under `commands/` using `mod.rs` re-exports so the external API doesn't change. Each new file contains a subset of the original functions. The `invoke_handler!` macro in `main.rs` references the same function names — only import paths change.

## Status Updates

*To be added during implementation*