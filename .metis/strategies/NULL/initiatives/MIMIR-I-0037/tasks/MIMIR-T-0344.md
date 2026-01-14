---
id: monster-type-refinement
level: task
title: "Monster Type Refinement"
short_code: "MIMIR-T-0344"
created_at: 2026-01-14T15:49:22.299897+00:00
updated_at: 2026-01-14T15:49:22.299897+00:00
parent: MIMIR-I-0037
blocked_by: [MIMIR-T-0343]
archived: false

tags:
  - "#task"
  - "#phase/todo"


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

- [ ] Replace `legendary_group: Option<Value>` with typed `LegendaryGroup` struct
- [ ] Replace `srd: Option<Value>` with typed `SrdValue` enum (bool or source string)
- [ ] Replace action `entries: Option<Value>` with typed `ActionEntry` enum
- [ ] Add `tracing::warn!` logging for any fallback/default paths
- [ ] Compare implementation against generated bestiary types
- [ ] Add deserialization tests for polymorphic monster fields
- [ ] MM book import succeeds without JSON errors
- [ ] All existing tests continue to pass

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

*To be added during implementation*