---
id: monster-type-refinement
level: task
title: "Monster Type Refinement"
short_code: "MIMIR-T-0344"
created_at: 2026-01-14T15:49:22.299897+00:00
updated_at: 2026-01-15T01:36:21.796678+00:00
parent: MIMIR-I-0037
blocked_by: [MIMIR-T-0343]
archived: true

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: MIMIR-I-0037
---

# Monster Type Refinement

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[MIMIR-I-0037]]

## Objective

Refine the monster model (`monster.rs`) to fix remaining `serde_json::Value` fields and serve as the reference implementation for other type migrations.

## Scope

**Target:** `crates/mimir-dm-core/src/models/catalog/monster.rs`

The monster model is already well-typed with polymorphic handling (ArmorClassValue, HitPointsValue, etc.), but has 3 remaining Value fields that need typing.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [x] Replace `legendary_group: Option<Value>` with typed `LegendaryGroup` struct
- [x] Replace `srd: Option<Value>` with typed `SrdValue` enum (bool or source string)
- [x] ~~Replace action `entries: Option<Value>` with typed `ActionEntry` enum~~ **Kept as Value** (see notes)
- [x] Add `tracing::warn!` logging for any fallback/default paths (N/A - new types have no fallback paths)
- [x] Compare implementation against generated bestiary types
- [x] Add deserialization tests for polymorphic monster fields
- [x] MM book import succeeds without JSON errors
- [x] All existing tests continue to pass

## Implementation Notes

### Current Value Fields in monster.rs
```rust
// These need to be replaced with typed alternatives:
pub legendary_group: Option<Value>,  // -> LegendaryGroup
pub srd: Option<Value>,              // -> SrdValue (bool | string)
pub entries: Option<Value>,          // In actions -> ActionEntry enum
```

### Reference Types
Use `types.rs` pattern for polymorphic types:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SrdValue {
    Boolean(bool),
    Source(String),
}
```

### Dependencies
- Depends on MIMIR-T-0343 (Schema Infrastructure) for generated type reference

### Risk Considerations
- Monster is the most complex catalog type
- Changes affect monster import, display, and PDF rendering
- Test with actual 5etools bestiary data

## Status Updates **[REQUIRED]**

### Session 1 - 2026-01-14

**Completed:**

1. **Added `LegendaryGroup` struct to types.rs**
   - Simple struct with `name` and `source` fields
   - References legendary group definitions (lair actions, regional effects)

2. **Added `SrdValue` enum to types.rs**
   - `SrdValue::Flag(bool)` - standard SRD inclusion marker
   - `SrdValue::Name(String)` - alternate SRD name (e.g., "Apparatus of the Crab")

3. **Updated monster.rs imports and fields**
   - `legendary_group: Option<LegendaryGroup>` (was `Option<Value>`)
   - `srd: Option<SrdValue>` (was `Option<Value>`)

4. **Documented architectural decision for action entries**
   - Kept `entries: Option<Vec<serde_json::Value>>` deliberately
   - Frontend handles 5etools tag processing ({@atk}, {@damage}, etc.)
   - Using typed Entry enum would lose information for unknown entry types
   - This is a pass-through field - Rust doesn't process the content

5. **Added 7 deserialization tests:**
   - `test_legendary_group_deserialization`
   - `test_srd_value_boolean`
   - `test_srd_value_string`
   - `test_monster_with_legendary_group`
   - `test_monster_without_legendary_group`
   - `test_monster_action_with_entries`
   - `test_full_monster_deserialization`

**Tests:** All 134 Rust core tests + 298 frontend tests pass

**Files Modified:**
- `crates/mimir-dm-core/src/models/catalog/types.rs` - Added LegendaryGroup, SrdValue
- `crates/mimir-dm-core/src/models/catalog/monster.rs` - Updated types, added tests