#!/bin/sh
# Build the editor bundle (MIMIR-T-0719): dist/mimir-editor.js. Trunk runs
# this before each build (Trunk.toml); it needs Node. npm ci runs only when
# node_modules is missing or older than the lockfile.
set -e
cd "$(dirname "$0")"
if [ ! -f node_modules/.package-lock.json ] || [ package-lock.json -nt node_modules/.package-lock.json ]; then
  npm ci --no-audit --no-fund
fi
npm run build --silent
