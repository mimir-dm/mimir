#!/bin/sh
# Check a built Mimir image (MIMIR-T-0706): it starts, becomes healthy, serves
# the web app and /readyz, runs under tini, and stops cleanly on SIGTERM.
#
#   scripts/image-smoke.sh <image>
set -eu
image="${1:?usage: image-smoke.sh <image>}"
name="mimir-smoke-$$"

cleanup() { docker rm -f -v "$name" >/dev/null 2>&1 || true; }
trap cleanup EXIT

docker run -d --name "$name" -p 127.0.0.1::8740 "$image" >/dev/null
port="$(docker port "$name" 8740/tcp | head -n1 | sed 's/.*://')"

echo "waiting for the health check..."
i=0
until [ "$(docker inspect -f '{{.State.Health.Status}}' "$name")" = "healthy" ]; do
  i=$((i + 1))
  if [ "$i" -gt 60 ]; then
    docker logs "$name" >&2
    echo "not healthy after 60 s" >&2
    exit 1
  fi
  sleep 1
done

curl -fsS "http://127.0.0.1:$port/readyz" | grep -q ready
# The embedded app, not the placeholder: its wasm loader is in the page.
curl -fsS "http://127.0.0.1:$port/" | grep -q "mimir-web"
curl -fsS "http://127.0.0.1:$port/api/config" | grep -q "\"version\""
# tini is PID 1.
docker exec "$name" cat /proc/1/comm | grep -q tini

docker stop -t 10 "$name" >/dev/null
code="$(docker inspect -f '{{.State.ExitCode}}' "$name")"
if [ "$code" != "0" ] && [ "$code" != "143" ]; then
  docker logs "$name" >&2
  echo "exit code $code on stop" >&2
  exit 1
fi
echo "image ok: $image"
