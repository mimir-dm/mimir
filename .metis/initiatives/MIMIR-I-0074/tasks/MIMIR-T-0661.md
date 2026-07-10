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

### 2026-07-10 — Scaffold working end-to-end; first design-review pass done (commit 7b4c202)

- Shim implemented **in-app** instead of via `addInitScript`: `src/harness/bridge-shim.ts` is imported first in `main.ts`, guarded by `import.meta.env.DEV && !('__TAURI_INTERNALS__' in window)` — tree-shaken from production builds, inert inside the real Tauri app, and it works for BOTH Playwright and live Chrome sessions. It installs `window.__TAURI_INTERNALS__` (invoke → `fetch` to the bridge, transformCallback, convertFileSrc passthrough, metadata).
- Plugin commands stubbed as planned: `plugin:event|listen/unlisten` no-op; dialog/shell reject with a console warning (harness-unsupported flows).
- `playwright.config.ts` + two capture specs (`capture.spec.ts` route walk, `capture-detail.spec.ts` interaction-driven: module detail, document viewer, sheet sub-tabs, homebrew statblock). 19 captures, all passing, real Frost Architect data. `npm run test:e2e` / `npm run screenshots`; screenshots gitignored; vitest excludes `playwright/`.
- Remaining for AC-complete: `webServer` orchestration in the config (today the bridge + Vite are started manually) and an explicit "The Frost Architect visible" assertion in a smoke spec — both small; natural to fold into T-0660's session script.
- Found during captures, fixed in the same commit: ASCII placeholder tab icons, a global `.campaign-dashboard { text-align: center }` leak, dark-on-dark selected document row, missing space after trait names in three stat panels, `beast , unaligned` comma bug, duplicated level in sheet subtitle, asterisk "has notes" marker.
- Route note for capture specs: `/modules/:id` is a stub view ("coming soon") — module screens are reached through the dashboard Modules tab selection, not the direct route.
