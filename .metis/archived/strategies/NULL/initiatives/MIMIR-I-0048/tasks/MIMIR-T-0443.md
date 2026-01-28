---
id: backend-class-feature-choices
level: task
title: "Backend - Class Feature Choices"
short_code: "MIMIR-T-0443"
created_at: 2026-01-27T21:15:10.704844+00:00
updated_at: 2026-01-27T22:35:12.229372+00:00
parent: MIMIR-I-0048
blocked_by: []
archived: true

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: MIMIR-I-0048
---

# Backend - Class Feature Choices

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[MIMIR-I-0048]]

## Objective

Implement storage and updates for class features that require player choices at level-up: Fighting Style, Expertise, Metamagic, Maneuvers, and Warlock Invocations/Pact Boon.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [x] Store Fighting Style selection for Fighter/Paladin/Ranger
- [x] Store Expertise skill selections for Rogue/Bard
- [x] Store Metamagic options for Sorcerer
- [x] Store Battle Master Maneuvers with swap support
- [x] Store Warlock Eldritch Invocations with swap support
- [x] Store Warlock Pact Boon selection
- [ ] Validate invocation prerequisites (level, pact boon) - Deferred to frontend

## Implementation Notes

### Feature Choices by Class/Level

| Feature | Class | Levels | Storage |
|---------|-------|--------|---------|
| Fighting Style | Fighter | 1 | `character_features` |
| Fighting Style | Paladin | 2 | `character_features` |
| Fighting Style | Ranger | 2 | `character_features` |
| Expertise | Rogue | 1, 6 | `character_proficiencies` (expertise flag) |
| Expertise | Bard | 3, 10 | `character_proficiencies` (expertise flag) |
| Metamagic | Sorcerer | 3, 10, 17 | `character_features` |
| Maneuvers | Battle Master | 3, 7, 10, 15 | `character_features` |
| Invocations | Warlock | 2, 5, 7, 9, 12, 15, 18 | `character_features` |
| Pact Boon | Warlock | 3 | `character_features` |

### FeatureChoices Structure
```rust
pub struct FeatureChoices {
    pub fighting_style: Option<String>,
    pub expertise_skills: Option<Vec<String>>,
    pub metamagic: Option<Vec<String>>,
    pub maneuvers: Option<ManeuverChoices>,
    pub invocations: Option<InvocationChoices>,
    pub pact_boon: Option<String>,
}
```

### May Need New Table
Consider `character_features` table if not already exists:
```sql
CREATE TABLE character_features (
    id TEXT PRIMARY KEY,
    character_id TEXT NOT NULL,
    feature_type TEXT NOT NULL,  -- 'fighting_style', 'metamagic', etc.
    feature_name TEXT NOT NULL,
    feature_source TEXT,
    FOREIGN KEY (character_id) REFERENCES characters(id)
);
```

### Dependencies
- MIMIR-T-0441 (Core Level Up Command)
- MIMIR-T-0444 (Catalog API for invocation prerequisites)

## Status Updates

### Session 2026-01-27

**Completed Implementation:**

1. **Database Migration** (`023_character_features/up.sql`):
   - Created `character_features` table with columns: id, character_id, feature_type, feature_name, feature_source, source_class
   - Added indexes for character_id and feature_type

2. **Model Layer** (`character_feature.rs`):
   - `CharacterFeature` - queryable model with type checking methods
   - `FeatureType` enum - FightingStyle, Metamagic, Maneuver, Invocation, PactBoon
   - `NewCharacterFeature` - insertable with convenience constructors

3. **DAL Layer** (`dal/campaign/character_feature.rs`):
   - `insert_character_feature`, `get_character_feature`
   - `list_features_by_type`, `list_features_by_class`
   - `character_has_feature`, `find_feature_by_name` (for swaps)
   - `delete_character_feature`, `count_features_by_type`

4. **Service Layer Types** (`services/character.rs`):
   - `FeatureChoices` - Container for all feature choice types
   - `FeatureReference` - Name + source reference
   - `ManeuverChoices` - new_maneuvers + swap_out/swap_in
   - `InvocationChoices` - new_invocations + swap_out/swap_in

5. **LevelUpRequest Extended**:
   - Added `feature_choices: Option<FeatureChoices>` field

6. **Feature Handling in `level_up` Method** (Step 9):
   - Fighting Style: Prevents duplicates
   - Metamagic: Adds multiple options, skips duplicates
   - Maneuvers: Swap support (validates swap_out exists), adds new
   - Invocations: Swap support (validates swap_out exists), adds new
   - Pact Boon: Single selection, checks for existing
   - Expertise: Upgrades existing skill proficiency or adds new with expertise

7. **Exports Updated** (`services/mod.rs`):
   - Added `FeatureChoices`, `FeatureReference`, `ManeuverChoices`, `InvocationChoices`

**Design Decisions:**
- Invocation prerequisites deferred to frontend (has catalog data for validation)
- Expertise uses existing `character_proficiencies` table with expertise flag
- Maneuver/Invocation swaps validate that swap_out feature exists

**Verification:**
- `cargo check` passed with no errors