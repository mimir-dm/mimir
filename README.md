# Mimir

<p align="center">
  <img src="assets/hyper-mimir.png" alt="Mimir" width="512" height="512">
</p>

Mimir is a local-first desktop application for managing D&D campaigns. Built with Rust and Tauri, it provides campaign workflow management, a searchable D&D 5e catalog, and PDF export capabilities. All data stays on your machine.

## Installation

**macOS / Linux**:
```bash
curl -sSL https://raw.githubusercontent.com/mimir-dm/mimir/main/scripts/install.sh | sh
```

**Windows**: Download and run the `.msi` installer from [GitHub Releases](https://github.com/mimir-dm/mimir/releases).

## Claude Code Plugin

Mimir includes a [Claude Code](https://claude.ai/code) plugin for AI-assisted campaign authoring directly from the terminal.

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

We welcome contributions. See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines on getting started, code style, and the pull request process.
