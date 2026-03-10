---
id: add-homebrew-spell-support-for
level: task
title: "Add homebrew spell support for characters"
short_code: "MIMIR-T-0542"
created_at: 2026-03-10T01:15:21.022765+00:00
updated_at: 2026-03-10T01:15:21.022765+00:00
parent: 
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/backlog"
  - "#feature"


exit_criteria_met: false
initiative_id: NULL
---

# Add homebrew spell support for characters

## Objective

Allow homebrew spells to be associated with characters — adding them to the character's known/prepared spell list, displaying them on the character sheet, including them in spell card PDF exports, and making them selectable during level-up.

## Backlog Item Details

### Type
- [x] Feature - New functionality or enhancement

### Priority
- [x] P2 - Medium (nice to have)

### Business Justification
- **User Value**: Homebrew spells are a core part of custom campaign content. Without character association, they exist in a vacuum — DMs can create them but players can't actually use them on their sheets.
- **Effort Estimate**: M

## Current State

The `character_spells` table exists with `spell_source` as a text field, so storing `"HB"` is structurally possible. However, nothing in the codebase currently supports it:

- **No direct add/remove spell command**: Spells can only be added via the level-up flow
- **Level-up spell selection UI**: Only pulls from `catalog_dal::list_spells_by_class` — homebrew spells never appear
- **Spell card PDF export**: Uses `catalog_dal::list_spells_by_class` exclusively — homebrew spells are invisible
- **MCP tools**: No tool for adding/removing spells directly on a character

### Key files
- `character_spells` table: `migrations/014_character_details/up.sql`
- Spell DAL: `crates/mimir-core/src/dal/campaign/character_spell.rs` (15 CRUD functions)
- Character service: `crates/mimir-core/src/services/character.rs` (level-up spell handling)
- Spell management composable: `frontend/src/features/characters/composables/useSpellManagement.ts`
- Level-up spell step: `frontend/src/features/characters/components/levelup/steps/SpellsStep.vue`
- Spell card PDF: `crates/mimir/src/commands/print/character.rs` (lines ~280-380)
- Homebrew spell DAL: `crates/mimir-core/src/dal/campaign/campaign_homebrew_spell.rs`
- Homebrew spell model: `crates/mimir-core/src/models/campaign/campaign_homebrew_spell.rs`

## Acceptance Criteria

- [ ] Tauri command to add a spell (catalog or homebrew) directly to a character's known spells
- [ ] Tauri command to remove a spell from a character's known spells
- [ ] MCP tool to add/remove spells on a character
- [ ] Level-up spell selection includes campaign homebrew spells alongside catalog spells
- [ ] Character sheet spell list displays homebrew spells correctly (name, level, school, description)
- [ ] Spell card PDF export renders homebrew spells when `spell_source == "HB"`
- [ ] Homebrew spells can be toggled prepared/unprepared like catalog spells

## Implementation Notes

### 1. Add/Remove Spell Commands (Tauri + MCP)

New Tauri commands: `add_character_spell`, `remove_character_spell`
- Accept: `character_id`, `spell_name`, `spell_source`, `source_class`, `prepared` (optional)
- For homebrew: `spell_source = "HB"`, validate spell exists in `campaign_homebrew_spells`
- For catalog: validate spell exists in catalog
- Reuse existing DAL functions: `insert_character_spell`, `delete_character_spell`

### 2. Level-Up Spell Selection

In `SpellsStep.vue` / `useSpellManagement.ts`:
- After loading catalog spells via `list_spells_by_class`, also fetch homebrew spells for the campaign
- Merge homebrew spells into the spell list, tagged with source "HB"
- Filter homebrew spells by level (same max-level logic as catalog)
- Display homebrew spells with a visual indicator (e.g., "HB" badge)

### 3. Spell Card PDF Export

In `character.rs` equipment card section (~line 331):
- After iterating catalog spells, also query `campaign_homebrew_spells` for the character's campaign
- For each homebrew spell the character knows (from `character_spells` where `spell_source == "HB"`), look up the full data from `campaign_homebrew_spells`
- Parse and include in `spell_data` for `SpellCardsSection`

### 4. Character Sheet Display

The character sheet spell list should already work if spells are in `character_spells` — it reads from the table regardless of source. But the detail view (clicking a spell) currently looks up catalog data. Need to add homebrew fallback when `spell_source == "HB"`.

## Status Updates

*To be added during implementation*