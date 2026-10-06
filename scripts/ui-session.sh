#!/usr/bin/env bash
# UI-harness session: run the invoke-over-HTTP bridge against a DISPOSABLE
# database seeded with the UI fixture (MIMIR-T-0660, MIMIR-T-0663).
#
# Development machines hold no real campaign data. Each session starts from an
# empty scratch dir; the bridge seeds it with the SRD catalog and "The Lost
# Mine of Phandelver" dev campaign (maps and their assets included), and the
# scratch dir is removed when the bridge exits. Nothing under the production
# app dir is read, and the bridge refuses to start on any path inside it.
#
# Env:
#   MIMIR_BRIDGE_PORT   bridge port (default 4175)
#
# Used as a Playwright webServer command (playwright.config.ts) and by
# `angreal dev screenshots`; also fine to run standalone.
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
PORT="${MIMIR_BRIDGE_PORT:-4175}"

SCRATCH="$(mktemp -d "${TMPDIR:-/tmp}/mimir-ui-session.XXXXXX")"
BRIDGE_PID=""
cleanup() {
  if [ -n "$BRIDGE_PID" ]; then
    kill "$BRIDGE_PID" 2>/dev/null || true
    wait "$BRIDGE_PID" 2>/dev/null || true
  fi
  rm -rf "$SCRATCH"
  echo "ui-session: scratch dir removed" >&2
}
trap cleanup EXIT INT TERM

# The bridge links the app crate, whose Tauri build script needs the MCP
# sidecar binary to exist.
if ! ls "$REPO_ROOT"/crates/mimir/binaries/mimir-mcp-* >/dev/null 2>&1; then
  echo "ui-session: building the mimir-mcp sidecar..." >&2
  bash "$REPO_ROOT/scripts/build-sidecar.sh" >&2
fi

echo "ui-session: building ui-bridge..." >&2
cargo build -p mimir-ui-bridge --quiet --manifest-path "$REPO_ROOT/Cargo.toml"

echo "ui-session: scratch dir $SCRATCH (fixture seeded by the bridge)" >&2
MIMIR_BRIDGE_APP_DIR="$SCRATCH" MIMIR_BRIDGE_PORT="$PORT" MIMIR_BRIDGE_SEED=fixture \
  "$REPO_ROOT/target/debug/ui-bridge" &
BRIDGE_PID=$!
wait "$BRIDGE_PID"
