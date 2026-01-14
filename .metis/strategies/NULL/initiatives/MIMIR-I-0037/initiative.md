---
id: schema-first-catalog-model
level: initiative
title: "Schema-First Catalog Model Hardening"
short_code: "MIMIR-I-0037"
created_at: 2026-01-05T02:27:26.213586+00:00
updated_at: 2026-01-14T15:49:12.354657+00:00
parent: MIMIR-V-0001
blocked_by: []
archived: false

tags:
  - "#initiative"
  - "#phase/decompose"


exit_criteria_met: false
estimated_complexity: L
strategy_id: NULL
initiative_id: schema-first-catalog-model
---

# Schema-First Catalog Model Hardening Initiative

*This template includes sections for various types of initiatives. Delete sections that don't apply to your specific use case.*

## Context

The 5etools JSON format is notoriously polymorphic - the same field can be a number, string, array, or object depending on context. Our current catalog models use ~30 `serde_json::Value` escape hatches across 13 files to handle this complexity, leading to:

- **Silent parsing failures** - Data lost without errors (e.g., MM book import fails on AC format `[12]` vs `[{"ac": 15}]`)
- **No type safety** - Runtime errors instead of compile-time catches
- **Fragile extraction** - Hand-coded `.unwrap_or()` chains that default without logging
- **Zero import tests** - No integration tests for actual JSON parsing paths

**Problem Files:**
- `class.rs` - 9 Value fields (hd, proficiency, class_features, equipment, multiclassing, etc.)
- `race.rs` - 4 Value fields (speed, lineage, height_and_weight)
- `monster.rs` - 3 Value fields (legendary_group, srd, action entries)
- `background.rs`, `spell.rs`, `optionalfeature.rs` - Various untyped fields

**Current State:**
- 23 catalog model files, ~6200 lines total
- `types.rs` has 17+ untagged enums (good pattern to follow)
- `monster.rs` shows well-typed polymorphic handling (ArmorClassValue, HitPointsValue, etc.)

## Goals & Non-Goals

**Goals:**
- Generate/derive Rust types from 5etools JSON Schema for authoritative type definitions
- Reduce `serde_json::Value` usage by 80% (from ~30 to ~6 escape hatches)
- Add import integration tests for all critical catalog types
- Zero deserialization failures on 5etools test corpus
- Maintain backward compatibility with existing database JSON columns

**Non-Goals:**
- Full schema coverage for all 50+ 5etools entity types (focus on high-value: monster, class, race, spell, item, background)
- Runtime schema validation (types provide compile-time safety instead)
- Breaking changes to existing database schema

## Architecture

### Schema Source
5etools provides pre-resolved JSON Schema at:
- `https://github.com/TheGiddyLimit/5etools-utils/tree/master/schema/brew-fast/`
- Uses standard JSON Schema Draft 2020-12 (no `$$merge` preprocessor)
- Modular structure with `$ref` composition

