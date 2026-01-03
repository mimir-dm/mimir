---
id: composable-character-sheet-printing
level: task
title: "Composable Character Sheet Printing"
short_code: "MIMIR-T-0270"
created_at: 2026-01-02T14:51:48.615420+00:00
updated_at: 2026-01-02T19:55:03.851045+00:00
parent: 
blocked_by: []
archived: false

tags:
  - "#task"
  - "#feature"
  - "#phase/active"


exit_criteria_met: false
strategy_id: NULL
initiative_id: NULL
---

# Composable Character Sheet Printing

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[Parent Initiative]]

## Objective

Create a character print system with selectable sections that can be composed into a single PDF, allowing users to customize which content they want to include.

## Wireframes

| Section | File |
|---------|------|
| Compact Sheet (2-page) | `docs/uat/print-character-sheet.md` |
| Long Form | `docs/uat/print-character-longform.md` |
| Spell Cards | `docs/uat/print-spell-cards.md` |
| Equipment Detail | `docs/uat/print-equipment-detail.md` |
| Print Dialog UI | `docs/uat/print-character-dialog.md` |

## Backlog Item Details

### Type
- [x] Feature - New functionality or enhancement  

### Priority
- [ ] P2 - Medium (nice to have)

### Business Justification
- **User Value**: Different gameplay scenarios need different character sheet outputs - quick reference at the table vs full background for roleplay vs spell cards for casters
- **Effort Estimate**: M

## Sections to Support

| Section | Description | Pages |
|---------|-------------|-------|
| **Compact Sheet** | 2-page WotC-style layout (see design below) | 2 |
| **Long Form Character** | Background, personality, appearance, roleplaying notes | 1-2 |
| **Spell Cards** | Individual spell cards for all known/prepared spells | Variable |
| **Equipment Detail** | Full inventory with categories, weights, descriptions | 1-2 |

## Compact Sheet Design (2-page WotC layout)

See `docs/uat/print-character-sheet.md` for ASCII mockup.

### Page 1: Combat & Stats

**Header Row**
- Character Name, Class & Level, Background
- Player Name, Race, Alignment, XP Points

**Left Column (Ability Scores)**
- 6 ability score boxes (STR, DEX, CON, INT, WIS, CHA)
- Each shows score + modifier

**Middle Column**
- Inspiration checkbox
- Proficiency Bonus
- Saving Throws (6 with proficiency indicators)
- Skills (18 skills with proficiency indicators + modifiers)
- Passive Perception
- Proficiencies & Languages

**Right Column**
- AC | Initiative | Speed boxes
- Current Hit Points (large box)
- Temp HP | Max HP
- Hit Dice (Total + Remaining)
- Death Saves (Success/Failure circles)
- Attacks & Spellcasting table (Name | ATK Bonus | Damage/Type)
- Spell Slots tracking (if character has spellcasting):
  ```
  ┌─────────────────────────────────────────────┐
  │  SPELL SLOTS                                │
  │  1st ○○○○  2nd ○○○  3rd ○○○  4th ○○        │
  │  5th ○○    6th ○    7th ○    8th ○   9th ○ │
  ├─────────────────────────────────────────────┤
  │  Spellcasting Ability: ___  Save DC: ___   │
  │  Spell Attack Bonus: ___                    │
  └─────────────────────────────────────────────┘
  ```
  - Circles pre-filled based on character's max slots per level
  - Non-casters: section omitted or shows "—" for each level
- Equipment (Currency: CP/SP/EP/GP/PP + item list)

**Footer - "Your Turn" Quick Reference (5-column layout)**
| Movement | Action | Bonus Action | Free Interact | Reaction |
|----------|--------|--------------|---------------|----------|
| Up to speed, can split around actions | Attack, Cast Spell, Dash, Disengage, Dodge, Help, Hide, Ready, Search, Use | Class/feature specific | One object (draw weapon, open door) | Opportunity Attack, Ready trigger (1/round) |

### Page 2: Character Details

**Header**
- Character Name
- Age | Height | Weight
- Eyes | Skin | Hair

**Left Side**
- Character Portrait area (placeholder/sketch box)

**Right Side**
- Personality Traits
- Ideals
- Bonds
- Flaws

**Full Width Sections**
- Character Backstory (large text area)
- Allies & Organizations (with symbol/emblem area) | Features & Traits (side by side)
- Additional Treasure

## Long Form Character Design

See `docs/uat/print-character-longform.md` for ASCII wireframe.

