# Mimir

<p align="center">
  <img src="assets/hyper-mimir.png" alt="Mimir" width="512" height="512">
</p>

Mimir is a local-first desktop application for managing D&D campaigns. Built with Rust and Tauri, it provides campaign workflow management, a searchable D&D 5e catalog, AI assistance via local LLMs, and PDF export capabilities. All data stays on your machine.

## Installation

**macOS / Linux**:
```bash
curl -sSL https://raw.githubusercontent.com/mimir-dm/mimir/main/scripts/install.sh | sh
```

**Windows**: Download and run the `.msi` installer from [GitHub Releases](https://github.com/mimir-dm/mimir/releases).

## Claude Code Plugin

Mimir includes a [Claude Code](https://claude.ai/code) plugin for AI-assisted campaign authoring directly from the terminal.

**Install from GitHub:**
```bash
claude plugins add github:colliery/mimir/crates/mimir-dm-mcp/plugin
```

**Or manually:**
```bash
# Build the MCP server
cargo build --release -p mimir-dm-mcp

# Set your database path
export MIMIR_DATABASE_PATH="$HOME/Library/Application Support/com.mimir.mimir/mimir.db"
```

Once installed, use natural language or slash commands:
- `/mimir-campaigns` - List your campaigns
- `/create-module "The Haunted Manor" mystery` - Create a new module
- `/search-monsters undead CR 5` - Search the monster catalog

See [plugin documentation](crates/mimir-dm-mcp/plugin/README.md) for full setup instructions.

## Documentation

Full documentation is available at [mimir-dm.github.io](https://mimir-dm.github.io).

## Contributing

We welcome contributions. See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines on getting started, code style, and the pull request process.
