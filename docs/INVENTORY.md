# Mimir Documentation Inventory

The surfaces of Mimir 1.0 (the web app) that the documentation must cover, with where each lives in the code. A surface is covered by a doc page or deferred with a reason.

## 1. Entrypoints & top-level structure

| Surface | Location |
|---|---|
| Cargo workspace (8 crates: mimir-core, mimir-mapgen, mimir-mcp, mimir-print, mimir-render, mimir-server, mimir-web, mimir-wire) | `Cargo.toml` |
| Server binary `mimir-server` | `crates/mimir-server/src/main.rs` |
| Router: `/healthz`, `/readyz`, `/api/config`, `/api/v1`, `/ws`, `/mcp`, SPA fallback | `crates/mimir-server/src/lib.rs` (`router`) |
| Web app (Leptos CSR, trunk) | `crates/mimir-web` (`src/app.rs`, `index.html`, `Trunk.toml`) |
| Editor bundle (TipTap, esbuild) | `crates/mimir-web/editor` |
| e2e harness (Playwright) | `crates/mimir-web/web-e2e` |
| mapgen CLI binary | `crates/mimir-mapgen/src/main.rs` |
| READMEs: root `README.md`, `DEVELOPMENT.md`, `CONTRIBUTING.md`, plugin `crates/mimir-mcp/plugin/README.md`, `crates/mimir-web/editor/README.md`, `crates/mimir-web/web-e2e/README.md` | — |
| Docs site: mdBook at `docs/` (theme navy, mermaid, search) | `docs/book.toml` |

Docs: [Architecture Overview](src/explanation/architecture-overview.md), [Architecture](src/developer/ARCHITECTURE.md), [Web App Architecture](src/developer/reference/frontend/README.md).

## 2. User-facing surfaces

### 2.1 Web app screens

Routes in `crates/mimir-web/src/app.rs`; the player page in `src/pages/player.rs`.

| Screen | Route | Doc |
|---|---|---|
| Sign-in | before the shell | `reference/ui/home-screen.md` |
| Campaigns | `/` | `reference/ui/home-screen.md` |
| Campaign dashboard (Campaign, Modules, NPCs, PCs, Homebrew) | `/campaigns/:id[/modules|npcs|pcs|homebrew]` | `reference/ui/campaign-dashboard.md` |
| Module page (Documents, NPCs, Maps, Dangers, Player notes) | `/modules/:id?section=` | `reference/ui/module-page.md` |
| DM map + initiative tracker | `/maps/:id` | `reference/ui/dm-map.md` |
| Player display | `/display/:campaign` | `reference/ui/player-display.md` |
| Player page (Display, Character, Party) | `/play/<token>` → `/player` | `reference/ui/player-page.md` |
| Characters, character sheet | `/characters`, `/characters/:id` | `reference/ui/characters.md` |
| Reference (Catalog, Books) | `/reference?mode=` | `reference/ui/reference-page.md` |
| Settings (Campaigns, Books, Integrations, About) | `/settings?section=` | `reference/ui/settings.md` |

Themes: light, dark, hyper (Aurora; stored by the browser). No app-wide keyboard shortcuts; editor shortcuts from TipTap StarterKit (`reference/keyboard-shortcuts.md`).

### 2.2 REST API

96 paths under `/api/v1`, each with its access (`Dm` or `Player`), in `crates/mimir-server/src/api/mod.rs` (`routes()`). Wire types in `crates/mimir-wire`. Internal to the web app; documented where a user or an integrator meets it: sources (`reference/sources/README.md`), PDF (`reference/pdf-export/README.md`), fog areas (`reference/data-model.md`). No full public API reference (deferred: the web app is the client).

### 2.3 Auth

DM bearer token (`MIMIR_API_TOKEN`), open mode without it, player links (hashed, per character) — `crates/mimir-server/src/auth.rs`, ADR MIMIR-A-0010. Docs: `how-to/run-server.md`, `how-to/characters/player-links.md`, `explanation/architecture-overview.md`.

