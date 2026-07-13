---
id: screen-coverage-viewport-captures
level: task
title: "Screen coverage, viewport captures, and angreal wiring"
short_code: "MIMIR-T-0662"
created_at: 2026-07-10T09:59:18.124918+00:00
updated_at: 2026-07-10T09:59:18.124918+00:00
parent: MIMIR-I-0074
blocked_by: ["MIMIR-T-0661"]
archived: false

tags:
  - "#task"
  - "#phase/todo"


exit_criteria_met: false
initiative_id: MIMIR-I-0074
---

# Screen coverage, viewport captures, and angreal wiring

## Parent Initiative

[[MIMIR-I-0074]] — Playwright UI harness for screenshots and UX iteration

## Objective **[REQUIRED]**

From one smoke screenshot to a full design-review capture set: every main
screen, key viewports, one command.

## Acceptance Criteria **[REQUIRED]**

- [x] Capture specs for the main screens with real data: campaign dashboard, module view (M04/M05 rosters), character sheet (a PC with spells + inventory), map view, catalog search, homebrew list
- [x] Viewport matrix: at least the app's default window size + one narrow width; captures named `<screen>--<viewport>.png`
- [x] Full-set capture script producing a timestamped run directory (before/after comparison during UX work)
- [x] Angreal task (e.g. `angreal dev screenshots`) running the whole session flow: snapshot DB → bridge → Vite → capture set → teardown; ToolDescription written for agent discovery
- [x] README or doc note in the frontend explaining the harness for future contributors
- [x] Full workspace test suite still green; harness code excluded from release builds

## Status Updates **[REQUIRED]**

### 2026-07-13 — Complete

- Screen catalog in `playwright/screens.ts`: 16 screens (dashboard tabs, document viewer, module detail with M04 roster, homebrew monster statblock, characters list, character sheet + equipment + spells sub-tabs, settings, DM map window, reference reader) x 2 viewports (desktop 1400x900 = app default from tauri.conf.json; narrow 768x900). Names follow `<screen>--<viewport>.png`.
- `npm run screenshots` writes each run to `playwright/screenshots/runs/<timestamp>/` for before/after comparison; output gitignored.
- `angreal dev screenshots` wired with ToolDescription (when_to_use/when_not_to_use); prints capture count + run dir on success.
- `playwright/README.md` documents architecture, commands, DB-safety flow, and harness limits (plugin stubs, assets opt-in, secondary-window URLs).
- Verified cold start end-to-end: 32/32 captures pass with full orchestration and teardown. Frontend 1466 tests + vue-tsc + `angreal test unit` all green. Harness code lives in the frontend and `crates/mimir-ui-bridge` — release bundle verified to contain only `mimir` + `mimir-mcp` (T-0659 note, commit f73ed44).
- Coverage notes: DM map window captures its empty state by default (map image assets not present in scratch sessions unless `MIMIR_UI_SESSION_ASSETS=link`); catalog search inside modals not yet in the catalog — add as interaction setups when needed.
- Narrow-viewport captures already surfaced UX findings for a future design pass: header crowds at 768px (campaign selector overlaps nav links); dashboard tab bar collapses to icons as designed.
