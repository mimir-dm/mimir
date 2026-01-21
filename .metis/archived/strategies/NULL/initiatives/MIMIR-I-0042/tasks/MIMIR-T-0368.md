---
id: create-mimir-core-crate-structure
level: task
title: "Create mimir-core crate structure"
short_code: "MIMIR-T-0368"
created_at: 2026-01-20T02:43:35.008309+00:00
updated_at: 2026-01-20T18:27:40.821918+00:00
parent: MIMIR-I-0042
blocked_by: []
archived: true

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: MIMIR-I-0042
---

# Create mimir-core crate structure

## Parent Initiative

[[MIMIR-I-0042]] - v0.5 Catalog Implementation

## Objective

Set up the `mimir-core` crate with proper directory structure, Cargo.toml, and module organization. This crate contains data models, Diesel schema, and basic CRUD operations (DAL).

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `crates/mimir-core/Cargo.toml` created with dependencies (diesel, serde, tokio, etc.)
- [ ] `crates/mimir-core/src/lib.rs` with module exports
- [ ] Module structure created:
  - `src/models/mod.rs` and `src/models/catalog/mod.rs`
  - `src/dal/mod.rs`
  - `src/generated/mod.rs` (placeholder for typify output)
  - `src/schema.rs` (placeholder for Diesel schema)
- [ ] `migrations/` directory created
- [ ] Crate compiles with `cargo build -p mimir-core`
- [ ] Workspace Cargo.toml updated to include mimir-core

## Implementation Notes

### Directory Structure
```
crates/mimir-core/
├── Cargo.toml
├── src/
│   ├── lib.rs
│   ├── schema.rs           # Diesel schema (generated)
│   ├── models/
│   │   ├── mod.rs
│   │   └── catalog/
│   │       └── mod.rs
│   ├── dal/
│   │   └── mod.rs
│   └── generated/
│       └── mod.rs          # typify output
└── migrations/
```

### Key Dependencies
```toml
[dependencies]
diesel = { version = "2", features = ["sqlite", "r2d2"] }
diesel_migrations = "2"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
tokio = { version = "1", features = ["rt-multi-thread"] }
anyhow = "1"
thiserror = "1"
chrono = { version = "0.4", features = ["serde"] }
```

### Dependencies
- None (foundational task)

## Status Updates

*To be added during implementation*