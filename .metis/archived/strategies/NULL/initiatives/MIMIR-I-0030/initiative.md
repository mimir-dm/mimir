---
id: campaign-authoring-framework
level: initiative
title: "Campaign Authoring Framework Refinement"
short_code: "MIMIR-I-0030"
created_at: 2026-01-03T03:55:52.207326+00:00
updated_at: 2026-01-03T13:47:11.855743+00:00
parent: MIMIR-V-0001
blocked_by: []
archived: true

tags:
  - "#initiative"
  - "#phase/completed"


exit_criteria_met: false
estimated_complexity: M
strategy_id: NULL
initiative_id: campaign-authoring-framework
---

# Campaign Authoring Framework Refinement Initiative

Refine the campaign generation documentation and module templates to produce complete, runnable adventures rather than just planning documents.

## Context

The "session planning" layer has been dropped - modules are now the atomic unit of play. DMs no longer need to plan individual sessions within modules. However, current templates are planning-focused skeletons that don't produce runnable content.

**Gap identified**: The current `module_overview.md` template produces planning docs, while real-world modules (e.g., Frost Architect Module 04) include full Adventure Scenes with read-aloud text, encounter stat blocks, puzzle solutions, and DM notes.

## Goals & Non-Goals

**Goals:**
- Remove session planning layer from documentation and templates
- Upgrade module templates to produce complete, runnable adventures
- Support both AI-assisted generation and human authoring
- Include read-aloud text, encounters, puzzles, rewards, and DM notes

**Non-Goals:**
- Changing the overall campaign framework philosophy
- Adding new module types (focus on improving existing templates)

## Alternatives Considered

1. **Keep templates planning-only, add separate "content" templates** - Rejected because it fragments the authoring experience
2. **Make session-level templates optional within modules** - Rejected because session boundaries add unnecessary complexity

## Implementation Plan

### Phase 1: Core Module Template Rewrite
- Rewrite `module_overview.md` with YAML front matter schema
- Define structured arrays for monsters, npcs, items catalog references
- Add Adventure Content sections with encounter tag references
- Use Frost Architect Module 04 as reference implementation

### Phase 2: Module Type Templates
- Update Mystery, Dungeon, Heist, Political, Horror templates
- Same YAML front matter schema as core template
- Add type-specific Adventure Content guidance
- Ensure all include read-aloud sections, encounter references, DM notes

### Phase 3: Documentation Cleanup
- Archive or merge session-management docs into module-running
- Update README files and template directories
- Remove Session Board from board-workflow-guide

### Phase 4: Backend Schema (Required)
- Define `ModuleFrontmatter` Rust struct for parsing YAML front matter
- Create `ModuleNPC` and `ModuleItem` models (following `ModuleMonster` pattern)
- Implement sync services to generate `npcs.md` and `items.md` files
- Integrate with document save workflow

## Files to Modify

**Templates:**
- `docs/src/campaign-framework/06-templates/templates/module_overview.md`
- `docs/src/campaign-framework/06-templates/templates/module_mystery.md`
- `docs/src/campaign-framework/06-templates/templates/module_dungeon.md`
- `docs/src/campaign-framework/06-templates/templates/module_heist.md`
- `docs/src/campaign-framework/06-templates/templates/module_political.md`
- `docs/src/campaign-framework/06-templates/templates/module_horror.md`
- `docs/src/campaign-framework/06-templates/templates/README.md`

**Documentation:**
- `docs/src/campaign-framework/04-session-management/` (archive/merge)
- `docs/src/campaign-framework/03-module-creation/module-creation-process.md`
- `docs/src/campaign-framework/board-workflow-guide.md`

**Backend (Phase 4):**
- `crates/mimir-dm-core/src/models/campaign/module_frontmatter.rs` (create)
- `crates/mimir-dm-core/src/models/campaign/module_npcs.rs` (create)
- `crates/mimir-dm-core/src/models/campaign/module_items.rs` (create)
- `crates/mimir-dm-core/src/services/module_npc_service.rs` (create)
- `crates/mimir-dm-core/src/services/module_item_service.rs` (create)
- `crates/mimir-dm-core/src/services/module_frontmatter_service.rs` (create)
- `crates/mimir-dm-core/migrations/0XX_create_module_npcs/` (create)
- `crates/mimir-dm-core/migrations/0XX_create_module_items/` (create)

## Key Principles

1. **Modules are atomic** - No session boundaries, DM runs until natural stopping point
2. **Runnable out of the box** - Templates produce content you can run, not just plan
3. **Read-aloud ready** - Include boxed text for key moments
4. **Catalog-linked** - Reference monsters/NPCs/items via YAML front matter, stats synced separately
5. **DM-friendly** - Include pacing, tone, and scaling guidance
6. **AI-compatible** - Structure supports AI generation; catalog references validated by system