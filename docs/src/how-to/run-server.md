# Run the Server

Mimir is one program, `mimir-server`. It serves the web app, the API under `/api`, the live socket at `/ws` and the MCP endpoint at `/mcp`. This guide shows you how to build it and run it.

## Prerequisites

- A Rust toolchain ([rustup](https://rustup.rs/))
- [trunk](https://trunkrs.dev/) (`cargo install trunk --locked`)
- Node.js and npm, for the editor bundle
- [angreal](https://github.com/angreal/angreal) (`pip install 'angreal>=2'`)

## Build One Binary

1. Build the web app for release:

   ```bash
   angreal web build --release
   ```

2. Build the server with the web app inside it:

   ```bash
   cargo build --release -p mimir-server --features embed-web
   ```

The binary is `target/release/mimir-server`. It holds the web app, so you copy one file.

## Choose the Settings

The server reads its settings from environment variables. See [Environment Variables](../reference/environment-variables.md).

| Variable | Use |
|----------|-----|
| `MIMIR_DATA_DIR` | The data directory. The database and the uploaded files go here. |
| `MIMIR_API_TOKEN` | The DM token. Without it, the server is in open mode: anyone who reaches it is the DM. |
| `MIMIR_BIND` | The address and port to listen on. Default `127.0.0.1:8740`. |

Set a DM token on any server that other people can reach. Use a long random value. Keep it secret: it gives full access.

## Start the Server

```bash
export MIMIR_DATA_DIR=/srv/mimir
export MIMIR_API_TOKEN="<a long random value>"
export MIMIR_BIND=0.0.0.0:8740
./mimir-server
```

The server makes the data directory, runs the database migrations and listens. Press Ctrl-C to stop it.

## Open Mimir

1. Open the server address in a browser, for example `http://192.0.2.10:8740` or `https://mimir.example.ts.net`.
2. Type the DM token and click **Sign in**.
3. Import the catalog data. See [Manage Source Books](./campaigns/manage-sources.md).

## Check the Server

| Path | Answer |
|------|--------|
| `/healthz` | `ok` while the process runs |
| `/readyz` | `ready` when the database answers |
| `/api/config` | The version and the sign-in mode (`open` or `token`) |

## Reach It From Other Devices

Players use their own devices and the TV shows the player display, so these devices must reach the server. Put the server on a private network, for example a Tailscale tailnet, and serve it over HTTPS. Do not put a server in open mode on the internet.

## Back Up

Everything is in the data directory: `data/mimir.db` and the `assets/` folder. Stop the server and copy the directory. To move one campaign, use a campaign archive instead. See [Export a Campaign](./campaigns/export-campaign.md).

## See Also

- [Environment Variables](../reference/environment-variables.md)
- [Troubleshooting](../reference/troubleshooting.md)
- [Architecture Overview](../explanation/architecture-overview.md)
