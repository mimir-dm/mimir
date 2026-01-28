---
id: character-source-rules
level: task
title: "Character Source/Rules Configuration"
short_code: "MIMIR-T-0450"
created_at: 2026-01-28T02:06:46.455194+00:00
updated_at: 2026-01-28T03:50:44.323406+00:00
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

# Character Source/Rules Configuration

Allow players to configure which rulebooks/sources their individual character uses. This is separate from campaign-level sources and allows for character-specific restrictions.

## Objective

Enable character-level source book configuration so that a character's available options (spells, features, etc.) can be filtered to their chosen sources, independent of campaign-wide settings.

## Backlog Item Details

### Type
- [x] Feature - New functionality or enhancement  

### Priority
- [x] P2 - Medium (nice to have)

### Business Justification
- **User Value**: Characters may use different source books than the campaign default (e.g., one player uses PHB 2014 character while others use PHB 2024). Allows granular control per character.
- **Business Value**: Supports mixed-edition play which is common during transition periods
- **Effort Estimate**: S

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [x] Character settings include a "Sources" configuration
- [x] UI to select sources for a character (CharacterSourcesModal with checkbox list, quick actions)
- [x] Character source settings persisted to database (character_sources table)
- [ ] Character spell lists, level-up options respect character's sources (future enhancement)
- [x] Default: all sources available if not explicitly configured

## Implementation Notes

### Technical Approach
- Add `character_sources` table (character_id, source_code)
- Add UI in character settings/edit to manage sources
- Source selector shows only sources allowed by campaign
- Modify spell/feature lookups to filter by character sources

### Dependencies
- MIMIR-T-0449: Campaign Source/Rules Configuration (should be implemented first)

## Status Updates

### Session 2026-01-27

**Completed Implementation:**

1. **Database Schema** - Added `character_sources` table to existing migration `012_characters`:
   - `id TEXT PRIMARY KEY NOT NULL`
   - `character_id TEXT NOT NULL REFERENCES characters(id) ON DELETE CASCADE`
   - `source_code TEXT NOT NULL REFERENCES catalog_sources(code)`
   - `UNIQUE(character_id, source_code)` constraint
   - Index on `character_id`

2. **Model** (`mimir-core/src/models/campaign/character_source.rs`):
   - `CharacterSource` struct with Queryable, Serialize, Deserialize
   - `NewCharacterSource` for insertions

3. **DAL** (`mimir-core/src/dal/campaign/character_source.rs`):
   - `insert_character_source` / `get_character_source`
   - `list_character_sources` / `list_character_source_codes`
   - `delete_character_source` / `delete_character_source_by_code` / `delete_all_character_sources`
   - `character_has_source` / `count_character_sources`

4. **Tauri Commands** (`mimir/src/commands/character.rs`):
   - `list_character_sources` - Get source codes for a character
   - `add_character_source` - Add a single source
   - `remove_character_source` - Remove a single source
   - `set_character_sources` - Replace all sources (bulk set)

5. **Frontend UI**:
   - `CharacterSourcesModal.vue` - Modal with checkbox list, quick actions (Select All, Select None, Core Only)
   - Integrated into `CharacterSheetView.vue` with "Sources" button in header
   - Uses same pattern as `CampaignSourcesModal`

**Files Modified:**
- `crates/mimir-core/migrations/012_characters/up.sql` - Added character_sources table
- `crates/mimir-core/migrations/012_characters/down.sql` - Added drop statement
- `crates/mimir-core/src/schema.rs` - Added table and joinable! macros
- `crates/mimir-core/src/models/campaign/character_source.rs` (new)
- `crates/mimir-core/src/models/campaign/mod.rs` - Export
- `crates/mimir-core/src/dal/campaign/character_source.rs` (new)
- `crates/mimir-core/src/dal/campaign/mod.rs` - Export
- `crates/mimir/src/commands/character.rs` - Added source commands
- `crates/mimir/src/main.rs` - Registered commands
- `crates/mimir/frontend/src/components/characters/CharacterSourcesModal.vue` (new)
- `crates/mimir/frontend/src/components/characters/index.ts` - Export
- `crates/mimir/frontend/src/features/characters/views/CharacterSheetView.vue` - Integrated modal

**Remaining Work:**
- Testing with actual character and UI verification

### Session 2026-01-27 (continued) - Spell Filtering

**Completed:**

6. **Spell Filtering by Character Sources** - Available spells are now filtered based on the character's configured sources:
   - `CharacterSheetView.vue` - `loadClassSpells()` now fetches character sources and filters spell list
   - `SpellsStep.vue` (level-up) - `loadSpells()` now filters cantrips and leveled spells by character sources
   - Default behavior: if no sources configured, show all spells
   - Filter applies to both the "Available Spells" display and the level-up spell selection

**Files Modified:**
- `crates/mimir/frontend/src/features/characters/views/CharacterSheetView.vue` - Added source filtering to loadClassSpells
- `crates/mimir/frontend/src/features/characters/components/levelup/steps/SpellsStep.vue` - Added source filtering to loadSpells

7. **PDF Export Spell Filtering** - Added source filtering to the `export_character` command in `print.rs`:
   - Fetches character's allowed sources via `dal::list_character_source_codes`
   - Filters spell cards to only include spells from allowed sources
   - Default behavior: if no sources configured, show all spells
   - Logging added for debugging source filtering

**Files Modified:**
- `crates/mimir/src/commands/print.rs` - Added character source filtering to spell cards section (~line 1645)