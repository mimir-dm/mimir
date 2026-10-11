# Deploy with Docker

This guide runs Mimir as a Docker Compose stack on a tailnet. The stack has the Mimir server, a Tailscale sidecar, a redirect from HTTP to HTTPS, and a backup service. Mimir is reachable only on your tailnet, at `https://<name>.<tailnet>.ts.net/`. No port opens on the host.

The files are in the `deploy/` folder of the repository.

## Prerequisites

- Docker with Compose v2
- A Tailscale account, with **HTTPS Certificates** on (admin console, **DNS**)
- A Tailscale auth key (admin console, **Settings**, **Keys**)

## First Start

1. Copy the `deploy/` folder to the host, for example to `~/Deploy/mimir/deploy`.
2. Make the settings file and protect it:

   ```bash
   cp .env.example .env
   chmod 600 .env
   ```

3. Edit `.env`:
   - `MIMIR_VERSION`: a release tag, for example `1.0.0`. Do not use `latest`.
   - `TAILSCALE_AUTHKEY`: the auth key. Without a key, the Tailscale node logs a login link instead: run `docker compose logs tailscale` and open the link within 15 minutes (`TAILSCALE_EXTRA_ARGS=--timeout=15m`). If your tailnet approves new devices, approve `mimir` in the admin console.
   - `TAILSCALE_HOSTNAME`: the node name. The default is `mimir`.
   - `MIMIR_API_TOKEN`: see [Set the DM Token](#set-the-dm-token).
4. Start the stack:

   ```bash
   docker compose --env-file .env up -d
   ```

5. Check that the server is healthy:

   ```bash
   docker compose ps
   docker compose logs mimir
   ```

6. In the Tailscale admin console, disable key expiry for the new node. You can then remove `TAILSCALE_AUTHKEY` from `.env`: the node keeps its identity in the `tailscale_state` volume.
7. Open `https://<name>.<tailnet>.ts.net/` and import the catalog data. See [Manage Source Books](./campaigns/manage-sources.md).

## Set the DM Token

- With `MIMIR_API_TOKEN` empty, the server is in open mode: anyone on the tailnet is the DM.
- With a value, the DM signs in with that value. Use a long random value, for example the output of `openssl rand -hex 32`.

Players do not use the DM token. Each player opens the link of their character. See [Player Links](./characters/player-links.md).

To change the token, edit `.env` and recreate the server:

```bash
docker compose --env-file .env up -d mimir
```

Recreating the server container also recreates the sidecars that share its network. Run `docker compose --env-file .env up -d --force-recreate tailscale tailscale-http-redirect` after it.

## Back Up

The `backup` service writes an archive every 6 hours into `MIMIR_BACKUP_DIR` on the host (default `./backups`). Each archive holds:

- `mimir.db`: a copy of the database, made with SQLite `.backup` while the server runs, and checked with `PRAGMA integrity_check`;
- `assets/`: the maps, the images and the other uploaded files.

The service keeps the newest 8 archives. Change this with `MIMIR_BACKUP_INTERVAL` (seconds) and `MIMIR_BACKUP_KEEP`.

On Linux, the backup folder must be writable by user ID 10001, the user of the image.

To make a backup now:

```bash
docker compose --env-file .env restart backup
```

## Restore a Backup

1. Stop the server: `docker compose --env-file .env stop mimir backup`.
2. Unpack the archive into the data volume:

   ```bash
   docker run --rm -v mimir_mimir_data:/data -v "$PWD/backups:/backups:ro" \
     debian:trixie-slim sh -c \
     'tar -xzf /backups/mimir-<stamp>.tar.gz -C /tmp && cp /tmp/mimir.db /data/data/mimir.db && cp -a /tmp/assets/. /data/assets/ && chown -R 10001:10001 /data'
   ```

3. Start the stack again: `docker compose --env-file .env up -d`.

## Upgrade

1. Read the release notes of the new version.
2. Make a backup (see above) and keep a copy of `.env`.
3. Set `MIMIR_VERSION` to the new tag in `.env`.
4. Pull and start:

   ```bash
   docker compose --env-file .env pull
   docker compose --env-file .env up -d
   docker compose --env-file .env up -d --force-recreate tailscale tailscale-http-redirect
   ```

The server runs the database migrations when it starts. Check `docker compose ps` and the logs. To go back, set the old tag and restore the backup.

## The Image

`ghcr.io/mimir-dm/mimir:<version>` is built for `linux/amd64` and `linux/arm64` when a version tag is pushed. Only version tags are published.

| Setting | Value in the image |
|---------|--------------------|
| `MIMIR_BIND` | `0.0.0.0:8740` |
| `MIMIR_DATA_DIR` | `/var/lib/mimir` (a volume) |
| User | `mimir`, user ID 10001 |
| Health check | `GET /readyz` |

To build the image yourself:

```bash
docker build -t mimir:local .
scripts/image-smoke.sh mimir:local
```
