---
id: migrate-character-level-up-workflow
level: initiative
title: "Migrate Character Level Up Workflow"
short_code: "MIMIR-I-0048"
created_at: 2026-01-27T21:00:12.452469+00:00
updated_at: 2026-01-27T21:19:06.800001+00:00
parent: MIMIR-V-0001
blocked_by: []
archived: false

tags:
  - "#initiative"
  - "#phase/active"


exit_criteria_met: false
estimated_complexity: M
strategy_id: NULL
initiative_id: migrate-character-level-up-workflow
---

# Migrate Character Level Up Workflow Initiative

## Context

The backup implementation (`mimir-dm-bu`) contains a full-featured character level-up workflow that needs to be migrated to the new v0.5 architecture. The current implementation has only a placeholder stub dialog.

### Backup Implementation Analysis

The backup has a sophisticated 6-step wizard (`LevelUpDialog.vue`) with:

1. **Class Selection** - Choose class to level in or multiclass into
   - Shows current classes with level progression
   - Validates multiclass prerequisites (D&D 5e rules)
   
2. **Hit Points** - HP gain method selection
   - Average: `(hit_die/2) + 1 + CON modifier`
   - Roll: Roll d[X] and add CON modifier
   - Manual: Enter external dice roll result

3. **Ability Scores** - ASI/Feat selection (at applicable levels)
   - ASI mode: +2 to one ability OR +1 to two abilities
   - Feat mode: Select from available feats
   - Prevents increases above 20

4. **Spells** (for Spells Known casters: Bard, Ranger, Sorcerer, Warlock)
   - SpellSelector component for spell choices
   - Updates known spells based on progression

5. **Features** - Display new class features gained

6. **Review** - Summary and confirmation

### Current State

- `LevelUpDialog.vue`: Placeholder saying "not yet available"
- `character_classes` table: Exists with `level`, `subclass_name`, `starting_class` fields
- `CharacterClass` model: Has `UpdateCharacterClass::set_level()` method
- Character proficiencies and spells tables exist
- No `level_up_character` backend command

## Rules Version

This implementation targets **D&D 5e 2014 PHB rules** for compatibility with the backup implementation. Key differences from 2024 rules:
- Subclass selection levels vary by class (not standardized to level 3)
- ASI is a class feature, not a feat
- Warlock Pact Boon is separate from Invocations

## Goals & Non-Goals

**Goals:**
- Implement full character level-up workflow matching D&D 5e 2014 rules
- Support single-class progression and multiclassing
- Handle HP gain (average/roll/manual methods)
- Handle ASI/Feat selection at appropriate class levels
- Support spell selection for all caster types (Known, Wizard spellbook, cantrips)
- Support class feature choices (Fighting Style, Expertise, Metamagic, Maneuvers, Invocations)
- Handle subclass selection at class-appropriate levels
- Validate multiclass prerequisites (both current AND target class)
- Update character data atomically

**Non-Goals:**
- Character versioning/history (removed in v0.5)
- Tracking HP/spell slot consumption (reference app design)
- 2024 PHB rules support (may add later)
- Custom homebrew class support

## Detailed Design

### Backend: LevelUpRequest Structure

```rust
pub struct LevelUpRequest {
    // Core level-up info
    pub class_name: String,
    pub class_source: String,
    pub hit_points_method: HpGainMethod,
    
    // Subclass (if at subclass level for this class)
    pub subclass: Option<SubclassChoice>,
    
    // ASI/Feat (if at ASI level for this class)
    pub asi_or_feat: Option<AsiOrFeat>,
    
    // Spells (for Spells Known, Wizard spellbook, cantrips)
    pub spell_changes: Option<SpellChanges>,
    
    // Class feature choices
    pub feature_choices: Option<FeatureChoices>,
}

pub enum HpGainMethod {
    Average,
    Roll(i32),      // Die roll result (1 to hit_die_value)
    Manual(i32),    // External roll entry
}

pub enum AsiOrFeat {
    AbilityScoreImprovement {
        ability1: String,
        increase1: i32,
        ability2: Option<String>,
        increase2: Option<i32>,
    },
    Feat { name: String, source: String },
}

pub struct SubclassChoice {
    pub name: String,
    pub source: String,
}

pub struct SpellChanges {
    // New spells learned (Spells Known casters, Wizard spellbook)
    pub new_spells: Vec<SpellReference>,
    // New cantrips learned
    pub new_cantrips: Vec<SpellReference>,
    // Spell to swap out (Spells Known casters only)
    pub swap_out: Option<SpellReference>,
    pub swap_in: Option<SpellReference>,
}

pub struct FeatureChoices {
    // Fighting Style (Fighter 1, Paladin 2, Ranger 2)
    pub fighting_style: Option<String>,
    // Expertise skills (Rogue 1/6, Bard 3/10)
    pub expertise_skills: Option<Vec<String>>,
    // Metamagic options (Sorcerer 3/10/17)
    pub metamagic: Option<Vec<String>>,
    // Battle Master maneuvers
    pub maneuvers: Option<ManeuverChoices>,
    // Warlock invocations
    pub invocations: Option<InvocationChoices>,
    // Warlock pact boon (level 3)
    pub pact_boon: Option<String>,
}

pub struct ManeuverChoices {
    pub new_maneuvers: Vec<String>,
    pub swap_out: Option<String>,
    pub swap_in: Option<String>,
}

pub struct InvocationChoices {
    pub new_invocations: Vec<String>,
    pub swap_out: Option<String>,
    pub swap_in: Option<String>,
}
```

