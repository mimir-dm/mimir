# Development Setup and Guide

How to build, run and test Mimir: the server (`mimir-server`) and the web app (`mimir-web`).

## Prerequisites

| Tool | Use |
|------|-----|
| Rust (via [rustup](https://rustup.rs/)) | Everything. CI pins Rust 1.93.0 for fmt and clippy. |
| `wasm32-unknown-unknown` target | The web app. `angreal web build` adds it. |
| [trunk](https://trunkrs.dev/) | Builds the web app: `cargo install trunk --locked` |
| Node.js LTS and npm | Only for the editor bundle (a trunk pre-build hook) and the Playwright e2e harness |
| [angreal](https://github.com/angreal/angreal) | The task runner: `pip install 'angreal>=2'` |
| [mdBook](https://rust-lang.github.io/mdBook/) and `mdbook-mermaid` | Only for the docs: `cargo install mdbook mdbook-mermaid` |

No system libraries are needed beyond a C toolchain: SQLite is bundled.

## First Build

```bash
git clone https://github.com/mimir-dm/mimir.git
cd mimir
cargo build --workspace
```

`cargo build --workspace` builds the server and checks the web app as a host library. It needs no wasm tools.

## Run the App in Development

```bash
angreal web serve
```

This command:

1. builds the web app with trunk (debug),
2. starts `mimir-server` with the `fixtures` feature, `MIMIR_SEED=fixture` and the data in `target/web-dev`,
3. runs `trunk watch`, so a code change rebuilds the bundle.

Open <http://127.0.0.1:8740>. Reload the page after a rebuild. Ctrl-C stops both.

The dev data is a fixture: the SRD catalog and the campaign "The Lost Mine of Phandelver". It never touches a real data directory.

| Option | Effect |
|--------|--------|
| `--port 8750` | Another port |
| `--token dev-token` | Token mode, to test the sign-in page. Without it the server is in open mode. |
| `--fresh` | Delete `target/web-dev` first |

To run the server alone on a bundle that you built:

```bash
angreal web build
MIMIR_WEB_DIST=crates/mimir-web/dist MIMIR_DATA_DIR=target/web-dev \
  cargo run -p mimir-server
```

## Release Build

```bash
angreal web build --release
cargo build --release -p mimir-server --features embed-web
```

`embed-web` bakes `crates/mimir-web/dist` into the binary. The result is one file: `target/release/mimir-server`. See [Run the Server](../how-to/run-server.md) and [Environment Variables](../reference/environment-variables.md).

## Gates

CI (`.github/workflows/ci.yml`) runs these. Run them before a pull request.

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo clippy -p mimir-web --target wasm32-unknown-unknown -- -D warnings
angreal test unit
angreal web e2e
```

| Gate | What it checks |
|------|----------------|
| `cargo fmt --all --check` | Formatting |
| `cargo clippy --workspace --all-targets -- -D warnings` | Lints on the host. Warnings are errors. |
| `cargo clippy -p mimir-web --target wasm32-unknown-unknown -- -D warnings` | Lints of the web app as wasm |
| `angreal test unit` | `cargo test --workspace -- --test-threads=1`. One thread avoids SQLite lock errors. |
| `angreal web e2e` | The Playwright smoke suite against a real server |

If a clippy lint does not fit, add `#[allow(clippy::…)]` with a comment that gives the reason.

CI also builds `mimir-wire` for wasm, builds the web app with `trunk build --release`, and, on pushes to `main`, runs the heavy SRD smoke test (`cargo test -p mimir-core --test srd_smoke_test`) and coverage.

## Tests

### Rust

```bash
angreal test unit                         # the whole workspace
cargo test -p mimir-core                  # one crate
cargo test -p mimir-server --test api_maps -- --test-threads=1   # one test file
cargo test -p mimir-core some_test_name   # by name
angreal test coverage --open              # cargo-tarpaulin (tarpaulin.toml)
```

| Where | What |
|-------|------|
| `crates/mimir-core/src/**` (`#[cfg(test)]`) | Unit tests of models, services, the DAL |
| `crates/mimir-core/tests/` | Catalog import, SRD smoke test |
| `crates/mimir-server/tests/` | The API, auth, player links, the live socket and MCP over HTTP, each against an in-process server and a temporary database |
| `crates/mimir-web/src/**` (`#[cfg(test)]`) | Pure logic of the web app: routing, vision, rules of the wizard and level-up. They run on the host. |
| `crates/mimir-mapgen/tests/` | CLI, format round trip, polygon snapshots |

### Web (Playwright)

```bash
angreal web e2e
```

It builds the bundle, installs the harness (`npm ci` in `crates/mimir-web/web-e2e` the first time) and runs the specs. Playwright starts `web-e2e/session.sh`. The script runs `mimir-server` on a new scratch data directory with the UI fixture, in token mode, and removes the directory after. Results of a failed run are in `crates/mimir-web/web-e2e/results`.

To install the browser the first time:

```bash
cd crates/mimir-web/web-e2e && npx playwright install chromium
```

### Screenshots

```bash
angreal web screenshots                         # light and dark themes
angreal web screenshots --themes light,dark,hyper
angreal web screenshots-diff --before <run>     # compare with the latest run
```

Each screen is captured at desktop (1280×800) and tablet (820×1100) width, in `crates/mimir-web/web-e2e/screenshots/runs/<timestamp>/`. `screenshots-diff` writes diff images to `<after>/diff`; `--fail` exits 1 on any change.

## Project Structure

```
crates/
├── mimir-server/   # axum server: API (/api/v1), auth, live socket (/ws), MCP (/mcp), serves the web app
├── mimir-web/      # Leptos CSR web app (trunk)
│   ├── src/        # app.rs (router), pages/, map/, character/, authoring/, reference/, admin/
│   ├── style/      # mimir.css: the few classes Aurora does not have
│   ├── editor/     # TipTap bundle (Node, esbuild) → editor/dist/mimir-editor.js
│   ├── web-e2e/    # Playwright specs and screenshot capture
│   ├── index.html  # trunk entry
│   └── Trunk.toml
├── mimir-wire/     # JSON types shared by server and web app (builds for wasm)
├── mimir-core/     # models, services, DAL, migrations, catalog import, seed fixture
├── mimir-mcp/      # MCP tool registry; plugin/ is the Claude Code plugin
├── mimir-print/    # PDF generation with Typst
├── mimir-render/   # 5etools JSON → HTML (catalog, reader, homebrew preview)
└── mimir-mapgen/   # procedural map generator (library and CLI)
docs/               # this book (mdBook)
```

See [Architecture](./ARCHITECTURE.md) and [Web App Architecture](./reference/frontend/README.md).

## Common Tasks

### Add an API Route

1. Add the request and answer types to `crates/mimir-wire` (serde only).
2. Write the handler in `crates/mimir-server/src/api/<area>.rs`. Call `mimir-core` services, never the DAL.
3. Add the route to `routes()` in `crates/mimir-server/src/api/mod.rs` with its `Access`: `Dm`, or `Player` (the handler then limits a player to their scope).
4. If the change matters to other clients, publish a change on the live hub (`crates/mimir-server/src/live.rs`).
5. Add a test in `crates/mimir-server/tests/`.
6. Call it from the web app through `crates/mimir-web/src/api.rs`.

### Add a Page

1. Add a component under `crates/mimir-web/src/pages/` (or the area module).
2. Add its `<Route>` in `crates/mimir-web/src/app.rs`. Add a side navigation link if it is a top-level page.
3. Use Aurora components; add a class to `style/mimir.css` only when Aurora has none.
4. Add an e2e spec or extend one in `crates/mimir-web/web-e2e/specs/`, and add it to the list in `.angreal/task_web.py` and `ci.yml`.

### Add a Database Migration

1. Add a directory `crates/mimir-core/migrations/NNN_name/` with `up.sql` and `down.sql`.
2. Update `crates/mimir-core/src/schema.rs` and the models.
3. The server runs migrations at start (`mimir_core::db::init_database`).

## Debugging

### Server

```bash
RUST_LOG=debug angreal web serve
RUST_LOG=info,mimir_core=debug cargo run -p mimir-server
```

The server logs to standard output.

### Web App

Use the browser developer tools. A panic in the app shows in the console (`console_error_panic_hook`). The network tab shows each `/api/v1` call and the `/ws` socket.

### Database

The dev database is `target/web-dev/data/mimir.db`:

```bash
sqlite3 target/web-dev/data/mimir.db
.tables
.schema campaigns
```

`angreal web serve --fresh` starts again from the fixture.

## Documentation

```bash
angreal docs serve        # http://localhost:3000, live reload
angreal docs build
angreal docs check        # build and report broken links
```

The docs deploy from `main` to the GitHub Pages site (`.github/workflows/docs.yml`).

## Getting Help

- Search the [GitHub issues](https://github.com/mimir-dm/mimir/issues).
- Ask in [GitHub Discussions](https://github.com/mimir-dm/mimir/discussions).
