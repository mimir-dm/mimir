# Mimir Documentation Inventory (Phase 1 — Discovery)

Generated 2026-06-09 against version 0.7.2 (commit 6b356db). Every surface listed
here must be covered by a doc or explicitly deferred with a reason. File:line
citations point at the source of truth.

## 1. Entrypoints & top-level structure

| Surface | Location |
|---|---|
| Cargo workspace (5 crates: mimir, mimir-core, mimir-mapgen, mimir-mcp, mimir-print) | `Cargo.toml:1-9` |
| Tauri desktop app entry | `crates/mimir/src/main.rs` |
| MCP server binary (stdio transport) | `crates/mimir-mcp/src/main.rs:16-79` |
| mapgen CLI binary | `crates/mimir-mapgen/src/main.rs:1-162` |
| mimir-print (library only, no binary) | `crates/mimir-print/src/lib.rs` |
| Frontend: 4 HTML entry points (index, sources, player-display, dm-map) | `crates/mimir/frontend/vite.config.ts:18-23` |
| READMEs: root `README.md`, `DEVELOPMENT.md`, `CONTRIBUTING.md`, plugin `crates/mimir-mcp/plugin/README.md` | — |
| Docs site: mdBook at `docs/` (theme navy, mermaid, search) | `docs/book.toml` |

## 2. User-facing surfaces

### 2.1 Tauri commands (in-app API) — 295 commands

Registered in `crates/mimir/src/main.rs:80-436` via `tauri::generate_handler!`.
All return `ApiResponse<T>` (`crates/mimir/src/commands/mod.rs:26-56`).
Domains (counts): campaigns (12), archive export/import (3), homebrew items/monsters/spells (16),
modules incl. tokens (21), characters incl. inventory/spells/sources (25), documents (9),
maps incl. UVTT/light/fog/traps/POIs (44), assets (6), catalog search across 23 entity
groups + level-up helpers (122), sources & book content (8), player display window (7),
DM map window (4), app info & dev tools (6), print/PDF export (12).
These are internal app plumbing — documented indirectly through UI reference/how-tos,
not as a public API (deferred: not a user-callable surface).

### 2.2 MCP tools — 54 tools (user-facing via Claude)

Registered in `crates/mimir-mcp/src/handler.rs:42-107`. Tool files in
`crates/mimir-mcp/src/tools/{campaign,module,document,character,map,homebrew,catalog,mapgen}.rs`.
- Campaign (10): list_campaigns, set_active_campaign, get_campaign_details, get_campaign_sources, create_campaign, update_campaign, delete_campaign, export_campaign, import_campaign, preview_archive
- Module (8): create_module, list_modules, get_module_details, update_module, delete_module, add_monster_to_module, remove_monster_from_module, add_item_to_module
- Document (6): list_documents, read_document, create_document, edit_document, delete_document, reorder_document
- Character (13): list_characters, get_character, create_character, edit_character, delete_character, level_up_character, get_character_inventory, add_item_to_character, remove_item_from_character, update_character_inventory, add_character_spell, remove_character_spell, list_character_spells
- Map (8): create_map, list_maps, get_map, update_map, delete_map, add_token_to_map, list_tokens_on_map, remove_token
- Homebrew (5): list_homebrew, get_homebrew, create_homebrew, update_homebrew, delete_homebrew
- Mapgen (3): generate_map, list_map_presets, validate_map_config
- Catalog (1): search_catalog (`crates/mimir-mcp/src/tools/catalog.rs:39`)
- Active-campaign requirement: most tools error with NoActiveCampaign until `set_active_campaign` (in-memory `McpContext.active_campaign_id`, `crates/mimir-mcp/src/context.rs`)

### 2.3 Claude Code plugin (crates/mimir-mcp/plugin)