### Backend: Level Up Service Logic

1. **Validate class choice**
   - If existing class: increment level
   - If multiclass: validate prerequisites, add new class entry

2. **Calculate HP gain**
   - Get hit die from class catalog
   - Apply method: average = `(hit_die/2) + 1`, roll/manual = provided value
   - Add CON modifier (minimum 1 total)

3. **Apply ASI/Feat (if at ASI level)**
   - Validate total increase = 2
   - Apply ability score increases (cap at 20)
   - Or add feat to character_feats table

4. **Update subclass (if provided)**
   - Set subclass_name and subclass_source on character_classes entry

5. **Update spells (if provided)**
   - Add new spells to character_spells table

6. **Update character_classes**
   - Increment level on existing class OR insert new multiclass entry

### Multiclass Prerequisites (D&D 5e PHB)

**IMPORTANT:** To multiclass, you must meet the prerequisites for BOTH your current class(es) AND the new class.

| Class | Prerequisites |
|-------|---------------|
| Barbarian | STR 13 |
| Bard | CHA 13 |
| Cleric | WIS 13 |
| Druid | WIS 13 |
| Fighter | STR 13 OR DEX 13 |
| Monk | DEX 13 AND WIS 13 |
| Paladin | STR 13 AND CHA 13 |
| Ranger | DEX 13 AND WIS 13 |
| Rogue | DEX 13 |
| Sorcerer | CHA 13 |
| Warlock | CHA 13 |
| Wizard | INT 13 |

**Example:** A Wizard (INT 13) multiclassing into Fighter needs INT 13 (to leave Wizard) AND STR/DEX 13 (to enter Fighter).

### Subclass Selection Levels (2014 PHB)

| Level | Classes |
|-------|---------|
| 1 | Cleric, Sorcerer, Warlock |
| 2 | Druid, Wizard |
| 3 | Barbarian, Bard, Fighter, Monk, Paladin, Ranger, Rogue, Artificer |

The level-up UI must show subclass selection when a character reaches the appropriate level for their class.

### ASI Levels by Class

- **Standard**: 4, 8, 12, 16, 19
- **Fighter**: 4, 6, 8, 12, 14, 16, 19 (7 total)
- **Rogue**: 4, 8, 10, 12, 16, 19 (6 total)

### HP Calculation

- **Average:** `(hit_die / 2) + 1 + CON_modifier`
  - d6 → 4, d8 → 5, d10 → 6, d12 → 7
- **Roll:** `roll_result + CON_modifier`
- **Manual:** `entered_value + CON_modifier`
- **Minimum:** Always gain at least 1 HP per level (even with negative CON)

### Spellcasting Types

Different caster types have different level-up spell mechanics:

#### Spells Known Casters (need spell selection UI)

| Class/Subclass | Type | Level-Up Behavior |
|----------------|------|-------------------|
| Bard | Full | Learn spells per table, can swap 1 |
| Ranger | Half | Learn spells per table, can swap 1 |
| Sorcerer | Full | Learn spells per table, can swap 1 |
| Warlock | Pact Magic | Learn spells per table, can swap 1 |
| Eldritch Knight | Third (Fighter) | Learn spells per table, can swap 1 |
| Arcane Trickster | Third (Rogue) | Learn spells per table, can swap 1 |

**Key feature:** Can replace ONE known spell with another when gaining a level.

#### Wizard (Spellbook - UNIQUE)

