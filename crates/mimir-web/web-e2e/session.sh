#!/usr/bin/env bash
# e2e session: mimir-server on a DISPOSABLE data dir seeded with the UI
# fixture (SRD catalog + "The Lost Mine of Phandelver"), serving the built
# web app. The scratch dir is removed when the server exits. It never reads
# or writes a real data dir: the data dir is always a new mktemp dir.
#
# Env:
#   MIMIR_E2E_PORT    port (default 8790)
#   MIMIR_E2E_TOKEN   DM token (default e2e-dm-token; a test value)
#
# Started by playwright.config.ts (webServer); fine to run standalone.
set -euo pipefail

WEB_CRATE="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
REPO_ROOT="$(cd "$WEB_CRATE/../.." && pwd)"
PORT="${MIMIR_E2E_PORT:-8790}"
TOKEN="${MIMIR_E2E_TOKEN:-e2e-dm-token}"

SCRATCH="$(mktemp -d "${TMPDIR:-/tmp}/mimir-e2e.XXXXXX")"
SERVER_PID=""
cleanup() {
  if [ -n "$SERVER_PID" ]; then
    kill "$SERVER_PID" 2>/dev/null || true
    wait "$SERVER_PID" 2>/dev/null || true
  fi
  rm -rf "$SCRATCH"
  echo "e2e session: scratch dir removed" >&2
}
trap cleanup EXIT INT TERM

if [ ! -f "$WEB_CRATE/dist/index.html" ]; then
  echo "e2e session: no bundle; building it (trunk build)..." >&2
  (cd "$WEB_CRATE" && trunk build >&2)
fi

echo "e2e session: building mimir-server (fixtures)..." >&2
cargo build -p mimir-server --features fixtures --quiet --manifest-path "$REPO_ROOT/Cargo.toml"

echo "e2e session: scratch dir $SCRATCH" >&2
MIMIR_BIND="127.0.0.1:$PORT" \
MIMIR_DATA_DIR="$SCRATCH" \
MIMIR_SEED=fixture \
MIMIR_API_TOKEN="$TOKEN" \
MIMIR_WEB_DIST="$WEB_CRATE/dist" \
  "$REPO_ROOT/target/debug/mimir-server" &
SERVER_PID=$!
wait "$SERVER_PID"