- `plugin.json` (name mimir-dm, v0.7.2): `.claude-plugin/plugin.json`
- `.mcp.json`: command `mimir-mcp`, env MIMIR_DATABASE_PATH
- Slash commands (5): /mimir-campaigns, /create-module, /search-monsters, /search-spells, /generate-map (`plugin/commands/*.md`)
- Skills (8): mimir-dm, encounter-balance, npc-network, loot-audit, continuity-check, session-prep, pressure-test, mapgen (`plugin/skills/*/SKILL.md`)

### 2.4 mapgen CLI

`crates/mimir-mapgen/src/main.rs:4-45` (clap):
- `generate [CONFIG] -o/--output <path=output.dungeondraft_map> -s/--seed <u64> -p/--preset <name>`
- `validate <CONFIG>`
- `list-presets`
- 12 presets: forest, grassland, cave, desert, lake, ice_lake, arctic, island_tropical, island_forest, island_arctic, swamp, forest_river
- YAML MapConfig: `crates/mimir-mapgen/src/pipeline.rs:24-95`; validation `pipeline.rs:150-204`; output format `.dungeondraft_map` JSON (`src/format/mod.rs:22-55`)
- Examples: `crates/mimir-mapgen/examples/`

### 2.5 Environment variables

| Var | Default / behavior | Source |
|---|---|---|
| MIMIR_DEV | unset; if set forces dev mode (dev/ data subdir) | `crates/mimir/src/state.rs:21` |
| RUST_LOG | "mimir=info,mimir_core=info" | `crates/mimir/src/main.rs:41-43` |
| MIMIR_DATABASE_PATH | unset → platform default DB path; supports `~/`/`$HOME/` expansion; empty/template values ignored | `crates/mimir-mcp/src/context.rs:29-103` |
| MIMIR_SEED_ASSETS | unset → walks parents for seed assets dir (dev seeding only) | `crates/mimir-core/src/seed/dev.rs:890` |
| HOME / APPDATA / XDG_DATA_HOME | platform path construction | `crates/mimir/src/main.rs:21-29`, `crates/mimir-mcp/src/context.rs:73-97` |
| TAURI_ENV_TARGET_TRIPLE | build-time, sidecar target selection | `scripts/build-sidecar.sh` |

### 2.6 Config / storage locations

- DB paths (prod vs dev `dev/` subdir; macOS/Windows/Linux table): `crates/mimir/src/state.rs:15-58`
- MCP server DB default per platform; Windows default NOT implemented (returns None): `crates/mimir-mcp/src/context.rs:88-102`
- Logs: daily-rotated `logs/mimir.log` under app data dir: `crates/mimir/src/main.rs:19-38`
- Assets dir, config dir: `crates/mimir/src/state.rs:28-109`
- Theme persistence: localStorage key `theme`, cross-window `theme-changed` event: `frontend/src/stores/theme.ts:8-94`
- SQLite WAL mode, FK on, per-call connections (no pool): `crates/mimir/src/state.rs:144-146`

### 2.7 Frontend surfaces

- Router: 15 routes (`frontend/src/app/router/index.ts`) — home, campaign create/detail/dashboard (+5 tabs), module detail, settings, characters list/sheet, player-display (the two ModulePlayView play routes were removed 2026-06-10)
- Windows: main (tauri.conf.json:14-22), player-display (`commands/player_display.rs:38`), dm-map (`commands/dm_map.rs:9`), sources (sources.html)
- Player-display IPC events: player-display:map-update / viewport-update / blackout (`commands/player_display.rs:125,141,157`)
- Keyboard shortcuts: DM map viewer `+/=`, `-`, `0`, `H`, `D`, `Escape` (`frontend/src/components/DmMapViewer.vue:1871-1908`); player display `F11`, `R` (`PlayerDisplayWindow.vue:380-391`); modal `Escape` (`shared/AppModal.vue:108-120`)
- Themes (3): light, dark, hyper (`frontend/src/assets/styles/themes/*.css`)
- Settings view sections: Theme, Integrations (Claude/MCP config generation), About, Dev Tools (dev only) (`frontend/src/views/SettingsView.vue:80-168`)
- Vision/fog system: composables `useVisionCalculation`, `useVisibilityPolygon`, `useTokenVision`, `useFog`, `useLightSources`, `useUvttMap`, `usePlayerDisplayEvents`, `usePlayerViewport` (`frontend/src/composables/map/`)
- Ambient light modes: Bright / Dim / Darkness

