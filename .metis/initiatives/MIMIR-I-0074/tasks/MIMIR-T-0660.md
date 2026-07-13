---
id: safe-screenshot-session-flow-db
level: task
title: "Safe screenshot-session flow (DB snapshot + env override)"
short_code: "MIMIR-T-0660"
created_at: 2026-07-10T09:59:15.561284+00:00
updated_at: 2026-07-10T09:59:15.561284+00:00
parent: MIMIR-I-0074
blocked_by: ["MIMIR-T-0659"]
archived: false

tags:
  - "#task"
  - "#phase/todo"


exit_criteria_met: false
initiative_id: MIMIR-I-0074
---

# Safe screenshot-session flow (DB snapshot + env override)

## Parent Initiative

[[MIMIR-I-0074]] — Playwright UI harness for screenshots and UX iteration

## Objective **[REQUIRED]**

Make it structurally impossible for a screenshot/UX session to touch the live
campaign. Sessions run against a disposable snapshot of the production DB.
(Memory: production DB deleted 2026-03-07 — never again.)

## Acceptance Criteria **[REQUIRED]**

- [x] The bridge/backend honors a DB-path override (e.g. MIMIR_DATABASE_PATH env var, mirroring the MCP server's behavior in crates/mimir-mcp/src/context.rs) — verify how crates/mimir/src/state.rs resolves the path today and add the override for dev builds
- [x] Session script (`scripts/ui-session.sh` or angreal-managed): snapshot prod DB via sqlite backup API → scratch dir → export override → launch bridge → on exit, tear down scratch copy
- [x] The bridge REFUSES to start if its DB path resolves to the production location (belt and suspenders — hard check on the canonical prod path)
- [x] Snapshot includes the assets dir reference or maps degrade gracefully (UVTT blobs live on disk; read-only access to real assets is acceptable — document the choice)
- [x] Documented one-command flow

## Status Updates **[REQUIRED]**

### 2026-07-13 — Complete

- DB-path override: implemented at the bridge (`MIMIR_BRIDGE_APP_DIR` scratch app dir) rather than in `state.rs` — the desktop app's path resolution stays untouched, which is structurally safer than teaching the app itself to accept overrides.
- `scripts/ui-session.sh`: sqlite `.backup` of the prod DB -> `mktemp` scratch dir -> builds+runs ui-bridge against the copy -> trap removes scratch and kills the bridge on exit. Verified: after a full run, no `/tmp/mimir-ui-session.*` remains and no bridge process survives.
- Hard refusal verified earlier (T-0659): bridge exits 1 if its DB canonicalizes under `~/Library/Application Support/com.mimir.app` — including the `dev/` subdir; sessions run on snapshots only.
- Assets decision (documented in playwright/README.md): the assets tree is ~6 GB, never copied. Default sessions run without assets — map views degrade to placeholders. `MIMIR_UI_SESSION_ASSETS=link` opt-in symlinks the real tree for map-image rendering, read-only by convention (asset mutations during a linked session would touch real files — warned in script output).
- One-command flow: `angreal dev screenshots` (or `npm run screenshots`); Playwright `webServer` invokes ui-session.sh + Vite automatically and reuses running instances.
- Note: during the ui-bridge crate split (bundler collision fix, commit f73ed44) the bridge moved to `crates/mimir-ui-bridge`; run with `cargo run -p mimir-ui-bridge`.
