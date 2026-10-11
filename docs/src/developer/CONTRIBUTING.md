# Contributing to Mimir

This guide covers how to contribute to Mimir: the workflow, the code style, the tests and the pull request process.

## Quick Links

- [Development Setup](./DEVELOPMENT.md): build, run and test
- [Architecture](./ARCHITECTURE.md): the crates and how they fit
- [GitHub Repository](https://github.com/mimir-dm/mimir)
- [Issue Tracker](https://github.com/mimir-dm/mimir/issues)

## Getting Started

1. **Find an issue.** Look for `good first issue` or `help wanted`.
2. **Comment on the issue** to say that you work on it.
3. **Fork and clone** the repository.
4. **Set up your environment** with the [Development Setup](./DEVELOPMENT.md).
5. **Make your change** on a branch.
6. **Open a pull request.**

## Development Workflow

### Branches

- `main` is the stable branch.
- Name a branch `feature/description` or `fix/description`.
- Branch from `main`.

```bash
git checkout main
git pull origin main
git checkout -b feature/my-new-feature
```

### Before You Submit

Run the gates that CI runs:

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo clippy -p mimir-web --target wasm32-unknown-unknown -- -D warnings
angreal test unit
angreal web e2e
```

For a change to the screens, also run `angreal web screenshots` before and after, and compare with `angreal web screenshots-diff`.

## Code Style and Standards

### Rust

- Format with `cargo fmt --all`. CI fails when `cargo fmt --all --check` finds a difference.
- Clippy must be clean, on the host and for wasm (`-p mimir-web --target wasm32-unknown-unknown`). CI runs clippy with Rust 1.93.0 (pinned in `.github/workflows/ci.yml`). If a lint does not fit, add `#[allow(clippy::...)]` with a comment that gives the reason.
- Write doc comments for public items.
- Keep functions short and focused.

### Server (`mimir-server`)

- Handlers call `mimir-core` services, never the DAL.
- Request and answer types live in `mimir-wire`.
- Each route in `api/mod.rs` names its `Access` (`Dm` or `Player`). A `Player` handler limits a player to their own scope.

### Web App (`mimir-web`)

- Use Aurora components and tokens. Add a class to `style/mimir.css` only when Aurora has none.
- Keep logic that can run on the host in plain functions with `#[cfg(test)]` tests, so `cargo test` covers it.
- Give controls accessible names (labels, `aria-label`, `title`). The e2e specs find controls by role and name.
- JavaScript is only in `crates/mimir-web/editor` (the TipTap bundle) and `crates/mimir-web/web-e2e` (Playwright).

### General Conventions

- **No emojis** in code, commit messages or technical writing.
- **No references to Claude or Anthropic** in commit messages.
- Plain, short language in docs, comments and commits.
- Follow the patterns already in the code.

## Testing Guidelines

- A new API route gets a test in `crates/mimir-server/tests/`.
- A new service function gets a unit test in `mimir-core`.
- A new screen or flow gets an e2e spec, or a step in an existing one, in `crates/mimir-web/web-e2e/specs/`. Add a new spec file to the list in `.angreal/task_web.py` and in `ci.yml`.
- Rust tests run with `--test-threads=1` to avoid SQLite lock errors.

See [Development Setup: Tests](./DEVELOPMENT.md#tests).

## Commit Messages

- Use the imperative mood ("Add feature", not "Added feature").
- Start with a capital letter. No period at the end.
- Keep the first line to 50 characters or less.
- Add a blank line and a longer description if needed.
- Reference the issue when there is one.

**Good:**
```
Add the turn order to the player display
Fix fog when a PC token is hidden
```

**Bad:**
```
Fixed stuff
WIP
```

## Pull Request Process

### 1. Prepare

- All gates pass.
- The docs describe the change.
- The description says what the PR does and why.

### 2. Description Template

```markdown
## Description
What this PR does

## Related Issues
Fixes #123

## Changes
- Added X
- Fixed Y

## Testing
- [ ] Rust tests added or updated
- [ ] e2e spec added or updated
- [ ] Tested in a desktop browser and on a tablet width

## Checklist
- [ ] Gates pass (fmt, clippy host and wasm, tests, e2e)
- [ ] Docs updated
```

### 3. Review

- A maintainer reviews the PR.
- Answer the feedback and push changes.

### 4. After Approval

- The PR is merged to `main`.
- The change ships in the next release.

## CI and Releases

### Continuous Integration

CI (`.github/workflows/ci.yml`) runs on pushes and pull requests to `main`:

- **Format and Clippy**: fmt, clippy on the host, clippy for wasm (Rust 1.93.0)
- **Rust Tests**: `angreal test unit`
- **Web App**: builds `mimir-wire` for wasm, builds the bundle with `trunk build --release`, runs the Playwright smoke suite
- **Integration Tests**: the SRD smoke test, only on pushes to `main`
- **Coverage**: cargo-tarpaulin on `mimir-core`. It reports and does not fail CI. The 50% floor in `tarpaulin.toml` applies to the local `angreal test coverage`.

### Releases

A tag matching `v*.*.*` runs `.github/workflows/release.yml`. It builds `mimir-mapgen` for each platform and makes a **draft** GitHub release with the binaries.

### Documentation

Pushes to `main` run `.github/workflows/docs.yml`. It builds the mdBook and deploys it to GitHub Pages.

## Release Process

### Version Numbers

Mimir follows [Semantic Versioning](https://semver.org/): MAJOR.MINOR.PATCH.

### Steps

1. Update the version in every crate `Cargo.toml` and in `crates/mimir-mcp/plugin/.claude-plugin/plugin.json`. Bump once per release.
2. Tag: `git tag vX.Y.Z`.
3. Push the tag: `git push origin vX.Y.Z`.
4. Check and publish the draft release on GitHub.

## Questions and Help

- Read this guide and the [Development Setup](./DEVELOPMENT.md).
- Search the issues, or open one.
- Ask in GitHub Discussions or in the pull request.

## License

By contributing to Mimir, you agree that your contributions are licensed under the license of the project (MIT).
