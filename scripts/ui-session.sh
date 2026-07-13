#!/usr/bin/env bash
# UI-harness session: run the invoke-over-HTTP bridge against a DISPOSABLE
# snapshot of the production database (MIMIR-T-0660).
#
# Flow: sqlite-backup prod DB -> scratch dir -> launch ui-bridge against the
# scratch copy -> tear the scratch dir down when the bridge exits. The live
# campaign is never opened writable; the bridge itself additionally refuses
# to start on any path under the production app dir.
#
# Env:
#   MIMIR_BRIDGE_PORT         bridge port (default 4175)
#   MIMIR_UI_SESSION_ASSETS   "link" to symlink the real assets dir into the
#                             scratch app dir so map images render. The assets
#                             tree is ~6 GB so it is never copied. CAUTION:
#                             through the symlink, asset uploads/deletes made
#                             during the session WOULD touch the real files —
#                             avoid asset mutations when linked. Default: no
#                             assets; map views degrade to placeholders.
#
# Used as a Playwright webServer command (playwright.config.ts) and by
# `angreal dev screenshots`; also fine to run standalone.
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
PROD_APP_DIR="$HOME/Library/Application Support/com.mimir.app"
PROD_DB="$PROD_APP_DIR/data/mimir.db"
PORT="${MIMIR_BRIDGE_PORT:-4175}"

if [ ! -f "$PROD_DB" ]; then
  echo "ui-session: production database not found at $PROD_DB" >&2
  exit 1
fi

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

mkdir -p "$SCRATCH/data"
sqlite3 "$PROD_DB" ".backup '$SCRATCH/data/mimir.db'"
echo "ui-session: snapshot -> $SCRATCH/data/mimir.db ($(du -h "$SCRATCH/data/mimir.db" | cut -f1))" >&2

if [ "${MIMIR_UI_SESSION_ASSETS:-none}" = "link" ]; then
  ln -s "$PROD_APP_DIR/assets" "$SCRATCH/assets"
  echo "ui-session: assets SYMLINKED to the real tree — do not upload/delete assets this session" >&2
fi

echo "ui-session: building ui-bridge..." >&2
cargo build -p mimir-ui-bridge --quiet --manifest-path "$REPO_ROOT/Cargo.toml"

MIMIR_BRIDGE_APP_DIR="$SCRATCH" MIMIR_BRIDGE_PORT="$PORT" \
  "$REPO_ROOT/target/debug/ui-bridge" &
BRIDGE_PID=$!
wait "$BRIDGE_PID"
