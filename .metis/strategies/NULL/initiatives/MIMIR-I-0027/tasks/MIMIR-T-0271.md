---
id: extend-characterdata-for-complete
level: task
title: "Extend CharacterData for complete PDF export"
short_code: "MIMIR-T-0271"
created_at: 2026-01-02T21:18:27.842533+00:00
updated_at: 2026-01-02T22:59:33.654918+00:00
parent: MIMIR-I-0027
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: MIMIR-I-0027
---

# Extend CharacterData for complete PDF export

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[MIMIR-I-0027]]

## Objective

Close the gap between stored character data and PDF export by adding missing fields for appearance, backstory, roleplay notes, and combat state. Currently many PDF sections render blank placeholders because CharacterData lacks corresponding fields

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] Add Appearance struct (age, height, weight, eyes, hair, skin, physical_description, distinctive_features)
- [ ] Add RoleplayNotes struct (voice_and_mannerisms, key_relationships, character_goals, play_reminders, allies_and_organizations, additional_treasure_notes)

- [ ] Add backstory, background_feature, player_name fields to CharacterData
- [ ] Update compact_sheet.rs to render new fields instead of placeholders
- [ ] Update character_longform.rs to render new fields instead of placeholders
- [ ] All new fields use #[serde(default)] for backward compatibility
- [ ] Existing character data deserializes without errors



## Implementation Notes

### Files to Modify
- `crates/mimir-dm-core/src/models/character/data.rs` - Add new structs and fields
- `crates/mimir-dm-core/src/models/character/mod.rs` - Re-export new types
- `crates/mimir-dm-print/src/sections/compact_sheet.rs` - Replace placeholders with data
- `crates/mimir-dm-print/src/sections/character_longform.rs` - Replace placeholders with data

### Technical Approach
1. Add Appearance, RoleplayNotes, CombatState structs to data.rs
2. Add new fields to CharacterData with #[serde(default)]
3. Update compact_sheet.rs placeholder locations (~lines 129, 349, 366, 545, 589, 700, 738)
4. Update character_longform.rs placeholder locations (~lines 84-147, 274, 294, 317, 330, 345, 360)

### Dependencies
- MIMIR-T-0270 (Composable Character Sheet Printing) - completed

## Status Updates **[REQUIRED]**

*To be added during implementation*