# syntax=docker/dockerfile:1

# The Mimir server image (MIMIR-T-0706, COLLIERY-I-0612). One binary,
# mimir-server, with the Leptos web app embedded (`embed-web`). It serves the
# app, /api, /ws, /mcp, /healthz and /readyz on 8740. The data (the SQLite
# database, the assets, the upload temp folder) is in /var/lib/mimir, a
# volume. Migrations run on boot.
#
# After the Kairos image (colliery-io/kairos Dockerfile).

# ---------------------------------------------------------------------------
# Stage 1: the web bundle and the release server binary
# ---------------------------------------------------------------------------
FROM rust:1.93-slim-trixie AS builder

# - nodejs + npm: Trunk's pre_build hook runs crates/mimir-web/editor/build.sh
#   (npm ci + esbuild, the TipTap bundle, MIMIR-T-0719).
# - ca-certificates: cargo, npm and trunk download over TLS.
# SQLite is bundled (libsqlite3-sys `bundled`), so no libsqlite3-dev.
# hadolint ignore=DL3008
RUN apt-get update && apt-get install -y --no-install-recommends \
        pkg-config \
        ca-certificates \
        nodejs \
        npm \
    && rm -rf /var/lib/apt/lists/*

# The version `angreal web build` uses.
ARG TRUNK_VERSION=0.21.14
RUN cargo install trunk --version "${TRUNK_VERSION}" --locked \
    && rustup target add wasm32-unknown-unknown

WORKDIR /build
COPY . .

# 1) The web bundle (release: wasm-opt -Oz) into crates/mimir-web/dist.
# 2) The server with `embed-web`: rust-embed reads dist at compile time, so
#    the order matters.
WORKDIR /build/crates/mimir-web
RUN trunk build --release
WORKDIR /build
RUN cargo build --release --locked -p mimir-server --features embed-web

# ---------------------------------------------------------------------------
# Stage 2: runtime
# ---------------------------------------------------------------------------
# The same Debian release as the builder.
FROM debian:trixie-slim AS runtime

# - fonts: the PDF export (mimir-print, typst) loads the system fonts; its
#   templates ask for Inter, then Helvetica or Arial, and JetBrains Mono or
#   Consolas. DejaVu is the fallback for the rest.
# - curl: the HEALTHCHECK.
# - sqlite3: the backup service of deploy/docker-compose.yaml runs this image
#   and copies the database with `.backup`.
# - tini: PID 1; it reaps orphans (a curl of the HEALTHCHECK) and passes
#   SIGTERM on.
# hadolint ignore=DL3008
RUN apt-get update && apt-get install -y --no-install-recommends \
        ca-certificates \
        curl \
        sqlite3 \
        tini \
        fonts-inter \
        fonts-jetbrains-mono \
        fonts-dejavu-core \
    && rm -rf /var/lib/apt/lists/* \
    && groupadd --system --gid 10001 mimir \
    && useradd --system --uid 10001 --gid 10001 --create-home --shell /usr/sbin/nologin mimir \
    && install -d -o 10001 -g 10001 /var/lib/mimir

LABEL org.opencontainers.image.title="mimir" \
      org.opencontainers.image.description="Mimir: a D&D campaign assistant for the DM and the players, served to the browser" \
      org.opencontainers.image.source="https://github.com/mimir-dm/mimir" \
      org.opencontainers.image.licenses="MIT"

COPY --from=builder /build/target/release/mimir-server /usr/local/bin/mimir-server

USER mimir

# Listen on all interfaces in the container; the data on the volume.
# MIMIR_API_TOKEN comes from the deployment (.env); unset is open mode.
ENV MIMIR_BIND=0.0.0.0:8740 \
    MIMIR_DATA_DIR=/var/lib/mimir

VOLUME ["/var/lib/mimir"]
EXPOSE 8740

# /readyz checks the database.
HEALTHCHECK --interval=10s --timeout=3s --start-period=20s --retries=5 \
    CMD curl -fsS http://localhost:8740/readyz || exit 1

ENTRYPOINT ["/usr/bin/tini", "-s", "--", "/usr/local/bin/mimir-server"]