Single page (expands to 2 if content is lengthy). Sections:

**Appearance**
- Height, Weight, Age, Eyes, Hair, Skin
- Physical Description (paragraph)
- Distinctive Features (unique visual traits)

**Personality** (2x2 grid)
- Personality Traits | Ideals
- Bonds | Flaws

**Background**
- Background Type + Feature name
- Backstory narrative (auto-expanding)

**Roleplaying Notes**
- Voice & Mannerisms
- Key Relationships (party, NPCs, orgs)
- Character Goals (short/long term)
- Play Reminders (bullet points for at-table reference)

## Spell Cards Design

See `docs/uat/print-spell-cards.md` for wireframe.

- 3x3 grid per page (9 cards), trading card size (2.5" x 3.5")
- Each card: Name, Level (diamonds), School, Casting Time, Range, Duration, Components
- Ritual/Concentration indicators
- Description text with "At Higher Levels" section
- Sorted by level, then alphabetically

## Equipment Detail Design

See `docs/uat/print-equipment-detail.md` for wireframe.

Simple flat list - no categorization (metadata may not support divisions):
- Currency row at top (CP/SP/EP/GP/PP)
- Item | Weight | Properties row for each item
- Special items: full-width description block with fluff text and special rules (vertical card-like)
- Mundane items: compact single row
- Total weight at bottom
- Page expands based on inventory size

## Print Dialog UI

See `docs/uat/print-character-dialog.md` for wireframe.

- Checkbox list for section selection
- Descriptions under each option
- Default: Compact Sheet + Spell Cards
- Spell Cards always visible - silently no-op if no spells
- Export button disabled if nothing selected
- PDF sections appear in fixed order regardless of selection order

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] CharacterPrintDialog.vue modal with checkboxes for each section
- [ ] CompactCharacterSheet section (2-page WotC-style layout)
- [ ] CharacterLongForm section (background, personality, RP notes)
- [ ] EquipmentDetailSection (full inventory with fluff text)
- [ ] SpellCardsSection already exists - make composable
- [ ] Backend CharacterExportOptions with flags for each section
- [ ] Default selection: Compact Sheet + Spell Cards
- [ ] Spell Cards silently no-op if spell list is empty

## Implementation Notes

### Files to Create
- `crates/mimir-dm-print/src/sections/compact_sheet.rs` - WotC-style 2-page layout
- `crates/mimir-dm-print/src/sections/character_longform.rs` - Narrative content
- `crates/mimir-dm-print/src/sections/equipment_detail.rs` - Detailed inventory
- `crates/mimir-dm/frontend/src/components/print/CharacterPrintDialog.vue` - Modal UI

### Files to Modify
- `crates/mimir-dm-print/src/sections/mod.rs` - Add exports
- `crates/mimir-dm-print/src/character.rs` - Add CharacterExportOptions
- `crates/mimir-dm-print/src/lib.rs` - Export new types
- `crates/mimir-dm/src/commands/print/mod.rs` - Update command
- `crates/mimir-dm/frontend/src/services/PrintService.ts` - Update interface
- `crates/mimir-dm/frontend/src/features/characters/views/CharacterSheetView.vue` - Use dialog

### Technical Approach
Use DocumentBuilder pattern (established in campaign.rs) to compose selected sections into single PDF. Each section implements Renderable trait.

## Status Updates **[REQUIRED]**

### 2026-01-02: Implementation Complete
- Created CharacterPrintDialog.vue with checkbox UI for 4 sections
- Created compact_sheet.rs (2-page WotC-style layout with combat stats, spell slots, equipment)
- Created character_longform.rs (appearance, personality 2x2 grid, background, RP notes)
- Created equipment_detail.rs (currency + inventory with descriptions)
- Updated character.rs with CharacterExportOptions and export_character_pdf()
- Added export_character Tauri command
- Wired CharacterSheetView.vue to use new dialog
- Fixed CampaignExportOptions type mismatch discovered during type checking
- All cargo check and npm type-check pass

### 2026-01-02: Bug Fix - Typst underscore escaping
- **Issue**: Typst compilation failing with "unclosed delimiter" errors
- **Root cause**: Underscore `_` in Typst is interpreted as subscript marker (like LaTeX)
- **Fix**: Escape underscores in fill-in blanks as `\_` in template strings
- **Also**: Added `_` to escape_typst() functions for user content safety
- Added 4 integration tests that compile full PDFs with all section combinations
- All 120 tests now pass