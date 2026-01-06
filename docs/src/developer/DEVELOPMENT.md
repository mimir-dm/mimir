# Development Setup and Guide

This comprehensive guide covers everything you need to know about developing Mimir, from initial setup to advanced development workflows.

## Quick Links

- [Main Development Guide](../../DEVELOPMENT.md) - Quick reference for development
- [Contributing Guide](CONTRIBUTING.md) - How to contribute to the project
- [GitHub Repository](https://github.com/mimir-dm/mimir)

## Table of Contents

1. [Environment Setup](#environment-setup)
2. [Building and Running](#building-and-running)
3. [Project Architecture](#project-architecture)
4. [Development Workflows](#development-workflows)
5. [Testing Strategy](#testing-strategy)
6. [Debugging Techniques](#debugging-techniques)
7. [Database Management](#database-management)
8. [Frontend Development](#frontend-development)
9. [Tauri-Specific Considerations](#tauri-specific-considerations)
10. [Common Tasks](#common-tasks)

## Environment Setup

### System Requirements

- **macOS**: 10.15 (Catalina) or later
- **Windows**: Windows 10 or later
- **Linux**: Any modern distribution with glibc 2.31+

### Required Tools

#### Rust Toolchain
```bash
# Install rustup (Rust version manager)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Verify installation
rustc --version
cargo --version
```

#### Node.js and npm
```bash
# macOS (via Homebrew)
brew install node

# Windows (via installer)
# Download from https://nodejs.org

# Linux (via package manager)
curl -fsSL https://deb.nodesource.com/setup_18.x | sudo -E bash -
sudo apt-get install -y nodejs

# Verify installation
node --version  # Should be v18 or higher
npm --version
```

### Platform-Specific Dependencies

#### macOS Setup
```bash
# Install Xcode Command Line Tools
xcode-select --install

# Install Homebrew (if not already installed)
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"

# Optional: Install additional development tools
brew install git gh
```

#### Linux Setup (Ubuntu/Debian)
```bash
# Update package lists
sudo apt-get update

# Install required dependencies
sudo apt-get install -y \
  libwebkit2gtk-4.1-dev \
  build-essential \
  curl \
  wget \
  file \
  libssl-dev \
  libgtk-3-dev \
  libayatana-appindicator3-dev \
  librsvg2-dev \
  patchelf

# Optional: Install git and gh
sudo apt-get install -y git gh
```

#### Linux Setup (Fedora)
```bash
sudo dnf install \
  webkit2gtk4.1-devel \
  openssl-devel \
  curl \
  wget \
  file \
  libappindicator-gtk3-devel \
  librsvg2-devel \
  patchelf

sudo dnf groupinstall "C Development Tools and Libraries"
```

#### Windows Setup
1. Install [Visual Studio Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/)
2. Install [Git for Windows](https://git-scm.com/download/win)
3. Install [Node.js](https://nodejs.org/)
4. Install Rust via [rustup-init.exe](https://rustup.rs/)

### Optional Development Tools

```bash
# Tauri CLI (faster startup for dev mode)
cargo install tauri-cli

# Angreal (test runner)
pip install 'angreal>=2'

# Diesel CLI (database migrations)
cargo install diesel_cli --no-default-features --features sqlite

# GitHub CLI (for PR management)
brew install gh  # macOS
# or download from https://cli.github.com/
```

### IDE Setup

#### VS Code (Recommended)
Install these extensions:
- **rust-analyzer** - Rust language support
- **Tauri** - Tauri development tools
- **Volar** - Vue 3 language support
- **ESLint** - JavaScript/TypeScript linting
- **Error Lens** - Inline error messages

Settings:
```json
{
  "rust-analyzer.checkOnSave.command": "clippy",
  "editor.formatOnSave": true,
  "[rust]": {
    "editor.defaultFormatter": "rust-lang.rust-analyzer"
  },
  "[typescript]": {
    "editor.defaultFormatter": "esbenp.prettier-vscode"
  },
  "[vue]": {
    "editor.defaultFormatter": "Vue.volar"
  }
}
```

#### IntelliJ IDEA / CLion
- Install Rust plugin
- Install Vue.js plugin
- Enable Tauri support

## Building and Running

### Initial Build

```bash
# Clone repository
git clone https://github.com/mimir-dm/mimir.git
cd mimir

# Build Rust workspace
cargo build

# Install frontend dependencies
cd crates/mimir-dm/frontend
npm install
cd ../../..
```

### Development Mode

```bash
# Run with hot reload (from project root)
cd crates/mimir-dm
cargo tauri dev

# Or using installed tauri-cli
cd crates/mimir-dm
tauri dev
```

This starts:
- Vite dev server for frontend (hot reload)
- Rust backend in debug mode
- Application window with DevTools available

### Production Build

```bash
# Build optimized release
cd crates/mimir-dm
cargo tauri build

# Output locations:
# macOS: target/release/bundle/dmg/
# Windows: target/release/bundle/msi/
# Linux: target/release/bundle/deb/ and target/release/bundle/appimage/
```

## Project Architecture

### Workspace Structure

```
mimir/
├── Cargo.toml                 # Workspace configuration
├── crates/
│   ├── mimir-dm/             # Main application
│   ├── mimir-dm-core/        # Business logic
│   ├── mimir-dm-llm/         # LLM abstraction
│   └── mimir-5etools-splitter/ # Data processing
├── docs/                      # Documentation
├── data/                      # D&D reference data
└── .metis/                   # Project management
```

### Architecture Layers

**Frontend Layer (Vue 3 + TypeScript)**
- Views: Page-level components
- Components: Reusable UI components
- Stores: Pinia state management
- Services: API wrappers for Tauri commands

**Application Layer (Tauri + Rust)**
- Commands: IPC handlers
- Services: Application-level business logic
- State: Application state management

**Domain Layer (mimir-dm-core)**
- Models: Domain entities
- Services: Business logic
- DAL: Data access layer
- Migrations: Database schema

**Infrastructure Layer**
- Database: SQLite with Diesel ORM
- LLM: Ollama integration
- File System: Campaign data storage

### Data Flow

```
User Interaction
      ↓
Vue Component
      ↓
Pinia Store / API Service
      ↓
Tauri Command (IPC)
      ↓
Command Handler
      ↓
Core Service
      ↓
Repository (DAL)
      ↓
Database
```

## Development Workflows

### Feature Development

1. **Create Feature Branch**
```bash
git checkout -b feature/my-feature
```

2. **Implement Feature**
- Add backend logic in `mimir-dm-core`
- Add Tauri command in `mimir-dm/src/commands`
- Add frontend UI in `mimir-dm/frontend/src`

3. **Test Locally**
```bash
cargo test --workspace
cd crates/mimir-dm/frontend && npm test
```

4. **Commit and Push**
```bash
git add .
git commit -m "Add feature description"
git push origin feature/my-feature
```

### Bug Fix Workflow

1. **Reproduce the Bug**
- Create a failing test
- Document reproduction steps

2. **Fix the Issue**
- Implement fix
- Verify test passes

3. **Test Edge Cases**
- Add additional tests
- Manual testing

4. **Submit PR**

### Adding a New Page

1. **Create Vue Component**
```bash
# In crates/mimir-dm/frontend/src/views/
touch MyNewView.vue
```

2. **Add Route**
```typescript
// In crates/mimir-dm/frontend/src/app/router.ts
{
  path: '/my-new-page',
  name: 'MyNewPage',
  component: () => import('@/views/MyNewView.vue')
}
```

3. **Add Navigation**
```vue
<!-- In layout component -->
<router-link to="/my-new-page">My New Page</router-link>
```

### Adding a New Tauri Command

1. **Create Command Function**
```rust
// In crates/mimir-dm/src/commands/my_commands.rs
#[tauri::command]
pub async fn my_new_command(
    db_service: State<'_, Arc<DatabaseService>>,
    param: String,
) -> Result<String, String> {
    let mut conn = db_service.get_connection()
        .map_err(|e| format!("Database error: {}", e))?;

    // Business logic here
    Ok(format!("Processed: {}", param))
}
```

2. **Register Command**
```rust
// In crates/mimir-dm/src/main.rs
.invoke_handler(tauri::generate_handler![
    commands::my_new_command,
    // ... other commands
])
```

3. **Call from Frontend**
```typescript
// In frontend service
import { invoke } from '@tauri-apps/api/core';

export async function myNewCommand(param: string): Promise<string> {
  return await invoke<string>('my_new_command', { param });
}
```

## Testing Strategy

### Rust Testing

#### Unit Tests
```rust
// In same file as implementation
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feature() {
        let result = my_function();
        assert_eq!(result, expected);
    }
}
```

#### Integration Tests
```rust
// In tests/integration_test.rs
use mimir_dm_core::*;

#[test]
fn test_database_workflow() {
    let mut conn = establish_connection(":memory:").unwrap();
    run_migrations(&mut conn).unwrap();

    // Test full workflow
}
```

#### Running Rust Tests
```bash
# All tests
cargo test --workspace

# Specific crate
cargo test -p mimir-dm-core

# Specific test
cargo test test_campaign_creation

# With output
cargo test -- --nocapture

# Unit tests only (via angreal)
angreal test unit
```

### Frontend Testing

#### Component Tests
```typescript
// In crates/mimir-dm/frontend/src/components/__tests__/
import { describe, it, expect } from 'vitest';
import { mount } from '@vue/test-utils';
import MyComponent from '../MyComponent.vue';

describe('MyComponent', () => {
  it('renders correctly', () => {
    const wrapper = mount(MyComponent, {
      props: { title: 'Test' }
    });
    expect(wrapper.text()).toContain('Test');
  });
});
```

#### Running Frontend Tests
```bash
cd crates/mimir-dm/frontend

# Run tests
npm test

# Watch mode
npm test -- --watch

# Coverage
npm run test:coverage

# UI mode
npm run test:ui
```

## Debugging Techniques

### Rust Debugging

#### Print Debugging
```rust
use tracing::{info, debug, warn, error};

debug!("Variable value: {:?}", my_var);
info!("Operation started");
warn!("Unusual condition: {}", condition);
error!("Operation failed: {}", err);
```

#### VS Code Debugging
1. Install CodeLLDB extension
2. Add `.vscode/launch.json`:
```json
{
  "version": "0.2.0",
  "configurations": [
    {
      "type": "lldb",
      "request": "launch",
      "name": "Debug Tauri",
      "cargo": {
        "args": ["build", "--manifest-path=crates/mimir-dm/Cargo.toml"]
      },
      "cwd": "${workspaceFolder}"
    }
  ]
}
```

### Frontend Debugging

#### Browser DevTools
- Right-click in app → "Inspect Element"
- Console tab for logs
- Network tab for Tauri commands
- Vue DevTools for component inspection

#### Console Logging
```typescript
console.log('Debug:', variable);
console.warn('Warning:', condition);
console.error('Error:', error);
```

### Database Debugging

```bash
# Open development database (macOS)
sqlite3 ~/Library/Application\ Support/com.mimir.mimir-test/mimir.db

# Common commands
.tables                # List tables
.schema campaigns      # Show table structure
SELECT * FROM campaigns;  # Query data
.quit                  # Exit
```

## Database Management

### Migrations

#### Creating a Migration
```bash
cd crates/mimir-dm-core
diesel migration generate add_new_feature

# Edit migrations/YYYY-MM-DD-HHMMSS_add_new_feature/up.sql
# Edit migrations/YYYY-MM-DD-HHMMSS_add_new_feature/down.sql
```

#### Running Migrations
```bash
# Apply migrations
diesel migration run

# Rollback last migration
diesel migration revert

# Test migration cycle
diesel migration redo
```

#### Development Database Location

- **macOS**: `~/Library/Application Support/com.mimir.mimir-test/mimir.db`
- **Linux**: `~/.local/share/com.mimir.mimir-test/mimir.db`
- **Windows**: `%APPDATA%\com.mimir.mimir-test\mimir.db`

### Resetting Development Database

```bash
# macOS
rm -rf ~/Library/Application\ Support/com.mimir.mimir-test/

# Linux
rm -rf ~/.local/share/com.mimir.mimir-test/

# Windows
# Delete folder: %APPDATA%\com.mimir.mimir-test\

# Restart app to recreate
cargo tauri dev
```

## Frontend Development

### Hot Reload

Changes to Vue components, TypeScript, and CSS hot reload automatically:
- Save file
- Browser refreshes automatically
- State may reset depending on change

### State Management (Pinia)

```typescript
// Define store
import { defineStore } from 'pinia';

export const useCampaignStore = defineStore('campaign', () => {
  const campaigns = ref<Campaign[]>([]);

  async function loadCampaigns() {
    campaigns.value = await invoke('list_campaigns');
  }

  return { campaigns, loadCampaigns };
});

// Use in component
const campaignStore = useCampaignStore();
await campaignStore.loadCampaigns();
```

### Calling Tauri Commands

```typescript
import { invoke } from '@tauri-apps/api/core';

// Basic invocation
const result = await invoke<string>('command_name', { param: value });

// With error handling
try {
  const data = await invoke<Campaign>('get_campaign', { id: campaignId });
} catch (error) {
  console.error('Failed to get campaign:', error);
}
```

## Tauri-Specific Considerations

### IPC Communication

Commands run in separate thread from UI:
- Always async
- Data must be serializable
- Large data may have performance impact

### File System Access

```rust
// Use tauri::api::path for cross-platform paths
use tauri::api::path::{app_data_dir, app_config_dir};

let data_dir = app_data_dir(&config).unwrap();
```

### Window Management

```typescript
import { getCurrent } from '@tauri-apps/api/window';

const appWindow = getCurrent();
await appWindow.setTitle('New Title');
await appWindow.maximize();
```

## Common Tasks

### Updating Dependencies

```bash
# Rust dependencies
cargo update

# Frontend dependencies
cd crates/mimir-dm/frontend
npm update
```

### Formatting Code

```bash
# Rust
cargo fmt

# TypeScript/Vue
cd crates/mimir-dm/frontend
npm run lint
```

### Running Clippy

```bash
cargo clippy --all-targets --all-features

# Fix auto-fixable issues
cargo clippy --fix
```

### Type Checking Frontend

```bash
cd crates/mimir-dm/frontend
npm run type-check
```

## Performance Optimization

### Profiling Rust Code

```bash
cargo build --release --profile=profiling
# Use profiling tools like flamegraph or perf
```

### Frontend Performance

- Use Vue DevTools performance tab
- Check bundle size: `npm run build -- --analyze`
- Lazy load routes and heavy components

## Additional Resources

- [Tauri Documentation](https://tauri.app/)
- [Vue 3 Documentation](https://vuejs.org/)
- [Rust Book](https://doc.rust-lang.org/book/)
- [Diesel Guide](https://diesel.rs/guides/)
- [Pinia Documentation](https://pinia.vuejs.org/)

## Getting Help

- Check [CONTRIBUTING.md](CONTRIBUTING.md) for contribution guidelines
- Search [GitHub Issues](https://github.com/mimir-dm/mimir/issues)
- Review crate READMEs for detailed architecture info
- Ask questions in pull request comments
