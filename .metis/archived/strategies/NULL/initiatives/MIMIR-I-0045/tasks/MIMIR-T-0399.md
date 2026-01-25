---
id: scaffold-tauri-v2-application-with
level: task
title: "Scaffold Tauri v2 application with Vue 3 frontend"
short_code: "MIMIR-T-0399"
created_at: 2026-01-21T16:13:09.776637+00:00
updated_at: 2026-01-21T17:24:31.115520+00:00
parent: MIMIR-I-0045
blocked_by: []
archived: true

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: MIMIR-I-0045
---

# Scaffold Tauri v2 application with Vue 3 frontend

## Parent Initiative

[[MIMIR-I-0045]] - Implement Tauri Application

## Objective

Set up the Tauri v2 Rust backend with `mimir-core` integration and migrate the existing Vue 3 frontend. Investigation (T-0360, T-0363) confirmed ~70% of the existing frontend is reusable.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

**Rust Backend:**
- [x] Create `crates/mimir` Cargo project with Tauri v2 dependencies
- [x] Configure `tauri.conf.json` with window settings (1400x900, min 1200x700)
- [x] Set up `AppState` with `Mutex<SqliteConnection>` and `app_data_dir: PathBuf`
- [x] Implement database initialization on app startup (run migrations via mimir-core::db)
- [x] Add `mimir-core` as workspace dependency
- [x] Add to workspace Cargo.toml

**Frontend Migration:**
- [x] Copy existing frontend from `mimir-dm-bu/mimir-dm/frontend/`
- [x] Update package.json with current Tauri v2 dependencies (already up to date)
- [x] Verify frontend builds with Vite
- [x] Verify app builds with `cargo build`

## Implementation Notes

### Rust Backend Structure

```
crates/mimir-tauri/
├── Cargo.toml
├── tauri.conf.json
├── build.rs
├── src/
│   ├── main.rs          # Tauri entry point
│   ├── lib.rs           # Library for commands
│   ├── state.rs         # AppState definition
│   └── commands/
│       └── mod.rs       # Command module (initially empty)
└── frontend/
    └── ...              # Vue app
```

### AppState Pattern

```rust
use diesel::SqliteConnection;
use std::path::PathBuf;
use std::sync::Mutex;

pub struct AppState {
    pub db: Mutex<SqliteConnection>,
    pub app_data_dir: PathBuf,
}
```

### Database Initialization

On startup:
1. Determine app data directory (`tauri::api::path::app_data_dir`)
2. Create directory if needed
3. Establish SQLite connection at `{app_data_dir}/mimir.db`
4. Run diesel migrations
5. Store connection in AppState

### Frontend Structure

```
frontend/
├── index.html
├── package.json
├── vite.config.ts
├── tsconfig.json
├── tailwind.config.js
└── src/
    ├── main.ts
    ├── App.vue
    ├── router/
    │   └── index.ts
    ├── stores/
    │   └── .gitkeep
    ├── views/
    │   └── HomeView.vue
    ├── components/
    │   └── .gitkeep
    └── types/
        └── .gitkeep
```

### Dependencies

**Cargo.toml:**
- `tauri` v2 with features: `["macos-private-api"]`
- `tauri-plugin-shell`
- `tauri-plugin-dialog`
- `mimir-core` (workspace)
- `diesel` with `sqlite` feature
- `serde`, `serde_json`

**package.json:**
- `vue` ^3.4
- `vue-router` ^4
- `pinia` ^2
- `@tauri-apps/api` v2
- `tailwindcss`, `postcss`, `autoprefixer`
- `typescript`, `vite`, `@vitejs/plugin-vue`

## Status Updates

### 2026-01-21

**Rust Backend Complete:**
- Created `crates/mimir/` with Tauri v2 scaffold
- `build.rs` - Tauri build integration
- `tauri.conf.json` - Window config (1400x900, min 1200x700), security capabilities
- `src/state.rs` - AppState with Mutex<SqliteConnection> and app_data_dir
- `src/error.rs` - CommandError type converting ServiceError to serializable format
- `src/lib.rs` - Module exports
- `src/main.rs` - Entry point with database initialization via mimir_core::db::init_database

**Database Module Added to mimir-core:**
- Created `crates/mimir-core/src/db.rs` with `MIGRATIONS` and `init_database()`
- Runs migrations on startup, enables foreign keys

**Frontend Migrated:**
- Copied `src/`, config files, and icons from backup
- Updated version to 0.5.0
- Placeholder `dist/index.html` for Tauri build
- Tauri dependencies already at v2.0.0

**Build Status:**
- `cargo check -p mimir` passes (warnings in mimir-core unrelated to this work)
- `npm install && npm run build` succeeds - frontend builds to 178KB sources bundle
- `cargo build` succeeds - full Tauri application compiles

**Files Created:**
```
crates/mimir/
├── build.rs              # Tauri build script
├── Cargo.toml            # Tauri v2 + mimir-core dependencies
├── tauri.conf.json       # Window config, capabilities
├── icons/                # App icons (copied from backup)
├── src/
│   ├── main.rs           # Entry point with db init
│   ├── lib.rs            # Module exports
│   ├── state.rs          # AppState struct
│   └── error.rs          # CommandError type
└── frontend/             # Vue 3 app (migrated from backup)
    ├── src/              # Vue components, stores, views
    ├── package.json      # v0.5.0, Tauri API v2
    └── dist/             # Built assets
```

**Also Created:**
- `crates/mimir-core/src/db.rs` - Exposed migrations for app startup

**Dev/Production Mode Detection:**
- `is_dev_mode()` - Returns true if `cfg!(debug_assertions)` or `MIMIR_DEV` env var set
- `AppPaths` - Handles path separation: dev uses `{app_data_dir}/dev/`, production uses `{app_data_dir}/`
- Separate databases and assets for dev vs production to prevent test data corruption
- Mode logged on startup: "Mimir DEVELOPMENT mode initialized" or "Mimir PRODUCTION mode initialized"