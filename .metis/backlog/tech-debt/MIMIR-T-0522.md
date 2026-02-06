---
id: split-print-rs-into-sub-domain
level: task
title: "Split print.rs into sub-domain modules"
short_code: "MIMIR-T-0522"
created_at: 2026-02-06T13:33:36.510206+00:00
updated_at: 2026-02-06T13:33:36.510206+00:00
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

# Split print.rs into sub-domain modules

## Objective

Split `commands/print.rs` (2734 lines) into focused sub-modules by print domain, following the same pattern used for `catalog.rs` and `map.rs`.

## Backlog Item Details

### Type
- [x] Tech Debt - Code improvement or refactoring

### Priority
- [x] P3 - Low (when time permits)

### Technical Debt Impact
- **Current Problems**: `print.rs` has 2734 lines handling 14 Tauri commands across 5 different domains (documents, characters, maps, monsters, traps). Navigation is difficult, and related functionality is spread throughout the file.
- **Benefits of Fixing**: Easier navigation, better code ownership, more focused files, clearer git blame history. Follows the pattern established by catalog.rs and map.rs splits.
- **Risk Assessment**: Very low — purely structural refactoring. No logic changes. Tauri command registration in `main.rs` just needs updated import paths.

## Acceptance Criteria

- [ ] Create `commands/print/` folder with submodules
- [ ] Extract `mod.rs` - Re-exports, shared types (ApiResponse, PrintResult), helper functions
- [ ] Extract `document.rs` - Campaign/module document export commands
- [ ] Extract `character.rs` - Character sheet export, D&D computation helpers
- [ ] Extract `map.rs` - Map printing commands
- [ ] Extract `monster.rs` - Monster card export commands
- [ ] Extract `trap.rs` - Trap card export commands
- [ ] Delete old `print.rs` 
- [ ] `main.rs` invoke_handler unchanged (re-exports maintain backwards compatibility)
- [ ] `cargo check -p mimir` passes

## Implementation Notes

### Proposed Structure
```
commands/print/
├── mod.rs          # Re-exports + shared types (ApiResponse, PrintResult, export options)
├── helpers.rs      # D&D computation helpers (hit_die_for_class, spell_slots, etc.)
├── document.rs     # export_campaign_document, export_campaign_documents, export_module_documents
├── character.rs    # export_character, generate_character_sheet
├── map.rs          # print_map, save_pdf
├── monster.rs      # export_module_monsters, export_monster_card
└── trap.rs         # export_trap_card, export_trap_cards
```

### Commands by Domain
- **Documents**: `list_print_templates`, `export_campaign_document`, `export_campaign_documents`, `export_module_documents`
- **Characters**: `export_character`, `generate_character_sheet`
- **Maps**: `print_map`, `save_pdf`
- **Monsters**: `export_module_monsters`, `export_monster_card`
- **Traps**: `export_trap_card`, `export_trap_cards`

## Status Updates

*To be added during implementation*