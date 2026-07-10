---
id: playwright-scaffold-browser-invoke
level: task
title: "Playwright scaffold + browser invoke shim + smoke screenshot"
short_code: "MIMIR-T-0661"
created_at: 2026-07-10T09:59:17.013044+00:00
updated_at: 2026-07-10T09:59:17.013044+00:00
parent: MIMIR-I-0074
blocked_by: ["MIMIR-T-0660"]
archived: false

tags:
  - "#task"
  - "#phase/todo"


exit_criteria_met: false
initiative_id: MIMIR-I-0074
---

# Playwright scaffold + browser invoke shim + smoke screenshot

## Parent Initiative

[[MIMIR-I-0074]] — Playwright UI harness for screenshots and UX iteration

## Objective **[REQUIRED]**

Playwright installed and configured in `crates/mimir/frontend`; the app renders
in Chromium with `invoke()` forwarded to the bridge; one smoke spec produces a
screenshot of the campaign dashboard showing real Frost Architect data.

## Acceptance Criteria **[REQUIRED]**

- [ ] `@playwright/test` dev-dependency + `playwright.config.ts` with webServer orchestration (bridge + Vite dev server, reuse-existing for local iteration)
- [ ] Invoke shim installed via `addInitScript` BEFORE app load: window.__TAURI_INTERNALS__/invoke intercepted (check what @tauri-apps/api v2 core.invoke actually calls in a non-Tauri context — shim at the right layer) and forwarded to the bridge via fetch
- [ ] Tauri plugins used by the frontend (dialog, shell) either shimmed to no-ops or the affected flows documented as harness-unsupported
- [ ] Smoke spec: load `/`, wait for campaign list, assert "The Frost Architect" visible, screenshot to `crates/mimir/frontend/playwright/screenshots/`
- [ ] Screenshots directory gitignored; `npm run test:e2e` and `npm run screenshots` scripts
- [ ] Works headless (agent-runnable) and headed (human-watchable)

## Status Updates **[REQUIRED]**

*To be added during implementation*
