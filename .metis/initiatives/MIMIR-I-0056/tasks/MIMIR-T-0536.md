---
id: add-ci-heavy-test-job-for-post
level: task
title: "Add CI heavy test job for post-merge integration tests"
short_code: "MIMIR-T-0536"
created_at: 2026-03-09T14:25:12.097564+00:00
updated_at: 2026-03-10T01:17:31.844593+00:00
parent: MIMIR-I-0056
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: MIMIR-I-0056
---

# Add CI heavy test job for post-merge integration tests

## Parent Initiative

[[MIMIR-I-0056]]

## Objective

Add a CI job that runs heavy integration tests (Rust SRD-seeded DB smoke tests + full frontend test suite) only on pushes to main, keeping PR checks fast while ensuring post-merge quality.

## Acceptance Criteria

## Acceptance Criteria

- [x] New `test-integration` job in `.github/workflows/ci.yml`
- [x] Job runs only on `push` to `main` (not on PRs)
- [x] Runs Rust integration tests: `cargo test -p mimir-core --test srd_smoke_test -- --test-threads=1`
- [x] Runs frontend tests: `npx vitest run`
- [x] Installs all required dependencies (Node, Rust, Ubuntu libs, npm)
- [x] Existing CI jobs (test-gui-build, test-frontend, test-other-crates, coverage) unchanged

## Implementation Notes

### Changes Made

**`.github/workflows/ci.yml`** — Added `test-integration` job:
- Condition: `if: github.event_name == 'push' && github.ref == 'refs/heads/main'`
- Runs on `ubuntu-latest`
- Steps: checkout, setup node (lts/*), install Rust stable, install Ubuntu deps (webkit2gtk, appindicator, librsvg, patchelf), npm ci, cargo test srd_smoke_test, vitest run
- Uses `--test-threads=1` for Rust tests to avoid SQLite concurrency issues with in-memory DBs

### Design Decisions
- **Post-merge only**: Heavy tests add ~2-3 min; not needed on every PR since lightweight tests catch most issues
- **Separate from existing test-frontend job**: The existing job runs on every PR; this integration job runs the same tests plus Rust integration tests, providing a full validation after merge
- **No sidecar build needed**: Integration tests don't exercise Tauri commands, only mimir-core DAL functions

## Status Updates

### 2026-03-09
- Edited `.github/workflows/ci.yml` to add `test-integration` job
- Verified all 4 existing jobs remain unchanged
- All acceptance criteria met