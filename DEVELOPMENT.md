# Development Guide

This guide covers setting up a development environment for Mimir and common development workflows.

## Prerequisites

### Required Software

- **Rust** (1.70 or higher) - [Install via rustup](https://rustup.rs/)
- **Node.js** (v18 or higher) - [Download from nodejs.org](https://nodejs.org/)
- **npm** (comes with Node.js)

### Platform-Specific Dependencies

#### macOS
```bash
# Install Xcode Command Line Tools
xcode-select --install
```

#### Linux (Ubuntu/Debian)
```bash
sudo apt-get update
sudo apt-get install -y \
  libwebkit2gtk-4.1-dev \
  libappindicator3-dev \
  librsvg2-dev \
  patchelf \
  build-essential \
  curl \
  wget \
  file
```

#### Windows
- Install [Microsoft Visual Studio C++ Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/)
- Or install Visual Studio with "Desktop development with C++" workload

## Initial Setup

### 1. Clone the Repository

```bash
git clone https://github.com/mimir-dm/mimir.git
cd mimir
```

### 2. Build the MCP Sidecar, Then Rust Dependencies

```bash
# Build the sidecar first — tauri.conf.json declares it as externalBin,
# so the mimir crate fails to build if the binary is missing
bash scripts/build-sidecar.sh

cargo build
```

This will download and compile all Rust dependencies. First build may take several minutes.

### 3. Install Frontend Dependencies

```bash
cd crates/mimir/frontend
npm install
cd ../../..
```

### 4. Install Angreal (Recommended)

Angreal is the project's task runner (`angreal dev launch`, `angreal test unit`, `angreal dev reset`, `angreal docs serve`). It requires Python and pip:

```bash
pip install 'angreal>=2'
```

Optional extras:

```bash
# Install Tauri CLI globally (optional, faster startup)
cargo install tauri-cli
```

## Running the Application

### Development Mode

The recommended way to run in development mode is the angreal task, from the project root:

```bash
angreal dev launch
```

This will:
- Install frontend dependencies if needed
- Start the Vite dev server with hot reload
- Build the MCP sidecar binary if it is missing
- Launch the app with `cargo run -p mimir --no-default-features`

> **Known issue:** `cargo tauri dev` runs `npm run dev` from `crates/` instead of
> `crates/mimir/frontend`, so the Vite dev server fails to start. If you can't use
> angreal, run the two halves manually:
>
> ```bash
> # Terminal 1
> cd crates/mimir/frontend && npm run dev
>
> # Terminal 2 (project root)
> cargo run -p mimir --no-default-features
> ```

### Frontend-Only Development

If you only need to work on the UI:

```bash
cd crates/mimir/frontend
npm run dev
```

This starts the Vite development server on http://localhost:5173, but Tauri commands won't work.

## Testing

### Run All Tests

```bash
# Rust tests (recommended)
angreal test unit
# Underlying command:
#   cargo test --workspace --exclude mimir -- --test-threads=1
# (excludes the Tauri crate, which needs the sidecar binary to build;
#  --test-threads=1 avoids SQLite locking issues)

# Frontend tests
cd crates/mimir/frontend && npm test

# Run tests with coverage
npm run test:coverage
```

### Run Specific Test Suites

```bash
# Specific crate tests
cargo test -p mimir-core -- --test-threads=1
cargo test -p mimir-print

# Specific integration test targets
cargo test -p mimir-core --test catalog_import -- --test-threads=1
cargo test -p mimir-core --test srd_smoke_test -- --test-threads=1
cargo test -p mimir-mapgen --test cli_integration
cargo test -p mimir-mapgen --test format_roundtrip
cargo test -p mimir-mapgen --test polygon_snapshots
```

### Frontend Testing

```bash
cd crates/mimir/frontend

# Run tests
npm test

# Run tests in watch mode
npm test -- --watch

# Run tests with UI
npm run test:ui

# Generate coverage report
npm run test:coverage
```

## Building for Production

### Full Build

```bash
cd crates/mimir
cargo tauri build
```

This creates platform-specific installers in `target/release/bundle/`.

### Frontend Build Only

```bash
cd crates/mimir/frontend
npm run build
```

Output goes to `crates/mimir/frontend/dist/`.

## Project Structure

```
mimir/
├── crates/                          # Rust workspace
│   ├── mimir/                      # Main Tauri application
│   │   ├── src/                    # Rust backend
│   │   │   ├── main.rs            # Application entry point
│   │   │   ├── commands/          # Tauri command handlers
│   │   │   └── state.rs           # Application state & paths
│   │   ├── frontend/               # Vue 3 frontend
│   │   │   ├── src/
│   │   │   │   ├── app/           # App setup and routing
│   │   │   │   ├── components/    # Reusable components
│   │   │   │   ├── composables/   # Vue composables
│   │   │   │   ├── views/         # Page views
│   │   │   │   ├── services/      # API services
│   │   │   │   ├── shared/        # Shared UI components
│   │   │   │   └── stores/        # Pinia stores
│   │   │   └── package.json
│   │   ├── tauri.conf.json        # Tauri configuration
│   │   └── Cargo.toml
│   │
│   ├── mimir-core/                 # Core business logic
│   │   ├── src/
│   │   │   ├── models/            # Domain models
│   │   │   ├── services/          # Business services
│   │   │   ├── dal/               # Data access layer
│   │   │   └── migrations/        # Database migrations
│   │   └── Cargo.toml
│   │
│   ├── mimir-mcp/                  # MCP server for Claude integration
│   │   ├── src/
│   │   │   ├── context.rs         # Database context
│   │   │   └── tools/             # MCP tool handlers
│   │   ├── plugin/                # Claude Code plugin definition
│   │   └── Cargo.toml
│   │
│   ├── mimir-mapgen/               # Procedural map generation (library + CLI)
│   │   ├── src/                   # Noise, terrain, paths, rooms, biome presets
│   │   ├── examples/              # YAML configs + generated maps
│   │   └── Cargo.toml
│   │
│   └── mimir-print/                # PDF export via Typst
│       ├── src/
│       │   └── sections/          # PDF section renderers
│       └── Cargo.toml
│
├── docs/                            # Documentation (mdBook)
│   └── src/                        # mdBook source
│
├── .metis/                         # Project management
└── Cargo.toml                      # Workspace configuration
```

## Common Development Tasks

### Adding a New Tauri Command

1. Create command handler in `crates/mimir/src/commands/`:
```rust
#[tauri::command]
pub async fn my_command(param: String) -> Result<String, String> {
    // Implementation
    Ok(format!("Result: {}", param))
}
```

2. Register in `crates/mimir/src/main.rs`:
```rust
.invoke_handler(tauri::generate_handler![
    commands::my_command,
    // ... other commands
])
```

3. Call from frontend:
```typescript
import { invoke } from '@tauri-apps/api/core';

const result = await invoke<string>('my_command', { param: 'value' });
```

### Adding a Database Migration

```bash
cd crates/mimir-core

# Create new migration
diesel migration generate migration_name

# Edit up.sql and down.sql in migrations/

# Run migration
diesel migration run

# Test rollback
diesel migration redo
```

### Adding a Frontend Component

```bash
cd crates/mimir/frontend/src/components
# Create MyComponent.vue
```

```vue
<template>
  <div class="my-component">
    {{ message }}
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue';

const message = ref('Hello');
</script>

<style scoped>
.my-component {
  /* styles */
}
</style>
```

## Troubleshooting

### Build Errors

**"failed to run custom build command for tauri"**
- Ensure all platform-specific dependencies are installed
- On Linux: verify webkit2gtk-4.1-dev is installed (not 4.0)
- On macOS: run `xcode-select --install`

**Frontend build fails**
```bash
cd crates/mimir/frontend
rm -rf node_modules package-lock.json
npm install
```

**Rust compilation errors**
```bash
cargo clean
cargo build
```

### Runtime Errors

**Database migration errors**
```bash
# Reset the dev database (deletes ONLY the dev DB, never production)
angreal dev reset

# Restart the app to recreate
```

**Frontend hot reload not working**
- Stop the app and restart with `angreal dev launch`
- Check console for build errors
- Ensure Vite dev server is running

### Development Mode

Force development mode with environment variable:
```bash
MIMIR_DEV=1 cargo run -p mimir --no-default-features
```

This uses a separate database at `com.mimir.app/dev/data/mimir.db` and enables debug logging.

## Code Style and Linting

### Rust

```bash
# Format code
cargo fmt

# Check formatting
cargo fmt -- --check

# Run linter
cargo clippy --all-targets --all-features

# Fix auto-fixable clippy warnings
cargo clippy --fix
```

### TypeScript/Vue

```bash
cd crates/mimir/frontend

# Lint
npm run lint

# Type check
npm run type-check
```

## Debugging

### Rust Backend

Use `tracing` for logging:
```rust
use tracing::{info, warn, error};

info!("Starting operation");
warn!("Something might be wrong");
error!("Operation failed: {}", err);
```

### Frontend

Use browser DevTools:
- Right-click in the app and select "Inspect Element"
- Use Console, Network, and Vue DevTools
- Check Application tab for local storage/database

### Database

```bash
# Connect to development database (macOS)
sqlite3 ~/Library/Application\ Support/com.mimir.app/dev/data/mimir.db

# Run SQL queries
.tables
SELECT * FROM campaigns;
.quit
```

## Additional Resources

- [Tauri Documentation](https://tauri.app/)
- [Vue 3 Documentation](https://vuejs.org/)
- [Rust Book](https://doc.rust-lang.org/book/)
- [Diesel ORM Guide](https://diesel.rs/guides/)

## Getting Help

- Check existing [GitHub Issues](https://github.com/mimir-dm/mimir/issues)
- Read the [CONTRIBUTING.md](CONTRIBUTING.md) guide
- Review crate-specific READMEs in `crates/*/README.md`
