---
id: playwright-ui-harness-for
level: initiative
title: "Playwright UI harness for screenshots and UX iteration"
short_code: "MIMIR-I-0074"
created_at: 2026-07-10T09:54:22.539886+00:00
updated_at: 2026-07-10T10:00:20.734444+00:00
parent: 
blocked_by: []
archived: false

tags:
  - "#initiative"
  - "#phase/active"


exit_criteria_met: false
estimated_complexity: M
initiative_id: playwright-ui-harness-for
---

# Playwright UI harness for screenshots and UX iteration Initiative

## Context **[REQUIRED]**

Goal (Dylan, 2026-07-10): a harness so an agent can run the Mimir UI, take
screenshots, and do UI/UX design work.

Stack recon: frontend is Vite + Vue 3 (`crates/mimir/frontend`), Vitest for unit
tests, `@tauri-apps/api` v2 for IPC. No Playwright present. There is an existing
`__tests__/helpers/mockInvoke.ts` — a routing layer over a mocked `invoke()`
with per-command handlers, plus a `__tests__/fixtures/` directory (monsters,
spells, characters, homebrew, etc.). This is the pattern to port to the browser
runtime.

**Hard constraint: `tauri-driver` (WebDriver for real Tauri windows) does not
support macOS** — wry/WKWebView has no WebDriver endpoint. Driving the actual
app window with Playwright is not possible on this machine. The harness must
drive the frontend in Chromium against the Vite dev server, with the Tauri IPC
layer shimmed in the browser.

## Goals & Non-Goals **[REQUIRED]**

**RULING (Dylan, 2026-07-10): real-campaign-data screenshots are the point** —
"kind of the only way this is useful." The HTTP bridge is therefore the CORE of
this initiative, not a follow-up; fixtures are demoted to an optional later
convenience for CI.

**Goals:**
- A dev-only **invoke-over-HTTP bridge**: the real Rust backend (services, real DB) reachable from a browser context, so the frontend renders actual campaign data (The Frost Architect)
- `npx playwright` harness in `crates/mimir/frontend`: boots Vite + the bridge, opens pages in Chromium, screenshots to a predictable directory
- **DB safety**: the harness NEVER points at the production DB directly — every session snapshots prod to a scratch copy (or uses the dev DB) and runs the bridge against that. The UI can mutate; a screenshot session must not be able to touch the live campaign. (See memory: production DB incident 2026-03-07.)
- Screenshot scripts an agent can run per-screen and per-viewport for design review
- Angreal task wiring (e.g. `angreal dev screenshots`) so the flow is one command

**Non-Goals:**
- Driving the real Tauri window (impossible on macOS; revisit only if CI adds Linux where tauri-driver works)
- Fixture-based rendering (optional later, for CI determinism — not needed for the design-work goal)
- Visual-regression gating in CI (later, once screenshots are stable)
- Exposing the bridge in release builds (feature-flagged, localhost-only, dev builds exclusively)

## Detailed Design **[REQUIRED]**

**Bridge transport — decide at design, two candidates:**
1. **`tauri-invoke-http`** (tauri-apps org crate): swaps Tauri's IPC transport to
   HTTP POST so remote/browser contexts can call invoke directly — near-zero
   per-command glue because dispatch goes through Tauri's own invoke handler.
   MUST VERIFY Tauri v2 compatibility first (design spike, web search + test).
2. **Minimal axum bridge** (fallback): feature-flagged endpoint in the mimir
   crate with a mirrored command dispatch table (one line per command, same
   shape as the existing `generate_handler!` list). More glue, fully in our
   control.

**Browser side:** Playwright `addInitScript` installs an `invoke()` shim that
forwards to the bridge over `fetch`. Frontend code unchanged.

**Session flow (safety-critical):** harness script = snapshot prod DB → scratch
copy → launch backend+bridge against the copy (requires a dev-build env
override for the DB path, e.g. honoring MIMIR_DATABASE_PATH in the app's
AppState the way the MCP server already does — verify state.rs at design) →
boot Vite → run Playwright specs → tear down. The copy is disposable.

Note: once the bridge serves the app in a browser, Claude's Chrome tools can
also drive it interactively for live UX work; Playwright is the scripted layer.

## Alternatives Considered **[REQUIRED]**

- **tauri-driver + WebDriver**: not available on macOS; rejected for now.
- **Screenshot the real app window via OS-level tooling (screencapture)**: no DOM access, no interaction scripting, flaky targeting; useful ad-hoc but not a harness.
- **Vitest browser mode**: closer to unit tests; Playwright gives real-page navigation, viewports, and first-class screenshots.

## Implementation Plan **[REQUIRED]**

Proposed decomposition (pending approval):
1. **Bridge spike + transport decision**: verify tauri-invoke-http v2 compatibility; pick transport; prove one real command (list_campaigns) answered over HTTP from a browser against a scratch DB copy
2. **Safe session flow**: DB-path env override in the app's dev build; snapshot-launch-teardown script; localhost-only + dev-feature-flag enforcement
3. **Playwright scaffold + invoke shim**: config, webServer orchestration (bridge + Vite), init-script shim, smoke spec screenshotting the dashboard with real Frost Architect data
4. **Screen coverage + design-review scripts**: per-screen/per-viewport captures (dashboard, module roster, character sheet, map view, catalog), output conventions, angreal task wiring

## Status Updates

- 2026-07-10: Created from Dylan's request. Discovery recon done (stack, mockInvoke pattern, macOS tauri-driver constraint). REORIENTED same day per Dylan: real-campaign-data screenshots are the point — bridge is core, fixtures demoted to optional. Awaiting decompose approval.