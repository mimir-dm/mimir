---
id: implement-pdf-export-functionality
level: task
title: "Implement PDF Export Functionality"
short_code: "MIMIR-T-0440"
created_at: 2026-01-26T21:11:57.736051+00:00
updated_at: 2026-01-26T21:15:15.851826+00:00
parent: MIMIR-I-0041
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/active"


exit_criteria_met: false
strategy_id: NULL
initiative_id: MIMIR-I-0041
---

# Implement PDF Export Functionality

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[MIMIR-I-0041]]

## Objective

Implement the actual PDF generation functionality in mimir-print crate. Currently has stub implementations that return "not yet implemented" errors.

## Backlog Item Details **[CONDITIONAL: Backlog Item]**

{Delete this section when task is assigned to an initiative}

### Type
- [ ] Bug - Production issue that needs fixing
- [ ] Feature - New functionality or enhancement  
- [ ] Tech Debt - Code improvement or refactoring
- [ ] Chore - Maintenance or setup work

### Priority
- [ ] P0 - Critical (blocks users/revenue)
- [ ] P1 - High (important for user experience)
- [ ] P2 - Medium (nice to have)
- [ ] P3 - Low (when time permits)

### Impact Assessment **[CONDITIONAL: Bug]**
- **Affected Users**: {Number/percentage of users affected}
- **Reproduction Steps**: 
  1. {Step 1}
  2. {Step 2}
  3. {Step 3}
- **Expected vs Actual**: {What should happen vs what happens}

### Business Justification **[CONDITIONAL: Feature]**
- **User Value**: {Why users need this}
- **Business Value**: {Impact on metrics/revenue}
- **Effort Estimate**: {Rough size - S/M/L/XL}

