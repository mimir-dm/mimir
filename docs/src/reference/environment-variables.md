# Environment Variables

The environment variables that Mimir reads.

## Server (`mimir-server`)

### `MIMIR_BIND`

The address and port to listen on.

```bash
export MIMIR_BIND=0.0.0.0:8740
```

**Default:** `127.0.0.1:8740`. Only this computer can connect. Use `0.0.0.0:8740` in a container or to serve other devices.

The server stops with an error if the value is not an address.

### `MIMIR_DATA_DIR`

The data directory.

```bash
export MIMIR_DATA_DIR=/srv/mimir
```

**Default:** `./mimir-data`, from the directory where the server starts.

| Path | Contents |
|------|----------|
| `<data dir>/data/mimir.db` | The SQLite database |
| `<data dir>/assets/` | Map images, token images and imported catalog images |

The server makes the directories and runs the database migrations at start.

### `MIMIR_API_TOKEN`

The DM token. A browser, the API and `/mcp` need it to act as the DM.

```bash
export MIMIR_API_TOKEN="<a long random value>"
```

**Default:** not set. With no token, or a blank one, the server is in **open mode**: every caller is the DM and there is no sign-in page. The server logs a warning at start. Use open mode only on a computer that no one else can reach.

Player links work in both modes.

### `MIMIR_WEB_DIST`

Serve the web app from this directory instead of the copy in the binary.

```bash
export MIMIR_WEB_DIST=crates/mimir-web/dist
```

**Default:** not set. The server then serves the web app built into the binary (the `embed-web` feature). Without both, it serves a placeholder page. `angreal web serve` sets this variable.

### `MIMIR_SEED`

Development and tests only.

```bash
export MIMIR_SEED=fixture
```

`fixture` seeds the SRD catalog and the test campaign "The Lost Mine of Phandelver" into an empty database. The server must be built with the `fixtures` feature. Any other value is an error.

**Default:** not set. No seeding.

### `MIMIR_SEED_ASSETS`

Development and tests only. The directory of the seed maps and token images for `MIMIR_SEED=fixture`.

**Default:** not set. The seeder looks for `crates/mimir-core/src/seed/assets` in the parent directories.

### `RUST_LOG`

How much the server logs, in the [tracing filter syntax](https://docs.rs/tracing-subscriber/latest/tracing_subscriber/filter/struct.EnvFilter.html).

```bash
export RUST_LOG=debug
export RUST_LOG=info,mimir_core=debug
```

**Default:** `info`. The server logs to standard output. The request log has the method and the path, never the query.

## Claude Code Plugin

The plugin in `crates/mimir-mcp/plugin` reads these in the shell that starts Claude Code.

| Variable | Use | Default |
|----------|-----|---------|
| `MIMIR_URL` | The server address | `http://127.0.0.1:8740` |
| `MIMIR_API_TOKEN` | The DM token | empty (open mode) |

See [MCP Server](./mcp-server.md).

## See Also

- [Run the Server](../how-to/run-server.md)
- [Troubleshooting](./troubleshooting.md)
- [Development Setup](../developer/DEVELOPMENT.md)