### Tooling
**Primary:** [Typify](https://github.com/oxidecomputer/typify) - JSON Schema to Rust type generator
- Active maintenance (v0.5.0, Oxide Computer)
- Handles `oneOf`, `allOf`, enums, format strings
- CLI and build.rs support

### Strategy: Hybrid Incremental Migration
1. **Generate types for high-value schemas only** (monster, class, race, spell, item, background)
2. **Place generated types in `generated/` module** as reference implementation
3. **Migrate hand-maintained types incrementally** using generated types as authoritative source
4. **Keep hand-maintained types for customization** (database mapping, summary extraction, `From` impls)

### Directory Structure
```
crates/mimir-dm-core/src/models/catalog/
├── mod.rs
├── types.rs          # Shared polymorphic types (expand here)
├── monster.rs        # Hand-maintained with typed fields
├── class.rs          # Migrate Value → typed
├── race.rs           # Migrate Value → typed
├── ...
├── schema/           # NEW: Schema management
│   ├── mod.rs
│   └── download.rs   # Schema fetching utilities
└── generated/        # NEW: Reference types from schema
    ├── mod.rs
    ├── bestiary.rs
    ├── class.rs
    └── ...
```

## Detailed Design

### Phase 1: Schema Infrastructure
- Create `schema/` module with download utilities for 5etools schemas
- Add `typify` to dev-dependencies with `generate-types` feature flag
- Download and vendor key schemas: bestiary.json, class.json, races.json, spells.json, items.json, backgrounds.json

### Phase 2: Monster Hardening (Reference Implementation)
- Generate types from bestiary.json schema
- Compare against existing monster.rs (already well-typed)
- Fix remaining Value fields: `legendary_group`, `srd`, action `entries`
- Add logging for silent failure paths

### Phase 3: Class Migration (Highest Priority)
Replace 9 Value fields in class.rs:
| Field | Target Type |
|-------|-------------|
| `hd` | `HitDice` struct |
| `proficiency` | `Vec<String>` or `Proficiency` enum |
| `class_features` | `Vec<ClassFeatureReference>` |
| `starting_proficiencies` | `StartingProficiencies` |
| `multiclassing` | `Multiclassing` with typed `requirements` |
| `class_table_groups` | `Vec<ClassTableGroup>` |
| `starting_equipment` | `StartingEquipment` |
| `optionalfeature_progression` | `Vec<OptionalFeatureProgression>` |
| `subclass_features` | `Vec<SubclassFeatureReference>` |

### Phase 4: Race Migration
Replace 4 Value fields in race.rs:
| Field | Target Type |
|-------|-------------|
| `speed` | `RaceSpeed` (number or SpeedObject) |
| `lineage` | `Lineage` enum (bool or source string) |
| `height_and_weight` | `HeightAndWeight` struct |

### Phase 5: Remaining Entities
- Background: `starting_equipment` → `Vec<StartingEquipmentEntry>`
- Spell: `MaterialComponent.consume` → `ConsumeValue` enum
- OptionalFeature: `additional_spells` → use existing `AdditionalSpells` type

### Phase 6: Testing Infrastructure
- Create `tests/integrations/import/` directory
- Add test fixtures (subset of real 5etools data)
- Write deserialization tests for each entity type
- Add round-trip serialization tests

## Testing Strategy

### Test Fixtures
- Vendor subset of real 5etools data (~10-20 entities per type)
- Store in `tests/integrations/import/fixtures/`

### Test Categories
1. **Deserialization tests** - Parse real JSON, verify typed fields populated
2. **Round-trip tests** - Serialize/deserialize equality
3. **Edge case tests** - Polymorphic variants (AC as number vs array vs object)
4. **Database integration** - Full import pipeline with catalog service

## Alternatives Considered

| Approach | Pros | Cons | Decision |
|----------|------|------|----------|
| **Minimal fix** | Fast, low risk | Whack-a-mole forever | Rejected |
| **Monster-only** | Focused | Doesn't fix class/race issues | Rejected |
| **Full schema generation** | Complete coverage | Massive bloat (50+ types) | Rejected |
| **Schema-first hybrid** | Targeted + authoritative | Some manual work | **Selected** |

## Implementation Plan

### Tasks to Create
1. **Schema Infrastructure** - Download utilities, typify setup, feature flag
2. **Monster Type Refinement** - Fix remaining Value fields, add logging
3. **Class Type Migration** - Replace 9 Value fields with typed structs
4. **Race Type Migration** - Replace 4 Value fields with typed structs  
5. **Remaining Entity Types** - Background, spell, optionalfeature
6. **Import Test Suite** - Fixtures, deserialization tests, round-trip tests

### Critical Files
- `crates/mimir-dm-core/src/models/catalog/types.rs` - Add new shared types
- `crates/mimir-dm-core/src/models/catalog/class.rs` - Highest priority migration
- `crates/mimir-dm-core/src/models/catalog/race.rs` - Second priority
- `crates/mimir-dm-core/src/models/catalog/monster.rs` - Reference implementation
- `crates/mimir-dm-core/Cargo.toml` - Add typify dependency

### Success Criteria
- [ ] Reduce `serde_json::Value` from ~30 to ~6 usages
- [ ] MM book import succeeds without JSON errors
- [ ] Import tests for monster, class, race, spell, item, background
- [ ] No database migrations required