# Contributing to Mimir

Thank you for your interest in contributing to Mimir! This document provides guidelines and instructions for contributing to the project.

## Code of Conduct

By participating in this project, you agree to maintain a respectful and inclusive environment for all contributors.

## How to Contribute

### Reporting Bugs

If you find a bug, please create an issue on GitHub with:
- A clear, descriptive title
- Steps to reproduce the issue
- Expected behavior vs actual behavior
- Your environment (OS, version, etc.)
- Any relevant logs or screenshots

### Suggesting Features

Feature requests are welcome! Please create an issue with:
- A clear description of the feature
- Use cases and benefits
- Any implementation ideas you have

### Pull Request Process

1. **Fork the repository** and create your branch from `main`
2. **Make your changes** following our code style guidelines
3. **Test your changes** - ensure all tests pass
4. **Update documentation** if you've changed APIs or added features
5. **Write a clear commit message** (see conventions below)
6. **Submit a pull request** with a description of your changes

## Development Setup

See [DEVELOPMENT.md](DEVELOPMENT.md) for detailed instructions on setting up your development environment.

Quick start:
```bash
# Clone the repository
git clone https://github.com/mimir-dm/mimir.git
cd mimir

# Install dependencies
cargo build
cd crates/mimir/frontend && npm install

# Run in development mode
cd ../.. && cargo tauri dev
```

## Code Style Guidelines

### Rust
- Follow standard Rust formatting: `cargo fmt`
- Run clippy and fix warnings: `cargo clippy --all-targets --all-features`
- Write doc comments for public APIs
- Use meaningful variable and function names
- Keep functions focused and reasonably sized

### TypeScript/Vue
- Follow the existing code style in the frontend
- Use TypeScript for type safety
- Write meaningful component and function names
- Keep components focused and composable

### General Conventions
- **No emojis** in code, commit messages, or technical writing
- **No references to Claude or Anthropic** in git commit messages
- Use clear, descriptive names
- Write self-documenting code with comments where necessary
- Follow existing patterns in the codebase

## Testing Requirements

Before submitting a pull request:

```bash
# Run Rust tests
cargo test --workspace

# Run frontend tests
cd crates/mimir/frontend && npm test

# Run unit tests only (faster, via angreal)
angreal test unit
```

All tests must pass before your PR can be merged.

## Commit Message Conventions

Write clear, concise commit messages:

**Good examples:**
```
Add campaign stage transition validation
Fix database migration error on fresh install
Update README with installation instructions
Refactor class service to use repository pattern
```

**Bad examples:**
```
Fixed stuff
WIP
Update code
Added feature
```

**Format:**
- Use imperative mood ("Add feature" not "Added feature")
- Start with a capital letter
- No period at the end
- First line should be 50 characters or less
- Add a blank line and detailed description if needed
- Reference issue numbers when applicable

## Project Structure

```
mimir/
├── crates/                      # Rust workspace
│   ├── mimir/                  # Main Tauri app (lib name: mimir_lib)
│   ├── mimir-core/             # Core business logic, DAL, models
│   ├── mimir-mcp/              # MCP server for Claude integration
│   └── mimir-print/            # PDF export / Typst rendering
├── docs/                        # Documentation (mdBook)
└── .metis/                     # Project management
```

## Documentation

- Update relevant documentation when adding or changing features
- Keep README files up to date
- Add doc comments to new public APIs

## Questions?

If you have questions about contributing, feel free to:
- Open an issue for discussion
- Ask in pull request comments
- Check existing issues and PRs for similar questions

## License

By contributing to Mimir, you agree that your contributions will be licensed under the same license as the project (MIT).
