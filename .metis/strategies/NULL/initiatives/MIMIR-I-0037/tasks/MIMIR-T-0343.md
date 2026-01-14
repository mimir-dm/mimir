---
id: schema-infrastructure-setup
level: task
title: "Schema Infrastructure Setup"
short_code: "MIMIR-T-0343"
created_at: 2026-01-14T15:49:22.176603+00:00
updated_at: 2026-01-14T15:49:22.176603+00:00
parent: MIMIR-I-0037
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/todo"


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

- [ ] Create `src/models/catalog/schema/` module with download utilities
- [ ] Add `typify` to dev-dependencies with appropriate feature flag
- [ ] Download and vendor key schemas: bestiary.json, class.json, races.json, spells.json, items.json, backgrounds.json
- [ ] Create `src/models/catalog/generated/` module structure
- [ ] Generate reference types from at least one schema (bestiary.json) as proof of concept
- [ ] Document schema update process in module comments
- [ ] All existing tests continue to pass

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

*To be added during implementation*