# Web App Architecture

The Mimir web app is the crate `crates/mimir-web`: a [Leptos](https://leptos.dev/) 0.8 app with client-side rendering (CSR), compiled to WebAssembly with [trunk](https://trunkrs.dev/). `mimir-server` serves it and its API from the same origin.

## Technology Stack

| Part | Choice |
|------|--------|
| Framework | Leptos 0.8, `csr` feature, `leptos_router` |
| Language | Rust, target `wasm32-unknown-unknown` |
| Design system | Aurora (`colliery-io-aurora`, imported as `aurora_leptos`): components, layout frame, tokens, the light, dark and hyper themes |
| HTTP | `gloo-net`, same-origin `/api` |
| Wire types | `mimir-wire`, shared with the server |
| Catalog HTML | `mimir-render`, shared with the server |
| Markdown view | `pulldown-cmark`, raw HTML escaped |
| Rich editor | TipTap 3 in a small JavaScript bundle (`editor/`), driven through wasm-bindgen |
| Build | trunk (`Trunk.toml`, `index.html`) |
| End-to-end tests | Playwright (`web-e2e/`) |

## Two Targets

The crate has a library and a binary:

- The **library** holds the whole app. `cargo clippy --workspace` and `cargo test --workspace` check it on the host, with no wasm tools.
- The **binary** `mimir-web` is what trunk compiles to wasm. It mounts `App`.

Keep logic that can run on the host in plain functions with `#[cfg(test)]` tests: routing, vision, the rules of the wizard and level-up, PDF option queries. CI also runs `cargo clippy -p mimir-web --target wasm32-unknown-unknown -- -D warnings`.

## Modules

`crates/mimir-web/src/`:

| Module | Contents |
|--------|----------|
| `app.rs` | The root component. It loads `/api/config`, passes the sign-in gate, then shows the player page (a player) or the shell with the router (the DM). The routes and the side navigation are here. |
| `auth.rs` | The token in `localStorage` (`mimir-dm-token`), the gate decision, `/play/<token>` links |
| `api.rs` | A thin client over `/api` and `/api/v1`. Each call sends the stored token as a bearer token. Errors become Aurora's `ApiError`. Downloads (PDFs, archives) and image blobs. |
| `live.rs` | The live socket `/ws?access_token=…`. It watches a campaign, hands each `ServerMsg` to a callback and reconnects (1 s up to 15 s). |
| `pages/` | One module per page: `home`, `campaign` (the dashboard tabs), `module`, `characters`, `player`, `sign_in`, `not_found` |
| `authoring/` | DM writing: campaign and module dialogs, the document workspace, map upload, module monsters, homebrew (list, editor, item form), the character wizard and its rules |
| `character/` | The character sheet (`sheet`), stats, inventory, level-up, notes, party |
| `map/` | The DM map (`dm`), the initiative tracker (`tracker`), the player display (`display`), pan and zoom (`viewport`), sight and light (`vision`) |
| `reference/` | The reference page: catalog browser, book reader, the detail drawer |
| `admin/` | Settings, the PDF dialog (`pdf`), the source books dialog (`sources`) |
| `editor.rs` | The `RichEditor` component over `window.MimirEditor` |
| `markdown.rs` | Safe markdown to HTML for document views |
| `components.rs` | Small pieces that pages share |

## Routes

| Path | Component | Frame |
|------|-----------|-------|
| `/` | `Home` | Shell |
| `/campaigns/:id` (+ `modules`, `npcs`, `pcs`, `homebrew`) | `CampaignDashboard` with tab outlets | Shell |
| `/modules/:id` | `ModulePage` (`?section=`) | Shell |
| `/maps/:id` | `DmMapPage` | Shell |
| `/characters`, `/characters/:id` | `CharactersPage`, `CharacterPage` | Shell |
| `/reference` | `ReferencePage` (`?mode=`) | Shell |
| `/settings` | `SettingsPage` (`?section=`) | Shell |
| `/display/:campaign` | `DisplayPage` | None (bare) |
| anything else | `NotFound` | Shell |

A player session never reaches the router: `App` shows `PlayerPage` with the tabs Display, Character and Party.

The shell is Aurora's `AppShell`: the brand, the header (theme switch, **Sign out**), the side navigation and the routes. The catalog detail drawer (`Reference::provide`) is available on every shell page.

## State and Data

- **Resources.** Pages load data with `LocalResource` over `api::…` calls. A `version` signal re-runs a resource after a change.
- **Server truth.** Pages send a change, then read the result. The rules live on the server (for example `CombatService`); the tracker only sends changes and shows the answer.
- **Live updates.** A page that shows shared state opens the live socket. A `ServerMsg` names what changed (for example the tokens of a map); the page reads that part again. A player gets the new player view directly.
- **Address as state.** Tabs and selections are in the query (`?section=`, `?doc=`, `?kind=`, `?hb=`, `?mode=`, `?at=`), so links and the back button work.

## Aurora and Styling

Use Aurora components (`Button`, `Select`, `Switch`, `Tabs`, `Table`, `Modal`, `ConfirmDialog`, `Drawer`, `PageHeader`, `ErrorState`, …) and Aurora tokens. `style/mimir.css` holds only the classes Aurora does not have, built on Aurora tokens. Aurora injects its CSS at runtime (`<AuroraStyles hyper=true />`).

`index.html` carries Aurora's theme script inline, so the stored theme applies before the first paint. A test in `src/lib.rs` keeps it equal to `aurora_leptos::theme::THEME_INIT_SCRIPT`.

## Trunk

`Trunk.toml`:

- builds `index.html` into `dist/`, with `public_url = "/"`;
- the Rust binary with `data-wasm-opt="z"` in release;
- a `pre_build` hook runs `sh editor/build.sh` (the editor bundle);
- `trunk watch` ignores `dist`, `web-e2e` and the editor's `node_modules` and `dist`.

`trunk serve` is not the dev flow: the app needs the same-origin API. Use `angreal web serve`, which runs `mimir-server` with `MIMIR_WEB_DIST=crates/mimir-web/dist` and `trunk watch`.

| Command | Effect |
|---------|--------|
| `angreal web build` | Debug bundle into `crates/mimir-web/dist` |
| `angreal web build --release` | Release bundle (what ships) |
| `angreal web serve` | Server on fixture data + rebuild on change |

For release, `cargo build --release -p mimir-server --features embed-web` bakes `dist/` into the server binary.

## The Editor Bundle

`crates/mimir-web/editor` is TipTap 3 without a framework:

- `src/index.js` sets `window.MimirEditor`: a `Handle` that takes an element, markdown, a placeholder, a label and a change callback.
- Extensions: StarterKit, Placeholder, Markdown, tables, task lists.
- `build.sh` runs `npm ci` when `node_modules` is missing or older than the lockfile, then esbuild. The output is `editor/dist/mimir-editor.js`. `index.html` loads it before the app.
- `src/editor.rs` binds the handle with wasm-bindgen. The toolbar (headings, bold, lists, tables, undo) is Leptos. Without the bundle, the editor falls back to a plain text area.

Markdown is the stored format. TipTap does not write the bytes it reads; the bundle fixes the known cases (`&amp;`, task lists, HTML comments, trailing blank lines). `web-e2e/specs/editor.spec.ts` checks that the fixture documents keep their structure and words, and that a second save changes nothing.

Node is needed only for this bundle and for the e2e harness.

## Map Rendering

The DM map and the player display draw one SVG scene in map pixels under a pan and zoom transform (`map/viewport.rs`). Pointer events drive pan, pinch and token drag, so mouse, touch and pen work.

`map/vision.rs` holds pure functions shared by both: walls and doors in pixels, light zones, token vision by light level, visibility polygons, the party's sight. They are tested on the host.

The DM map computes the fog shade from all tokens. The player display draws only the player view the server sends; it computes the sight from that view and the map geometry.

## End-to-End Harness

`crates/mimir-web/web-e2e`:

| File | Purpose |
|------|---------|
| `playwright.config.ts` | Playwright config; starts `session.sh` as the web server |
| `session.sh` | Runs `mimir-server` on a new `mktemp` data directory with `MIMIR_SEED=fixture`, in token mode (`e2e-dm-token`), serving `../dist`. Removes the directory on exit. |
| `specs/session.ts` | The token, the fixture names, a `test` that fails on page or console errors |
| `specs/*.spec.ts` | The suites: smoke, map, display, combat, players, characters, party, editor, authoring, characters-dm, reference, homebrew, settings |
| `specs/capture.spec.ts` | Screenshot capture at desktop (1280×800) and tablet (820×1100), per theme |
| `diff.mjs` | Pixel diff of two capture runs |

| Command | Effect |
|---------|--------|
| `angreal web e2e` | Build, then run the smoke suite (CI runs the same list) |
| `angreal web screenshots [--themes light,dark,hyper]` | Capture into `screenshots/runs/<timestamp>/` |
| `angreal web screenshots-diff --before <run> [--after <run>] [--fail]` | Compare two runs; diff images in `<after>/diff` |

Specs find controls by role and accessible name, so give every control a label, an `aria-label` or a `title`.

## Error Handling

API errors arrive as `mimir_wire::ErrorBody` with an error code. `api.rs` turns them into Aurora's `ApiError`. Pages show them with `ErrorState` (with **Retry** where it helps) or an `Alert`. A 401 on a stored token sends the app back to the sign-in page.
