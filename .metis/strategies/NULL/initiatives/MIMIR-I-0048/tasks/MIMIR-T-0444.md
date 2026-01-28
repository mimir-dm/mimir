---
id: catalog-api-enhancement-for-level
level: task
title: "Catalog API Enhancement for Level Up"
short_code: "MIMIR-T-0444"
created_at: 2026-01-27T21:15:11.216912+00:00
updated_at: 2026-01-27T21:37:12.651336+00:00
parent: MIMIR-I-0048
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: MIMIR-I-0048
---

# Catalog API Enhancement for Level Up

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[MIMIR-I-0048]]

## Objective

Ensure the catalog API provides all data needed for level-up decisions: class info, spell progression, subclasses, feats, and class feature options (fighting styles, metamagic, maneuvers, invocations).

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [x] `get_class_info` returns: hit_die, subclass_level, multiclass_prereqs, ASI_levels, spellcasting_type
- [x] `get_class_spellcasting` returns spell slots and spells known per level
- [x] `list_subclasses_by_class` returns subclasses for a given class (already existed)
- [x] `list_feats_with_prereqs` returns feats with prerequisites
- [x] `list_fighting_styles` returns all fighting style options
- [x] `list_metamagic` returns all metamagic options
- [x] `list_maneuvers` returns all Battle Master maneuvers
- [x] `list_invocations` returns invocations with level/pact prerequisites

## Implementation Notes

### Required Catalog Commands

| Command | Returns | Used For |
|---------|---------|----------|
| `get_class` | Class details | HP, prereqs, ASI levels |
| `get_class_spellcasting` | Spell progression table | Spells known, slots |
| `list_subclasses` | Subclass options | Subclass selection step |
| `list_feats` | Feats with prereqs | ASI/Feat step |
| `list_fighting_styles` | Fighting styles | Fighter/Paladin/Ranger |
| `list_metamagic` | Metamagic options | Sorcerer |
| `list_maneuvers` | Maneuvers | Battle Master |
| `list_invocations` | Invocations with prereqs | Warlock |

### Class Data Structure
```rust
pub struct ClassInfo {
    pub name: String,
    pub hit_die: i32,              // 6, 8, 10, or 12
    pub subclass_level: i32,       // 1, 2, or 3
    pub asi_levels: Vec<i32>,      // [4, 8, 12, 16, 19] or class-specific
    pub multiclass_prereqs: MulticlassPrereqs,
    pub spellcasting_type: Option<SpellcastingType>,
}

pub enum SpellcastingType {
    Full,       // Bard, Cleric, Druid, Sorcerer, Wizard
    Half,       // Paladin, Ranger
    Third,      // Eldritch Knight, Arcane Trickster
    PactMagic,  // Warlock
}
```

### Invocation Prerequisites
```rust
pub struct Invocation {
    pub name: String,
    pub description: String,
    pub level_prereq: Option<i32>,
    pub pact_prereq: Option<String>,  // "Blade", "Chain", "Tome"
}
```

### Data Source
Most data comes from 5etools JSON in the catalog database.

## Status Updates

### 2026-01-27: Implementation Complete

All acceptance criteria have been implemented:

**New Tauri Commands Added:**

1. **`get_class_info`** - Returns structured class data for level-up:
   - `hit_die`: The class hit die size (6, 8, 10, or 12)
   - `subclass_level`: Level at which subclass is chosen (1, 2, or 3)
   - `asi_levels`: Array of levels where ASI/Feat choices occur
   - `multiclass_prereqs`: Ability score requirements for multiclassing
   - `spellcasting_type`: Full, Half, Third, PactMagic, or null
   - `spellcasting_ability`: The ability used for spellcasting
   - `optional_feature_progression`: For classes with invocations, metamagic, etc.
   - `cantrip_progression`: Cantrips known per level
   - `spells_known_progression`: Spells known per level

2. **`get_class_spellcasting`** - Returns full spellcasting progression:
   - `is_spellcaster`: Boolean indicating if class has spellcasting
   - `caster_type`: full, 1/2, 1/3, or pact
   - `spellcasting_ability`: The casting ability
   - `cantrip_progression`: Array indexed by level
   - `spells_known_progression`: Array indexed by level (for known casters)
   - `prepared_spells`: Formula for prepared casters
   - `spell_slots_by_level`: Standard spell slot table

3. **`list_fighting_styles`** - Returns all fighting styles with:
   - Standard optional feature data
   - `available_to_classes`: Array of class names that can use the style

4. **`list_metamagic`** - Returns all metamagic options (feature_type = "MM")

5. **`list_maneuvers`** - Returns all Battle Master maneuvers (feature_type starts with "MV")

6. **`list_invocations`** - Returns all Eldritch Invocations with:
   - Standard optional feature data including prerequisite JSON
   - `level_prereq`: Parsed warlock level requirement
   - `pact_prereq`: Required pact boon (Blade, Chain, Tome)
   - `spell_prereq`: Required spell (e.g., eldritch blast)

7. **`list_feats_with_prereqs`** - Returns feats with parsed prerequisites:
   - Standard feat data
   - `parsed_prereqs`: Human-readable array of requirements

**Existing Commands (already satisfy requirements):**
- `list_subclasses_by_class` - Already existed, returns subclasses for a given class

**Helper Functions Added:**
- `find_subclass_level()` - Parses classFeatures to find subclass gain level
- `extract_multiclass_prereqs()` - Extracts multiclassing requirements
- `determine_asi_levels()` - Returns ASI levels (with Fighter/Rogue special handling)
- `extract_spell_slots_from_table()` - Parses classTableGroups for spell slots
- `generate_spell_slot_progression()` - Generates standard spell slot tables by caster type
- `extract_fighting_style_classes()` - Maps feature_type codes to class names
- `extract_invocation_prereqs()` - Parses invocation prerequisites
- `extract_feat_prereqs()` - Parses feat prerequisites into readable strings

**Files Modified:**
- `crates/mimir/src/commands/catalog.rs` - Added 7 new commands and helper functions
- `crates/mimir/src/main.rs` - Registered new commands

All code compiles successfully.