- Learns **2 free spells** to add to spellbook per level
- Prepares spells from spellbook daily (no level-up selection for prepared)
- Requires spell selection UI at every level

#### Prepared Casters (NO spell selection at level-up)

| Class | Notes |
|-------|-------|
| Cleric | Prepares from full cleric list |
| Druid | Prepares from full druid list |
| Paladin | Prepares from full paladin list |

These classes automatically have access to all class spells - no selection needed at level-up.

#### Cantrips (All Spellcasters)

Cantrips are gained at specific levels and are permanent choices. Most casters gain cantrips at levels 1, 4, and 10. Cantrip selection is separate from leveled spells.

### Class Features with Choices

Several class features require player choices at level-up (not just display):

| Feature | Class | Levels | Choices |
|---------|-------|--------|---------|
| **Fighting Style** | Fighter | 1 | Choose one style |
| **Fighting Style** | Paladin | 2 | Choose one style |
| **Fighting Style** | Ranger | 2 | Choose one style |
| **Expertise** | Rogue | 1, 6 | 2 skills each time |
| **Expertise** | Bard | 3, 10 | 2 skills each time |
| **Metamagic** | Sorcerer | 3, 10, 17 | 2, +1, +1 options |
| **Maneuvers** | Battle Master | 3, 7, 10, 15 | 3, +2, +2, +2 |
| **Eldritch Invocations** | Warlock | 2, 5, 7, 9, 12, 15, 18 | Per table |
| **Pact Boon** | Warlock | 3 | Blade/Chain/Tome |

**Swap mechanics:**
- Maneuvers: Can swap 1 when learning new ones
- Invocations: Can swap 1 per level
- Metamagic: Cannot swap (RAW)

### Warlock Special Handling

Warlock has unique level-up requirements:

1. **Pact Magic:** Separate spell slot progression (doesn't combine with other caster levels)
2. **Eldritch Invocations:** 
   - Gained at levels 2, 5, 7, 9, 12, 15, 18
   - Can swap one invocation per level-up
   - Some have prerequisites (level, pact boon)
3. **Pact Boon:** Choose at level 3 (Blade, Chain, or Tome)
4. **Mystic Arcanum:** Higher-level spells at 11, 13, 15, 17 (once per long rest each)

### Frontend: Wizard Steps

The wizard dynamically shows/hides steps based on what choices are available for the selected class at the new level.

```
LevelUpDialog.vue
├── Step 1: ClassSelectionStep.vue
│   ├── Current classes with level → level+1
│   ├── Available multiclasses with prereq validation
│   └── Shows prereqs for BOTH current class(es) AND target class
│
├── Step 2: SubclassStep.vue (CONDITIONAL)
│   ├── Show if: reaching subclass level for selected class
│   ├── Cleric/Sorcerer/Warlock: level 1 (only for new multiclass)
│   ├── Druid/Wizard: level 2
│   └── All others: level 3
│
├── Step 3: HitPointsStep.vue
│   ├── Average HP option (recommended)
│   ├── Roll option (shows die, click to roll, adds CON mod)
│   ├── Manual entry option (for physical dice)
│   └── Shows calculated total with minimum 1 HP rule
│
├── Step 4: AbilityScoreStep.vue (CONDITIONAL)
│   ├── Show if: at ASI level for selected class
│   ├── ASI mode: +2 single OR +1/+1 split (cap at 20)
│   └── Feat mode: Selection from catalog with prerequisites
│
├── Step 5: SpellsStep.vue (CONDITIONAL)
│   ├── Show if: spellcaster gaining spells at this level
│   ├── Spells Known: New spells + optional swap
│   ├── Wizard: 2 spells to add to spellbook
│   ├── Cantrips: If gaining cantrips at this level
│   └── Spell level limits based on class progression
│
├── Step 6: FeatureChoicesStep.vue (CONDITIONAL)
│   ├── Show if: class feature requires choice at this level
│   ├── Fighting Style (Fighter 1, Paladin 2, Ranger 2)
│   ├── Expertise (Rogue 1/6, Bard 3/10)
│   ├── Metamagic (Sorcerer 3/10/17)
│   ├── Maneuvers (Battle Master 3/7/10/15)
│   ├── Invocations (Warlock 2/5/7/9/12/15/18)
│   └── Pact Boon (Warlock 3)
│
├── Step 7: FeaturesDisplayStep.vue
│   └── Display all features gained at this level (info only)
│
└── Step 8: ReviewStep.vue
    ├── Summary of all choices
    ├── HP change, ability scores, new features
    └── Confirm button
```

**Step visibility logic:**
- Steps 2, 4, 5, 6 are conditional based on class/level
- Minimum flow: Class → HP → Features Display → Review (4 steps)
- Maximum flow: All 8 steps (rare, e.g., multiclassing into Warlock at a specific level)

### Frontend: Catalog Lookups Required

| Data | Catalog Command | Purpose |
|------|-----------------|---------|
| Hit Die | `get_class` | HP calculation |
| ASI Levels | `get_class` | Determine if ASI available |
| Subclass Level | `get_class` | When to show subclass step |
| Spellcasting Type | `get_class` | Full/Half/Third/Pact/None |
| Spell Progression | `get_class_spellcasting` | Spells known, slots per level |
| Multiclass Prereqs | `get_class` | Prerequisite ability scores |
| Class Features | `get_class_feature` | Features display by level |
| Subclasses | `list_subclasses` | Subclass selection for class |
| Spells | `search_spells` | Spell selection by class/level |
| Cantrips | `search_spells` | Cantrip selection (level 0) |
| Feats | `list_feats` | Feat selection with prereqs |
| Fighting Styles | `list_fighting_styles` | Fighter/Paladin/Ranger |
| Metamagic | `list_metamagic` | Sorcerer options |
| Maneuvers | `list_maneuvers` | Battle Master options |
| Invocations | `list_invocations` | Warlock options with prereqs |

## Alternatives Considered

### 1. Simplified Level Up (Rejected)
Just increment class level without wizard steps. Rejected because D&D 5e level-up involves meaningful choices (HP, ASI/feat, spells) that shouldn't be skipped.

### 2. Port Backup Code Directly (Rejected)
Copy-paste from backup. Rejected because backup has different data model (versioning, different service layer) that won't integrate cleanly.

### 3. Level Up as Separate Character Edit (Rejected)
Use UpdateCharacter with level changes. Rejected because level-up is a distinct workflow with its own validation rules and UI needs.

## Implementation Plan

### Task 1: Backend - Core Level Up Command
- Add `level_up_character` Tauri command with `LevelUpRequest`
- Implement `CharacterService::level_up()` method
- HP calculation with all methods and minimum 1 HP rule
- Multiclass prerequisite validation (current AND target class)
- Update `character_classes` (increment level or add new class)
- Update character ability scores for ASI

### Task 2: Backend - Spell and Feature Updates
- Add spell changes to `character_spells` table
- Support spell swap for Spells Known casters
- Support Wizard spellbook additions (2 per level)
- Add cantrip handling
- Add feat to `character_feats` table when chosen

### Task 3: Backend - Class Feature Choices
- Fighting style storage/updates
- Expertise skill additions
- Metamagic options (Sorcerer)
- Maneuvers (Battle Master)
- Invocations and Pact Boon (Warlock)

### Task 4: Catalog API Enhancement
- Ensure `get_class` returns: hit die, ASI levels, subclass level, spellcasting type
- Add `get_class_spellcasting` for spell progression tables
- Ensure `list_subclasses` works for subclass selection
- Add `list_feats` with prerequisite info
- Add `list_fighting_styles`, `list_metamagic`, `list_maneuvers`, `list_invocations`

### Task 5: Frontend - Wizard Framework
- Implement dynamic step wizard in LevelUpDialog.vue
- Step visibility logic based on class/level
- Progress indicator showing current/total steps
- Back/Next navigation with validation

### Task 6: Frontend - Core Steps
- ClassSelectionStep: Current classes + multiclass options with dual prereq display
- SubclassStep: Subclass selection when at appropriate level
- HitPointsStep: Average/Roll/Manual with CON modifier display
- AbilityScoreStep: ASI (+2 or +1/+1) vs Feat selection

### Task 7: Frontend - Spells and Features
- SpellsStep: New spells, cantrips, and spell swap UI
- FeatureChoicesStep: Dynamic UI for Fighting Style, Expertise, Metamagic, etc.
- FeaturesDisplayStep: Read-only display of gained features
- ReviewStep: Summary of all changes with confirm

### Task 8: Testing
- Single-class progression (Fighter 1→20)
- Multiclass scenarios (Fighter/Wizard, Rogue/Warlock)
- All HP methods
- ASI at correct levels (standard, Fighter, Rogue)
- Subclass at correct levels (1, 2, 3 depending on class)
- Spells Known swap mechanics
- Wizard spellbook additions
- Warlock invocation/pact boon flow
- Edge cases: negative CON, multiclass prereqs not met