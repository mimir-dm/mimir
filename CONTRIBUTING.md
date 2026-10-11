# Contributing to Mimir

Thank you for your interest in Mimir. This document explains how to contribute.

## Code of Conduct

By taking part in this project, you agree to keep a respectful and inclusive environment for all contributors.

## How to Contribute

### Reporting Bugs

Open an issue on GitHub with:
- A clear title
- Steps to reproduce the problem
- What you expected and what happened
- Your browser and the Mimir version (**Settings**, **About**)
- Logs or screenshots, if they help

### Suggesting Features

Open an issue with:
- A clear description of the feature
- The use cases
- Ideas for the implementation, if you have them

### Pull Request Process

1. **Fork the repository** and branch from `main`.
2. **Make your change** in the style of the code.
3. **Run the gates** (below). All must pass.
4. **Update the documentation** if you changed a feature or an API.
5. **Write a clear commit message** (see below).
6. **Open a pull request** that describes the change.

## Development Setup

See [DEVELOPMENT.md](DEVELOPMENT.md) for the full guide.

Quick start:
```bash
git clone https://github.com/mimir-dm/mimir.git
cd mimir

# Tools: Rust, trunk, Node.js (editor bundle and e2e), angreal
cargo install trunk --locked
pip install 'angreal>=2'

# Build the web app, run the server on fixture data, rebuild on change
angreal web serve        # http://127.0.0.1:8740
```

## Code Style

### Rust
- Format with `cargo fmt --all`. CI fails when `cargo fmt --all --check` finds a difference.
- Clippy must be clean on the host and for wasm. CI runs it with Rust 1.93.0 (pinned in `.github/workflows/ci.yml`). If a lint does not fit, add `#[allow(clippy::...)]` with a comment that gives the reason.
- Write doc comments for public items.
- Keep functions short and focused.

### Web App
- The web app is Rust (Leptos) in `crates/mimir-web`. Use Aurora components and tokens.
- JavaScript is only in the editor bundle (`crates/mimir-web/editor`) and the Playwright harness (`crates/mimir-web/web-e2e`).
- Give controls accessible names; the e2e specs find them by role and name.

### General Conventions
- **No emojis** in code, commit messages or technical writing
- **No references to Claude or Anthropic** in git commit messages
- Clear names, and comments where the code needs them
- Follow the patterns already in the code

## Gates

Run these before a pull request. CI runs them too.

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo clippy -p mimir-web --target wasm32-unknown-unknown -- -D warnings
angreal test unit       # cargo test --workspace -- --test-threads=1
angreal web e2e         # Playwright smoke suite against a real server
```

`--test-threads=1` avoids SQLite lock errors.

## Commit Messages

**Good:**
```
Add the turn order to the player display
Fix fog when a PC token is hidden
Update README with the server setup
```

**Bad:**
```
Fixed stuff
WIP
Update code
```

**Format:**
- Use the imperative mood ("Add feature", not "Added feature")
- Start with a capital letter
- No period at the end
- First line of 50 characters or less
- A blank line and a longer description if needed
- Reference the issue when there is one

## Project Structure

```
mimir/
├── crates/
│   ├── mimir-server/   # axum server: API, auth, live socket, MCP at /mcp, serves the web app
│   ├── mimir-web/      # Leptos web app (trunk), editor bundle, Playwright e2e
│   ├── mimir-wire/     # JSON types shared by the server and the web app
│   ├── mimir-core/     # Models, services, data access, migrations
│   ├── mimir-mcp/      # MCP tools for Claude; plugin/ is the Claude Code plugin
│   ├── mimir-print/    # PDF export with Typst
│   ├── mimir-render/   # 5etools JSON to HTML
│   └── mimir-mapgen/   # Procedural map generation (library and CLI)
└── docs/               # Documentation (mdBook)
```

## Documentation

- Update the docs when you add or change a feature.
- Keep the READMEs current.
- Add doc comments to new public items.

## Questions?

- Open an issue for discussion.
- Ask in the pull request.
- Look for similar issues and pull requests.

## License

By contributing to Mimir, you agree that your contributions are licensed under the license of the project (MIT).
