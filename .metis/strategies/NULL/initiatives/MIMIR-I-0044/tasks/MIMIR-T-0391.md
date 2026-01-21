---
id: implement-catalogservice-trait-and
level: task
title: "Implement CatalogService trait and MonsterService"
short_code: "MIMIR-T-0391"
created_at: 2026-01-21T03:02:29.834972+00:00
updated_at: 2026-01-21T03:13:48.800695+00:00
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

# Implement CatalogService trait and MonsterService

## Parent Initiative

[[MIMIR-I-0044]] - Implement Service Layer for mimir-core

## Objective

Define the `CatalogEntityService` trait that provides a generic interface for catalog entity access, then implement `MonsterService` as the first concrete implementation. This establishes the pattern for all other catalog services.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `CatalogEntityService` trait defined with associated types
- [ ] `MonsterService` struct implemented with stateful connection pattern
- [ ] Search with filters (name, source, CR, type, size)
- [ ] Pagination support via `search_paginated()`
- [ ] Get by ID, get by name+source
- [ ] List available sources
- [ ] Unit tests using `test_utils::setup_test_db_with_sources()`
- [ ] `DEFAULT_QUERY_LIMIT = 1000` constant for memory protection

## Implementation Notes

### Files to Create

```
crates/mimir-core/src/services/
├── mod.rs              # Add catalog module export
├── catalog/
│   ├── mod.rs          # CatalogEntityService trait, re-exports
│   └── monster.rs      # MonsterService implementation
```

### CatalogEntityService Trait

```rust
pub trait CatalogEntityService {
    type Entity;
    type Filter: Default;
    type Summary;
    
    fn search(&mut self, filter: &Self::Filter) -> ServiceResult<Vec<Self::Summary>>;
    fn search_paginated(&mut self, filter: &Self::Filter, limit: i64, offset: i64) -> ServiceResult<Vec<Self::Summary>>;
    fn get(&mut self, id: i32) -> ServiceResult<Option<Self::Entity>>;
    fn get_by_name_and_source(&mut self, name: &str, source: &str) -> ServiceResult<Option<Self::Entity>>;
    fn list_sources(&mut self) -> ServiceResult<Vec<String>>;
    fn count(&mut self) -> ServiceResult<i64>;
}
```

### MonsterService Pattern

```rust
pub struct MonsterService<'a> {
    conn: &'a mut SqliteConnection,
}

impl<'a> MonsterService<'a> {
    pub fn new(conn: &'a mut SqliteConnection) -> Self {
        Self { conn }
    }
}

impl<'a> CatalogEntityService for MonsterService<'a> {
    type Entity = Monster;
    type Filter = MonsterFilter;
    type Summary = Monster; // Or a lighter MonsterSummary if needed
    // ... implementations delegate to DAL
}
```

### Dependencies

- MIMIR-T-0390 (ServiceError type)
- Existing `dal::catalog::monster` module
- Existing `models::catalog::Monster`, `MonsterFilter`

## Status Updates

*To be added during implementation*