### Technical Debt Impact **[CONDITIONAL: Tech Debt]**
- **Current Problems**: {What's difficult/slow/buggy now}
- **Benefits of Fixing**: {What improves after refactoring}
- **Risk Assessment**: {Risks of not addressing this}

## Acceptance Criteria

## Acceptance Criteria **[REQUIRED]**

- [ ] {Specific, testable requirement 1}
- [ ] {Specific, testable requirement 2}
- [ ] {Specific, testable requirement 3}

## Test Cases **[CONDITIONAL: Testing Task]**

{Delete unless this is a testing task}

### Test Case 1: {Test Case Name}
- **Test ID**: TC-001
- **Preconditions**: {What must be true before testing}
- **Steps**: 
  1. {Step 1}
  2. {Step 2}
  3. {Step 3}
- **Expected Results**: {What should happen}
- **Actual Results**: {To be filled during execution}
- **Status**: {Pass/Fail/Blocked}

### Test Case 2: {Test Case Name}
- **Test ID**: TC-002
- **Preconditions**: {What must be true before testing}
- **Steps**: 
  1. {Step 1}
  2. {Step 2}
- **Expected Results**: {What should happen}
- **Actual Results**: {To be filled during execution}
- **Status**: {Pass/Fail/Blocked}

## Documentation Sections **[CONDITIONAL: Documentation Task]**

{Delete unless this is a documentation task}

### User Guide Content
- **Feature Description**: {What this feature does and why it's useful}
- **Prerequisites**: {What users need before using this feature}
- **Step-by-Step Instructions**:
  1. {Step 1 with screenshots/examples}
  2. {Step 2 with screenshots/examples}
  3. {Step 3 with screenshots/examples}

### Troubleshooting Guide
- **Common Issue 1**: {Problem description and solution}
- **Common Issue 2**: {Problem description and solution}
- **Error Messages**: {List of error messages and what they mean}

### API Documentation **[CONDITIONAL: API Documentation]**
- **Endpoint**: {API endpoint description}
- **Parameters**: {Required and optional parameters}
- **Example Request**: {Code example}
- **Example Response**: {Expected response format}

## Implementation Notes

### Current State
- `mimir-print` crate exists with stub commands
- All print commands registered in main.rs
- Frontend dialogs exist and call the commands
- Typst templates preserved in `mimir-print/templates/_shared/`
- Old implementation preserved in `mimir-dm-bu/mimir-dm-print/` for reference

### Implementation Tasks

1. **Core PDF Infrastructure**
   - Port `service.rs` (PrintService with Typst integration)
   - Port `world.rs` (MimirTypstWorld for Typst compilation)
   - Port `builder.rs` (DocumentBuilder pattern)
   - Set up font loading

2. **Character Export**
   - Port compact sheet section
   - Port long form section  
   - Port battle card section
   - Port spell cards section
   - Port equipment cards/detail sections
   - Adapt to new Character model (simpler, flat structure)

3. **Module Export**
   - Port markdown section
   - Port monster appendix/cards
   - Port trap cards
   - Port NPC cards
   - Port map preview/tiled sections

4. **Campaign Export**
   - Combine module exports
   - Campaign-level documents
   - NPC index

5. **Map Export**
   - Port map_renderer.rs (LOS walls, tokens)
   - Port tiled map slicing
   - Port token cutout sheets

### Key Differences from v1
- New `mimir_core` has simplified Character model (no versioning, flat structure)
- Database-first approach (less filesystem operations)
- Need to adapt data fetching to use new DAL

### Dependencies
- mimir-core types and DAL
- Typst 0.12 for PDF compilation

## Status Updates **[REQUIRED]**

### Session 2 - Core Infrastructure Complete (2026-01-26)
- Ported world.rs (MimirTypstWorld) - custom Typst World for file/font resolution
- Ported service.rs (PrintService) - template-based PDF generation
- Ported builder.rs (DocumentBuilder) - composable multi-section document assembly
- Updated Cargo.toml with Typst dependencies (typst v0.12, typst-pdf, ecow, fontdb, imageproc, rusttype)
- Updated lib.rs to export new modules
- Both mimir-print and main mimir crate compile successfully
- Next: Port markdown conversion for document sections

### Session 2 - Document Export Working (2026-01-26)
- Ported markdown.rs (markdown to Typst conversion with frontmatter)
- Created sections/markdown.rs (MarkdownSection renderable)
- Created new print commands in mimir crate (can access AppState for DB)
- Implemented `export_campaign_document` - exports single document to PDF
- Implemented `export_campaign_documents` - exports all campaign docs to PDF
- Implemented `export_module_documents` - exports all module docs to PDF
- All document export commands now functional
- Remaining: character export, map export, monster cards, trap cards

### Session 2 - Character Export Complete (2026-01-26)
- Created CharacterSection in `sections/character.rs`
- CharacterData struct maps to new normalized Character model
- ClassInfo struct handles CharacterClass table (multiclass support)
- InventoryItem struct handles CharacterInventory table
- Generates Typst with: header, ability score boxes, currency, roleplay elements, NPC details, inventory
- Implemented `export_character` command using CharacterSection
- Fetches character, classes, and inventory from database
- All 37 tests pass for mimir-print crate
- Remaining: map export, monster cards, trap cards

### Session 3 - Monster/Trap Cards and Map Export Complete (2026-01-26)
- Ported `sections/monster_cards.rs` - half-page monster stat cards (3.875" x 5.125")
  - HP tracker with tiered rendering (individual boxes for <20HP, 5-HP boxes for 21-100, 10-HP boxes for 101+)
  - Foldable card support for complex monsters with overflow content
  - 5etools tag stripping for abilities
- Ported `sections/trap_cards.rs` - half-page trap/hazard reference cards
  - Threat level badges (simple, moderate, dangerous, deadly)
  - DC block rendering (detect, disable, save DCs)
  - Foldable card support for complex traps
- Ported `map_renderer.rs` - map image rendering with:
  - Grid overlay (square grids)
  - LOS wall rendering (red semi-transparent lines)
  - Token rendering with type-based colors
  - UVTT file handling (embedded base64 image extraction)
- Ported `sections/map.rs`:
  - MapPreview - single-page map preview fit to page
  - TiledMapSection - multi-page tiles at true scale (1 grid = 1 inch)
  - Assembly guide generation for tiled maps
- Implemented Tauri commands in `print.rs`:
  - `print_map` - full map export with preview/tiled modes and LOS walls
  - `export_module_monsters` - exports all monsters in a module as cards
  - `export_monster_card` - exports single monster card by name/source
  - `export_trap_card` - exports single trap card by name/source
  - `export_trap_cards` - exports multiple traps as cards
- All 63 tests pass for mimir-print crate
- All commands registered in main.rs

### Session 4 - Embedded Templates and Campaign Export Fix (2026-01-26)
- **Fixed template not found error**: Templates were looking for filesystem `_shared/*.typ` files
  - Created `embedded_templates.rs` - embeds STYLES_TYP, COMPONENTS_TYP, ICONS_TYP directly in binary
  - Updated `world.rs` to check embedded templates before filesystem lookups
  - Templates now resolved via `get_embedded_template()` function
  - No external template files needed for PDF generation

- **Enhanced campaign export with tiled maps**:
  - Added campaign tiled maps (opts.include_campaign_tiled_maps) - regional/world maps at 1"=5ft scale
  - Added module tiled maps (opts.include_module_tiled_maps) - dungeon maps at 1"=5ft scale
  - Both use TiledMapSection for multi-page true-scale printing
  - Token cutouts placeholder added (requires more complex monster/NPC lookups)

- **Campaign export options now working**:
  - include_campaign_docs ✓
  - include_module_content (docs + monsters) ✓
  - include_npcs ✓
  - include_campaign_map_previews ✓
  - include_module_map_previews ✓
  - include_campaign_tiled_maps ✓
  - include_module_tiled_maps ✓
  - include_token_cutouts - TODO (needs monster/NPC token image lookups)

- All 67 tests pass
- Build succeeds with only warnings

### Session 5 - Character Sheet Migration Phase 1 Complete (2026-01-27)
**Backend Changes:**
- Added `proficiencies: Vec<CharacterProficiency>` to `CharacterResponse` struct
- Updated `from_character()` to accept proficiencies parameter
- Updated all character commands (list_characters, list_pcs, list_npcs, get_character, update_character) to fetch and include proficiencies
- Proficiencies already exist in database and DAL (character_proficiencies table, list_character_proficiencies function)

**Frontend Types:**
- Added `CharacterProficiency` interface to `types/character.ts`
- Added `proficiencies` array to `Character` interface  
- Added proficiency helper functions:
  - `hasExpertise()`, `getSkillProficiencies()`, `getSaveProficiencies()`
  - `getToolProficiencies()`, `getWeaponProficiencies()`, `getArmorProficiencies()`, `getLanguages()`
  - `isProficientInSkill()`, `hasSkillExpertise()`, `isProficientInSave()`
  - `proficiencyBonus()` - calculates from total level

**Bug Fix:**
- Fixed MapPoi type mismatch (missing created_at/updated_at fields in PoiEditModal and MapTokenSetupModal)

### Session 6 - Character Sheet Migration Phase 2 Complete (2026-01-27)
**Created `utils/characterUtils.ts`:**
- D&D 5e constants: ALL_SKILLS (18 skills with abilities), ABILITIES array
- Core calculations: getModifier(), formatModifier(), getProficiencyBonus(), getTotalLevel()
- Proficiency helpers: getProficienciesByType(), isProficientInSkill(), hasSkillExpertise(), isProficientInSave()
- Skill/save calculations: getSkillBonus(), getSaveBonus(), getPassivePerception()
- Combat: getArmorAC() (handles light/medium/heavy + magic bonuses), getWeaponDamage(), isFinesse(), isRanged()
- Spellcasting: SPELLCASTING_ABILITY map, isSpellcaster(), getSpellcastingAbility(), getSpellSaveDC(), getSpellAttackBonus()
- Hit dice: CLASS_HIT_DICE map, getHitDiceString()

**Rewrote `CharacterSheetView.vue`:**
- Tab navigation: Character, Equipment, Spells, Details
- Three-column layout for Character tab:
  - Left: Ability scores with modifiers, Combat stats (AC, Initiative, Speed, Passive Perception, Hit Dice, Proficiency Bonus), Saving throws with proficiency indicators, Attacks from equipped weapons
  - Middle: All 18 skills with computed bonuses, proficiency/expertise indicators
  - Right: Proficiencies (armor, weapons, tools, languages), Spellcasting summary for casters, Personality traits
- Equipment tab: Currency display, Equipped items, Full inventory list
- Details tab: Background, NPC info, Classes with levels
- Note: Speed defaults to 30ft (race catalog lookup not yet implemented)
- Note: Spells tab shows placeholder (full spell management deferred)

**Character Sheet Migration Status:**
- Phase 1 Backend: COMPLETE (proficiencies in CharacterResponse)
- Phase 2 Frontend: COMPLETE (character sheet view with all calculations)
- Both `cargo build` and `npm run type-check` pass

### Session 7 - Catalog Lookups and Spell Slots (2026-01-27)
**Added catalog data fetching:**
- `loadRaceData()` - Fetches race JSON from catalog via `get_race_by_name`
- `loadClassData()` - Fetches class JSON for each class via `get_class_by_name`
- Parallel loading with `Promise.all()` for better performance

**Speed now from race catalog:**
- Extracts speed from 5etools race JSON (handles both number and {walk: number} formats)
- Falls back to 30ft if no race data available

**Spell slots table implemented:**
- Full caster progression (bard, cleric, druid, sorcerer, wizard) - levels 1-20
- Half caster progression (paladin, ranger) - starts at level 2
- Warlock pact magic (all slots same level, increases with level)
- Visual slot boxes in Spells tab for tracking

**Updated Spells tab:**
- Shows spellcasting stats (DC, attack bonus, ability)
- Displays spell slot boxes per level with counts
- Note about tracking on paper

**Class features display added:**
- `loadClassFeatures()` - Fetches class features via `list_class_features`
- Filters features by character's class level (only shows gained features)
- Sorted by level then name
- Displayed in Character tab with scrollable list

**Character Sheet Migration - COMPLETE:**
- Phase 1: Backend proficiencies - DONE
- Phase 2: Character sheet view - DONE
- Phase 3: Catalog lookups (race speed, class features) - DONE
- Phase 4: Spell slots display - DONE

### Session 8 - Character Sheet Information Expansion (2026-01-27)
**Equipment Tab Expansion:**
- Expandable item cards with click-to-reveal details
- Fetches item data from catalog via `get_item_by_name`
- Shows: rarity, properties (finesse, heavy, attunement, etc.), description
- Separate sections for Equipped Items and full Inventory
- Item notes display

**Spells Tab Expansion:**
- Loads all available spells for character's spellcasting class(es)
- Filters spells by class from 5etools JSON data
- Groups spells by level (Cantrip through 9th)
- Expandable spell cards showing:
  - Casting time, range, components, duration
  - Full spell description
  - Ritual (R) and Concentration (C) tags
- Max spell level calculated based on class and level

**Details Tab Expansion:**
- Background details from catalog:
  - Skill and tool proficiencies
  - Language options
  - Starting equipment
  - Background feature with description
- Class details with descriptions from catalog
- Subclass details with descriptions
- Compact feature list per class showing all gained features

**Remaining (future enhancement):**
- Full spell list management (known/prepared spells tracking)
- Spell slot tracking (currently reference only)

### Session 9 - Character PDF and Print Logging (2026-01-27)
**Character PDF Enhancements:**
- Added `Proficiencies` and `ProficiencyEntry` structs to character section
- Enhanced `CharacterData` with proficiencies and speed fields
- Updated character PDF template with:
  - Combat stats (AC, Speed, Proficiency Bonus, Initiative)
  - Saving throws with proficiency calculation
  - Full proficiencies section (skills, saves, languages, armor, weapons, tools)
  - Equipment with currency display
  - NPC badge and info box for NPC characters

**Spell Cards Fix:**
- Changed from `list_character_spells` (empty table) to `list_spells_by_class` (catalog spells)
- Now shows ALL available spells for character's classes
- Added `max_spell_level_for_class()` helper for spell level filtering:
  - Full casters (Bard, Cleric, etc.): `(level + 1) / 2`, max 9
  - Half casters (Paladin, Ranger): Custom progression, max 5
  - Third casters (Artificer): Starts at level 1, max 5
  - Warlock: Custom pact magic progression

**Comprehensive Logging Added to ALL Print Functions:**
- `export_character` - Full logging with options, character data, and all sections
- `export_campaign_documents` - Options logging and all 8 section types
- `export_module_documents` - Options logging and all section types
- `print_map` - Options, map data (dimensions, grid, LOS walls), sections
- `export_module_monsters` - Options, module data, monster list, sections
- `export_monster_card` - Monster name/source, options, sections
- `export_trap_card` - Trap name/source, options, sections
- `export_trap_cards` - Trap list, options, sections
- `save_pdf` - File path and size information

**Logging Pattern:**
- `=== function_name called ===` header
- `=== Options ===` for parameters
- `=== Data ===` for loaded data
- `[SECTION]` prefix for section additions
- `=== Building PDF ===` before final build

All print functions now have consistent debugging output for easier troubleshooting.