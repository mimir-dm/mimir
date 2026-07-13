# UI Harness (Playwright + invoke-over-HTTP bridge)

Runs the Mimir frontend in a real browser against the **real Rust backend and
real campaign data**, without Tauri. Used for screenshots, design review, and
agent-driven UX work. (`tauri-driver` doesn't exist on macOS, so the real app
window can't be driven directly — this is the workaround, see MIMIR-I-0074.)

## How it fits together

```
Playwright/Chrome ──▶ Vite dev server (localhost:5173)
        │                    │  frontend + src/harness/bridge-shim.ts
        │                    ▼  (installs window.__TAURI_INTERNALS__ when
        │                        there's no Tauri runtime; dev builds only)
        └── invoke() ──▶ ui-bridge (127.0.0.1:4175)
                             │  POST /invoke/{command} → the app's real
                             │  generate_handler! pipeline (tauri::test)
                             ▼
                     scratch COPY of the production DB
```

**DB safety:** `scripts/ui-session.sh` snapshots the production database with
sqlite's backup API into a throwaway scratch dir, runs the bridge against the
copy, and deletes it on exit. The bridge refuses to start on any path under
the production app dir. UI mutations during a session hit the copy, never the
live campaign.

## Commands

```sh
angreal dev screenshots      # full capture set, fully orchestrated, one command
npm run screenshots          # same, from this directory (timestamped run dir)
npm run test:e2e             # all Playwright specs (smoke + captures + repros)
npx playwright test playwright/smoke.spec.ts   # quick boot check
```

Playwright's `webServer` config starts the bridge (via ui-session.sh) and Vite
automatically and reuses them if already running — for iteration, keep both up:

```sh
bash ../../scripts/ui-session.sh   # terminal 1 (from crates/mimir)
npm run dev                        # terminal 2
```

## Captures

- Screen catalog + viewport matrix live in `playwright/screens.ts`
  (desktop 1400×900 = app default, narrow 768).
- `npm run screenshots` writes `screens.ts × viewports` to
  `playwright/screenshots/runs/<timestamp>/` as `<screen>--<viewport>.png` —
  compare directories for before/after during UX work.
- Screenshot output is gitignored.

## Known harness limits

- Plugin IPC is stubbed by the shim: event listeners never fire; dialogs are
  unavailable except **save**, which becomes a browser download; shell-open is
  rejected. Flows depending on those are harness-unsupported.
- Map images: the assets tree (~6 GB) is not copied. Maps render without
  images unless you opt in: `MIMIR_UI_SESSION_ASSETS=link` symlinks the real
  assets dir (read-only *by convention* — don't upload/delete assets during a
  linked session).
- The catalog/reference reader and DM map are separate window entries — load
  them as `/sources.html` and `/dm-map.html?moduleId=…&campaignId=…`.