## 3. Data model (mimir-core)

Schema: `crates/mimir-core/src/schema.rs`; 29 migrations at `crates/mimir-core/migrations/001-029`.

- Catalog tables (26 entity types + join/support tables): catalog_sources, actions, backgrounds, books, classes, class_features, subclasses, subclass_features, conditions, cults, deities, diseases, feats, hazards, items, item_attunement_classes, languages, monsters, objects, optional_features, psionics, races, rewards, senses, skills, spells, spell_classes, spell_subclasses, tables, traps, variant_rules, vehicles
- Campaign tables: campaigns, campaign_sources, modules, documents (+documents_fts FTS5 porter), campaign_assets, maps, token_placements, light_sources, map_traps, map_pois, fog_revealed_areas
- Character tables: characters, character_sources, character_classes, character_inventory, character_spells, character_proficiencies, character_features, character_feats
- Module tables: module_monsters, module_npcs
- Homebrew tables: campaign_homebrew_items / _monsters / _spells (clone-from-catalog deep merge: `services/homebrew.rs:143-150`)

## 4. Main workflows (end to end)

| Workflow | Path through code |
|---|---|
| Campaign creation → 11 template docs auto-created | `services/campaign.rs:83`, templates `src/templates/mod.rs:96-152` |
| Module creation → type-specific overview doc (6 types: general, mystery, dungeon, heist, horror, political) + play notes | `services/module.rs:16-42,150+`, `templates/mod.rs:155-236` |
| Character create / level up (multiclass, ASI/feat, spells, features, HP method Average/Roll/Fixed) | `services/character.rs:278-365` |
| Catalog import from 5etools zip (discovery → collection → per-source transactional import + FTS) | `src/import/{discovery,collector,service}.rs`; 26 entity types `discovery.rs:178-387`; only groups core/supplement are imported (`DEFAULT_ALLOWED_GROUPS`, `import/service.rs:83`); setting/adventure/screen/homebrew skipped (note: rustdoc comment at service.rs:101 is stale on this) |
| Campaign archive export/import (.tar.gz, manifest v2.0, data.json, asset files, UUID remap on import, catalog refs via `{@type name|source}` regex) | `services/archive.rs:33-498` |
| Map upload (UVTT as campaign_asset) → grid/lighting config → tokens/light/fog/traps/POIs → play | `services/map.rs`, `commands/map/*` |
| Play mode: Play button opens the DM Map window (`ModulesTab.vue` handlePlayModule → openDmMapWindow). The former in-window ModulePlayView and its routes were removed 2026-06-10 as unreachable dead code. Player display window, viewport sync, blackout, fog/vision | `commands/{player_display,dm_map}.rs`, `frontend/src/components/{DmMapViewer,DmMapWindow,PlayerDisplayWindow}.vue` |
| PDF export via Typst: documents, character sheets, battle cards, monster/spell/equipment/trap cards, map previews, tiled maps, token cutouts | `crates/mimir-print/src/sections/*`, `service.rs:54-99`, embedded templates `embedded_templates.rs` |
| Mapgen: YAML/preset → noise/terrain/water/objects/paths/rooms/polygons/lights pipeline → .dungeondraft_map | `crates/mimir-mapgen/src/pipeline.rs`, module files per generator |
| AI workflow: Claude Code plugin → mimir-mcp (sidecar or standalone) → same DB | `plugin/`, `context.rs` |

## 5. Build / test / deploy lifecycle