### 2.4 Live socket

`/ws` (`crates/mimir-server/src/live.rs`, `crates/mimir-web/src/live.rs`): change notices to the DM, the player view to players, display state in memory. Docs: `explanation/screens-and-devices.md`.

### 2.5 MCP

Streamable HTTP at `/mcp`, DM only (`crates/mimir-server/src/mcp.rs`), over the tool registry in `crates/mimir-mcp/src/registry.rs` (tool families in `src/tools/`). Docs: `reference/mcp-server.md`, `how-to/ai-assistant/README.md`.

### 2.6 Claude Code plugin (`crates/mimir-mcp/plugin`)

- `.claude-plugin/plugin.json` (name mimir-dm)
- `.mcp.json`: HTTP server at `${MIMIR_URL}/mcp` with `Authorization: Bearer ${MIMIR_API_TOKEN}`
- Slash commands (5): /mimir-campaigns, /create-module, /search-monsters, /search-spells, /generate-map
- Skills (8): mimir-dm, encounter-balance, npc-network, loot-audit, continuity-check, session-prep, pressure-test, mapgen

### 2.7 mapgen CLI

`crates/mimir-mapgen/src/main.rs`: `generate`, `validate`, `list-presets`; 12 presets; YAML config in `src/pipeline.rs`; output `.dungeondraft_map`. Docs: `reference/mapgen.md`, `how-to/maps/generate-map.md`, `how-to/maps/mapgen-standalone.md`.

### 2.8 Environment variables

| Var | Default / behavior | Source |
|---|---|---|
| MIMIR_BIND | `127.0.0.1:8740` | `crates/mimir-server/src/config.rs` |
| MIMIR_DATA_DIR | `./mimir-data`; `data/mimir.db`, `assets/` | `config.rs` |
| MIMIR_API_TOKEN | unset or blank → open mode (warning logged) | `config.rs`, `main.rs` |
| MIMIR_WEB_DIST | unset → embedded app (`embed-web`) or placeholder | `config.rs`, `web.rs` |
| MIMIR_SEED | `fixture` → SRD + test campaign (needs `fixtures` feature) | `config.rs`, `lib.rs` |
| MIMIR_SEED_ASSETS | seed asset dir, else found from parent dirs | `crates/mimir-core/src/seed/dev.rs` |
| RUST_LOG | `info` | `main.rs` |
| MIMIR_URL (plugin) | `http://127.0.0.1:8740` | `plugin/.mcp.json` |

Docs: `reference/environment-variables.md`.

## 3. Data model (mimir-core)

Schema: `crates/mimir-core/src/schema.rs`; migrations in `crates/mimir-core/migrations/` (latest: 030 combat, 031 player links, 032 player notes and sheet shares). Docs: `reference/data-model.md`.

## 4. Main workflows

| Workflow | Code | Docs |
|---|---|---|
| Campaign creation with 11 template documents | `services/campaign.rs`, `templates/` | `how-to/campaigns/create-campaign.md` |
| Module creation (6 types) with overview and Play Notes | `services/module.rs` | `how-to/modules/create-module.md` |
| Documents with the rich editor, autosave, reorder | `src/authoring/documents.rs`, `src/editor.rs` | `how-to/campaigns/manage-documents.md` |
| Character wizard (PC at level 1, NPC form), level-up | `src/authoring/wizard*.rs`, `src/character/levelup.rs`, `services/character/` | `how-to/characters/*` |
| Catalog import (data and images) as a background job | `api/admin.rs`, `jobs.rs`, `mimir-core/src/import/` | `how-to/campaigns/manage-sources.md` |
| Campaign archive export/import | `services/archive.rs`, `api/admin.rs` | `how-to/campaigns/export-campaign.md`, `reference/campaign-archive-format.md` |
| Map upload (UVTT or image with grid), tokens, lights, traps, POIs, fog | `src/authoring/maps.rs`, `src/map/dm.rs`, `api/maps.rs` | `how-to/maps/*`, `how-to/play-mode/*` |
| Player display and player view | `src/map/display.rs`, `api/maps.rs` (`build_player_view`), `live.rs` | `how-to/play-mode/use-player-display.md` |
| Combat (initiative tracker) | `src/map/tracker.rs`, `api/combat.rs`, `services/combat.rs` | `how-to/play-mode/manage-encounters.md` |
| Player links, notes, party sharing | `api/links.rs`, `api/party.rs`, `src/character/{notes,party}.rs` | `how-to/characters/player-links.md` |
| Homebrew items, monsters, spells | `src/authoring/homebrew*.rs`, `api/homebrew.rs` | `how-to/homebrew/*` |
| PDF export via Typst | `crates/mimir-server/src/print/`, `crates/mimir-print` | `reference/pdf-export/README.md` |
| Mapgen | `crates/mimir-mapgen` | `how-to/maps/generate-map.md` |
| AI workflow: Claude Code → `/mcp` → same database | `mcp.rs`, `plugin/` | `how-to/ai-assistant/README.md` |

