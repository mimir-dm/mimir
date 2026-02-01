---
id: homebrew-spell-creation-with
level: task
title: "Homebrew spell creation with catalog cloning"
short_code: "MIMIR-T-0507"
created_at: 2026-02-01T03:16:42.053007+00:00
updated_at: 2026-02-01T03:47:42.817016+00:00
parent: 
blocked_by: []
archived: true

tags:
  - "#task"
  - "#feature"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: NULL
---

# Homebrew spell creation with catalog cloning

## Objective

Add homebrew spell support to campaigns following the same pattern as homebrew items (MIMIR-T-0506) and homebrew monsters. Users clone from the spell catalog and edit the JSON. The HomebrewTab gets a third "Spells" sub-tab.

## Backlog Item Details

### Type
- [x] Feature - New functionality or enhancement

### Priority
- [ ] P2 - Medium (nice to have)

### Business Justification
- **User Value**: DMs frequently homebrew custom spells; this completes the homebrew trifecta (items, monsters, spells)
- **Effort Estimate**: M — follows established pattern from items/monsters

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] Migration creates `campaign_homebrew_spells` table (id, campaign_id, name, level, school, data, cloned_from_name/source, timestamps)
- [ ] Model, DAL, Tauri commands, and MCP tools for CRUD (following homebrew_monster pattern)
- [ ] Archive export/import includes homebrew spells
- [ ] Frontend service `HomebrewSpellService.ts` with data events
- [ ] HomebrewTab has "Spells" sub-tab with clone-from-catalog, JSON editor, and spell detail preview
- [ ] Selected card highlights use `color-mix()` for theme compatibility
- [ ] `cargo check` passes for mimir-core, mimir, mimir-mcp
- [ ] `cargo test -p mimir-mcp` passes (EXPECTED_TOOLS updated)
- [ ] `npx vue-tsc --noEmit` introduces no new errors

## Implementation Notes

### Technical Approach
Clone-from-catalog only (same as monsters). Search the spell catalog via `search_spells`, clone entry, edit JSON in textarea. Reuse spell rendering components if available, otherwise build a simple inline spell detail block.

### Dependencies
- Homebrew items pattern (MIMIR-T-0506) — established
- Homebrew monsters pattern — just implemented

## Status Updates

### Session 1
- All backend complete: migration 026, model, DAL, schema.rs, archive.rs, Tauri commands, MCP tools
- All registrations done: mod.rs files, main.rs invoke_handler, handler.rs get_tools/execute_tool/EXPECTED_TOOLS
- All 3 crates compile, 16 MCP tests pass
- Frontend complete: HomebrewSpellService.ts, dataEvents, HomebrewSpellsSubTab.vue, SpellStatBlock.vue
- HomebrewTab.vue updated with "Spells" sub-tab button
- SpellStatBlock uses `processFormattingTags` to render `{@...}` 5etools tags via v-html
- Uses `name_contains` (not camelCase) for SpellFilter search — verified SpellFilter has no rename_all
- Selected card highlights use `color-mix(in srgb, var(--color-primary-400) 12%, var(--color-surface))`
- Awaiting vue-tsc check and manual UI testing