- Angreal tasks (authoritative): `dev launch`, `dev reset` (`.angreal/task_dev.py:29-101`); `test unit [--watch|--core|--ui|--all]`, `test coverage [--core|--ui|--all|--open]` (`.angreal/task_test.py:25-173`); `docs build/serve/watch/check/clean/init/production/test` (`.angreal/task_docs.py`)
- `dev launch` = npm ci (if needed) + Vite dev server + build sidecar if missing + `cargo run -p mimir --no-default-features`
- Feature flag: mimir default `custom-protocol`; dev disables it (`crates/mimir/Cargo.toml:55`, `task_dev.py:94`)
- Sidecar build scripts: `scripts/build-sidecar.{sh,ps1,mjs}` → `crates/mimir/binaries/mimir-mcp-<target-triple>`; required by `tauri.conf.json:65-67` externalBin BEFORE building mimir
- Installer: `scripts/install.sh` (macOS/Linux, GitHub Releases `app-v<version>` tags, installs app + mimir-mcp CLI to ~/.local/bin)
- Fixtures: `scripts/extract-srd-fixtures.py` (SRD-only, OGL-safe)
- CI (`.github/workflows/ci.yml`): test-gui-build matrix (macOS arm/x64, Ubuntu 22.04, Windows), test-frontend (vitest), test-other-crates (`angreal test unit --core`), test-integration (SRD smoke, main pushes only), coverage (tarpaulin → artifacts)
- Release (`.github/workflows/release.yml`): tag `v*.*.*` → desktop matrix build (tauri-action, draft release `app-v<ver>`), sidecar + mapgen binaries uploaded to release
- Docs deploy (`.github/workflows/docs.yml`): mdBook build → push to external repo mimir-dm/mimir-dm.github.io via PAGES_DEPLOY_TOKEN
- Tests: Rust `--test-threads=1` everywhere (SQLite locking); tarpaulin only mimir-core, fail-under 50% (`tarpaulin.toml`); frontend vitest + jsdom, fixtures in `frontend/__tests__/`
- mimir-core integration tests: `tests/{catalog_import,srd_smoke_test}.rs`; mapgen: `tests/{cli_integration,format_roundtrip,polygon_snapshots}.rs`

## 6. Dependencies & integration points

- Tauri v2 (+plugins: dialog, fs, window perms per tauri.conf.json capabilities), Diesel/SQLite, rust-mcp-sdk 0.8 (stdio), Typst (typst + typst_pdf), noise/Perlin, clap, Vue 3 + Pinia + Vue Router + Vite + Vitest
- 5etools data tarball (catalog source); Dungeondraft (consumes .dungeondraft_map; exports UVTT)
- Claude Code (plugin host); GitHub Releases (distribution); external Pages repo (docs)

## 7. Implicit / tribal knowledge

1. Sidecar MUST exist at `crates/mimir/binaries/mimir-mcp-<triple>` before `cargo build -p mimir` (externalBin), or the build fails. `angreal dev launch` handles this.
2. Dev vs prod DB separation: dev mode = `cfg!(debug_assertions) || MIMIR_DEV` → `dev/` subdir (`state.rs:15-58`). `angreal dev reset` only ever deletes the dev DB.
3. `cargo tauri dev` from `crates/mimir` is the documented-but-suboptimal path; the supported dev loop is `angreal dev launch` (manual Vite + `cargo run -p mimir --no-default-features`). DEVELOPMENT.md/CONTRIBUTING.md still lead with `cargo tauri dev` (flagged stale).
4. Rust tests must run `--test-threads=1` (SQLite locking).
5. Image-processing deps pinned to opt-level 3 in dev profile to avoid 10-50x slowdown (`Cargo.toml:103-110`).
6. MCP server on Windows has no default DB path — MIMIR_DATABASE_PATH is mandatory there (`context.rs:99-102`).
7. Most MCP tools require `set_active_campaign` first; active campaign is in-memory per server process.
8. SRD smoke test is heavy and only runs on pushes to main, not PRs (`ci.yml:118`).
9. Dev seeding commands (seed/reseed/clear) exist only in debug builds (`commands/dev.rs`).
10. Docs deploy goes to a *different repo* (mimir-dm.github.io) — pushing docs/ to main triggers it.
