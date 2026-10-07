# UI Harness (Playwright + invoke-over-HTTP bridge)

Runs the Mimir frontend in a real browser against the **real Rust backend and a
seeded fixture campaign**, without Tauri. Used for screenshots, design review,
and agent-driven UX work. (`tauri-driver` doesn't exist on macOS, so the real app
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
                     scratch DB seeded with the UI fixture
```

## Data: the UI fixture

Development machines hold **no real campaign data**; real campaigns live on a
different computer. Every session starts from an empty scratch dir, and the
bridge seeds it at startup (`MIMIR_BRIDGE_SEED=fixture`,
`mimir_core::seed::seed_ui_fixture`):

- **SRD catalog** from `crates/mimir-core/tests/fixtures/srd_*.json`: classes,
  subclasses, features, backgrounds, races, 44 items, 43 spells with their class
  lists, 17 monsters.
- **"The Lost Mine of Phandelver"** dev campaign (`mimir-core/src/seed/dev.rs`):
  the 11 template documents, PCs (Thorin, Elara, Finn, Sister Helena) with
  classes, gear and spells, NPCs, homebrew items and the Cragmaw Mutant, the
  Cragmaw Hideout module with its roster, and the Goblin Hideout map with
  tokens, lights, traps and POIs. Map assets are copied from
  `mimir-core/src/seed/assets` into the scratch dir.

IDs are new every session. Specs name fixture entities in `fixture.ts`
(`FIXTURE`) and resolve their IDs through the bridge (`resolveFixture()`).

**Safety:** `scripts/ui-session.sh` reads nothing under the production app dir,
and the bridge refuses to start on any path inside it. The scratch dir is
deleted when the session ends (Playwright stops it with SIGTERM so the cleanup
runs).

## Commands

```sh
angreal dev screenshots      # full capture set, fully orchestrated, one command
angreal dev screenshots --themes light,dark,hyper   # the same, in each theme
angreal dev screenshots-diff --before <run>         # diff the latest run against <run>
npm run screenshots          # same as the first, from this directory (timestamped run dir)
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
- `npm run screenshots` writes `screens.ts × viewports × themes` to
  `playwright/screenshots/runs/<timestamp>/` as
  `<screen>--<viewport>--<theme>.png`. The `THEMES` environment variable
  selects the themes (comma-separated, default `light`). The harness sets the
  theme in `localStorage` before the app starts.
- The DM map and the player display always use the dark theme. Their screens
  have `fixedTheme: true` and are captured once, as `<screen>--<viewport>.png`.
- Screenshot output is gitignored.

## Visual diff

To show that a change has no visible effect, capture a baseline before the
change and a second run after it, in the same themes. Then compare them:

```sh
angreal dev screenshots --themes light,dark,hyper          # before: note the run dir
# ...make the change...
angreal dev screenshots --themes light,dark,hyper          # after
angreal dev screenshots-diff --before <before-run>         # after = the latest run
```

The diff (`playwright/diff.mjs`) prints the changed-pixel count of each image
and writes a diff image of each changed one to `<after-run>/diff/`. Images that
are new, missing or of a different size are also reported. Add `--fail` to
exit 1 when something changed. The fixture database is seeded fresh for each
run, so two runs of the same code give the same images.

## Known harness limits

- Plugin IPC is stubbed by the shim. Events are relayed between the harness
  pages of one browser (a DM page and a player-display page talk as the app's
  windows do), but events emitted by the Rust side never arrive. Dialogs are
  unavailable except **save**, which becomes a browser download; shell-open is
  rejected. Flows depending on those are harness-unsupported.
- Map images: the bridge serves backend file paths at `GET /file?path=…`
  (only files inside the scratch dir); the shim's `convertFileSrc` points
  there, so fixture maps render with their images.
- Fixture coverage: monsters outside the SRD (Bugbear, Adult Amethyst Dragon)
  are on the roster without a catalog statblock. The Spells tab lists the
  class spell list in collapsed level groups.
- Captures are viewport-tall: the app scrolls inside its own container, so
  `fullPage` does not reach content below the fold.
- The catalog/reference reader and DM map are separate window entries — load
  them as `/sources.html` and `/dm-map.html?moduleId=…&campaignId=…`.
