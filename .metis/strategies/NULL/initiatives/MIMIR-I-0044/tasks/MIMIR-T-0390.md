---
id: create-services-module-structure
level: task
title: "Create services module structure and ServiceError type"
short_code: "MIMIR-T-0390"
created_at: 2026-01-21T03:02:29.664205+00:00
updated_at: 2026-01-21T03:08:46.748661+00:00
parent: MIMIR-I-0044
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: MIMIR-I-0044
---

# Create services module structure and ServiceError type

## Parent Initiative

[[MIMIR-I-0044]] - Implement Service Layer for mimir-core

## Objective

Create the foundational services module structure and unified error type that all services will use. This enables the rest of the service layer implementation.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [x] `src/services/mod.rs` exists and is exported from `lib.rs`
- [x] `ServiceError` enum covers all error cases (NotFound, Validation, Database, Io)
- [x] Error type implements `std::error::Error` and `thiserror::Error`
- [x] Module compiles without errors
- [x] Basic unit test for error creation/display

## Implementation Notes

### Files to Create

```
crates/mimir-core/src/services/
├── mod.rs          # Module exports, ServiceError definition
└── (future files)  # Other services added in subsequent tasks
```

### ServiceError Definition

```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ServiceError {
    #[error("Not found: {entity_type} with id {id}")]
    NotFound { entity_type: String, id: String },
    
    #[error("Validation error: {0}")]
    Validation(String),
    
    #[error("Database error: {0}")]
    Database(#[from] diesel::result::Error),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

pub type ServiceResult<T> = Result<T, ServiceError>;
```

### Dependencies

- `thiserror` already in workspace dependencies
- No new crate dependencies needed

## Status Updates

### 2026-01-21: Completed
- Created `src/services/mod.rs` with `ServiceError` enum
- Added `DEFAULT_QUERY_LIMIT = 1000` constant
- Added helper constructors `ServiceError::not_found()` and `ServiceError::validation()`
- Exported module from `lib.rs`
- All 4 unit tests passing