---
id: test-coverage-improvement
level: initiative
title: "Test Coverage Improvement"
short_code: "MIMIR-I-0021"
created_at: 2025-12-17T13:34:02.370129+00:00
updated_at: 2026-01-19T21:58:52.005835+00:00
parent: MIMIR-V-0001
blocked_by: []
archived: true

tags:
  - "#initiative"
  - "#phase/completed"


exit_criteria_met: false
estimated_complexity: M
strategy_id: NULL
initiative_id: test-coverage-improvement
---

# Test Coverage Improvement Initiative

## Context

A code review revealed gaps in test coverage and quality issues in existing tests:
- **Missing integration tests** for critical services (DocumentService, TableService)
- **501 `.unwrap()` calls** in tests that mask errors and make debugging harder
- **Test setup duplication** - same ~15 lines copied across 22+ test files
- **Limited error path testing** - only 1 `.is_err()` assertion found
- **No shared test utilities** - each test file recreates fixtures from scratch

## Goals & Non-Goals

**Goals:**
- Add integration tests for untested services (DocumentService, TableService, CatalogTrait)
- Create shared test fixtures and utilities to reduce duplication
- Add error path testing for all major services
- Replace `.unwrap()` with `.expect()` for better error messages

**Non-Goals:**
- 100% code coverage (focus on critical paths)
- E2E/UI testing (separate initiative)
- Performance/load testing

## Detailed Design

### Phase 1: Create Shared Test Infrastructure

Expand `tests/integrations/common/mod.rs`:
```rust
// tests/common/fixtures.rs
pub struct TestContext {
    pub db: TestDatabase,
    pub conn: SqliteConnection,
}

impl TestContext {
    pub fn new() -> Self { ... }
    pub fn with_campaign() -> Self { ... }
    pub fn with_documents() -> Self { ... }
    pub fn with_character() -> Self { ... }
}

// Builder pattern for test data
pub struct CampaignBuilder { ... }
pub struct DocumentBuilder { ... }
```

### Phase 2: Add Missing Integration Tests

| Service | Lines | Priority | Focus Areas |
|---------|-------|----------|-------------|
| DocumentService | 321 | High | CRUD, filtering, level-based queries |
| TableService | ~100 | High | Basic CRUD operations |
| CatalogTrait | Interface | Medium | Trait compliance verification |
| Error handling | 92 | Medium | `is_unique_violation()`, `is_foreign_key_violation()` |

### Phase 3: Add Error Path Tests

For each service, add tests for:
- Not found scenarios
- Constraint violations (unique, foreign key)
- Invalid input handling
- Transaction rollback scenarios

## Alternatives Considered

1. **Add mocking framework (mockall)**: Deferred - real database tests provide better coverage
2. **Separate test crate**: Rejected - adds complexity, current structure is adequate
3. **Property-based testing (proptest)**: Could be added later for edge cases

## Implementation Plan

1. Create shared test fixtures in `common/fixtures.rs`
2. Refactor existing tests to use shared fixtures
3. Add DocumentService integration tests
4. Add TableService integration tests  
5. Add error path tests to existing service tests
6. Replace `.unwrap()` with `.expect()` across all test files