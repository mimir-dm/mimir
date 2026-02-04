---
id: enable-wal-mode-and-remove-global
level: task
title: "Enable WAL mode and remove global mutex for concurrent reads"
short_code: "MIMIR-T-0512"
created_at: 2026-02-02T01:25:05.047047+00:00
updated_at: 2026-02-04T02:19:15.479672+00:00
parent: 
blocked_by: []
archived: false

tags:
  - "#task"
  - "#tech-debt"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: NULL
---

# Enable WAL mode and remove global mutex for concurrent reads

## Objective

Enable SQLite WAL (Write-Ahead Logging) mode and remove the global `Mutex<SqliteConnection>` to allow concurrent database reads. Currently every Tauri command and MCP tool acquires a mutex lock, meaning all requests — even reads — serialize through one lock.

## Backlog Item Details

### Type
- [x] Tech Debt - Code improvement or refactoring

### Priority
- [x] P1 - High (important for user experience)

### Technical Debt Impact
- **Current Problems**: Every command does `state.db.lock()` on a single `Mutex<SqliteConnection>`. The dashboard loads multiple API calls concurrently (characters, modules, documents, homebrew) — all serialize through one lock. MCP has the same issue via `ctx.db()`.
- **Benefits of Fixing**: Concurrent reads with WAL mode. Dashboard loads faster. MCP can handle parallel tool calls. Simpler code without mutex boilerplate.
- **Risk Assessment**: Low. WAL mode is SQLite's recommended mode for concurrent access. No new dependencies needed.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [x] SQLite WAL mode enabled at database initialization (`PRAGMA journal_mode=WAL`)
- [x] Global `Mutex<SqliteConnection>` removed from `AppState`
- [x] Tauri commands create connections on-demand (or use a simple pool without r2d2)
- [x] `McpContext` uses the same pattern
- [x] All tests pass (MCP tests; pre-existing failures in mimir-core unrelated to this change)
- [x] Concurrent reads work without blocking each other

## Implementation Notes

### Why WAL, not r2d2?

**r2d2 is overkill for SQLite:**
- Connection pools help when connections are expensive (network DBs) — SQLite connections are cheap (~microseconds)
- Pools help when you need many concurrent connections — SQLite only allows one writer anyway
- Adds dependency complexity for minimal benefit

**WAL mode is the right fix:**
- Allows concurrent readers while writing (readers don't block writers, writers don't block readers)
- Multiple connections can read simultaneously
- Writes still serialize (SQLite's design), but that's fine
- One pragma, no new dependencies

### Technical Approach
1. Enable WAL mode in `db.rs` after connection: `conn.execute("PRAGMA journal_mode=WAL")`
2. Remove `Mutex<SqliteConnection>` from `AppState`
3. Create connections on-demand in each command using `establish_connection()`
4. Or: keep a simple connection factory in state (not a pool, just the DB path)
5. Update `McpContext` similarly

### Risk Considerations
- SQLite with WAL: multiple concurrent readers, one writer at a time
- This is strictly better than current Mutex (one reader OR writer at a time)
- Connection creation is cheap for SQLite, no need to pool

## Status Updates

### 2026-02-03 Analysis

**Current Architecture:**
- `AppState` in `state.rs`: `db: Mutex<SqliteConnection>`, `paths: AppPaths`
- `McpContext` in `context.rs`: `db: Mutex<SqliteConnection>`, plus `assets_dir`, `active_campaign_id`
- Every command does `state.db.lock()` - serializes ALL access
- `init_database()` in `db.rs`: creates connection, runs migrations, sets `PRAGMA foreign_keys = ON`

**Plan:**
1. Enable WAL mode in `init_database()` 
2. Change `AppState.db` from `Mutex<SqliteConnection>` to store just the db path
3. Add `connect()` method that creates connections on-demand
4. Update all Tauri commands to call `state.connect()` instead of `state.db.lock()`
5. Same pattern for `McpContext`
6. Each connection enables foreign_keys + uses WAL mode

**Completed:**

### Step 1: Enable WAL mode in `db.rs`
- Added `configure_connection()` function that sets:
  - `PRAGMA journal_mode=WAL` for concurrent reads
  - `PRAGMA foreign_keys=ON` for referential integrity
  - `PRAGMA synchronous=NORMAL` for better performance with WAL
- Added `create_connection()` function for on-demand connections after startup
- `init_database()` now calls `configure_connection()`

### Step 2: Remove Mutex from AppState (`state.rs`)
- Changed `db: Mutex<SqliteConnection>` to `db_url: String`
- Added `connect()` method that creates connections on-demand via `create_connection()`
- Added `active_campaign_id: Mutex<Option<String>>` for consistency with MCP
- `AppState::new()` now takes just `AppPaths` (no connection)

### Step 3: Update Tauri commands
- Updated all 14 command files (268 occurrences)
- Changed `state.db.lock()` to `state.connect()` 
- Error messages simplified from "Database lock error" to just the connection error

### Step 4: Update MCP context (`context.rs`)
- Changed `db: Mutex<SqliteConnection>` to `db_url: String`
- Added `connect()` method mirroring AppState pattern
- Removed deprecated `db()` method (was just an alias)
- Added `for_testing()` constructor for test contexts

### Step 5: Update MCP tools
- Updated all 9 tool files (61 occurrences)
- Changed `ctx.db()` to `ctx.connect()`

### Step 6: Update main.rs
- `init_database()` now just initializes (runs migrations) and drops connection
- `AppState::new()` receives only paths, not a connection

### Verification
- `cargo check -p mimir-core` - passes
- `cargo check -p mimir` - passes  
- `cargo check -p mimir-mcp` - passes
- `cargo test -p mimir-mcp` - all 16 tests pass

**Note:** Pre-existing test failures in mimir-core (47 tests) related to schema mismatch (`campaign_assets.description` column missing). These failures existed before this change - confirmed by stashing changes and re-running tests.

### 2026-02-03 Test Infrastructure Fix

Fixed pre-existing test failures caused by DAL test files using hardcoded SQL instead of real migrations.

**Root Cause:** 33 DAL test files had `setup_test_db()` functions with hardcoded CREATE TABLE SQL that didn't stay in sync with actual migrations (missing `description` column on `campaign_assets` table).

**Solution:** Replaced `setup_test_db()` with:
1. `test_connection()` from `crate::db` - runs real embedded migrations
2. `setup_test_data()` - uses DAL functions to insert required seed data

**Files Fixed:**
- 19 campaign DAL files (campaign, module, character, map, document, etc.)
- 14 catalog DAL files (source, action, book, skill, deity, spell_list, etc.)
- 6 service test files (background, class, feat, monster, race catalog services)
- Added missing catalog sources to `test_utils.rs` (TCE, VGM, MPMM, GoS, CR)
- Fixed map delete order in `services/map.rs` (delete map before asset to avoid FK violation)

**Result:** All 797 mimir-core tests pass.