---
id: schema-infrastructure-setup
level: task
title: "Schema Infrastructure Setup"
short_code: "MIMIR-T-0343"
created_at: 2026-01-14T15:49:22.176603+00:00
updated_at: 2026-01-14T21:22:06.663297+00:00
parent: MIMIR-I-0037
blocked_by: []
archived: true

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: MIMIR-I-0037
---

# Schema Infrastructure Setup

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[MIMIR-I-0037]]

## Objective

Set up the foundational infrastructure for schema-first type generation, including downloading 5etools JSON schemas and configuring the typify code generator.

## Scope

**Target:** `crates/mimir-dm-core/`

This task creates the foundation for all subsequent type migration tasks by establishing:
- Schema download and management utilities
- Typify configuration for Rust type generation
- Directory structure for generated types

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [x] Create `src/models/catalog/schema/` module with download utilities
- [x] Add `typify` to dev-dependencies with appropriate feature flag
- [x] Download and vendor key schemas: bestiary.json, class.json, races.json, spells.json, items.json, backgrounds.json
- [x] Create `src/models/catalog/generated/` module structure
- [x] Generate reference types from at least one schema (bestiary.json) as proof of concept
- [x] Document schema update process in module comments
- [x] All existing tests continue to pass

## Implementation Notes

### Schema Source
5etools provides pre-resolved JSON Schema at:
- `https://github.com/TheGiddyLimit/5etools-utils/tree/master/schema/brew-fast/`
- Uses standard JSON Schema Draft 2020-12 (no `$$merge` preprocessor)

### Directory Structure
```
crates/mimir-dm-core/src/models/catalog/
├── mod.rs
├── types.rs
├── monster.rs
├── class.rs
├── ...
├── schema/           # NEW
│   ├── mod.rs
│   └── download.rs   # Schema fetching utilities
└── generated/        # NEW
    ├── mod.rs
    └── bestiary.rs   # Proof of concept
```

### Typify Configuration
```toml
[dev-dependencies]
typify = { version = "0.5", optional = true }

[features]
generate-types = ["typify"]
```

### Risk Considerations
- Schema format may change between 5etools versions
- Generated types may need manual adjustment for serde compatibility
- Keep generated module behind feature flag to avoid build-time overhead

## Status Updates **[REQUIRED]**

### Session 1 - 2026-01-14

**Completed:**
1. Created `schema/mod.rs` with constants for schema URLs and paths
2. Created `schema/download.rs` with async download utilities (behind `schema-download` feature)
3. Added feature flags to Cargo.toml: `schema-download` and `generated-types`
4. Downloaded 8 vendored schemas from 5etools-utils repository:
   - bestiary.json (106KB), class.json (64KB), races.json (41KB)
   - spells.json (43KB), items.json (85KB), backgrounds.json (12KB)
   - entry.json (79KB), util.json (80KB)
5. Created `generated/mod.rs` documenting the type generation approach
6. Created proof-of-concept schema (`alignment_poc.json`) and generated Rust types

**Key Discovery:**
- typify doesn't support external `$ref` references (e.g., `../util.json#/$defs/alignment`)
- The 5etools schemas use extensive cross-file references
- Documented three workarounds in `generated/mod.rs`:
  1. Schema bundling (resolve all refs into one file)
  2. Manual extraction (extract specific types)
  3. Reference comparison (use schemas as documentation)
- Created self-contained PoC schema to demonstrate typify works for bundled schemas
- Generated ~1150 lines of Rust types from PoC showing: enums for alignment, creature types, sizes, damage types, conditions, skills, ability scores

**Tests:** All 156 Rust workspace tests + 298 frontend tests pass