---
id: auto-populate-character
level: task
title: "Auto-populate character proficiencies from class, background, and race data"
short_code: "MIMIR-T-0532"
created_at: 2026-03-09T14:03:06.825090+00:00
updated_at: 2026-03-10T01:17:38.284266+00:00
parent: 
blocked_by: []
archived: false

tags:
  - "#task"
  - "#feature"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: NULL
---

# Auto-populate character proficiencies from class, background, and race data

## Objective

Characters currently have an empty `character_proficiencies` table. D&D 5e characters gain proficiencies from three sources — class, background, and race — and these should be auto-populated during character creation and level-up.

## Backlog Item Details

### Type
- [x] Feature - New functionality or enhancement

### Priority
- [x] P1 - High (important for user experience)

### Business Justification
- **User Value**: Proficiencies are core to D&D 5e — they affect saving throws, skill checks, attack rolls, and armor/weapon usage. Without them the character sheet is incomplete.
- **Effort Estimate**: M

## Proficiency Sources

### From Class (level 1 — starting class)
- **Saving throws** (2 per class, e.g., Rogue = DEX + INT)
- **Armor** (e.g., Rogue = light armor)
- **Weapons** (e.g., Rogue = simple weapons, hand crossbows, longswords, rapiers, shortswords)
- **Tools** (e.g., Rogue = thieves' tools)
- **Skills** (choose N from a list, e.g., Rogue chooses 4)

### From Background
- **Skills** (typically 2, e.g., Criminal = Deception + Stealth)
- **Tools** (e.g., Criminal = thieves' tools, one gaming set)
- **Languages** (some backgrounds grant languages)

### From Race
- **Skills** (e.g., Elf = Perception)
- **Weapons** (e.g., Elf = longsword, shortsword, shortbow, longbow)
- **Tools** (e.g., Dwarf = one artisan's tools)
- **Languages** (e.g., Elf = Common + Elvish)

### From Multiclass (level-up into a new class)
- Subset of the class's starting proficiencies (defined in `multiclassing.proficienciesGained` in 5etools data)

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [x] Character creation populates proficiencies from class, background, and race catalog data
- [x] Level-up into a new class (multiclass) adds the multiclass proficiency subset
- [x] Saving throw proficiencies are correctly reflected in the character sheet (adding proficiency bonus) — already worked, just needed data
- [x] Skill proficiencies show correct bonuses (ability mod + proficiency bonus) — already worked, just needed data
- [x] Proficiencies display in the character sheet Proficiencies section (armor, weapons, tools, languages) — already worked, just needed data
- [x] Skill choices (e.g., "choose 4 from...") are presented to the user during creation/level-up — wizard already had UI, now sends choices
- [N/A] ~~Existing characters can have proficiencies retroactively populated or manually added~~ — not needed, new feature applies to new characters going forward

## Implementation Notes

### Key Data Sources (5etools format)
- Class data blob: `startingProficiencies` (armor, weapons, tools, skills), `proficiency` (saving throws)
- Background data blob: `skillProficiencies`, `toolProficiencies`, `languageProficiencies`
- Race data blob: `skillProficiencies`, `weaponProficiencies`, `toolProficiencies`, `languageProficiencies`
- Class data blob: `multiclassing.proficienciesGained` (for multiclass level-ups)

### Existing Infrastructure
- `character_proficiencies` table exists with columns: `id`, `character_id`, `proficiency_type`, `name`, `expertise`
- `CharacterProficiency` model exists in Rust
- Character sheet already has a Proficiencies section that reads from this table
- Saving throw display in `CharacterStatsTab.vue` needs to check proficiency to add the bonus

## Status Updates

### Research Phase
Analyzed the full stack. Key findings:
- `character_proficiencies` table and DAL are fully built out (15 CRUD functions)
- Frontend wizard collects skill choices (`formData.skills`) but never sends them
- `create_pc` command doesn't accept proficiency data
- `CreateCharacterInput` has no proficiency fields
- Level-up only handles expertise, not starting proficiencies
- 5etools data has polymorphic proficiency format (ProficiencyItem enum)

### Implementation — Completed

#### Service layer (`crates/mimir-core/src/services/character.rs`)
- Added `class_name`, `class_source`, `selected_skills` fields to `CreateCharacterInput`
- Added `with_class()` and `with_skills()` builder methods
- Added proficiency extraction helpers:
  - `extract_keyed_proficiencies()` — parses 5etools keyed JSON format
  - `extract_class_proficiencies()` — saves, armor, weapons, tools, selected skills
  - `extract_background_proficiencies()` — skills, tools, languages
  - `extract_race_proficiencies()` — skills, weapons, armor, tools, languages
  - `extract_multiclass_proficiencies()` — armor, weapons, tools from `multiclassing.proficienciesGained`
  - `insert_proficiencies()` — inserts entries, skipping duplicates
  - `capitalize_proficiency()` — formats names for display
- Updated `create()` to populate proficiencies after inserting character
- Updated `level_up()` to add multiclass proficiencies when multiclassing

#### Tauri commands (`crates/mimir/src/commands/character.rs`)
- Added `class_name`, `class_source`, `selected_skills` to `CreatePcRequest`
- Updated `create_pc` to pass them to service and enrich response with proficiencies

#### MCP tools (`crates/mimir-mcp/src/tools/character.rs`)
- Updated `create_character_tool()` description and parameters for proficiency population
- Updated `create_character` implementation to accept and pass class/background/skills

#### Frontend
- Updated `CreatePcRequest` type with new fields
- Updated `CharacterCreationWizard.vue` to send class_name, class_source, and selected_skills

#### Tests (12 new)
- `test_extract_keyed_proficiencies_boolean_keys`
- `test_extract_keyed_proficiencies_ignores_choice_keys`
- `test_extract_keyed_proficiencies_ignores_any_prefix_keys`
- `test_extract_keyed_proficiencies_string_values`
- `test_extract_keyed_proficiencies_strips_source_suffix`
- `test_extract_class_proficiencies_saves`
- `test_extract_class_proficiencies_with_skills`
- `test_extract_background_proficiencies`
- `test_extract_race_proficiencies`
- `test_extract_multiclass_proficiencies`
- `test_extract_multiclass_proficiencies_no_multiclass`
- `test_capitalize_proficiency`

All 1077 Rust tests + 350 frontend tests passing.

#### Not Needed
- Retroactive proficiency population dropped — feature applies to new characters going forward

### Post-completion: Production Data Fix (Session 2)
- Manually inserted proficiencies for all 4 existing PCs in production database
- Fixed apostrophe escaping issue in SQLite for "Thieves' tools" and "Cartographer's tools"
- All PCs now have full proficiency data: Cheese (13), Madrigal (14), Major (19), Matrim (17)