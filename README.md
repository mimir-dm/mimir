# Mimir

<p align="center">
  <img src="assets/hyper-mimir.png" alt="Mimir" width="512" height="512">
</p>

Mimir is a D&D 5e campaign manager for Dungeon Masters. It is a web app that you run on your own server. Prepare campaigns, modules, characters and maps in the browser; run sessions from the DM map; show the map on a TV; and give each player a link to their own character. It has a searchable D&D 5e catalog and PDF export. All data stays on your server.

## Run It

Mimir is one program, `mimir-server`. It serves the web app, its API and an MCP endpoint for Claude.

```bash
angreal web build --release
cargo build --release -p mimir-server --features embed-web

MIMIR_DATA_DIR=/srv/mimir \
MIMIR_API_TOKEN="<a long random value>" \
MIMIR_BIND=0.0.0.0:8740 \
  ./target/release/mimir-server
```

Open the server address in a browser and sign in with the DM token. Without `MIMIR_API_TOKEN` the server is in open mode: anyone who reaches it is the DM. Put the server on a private network.

See [Run the Server](docs/src/how-to/run-server.md) and [Environment Variables](docs/src/reference/environment-variables.md).

## Claude Code Plugin

Mimir includes a [Claude Code](https://claude.ai/code) plugin for AI-assisted campaign authoring from the terminal.

**Install from GitHub** (inside a Claude Code session):
```
/plugin marketplace add mimir-dm/mimir
/plugin install mimir-dm@mimir
```

The plugin connects to the Mimir server's `/mcp` endpoint. Set the server's
address and the DM token first:
```bash
export MIMIR_URL="http://127.0.0.1:8740"   # the server's address
export MIMIR_API_TOKEN="…"                 # the DM token (empty in open mode)
```

**Or add the server directly:**
```bash
claude mcp add --transport http mimir "$MIMIR_URL/mcp" \
  --header "Authorization: Bearer $MIMIR_API_TOKEN"
```

Once installed, use natural language or slash commands:
- `/mimir-campaigns` - List your campaigns
- `/create-module "The Haunted Manor" mystery` - Create a new module
- `/search-monsters undead CR 5` - Search the monster catalog

See [plugin documentation](crates/mimir-mcp/plugin/README.md) for full setup instructions.

## Documentation

Full documentation is available at [mimir-dm.github.io](https://mimir-dm.github.io).

## Contributing

We welcome contributions. See [CONTRIBUTING.md](CONTRIBUTING.md) and [DEVELOPMENT.md](DEVELOPMENT.md) for the setup, the code style and the pull request process.
