---
id: replace-single-mutex-db-connection
level: task
title: "Replace single mutex DB connection with r2d2 connection pool"
short_code: "MIMIR-T-0512"
created_at: 2026-02-02T01:25:05.047047+00:00
updated_at: 2026-02-02T01:25:05.047047+00:00
parent: 
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/backlog"
  - "#tech-debt"


exit_criteria_met: false
strategy_id: NULL
initiative_id: NULL
---

# Replace single mutex DB connection with r2d2 connection pool

## Objective

Replace the single mutex-locked `SqliteConnection` with an `r2d2` connection pool to eliminate serialized database access. Currently every Tauri command and MCP tool acquires a global mutex lock, meaning concurrent requests queue behind each other.

## Backlog Item Details

### Type
- [x] Tech Debt - Code improvement or refactoring

### Priority
- [x] P1 - High (important for user experience)

### Technical Debt Impact
- **Current Problems**: Every command does `state.db.lock()` on a single `Mutex<SqliteConnection>`. The dashboard loads multiple API calls concurrently (characters, modules, documents, homebrew) — all serialize through one lock. MCP has the same issue via `ctx.db()`.
- **Benefits of Fixing**: Concurrent reads (SQLite WAL mode supports this). Dashboard loads faster. MCP can handle parallel tool calls. Eliminates the 4-line mutex boilerplate repeated 50+ times.
- **Risk Assessment**: Low-medium. `diesel::r2d2` is well-established. SQLite needs WAL mode enabled for concurrent readers. Write contention is inherent to SQLite but no worse than current mutex approach.

## Acceptance Criteria

- [ ] `AppState` uses `r2d2::Pool<ConnectionManager<SqliteConnection>>` instead of `Mutex<SqliteConnection>`
- [ ] SQLite WAL mode enabled at pool initialization
- [ ] All Tauri commands use `state.pool.get()` instead of `state.db.lock()`
- [ ] `McpContext` uses the same pool pattern
- [ ] All tests pass
- [ ] No deadlocks or connection leaks under concurrent access

## Implementation Notes

### Technical Approach
1. Add `r2d2` dependency to mimir-core: `diesel = { features = ["r2d2"] }`
2. Create pool in `db.rs` with `Pool::builder().max_size(4).build(manager)`
3. Enable WAL: `PRAGMA journal_mode=WAL` on connection initialization
4. Replace `Mutex<SqliteConnection>` in `AppState` with `Pool`
5. Each command calls `state.pool.get()?` to get a pooled connection
6. Update `McpContext` similarly

### Risk Considerations
- SQLite with WAL supports multiple concurrent readers but only one writer. This is fine — current behavior is strictly worse (one reader OR writer at a time).
- Pool size of 4 is reasonable for a desktop app. Reader connections can run in parallel; writer will briefly block others.

## Status Updates

*To be added during implementation*