## 5. Build / test / deploy lifecycle

- Angreal tasks: `web build [--release]`, `web serve [--port --token --fresh]`, `web e2e`, `web screenshots [--themes]`, `web screenshots-diff --before [--after --fail]` (`.angreal/task_web.py`); `test unit`, `test coverage [--open]` (`.angreal/task_test.py`); `docs build/serve/watch/check/clean/init/production/test` (`.angreal/task_docs.py`)
- Release binary: `angreal web build --release`, then `cargo build --release -p mimir-server --features embed-web`
- CI (`.github/workflows/ci.yml`): fmt; clippy host and wasm (Rust 1.93.0); `angreal test unit`; wasm build of `mimir-wire`, `trunk build --release`, Playwright smoke suite; SRD smoke test (pushes to main only); coverage (tarpaulin, does not fail CI)
- Release (`.github/workflows/release.yml`): tag `v*.*.*` → `mimir-mapgen` binaries per platform → draft GitHub release. The workflow does not build the server.
- Docs deploy (`.github/workflows/docs.yml`): mdBook build → GitHub Pages
- Tests: Rust `--test-threads=1` (SQLite locking); `mimir-server/tests/` for API, auth, live, MCP; host tests of web logic; Playwright specs in `web-e2e/specs/`

Docs: `developer/DEVELOPMENT.md`, `developer/CONTRIBUTING.md`.

## 6. Dependencies & integration points

- axum 0.8, tokio, rmcp (streamable HTTP), Diesel/SQLite (bundled), Typst, rust-embed (`embed-web`)
- Leptos 0.8 CSR, leptos_router, Aurora (`colliery-io-aurora`), gloo-net, wasm-bindgen, web-sys, qrcode
- TipTap 3 + esbuild (editor bundle), Playwright (e2e) — Node.js only for these two
- 5etools data and image archives (catalog source); Dungeondraft (UVTT export)
- Claude Code (plugin host); GitHub Releases (mapgen binaries); GitHub Pages (docs)

## 7. Implicit knowledge

1. `angreal web serve` uses fixture data in `target/web-dev`; it never touches a real data directory. The e2e harness uses a new temporary directory each run.
2. Open mode (no `MIMIR_API_TOKEN`) makes every caller the DM. Player links still work.
3. The display state (shown map, blackout, turn order on display) and import jobs are in memory; a restart clears them.
4. The Line of sight switch, the light control and door clicks on the DM map change the DM's view only.
5. Rust tests run with `--test-threads=1`.
6. Image-processing crates are built at opt-level 3 in the dev profile (`Cargo.toml`), else map work is 10–50× slower.
7. `cargo build --workspace` needs no wasm tools; the web app's library builds on the host.
8. The trunk pre-build hook needs Node (`editor/build.sh`).
9. Most MCP tools need `set_active_campaign` first; the active campaign belongs to the MCP session.
10. The SRD smoke test runs only on pushes to `main`.
