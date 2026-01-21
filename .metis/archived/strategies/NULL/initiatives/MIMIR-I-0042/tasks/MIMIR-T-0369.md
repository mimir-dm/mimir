---
id: create-mimir-services-crate
level: task
title: "Create mimir-services crate structure"
short_code: "MIMIR-T-0369"
created_at: 2026-01-20T02:43:35.369480+00:00
updated_at: 2026-01-20T18:27:41.085960+00:00
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

# Create mimir-services crate structure

## Parent Initiative

[[MIMIR-I-0042]] - v0.5 Catalog Implementation

## Objective

Set up the `mimir-services` crate with proper directory structure. This crate contains business logic, the 5etools import pipeline, and search services. Depends on `mimir-core`.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `crates/mimir-services/Cargo.toml` created with dependency on mimir-core
- [ ] `crates/mimir-services/src/lib.rs` with module exports
- [ ] Module structure created:
  - `src/catalog/mod.rs`
  - `src/catalog/import.rs` (placeholder)
  - `src/catalog/search.rs` (placeholder)
- [ ] Crate compiles with `cargo build -p mimir-services`
- [ ] Workspace Cargo.toml updated to include mimir-services

## Implementation Notes

### Directory Structure
```
crates/mimir-services/
├── Cargo.toml
└── src/
    ├── lib.rs
    └── catalog/
        ├── mod.rs
        ├── import.rs       # 5etools import logic
        └── search.rs       # FTS query building
```

### Key Dependencies
```toml
[dependencies]
mimir-core = { path = "../mimir-core" }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
tokio = { version = "1", features = ["rt-multi-thread", "fs"] }
anyhow = "1"
thiserror = "1"
tracing = "0.1"
```

### Dependencies
- Requires [[MIMIR-T-0368]] (mimir-core crate)

## Status Updates

*To be added during implementation*