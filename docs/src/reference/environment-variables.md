# Environment Variables

All environment variables recognized by Mimir and its components.

## Application

### `MIMIR_DEV`

Forces development mode regardless of build type. When set (any value), Mimir uses the dev database directory instead of production.

```bash
export MIMIR_DEV=1
```

**Effect:** Database and assets are read from `{app_dir}/dev/data/` instead of `{app_dir}/data/`. This prevents development from modifying production campaign data.

**Default:** Not set. Dev mode is normally detected automatically via Rust's `cfg!(debug_assertions)` — debug builds use dev paths, release builds use production paths.

### `RUST_LOG`

Controls log verbosity using the [tracing](https://docs.rs/tracing-subscriber/latest/tracing_subscriber/filter/struct.EnvFilter.html) filter syntax.

```bash
# Show debug logs for all Mimir crates
export RUST_LOG=mimir=debug,mimir_core=debug

# Trace-level for the core crate only
export RUST_LOG=mimir_core=trace

# Quiet mode — errors only
export RUST_LOG=mimir=error
```

**Default:** `mimir=info,mimir_core=info` (all builds)

**Log output:** Written to files in the logs directory:
- **macOS:** `~/Library/Application Support/com.mimir.app/logs/`
- **Linux:** `~/.local/share/com.mimir.app/logs/`
- **Windows:** `%APPDATA%/com.mimir.app/logs/`

## MCP Server

### `MIMIR_DATABASE_PATH`

Overrides the database path for the MCP sidecar server. Useful when running the MCP server outside of Tauri (e.g., as a standalone process for Claude Code).

```bash
export MIMIR_DATABASE_PATH=~/Library/Application\ Support/com.mimir.app/data/mimir.db
```

**Default:** Platform-specific:
| Platform | Default Path |
|----------|-------------|
| macOS | `~/Library/Application Support/com.mimir.app/data/mimir.db` |
| Linux | `$XDG_DATA_HOME/com.mimir.app/data/mimir.db` (falls back to `~/.local/share/...`) |
| Windows | No default — `MIMIR_DATABASE_PATH` must be set explicitly |

The path supports `~` and `$HOME` expansion.

> **Note on Linux:** The MCP server respects the `XDG_DATA_HOME` environment variable. If unset, it falls back to `~/.local/share/`.

### `MIMIR_SEED_ASSETS`

Development only. Path to the seed-asset directory (sample UVTT maps, token images) used when seeding the dev database from the in-app Dev Tools.

```bash
export MIMIR_SEED_ASSETS=/path/to/mimir/crates/mimir-core/src/seed/assets
```

**Default:** Not set. The seeder walks parent directories looking for `crates/mimir-core/src/seed/assets`; the variable is only needed when running from a location where that walk fails.

## Build System

### `TAURI_ENV_TARGET_TRIPLE`

Set by Tauri during builds. Used by the sidecar build scripts to determine the target architecture.

```bash
# Set automatically by Tauri, but can be overridden for cross-compilation
export TAURI_ENV_TARGET_TRIPLE=aarch64-apple-darwin
```

**Supported values:**
- `aarch64-apple-darwin` (macOS ARM64)
- `x86_64-apple-darwin` (macOS Intel)
- `x86_64-unknown-linux-gnu` (Linux x86_64)
- `aarch64-unknown-linux-gnu` (Linux ARM64)
- `x86_64-pc-windows-msvc` (Windows x86_64)
- `aarch64-pc-windows-msvc` (Windows ARM64)

**Default:** Auto-detected from the current platform when not set.

### `CARGO_TERM_COLOR`

Standard Cargo variable for controlling colored output. Set to `always` in CI workflows.

```bash
export CARGO_TERM_COLOR=always
```

## Platform Directories

Mimir uses platform-appropriate directories following OS conventions. These aren't environment variables, but they're the default paths that environment variables override:

### macOS

| Purpose | Path |
|---------|------|
| App directory | `~/Library/Application Support/com.mimir.app/` |
| Database | `{app_dir}/data/mimir.db` |
| Assets | `{app_dir}/assets/` |
| Logs | `{app_dir}/logs/` |
| Config | `{app_dir}/config/` |

### Linux

| Purpose | Path |
|---------|------|
| App directory | `~/.local/share/com.mimir.app/` |
| Database | `{app_dir}/data/mimir.db` |
| Assets | `{app_dir}/assets/` |
| Logs | `{app_dir}/logs/` |
| Config | `{app_dir}/config/` |

### Windows

| Purpose | Path |
|---------|------|
| App directory | `%APPDATA%/com.mimir.app/` |
| Database | `{app_dir}/data/mimir.db` |
| Assets | `{app_dir}/assets/` |
| Logs | `{app_dir}/logs/` |
| Config | `{app_dir}/config/` |

## See Also

- [Troubleshooting](./troubleshooting.md) — Common issues related to paths and configuration
- [Development Setup](../developer/DEVELOPMENT.md) — Setting up a development environment
- [MCP Server Reference](./mcp-server.md) — MCP server configuration
