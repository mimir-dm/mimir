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

- [ ] The bridge/backend honors a DB-path override (e.g. MIMIR_DATABASE_PATH env var, mirroring the MCP server's behavior in crates/mimir-mcp/src/context.rs) — verify how crates/mimir/src/state.rs resolves the path today and add the override for dev builds
- [ ] Session script (`scripts/ui-session.sh` or angreal-managed): snapshot prod DB via sqlite backup API → scratch dir → export override → launch bridge → on exit, tear down scratch copy
- [ ] The bridge REFUSES to start if its DB path resolves to the production location (belt and suspenders — hard check on the canonical prod path)
- [ ] Snapshot includes the assets dir reference or maps degrade gracefully (UVTT blobs live on disk; read-only access to real assets is acceptable — document the choice)
- [ ] Documented one-command flow

## Status Updates **[REQUIRED]**

*To be added during implementation*
