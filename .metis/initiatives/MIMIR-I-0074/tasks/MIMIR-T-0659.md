---
id: invoke-over-http-bridge-spike
level: task
title: "Invoke-over-HTTP bridge spike + transport decision"
short_code: "MIMIR-T-0659"
created_at: 2026-07-10T09:59:14.150812+00:00
updated_at: 2026-07-10T10:00:34.147020+00:00
parent: MIMIR-I-0074
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/active"


exit_criteria_met: false
initiative_id: MIMIR-I-0074
---

# Invoke-over-HTTP bridge spike + transport decision

## Parent Initiative

[[MIMIR-I-0074]] — Playwright UI harness for screenshots and UX iteration

## Objective **[REQUIRED]**

Settle the core design risk: how the real Rust backend answers `invoke()` calls
from a plain browser. Verify whether `tauri-invoke-http` supports Tauri v2; if
yes, use it; if no, build the minimal axum dispatch bridge. Prove the choice
end-to-end with one real command.

## Acceptance Criteria

## Acceptance Criteria **[REQUIRED]**

- [x] Transport decision documented here with evidence (crate version/compat check via web + a compile test — not assumption)
- [x] Feature-flagged bridge (`ui-harness` cargo feature or equivalent) in the mimir crate: OFF by default, never in release builds, binds 127.0.0.1 only
- [x] Proof: `curl` (or fetch from a browser page) invoking `list_campaigns` returns The Frost Architect from a SCRATCH COPY of the DB — never the live production file
- [x] CORS configured for the Vite dev origin (localhost:5173)
- [x] Whatever dispatch mechanism is chosen, adding a command later must be at most one line (mirror of generate_handler!) or zero lines (tauri-invoke-http)

## Implementation Notes

- Commands take `State<AppState>`; if hand-rolling the bridge, construct AppState against the scratch DB path and dispatch by command name with serde_json args (same shapes the frontend sends — camelCase per the request structs).
- The app is a GUI binary; the bridge may live in a separate dev bin target (`cargo run -p mimir --bin ui-bridge --features ui-harness`) to avoid launching a window headlessly.

## Status Updates **[REQUIRED]**

### 2026-07-10 — Transport decision + working bridge

**Decision: reject `tauri-invoke-http`, build a dev-only bin on `tauri::test` (MockRuntime + `get_ipc_response`).**

Evidence against `tauri-invoke-http`: latest release is v2.0.0-rc.1 (Sept 2024, targeting a Tauri v2 RC, not stable v2), repo has an open unresolved v2-support issue and no commits since — stale. It also reshapes the production invoke transport rather than being a dev-only side door.

Evidence for the chosen path (compile + runtime tested, not assumed): `tauri = 2.11.2` ships a `test` feature with `mock_builder()` / `mock_context()` / `get_ipc_response()`, which pushes an `InvokeRequest` through the **real `generate_handler!` pipeline** — arg deserialization, `State<AppState>` extraction, serialization all identical to the desktop app. Zero per-command glue.

**What was built (branch `feat/ui-harness`):**

- `crates/mimir/src/ipc.rs` — the full `generate_handler!` list hoisted out of `main.rs` into `mimir_lib::ipc::invoke_handler<R: Runtime>()`. Single source of truth: the Wry app and the MockRuntime bridge share it, so adding a command stays exactly one line, once. Required genericizing the 11 window commands in `player_display.rs`/`dm_map.rs` over `R: Runtime` (mechanical).
- `crates/mimir/src/bin/ui_bridge.rs` — axum server, `POST /invoke/{cmd}` (JSON args, camelCase as the frontend sends) + `GET /health`. A dedicated dispatch thread owns the headless MockRuntime app; HTTP handlers forward via channel to `get_ipc_response`. Binds 127.0.0.1 only; CORS allow-origin for `http://localhost:5173` / `http://127.0.0.1:5173`.
- Cargo: `ui-harness = ["tauri/test", "dep:axum", "dep:tower-http"]`, OFF by default; `[[bin]] ui-bridge` has `required-features = ["ui-harness"]`, so release builds never contain it.
- Safety rails (previewing T-0660): requires `MIMIR_BRIDGE_APP_DIR` (scratch app dir); REFUSES to start (exit 1, verified) if the resolved DB canonicalizes to anywhere under the production app dir `~/Library/Application Support/com.mimir.app` — including the `dev/` subdir, which is intentional: sessions run on disposable snapshots only.

**Proof run:** snapshotted prod DB via sqlite `.backup` to a scratch dir, then
`MIMIR_BRIDGE_APP_DIR=<scratch> cargo run -p mimir --features ui-harness --bin ui-bridge`:
- `curl -X POST 127.0.0.1:4175/invoke/list_campaigns -d '{}'` → `{"success":true,"data":[{"id":"cf8be92a-…","name":"The Frost Architect",…}]}`
- `get_campaign` with JSON args works; unknown command → 400; OPTIONS preflight and POST both return `access-control-allow-origin: http://localhost:5173`.

**Gotcha worth remembering:** `InvokeRequest.url` must be the platform's local webview origin (`tauri://localhost` on macOS/Linux, `http://tauri.localhost` on Windows) or Tauri's ACL treats the invoke as remote and rejects with "not allowed. Plugin not found".

**Known limits (deferred, noted for T-0661):** plugin commands (dialog/shell) aren't registered on the mock app — the browser shim should no-op them; `InvokeResponseBody::Raw` responses (e.g. `read_asset_file`) are passed through as octet-stream but untested.

`angreal test unit` green. Port configurable via `MIMIR_BRIDGE_PORT` (default 4175).