---
id: implement-modulefrontmatter-schema
level: task
title: "Implement ModuleFrontmatter schema and sync service"
short_code: "MIMIR-T-0282"
created_at: 2026-01-03T13:18:17.385210+00:00
updated_at: 2026-01-03T13:46:54.278536+00:00
parent: MIMIR-I-0030
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: MIMIR-I-0030
---

# Implement ModuleFrontmatter schema and sync service

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[MIMIR-I-0030]]

## Objective

Implement backend infrastructure to parse YAML front matter from module documents and sync catalog references (monsters, NPCs, items) to database tables. This enables the module template's machine-readable front matter to drive the catalog linking system.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] Define `ModuleFrontmatter` Rust struct with serde for parsing YAML front matter
- [ ] Parse monsters array from front matter and sync to `module_monsters` table
- [ ] Define `ModuleNPC` model and `module_npcs` table (following monster pattern)
- [ ] Define `ModuleItem` model and `module_items` table (following monster pattern)
- [ ] Create `ModuleFrontmatterService` to orchestrate parsing and sync
- [ ] Extend sync to generate `npcs.md` and `items.md` files (like `monsters.md`)
- [ ] Integration with document save workflow to trigger sync

## Implementation Notes

### Technical Approach

1. **ModuleFrontmatter struct** (`crates/mimir-dm-core/src/models/campaign/module_frontmatter.rs`):
   - Parse YAML using existing `gray_matter` crate
   - Serde derive for `MonsterReference`, `NpcReference`, `ItemReference`

2. **New models** (following `module_monsters.rs` pattern):
   - `ModuleNPC` struct with role, name, source, location, notes
   - `ModuleItem` struct with location, name, source, quantity, notes

3. **Migrations**:
   - `module_npcs` table
   - `module_items` table

4. **Services**:
   - `ModuleNpcService` (mirroring `ModuleMonsterService`)
   - `ModuleItemService` (mirroring `ModuleMonsterService`)
   - `ModuleFrontmatterService` to parse front matter and delegate to entity services

5. **Sync to files**:
   - `npcs.md` - rendered NPC cards
   - `items.md` - rendered item cards

### Dependencies

- MIMIR-T-0278: Core template defines the YAML schema format
- Existing `ModuleMonsterService` pattern to follow

### Files to Create/Modify

| File | Action |
|------|--------|
| `models/campaign/module_frontmatter.rs` | Create - front matter schema |
| `models/campaign/module_npcs.rs` | Create - NPC model |
| `models/campaign/module_items.rs` | Create - Item model |
| `services/module_npc_service.rs` | Create |
| `services/module_item_service.rs` | Create |
| `services/module_frontmatter_service.rs` | Create |
| `migrations/0XX_create_module_npcs/` | Create |
| `migrations/0XX_create_module_items/` | Create |

## Status Updates **[REQUIRED]**

*To be added during implementation*