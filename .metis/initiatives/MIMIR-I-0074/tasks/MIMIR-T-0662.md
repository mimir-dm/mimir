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

- [ ] Capture specs for the main screens with real data: campaign dashboard, module view (M04/M05 rosters), character sheet (a PC with spells + inventory), map view, catalog search, homebrew list
- [ ] Viewport matrix: at least the app's default window size + one narrow width; captures named `<screen>--<viewport>.png`
- [ ] Full-set capture script producing a timestamped run directory (before/after comparison during UX work)
- [ ] Angreal task (e.g. `angreal dev screenshots`) running the whole session flow: snapshot DB → bridge → Vite → capture set → teardown; ToolDescription written for agent discovery
- [ ] README or doc note in the frontend explaining the harness for future contributors
- [ ] Full workspace test suite still green; harness code excluded from release builds

## Status Updates **[REQUIRED]**

*To be added during implementation*
