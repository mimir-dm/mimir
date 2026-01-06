# Contributing to Mimir

Welcome to the Mimir contributor guide! This comprehensive guide covers everything you need to know about contributing to the project.

## Quick Links

- [Main Contributing Guide](../../CONTRIBUTING.md) - Quick reference for contributors
- [Development Setup](DEVELOPMENT.md) - Detailed development environment setup
- [GitHub Repository](https://github.com/mimir-dm/mimir)
- [Issue Tracker](https://github.com/mimir-dm/mimir/issues)

## Table of Contents

1. [Getting Started](#getting-started)
2. [Development Workflow](#development-workflow)
3. [Code Style and Standards](#code-style-and-standards)
4. [Testing Guidelines](#testing-guidelines)
5. [Pull Request Process](#pull-request-process)
6. [Architecture Overview](#architecture-overview)
7. [Design Principles](#design-principles)
8. [Release Process](#release-process)

## Getting Started

### First-Time Contributors

Welcome! Here's how to make your first contribution:

1. **Find an issue** - Look for issues labeled `good first issue` or `help wanted`
2. **Comment on the issue** - Let us know you're working on it
3. **Fork and clone** - Create your own fork and clone it locally
4. **Set up your environment** - Follow the [Development Setup](DEVELOPMENT.md) guide
5. **Make your changes** - Implement your fix or feature
6. **Submit a PR** - Create a pull request with your changes

### Finding Work

- **Good First Issues** - Tagged issues suitable for newcomers
- **Help Wanted** - Issues where we'd appreciate community help
- **Bug Reports** - Always welcome to fix reported bugs
- **Feature Requests** - Check if a feature request needs implementation
- **Documentation** - Help improve our docs

## Development Workflow

### Branch Strategy

- `main` - Stable release branch
- Feature branches - `feature/description` or `fix/description`
- Always branch from `main`

```bash
git checkout main
git pull origin main
git checkout -b feature/my-new-feature
```

### Making Changes

1. **Write code** following our style guidelines
2. **Test your changes** locally
3. **Commit frequently** with clear messages
4. **Push to your fork** regularly
5. **Keep your branch updated** with main

```bash
# Keep your branch up to date
git fetch origin
git rebase origin/main

# Or merge if you prefer
git merge origin/main
```

### Before Submitting

Checklist before creating a pull request:

- [ ] All tests pass locally
- [ ] Code follows style guidelines
- [ ] New tests added for new functionality
- [ ] Documentation updated if needed
- [ ] Commit messages follow conventions
- [ ] No merge conflicts with main
- [ ] Changes are focused and minimal

## Code Style and Standards

### Rust Code Style

**Formatting:**
```bash
# Format all Rust code
cargo fmt

# Check formatting without changing
cargo fmt -- --check
```

**Linting:**
```bash
# Run clippy
cargo clippy --all-targets --all-features

# Fix auto-fixable warnings
cargo clippy --fix
```

**Best Practices:**
- Use meaningful variable and function names
- Write doc comments for public APIs
- Keep functions focused and reasonably sized
- Prefer explicit error handling over unwrap()
- Use `Result<T, E>` for operations that can fail
- Follow the Rust API Guidelines

**Example:**
```rust
/// Retrieves a campaign by ID from the database.
///
/// # Arguments
/// * `conn` - Database connection
/// * `id` - Campaign ID to retrieve
///
/// # Returns
/// * `Ok(Campaign)` - The campaign if found
/// * `Err(String)` - Error message if not found
pub fn get_campaign(conn: &mut SqliteConnection, id: i32) -> Result<Campaign, String> {
    campaigns::table
        .find(id)
        .first(conn)
        .map_err(|e| format!("Failed to get campaign: {}", e))
}
```

### TypeScript/Vue Code Style

**Type Safety:**
- Always use TypeScript, not JavaScript
- Define interfaces for all data structures
- Avoid `any` type unless absolutely necessary
- Use strict mode in tsconfig.json

**Vue Best Practices:**
- Use Composition API with `<script setup>`
- Keep components focused and single-purpose
- Props should have types and validators
- Emit events with proper typing
- Use composables for reusable logic

**Example:**
```vue
<script setup lang="ts">
import { ref, computed } from 'vue';
import type { Campaign } from '@/types/campaigns';

interface Props {
  campaign: Campaign;
  editable?: boolean;
}

const props = withDefaults(defineProps<Props>(), {
  editable: false
});

const emit = defineEmits<{
  save: [campaign: Campaign];
  cancel: [];
}>();

const localCampaign = ref({ ...props.campaign });

const hasChanges = computed(() => {
  return JSON.stringify(localCampaign.value) !== JSON.stringify(props.campaign);
});

function handleSave() {
  emit('save', localCampaign.value);
}
</script>
```

### General Conventions

**Naming:**
- Rust: `snake_case` for functions/variables, `PascalCase` for types
- TypeScript: `camelCase` for variables/functions, `PascalCase` for classes/interfaces
- Files: Match the primary export name

**Comments:**
- Write self-documenting code
- Add comments for complex logic
- Doc comments for public APIs
- No commented-out code in commits

**Prohibited:**
- No emojis in code, comments, or commit messages
- No references to Claude or Anthropic in commit messages
- No hardcoded secrets or credentials
- No console.log in production code (use logging frameworks)

## Testing Guidelines

### Rust Testing

**Unit Tests:**
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_campaign_creation() {
        let campaign = Campaign::new("Test Campaign");
        assert_eq!(campaign.name, "Test Campaign");
        assert_eq!(campaign.stage, CampaignStage::Spark);
    }
}
```

**Integration Tests:**
Place in `tests/` directory:
```rust
use mimir_dm_core::{establish_connection, run_migrations};

#[test]
fn test_database_integration() {
    let mut conn = establish_connection(":memory:").unwrap();
    run_migrations(&mut conn).unwrap();
    // Test database operations
}
```

### Frontend Testing

**Component Tests:**
```typescript
import { describe, it, expect } from 'vitest';
import { mount } from '@vue/test-utils';
import CampaignCard from '@/components/CampaignCard.vue';

describe('CampaignCard', () => {
  it('renders campaign name', () => {
    const wrapper = mount(CampaignCard, {
      props: {
        campaign: { id: 1, name: 'Test Campaign' }
      }
    });
    expect(wrapper.text()).toContain('Test Campaign');
  });
});
```

### Running Tests

```bash
# All tests
cargo test --workspace
cd crates/mimir-dm/frontend && npm test

# Unit tests only
angreal test unit

# With coverage
npm run test:coverage

# Specific test
cargo test test_campaign_creation
```

## Pull Request Process

### 1. Prepare Your PR

- Ensure all tests pass
- Update documentation
- Write a clear PR description
- Reference related issues

### 2. PR Description Template

```markdown
## Description
Brief description of what this PR does

## Related Issues
Fixes #123
Related to #456

## Changes Made
- Added feature X
- Fixed bug Y
- Updated documentation for Z

## Testing
- [ ] Unit tests added/updated
- [ ] Integration tests added/updated
- [ ] Manually tested on macOS/Windows/Linux

## Checklist
- [ ] Code follows style guidelines
- [ ] Self-review completed
- [ ] Documentation updated
- [ ] No new warnings
- [ ] Tests pass locally
```

### 3. Review Process

- Maintainers will review your PR
- Address feedback and requested changes
- Keep discussion focused and professional
- Be patient - reviews may take a few days

### 4. After Approval

- PR will be merged to main
- Your contribution will be included in the next release
- Thank you for contributing!

## Architecture Overview

### System Architecture

Mimir follows a clean architecture pattern with clear separation of concerns:

```
┌─────────────────────────────────────────┐
│         Frontend (Vue 3 + TS)          │
│  ┌──────────────────────────────────┐  │
│  │ Components, Views, Stores        │  │
│  └──────────────────────────────────┘  │
└─────────────────┬───────────────────────┘
                  │ Tauri IPC
┌─────────────────▼───────────────────────┐
│      Tauri Desktop Shell (Rust)        │
│  ┌──────────────────────────────────┐  │
│  │ Commands, Services, State        │  │
│  └──────────────┬───────────────────┘  │
└─────────────────┼───────────────────────┘
                  │
┌─────────────────▼───────────────────────┐
│      mimir-dm-core (Business Logic)    │
│  ┌──────────────────────────────────┐  │
│  │ Services, DAL, Domain Models     │  │
│  └──────────────┬───────────────────┘  │
└─────────────────┼───────────────────────┘
                  │
┌─────────────────▼───────────────────────┐
│      Database (SQLite + Diesel)        │
└─────────────────────────────────────────┘
```

### Key Crates

- **mimir-dm** - Tauri app shell and command handlers
- **mimir-dm-core** - Business logic, domain models, database
- **mimir-dm-llm** - LLM provider abstraction
- **mimir-5etools-splitter** - Data processing utility

See individual crate READMEs for detailed information.

## Design Principles

1. **Local-First** - All data stored locally, no cloud dependencies
2. **Privacy-First** - No telemetry, no tracking, user data stays local
3. **Type Safety** - Strong typing in both Rust and TypeScript
4. **Separation of Concerns** - Clear boundaries between layers
5. **Testability** - Write testable code with good test coverage
6. **Domain-Driven Design** - Model the D&D campaign management domain
7. **Progressive Enhancement** - Core features work without LLM
8. **Cross-Platform** - Native experience on Windows, macOS, Linux

## Release Process

### Version Numbering

We follow [Semantic Versioning](https://semver.org/):
- MAJOR.MINOR.PATCH (e.g., 0.1.0)
- MAJOR: Breaking changes
- MINOR: New features, backwards compatible
- PATCH: Bug fixes, backwards compatible

### Release Steps

1. Update version in all `Cargo.toml` files
2. Update version in `package.json`
3. Update version in `tauri.conf.json`
4. Create git tag: `git tag vX.Y.Z`
5. Push tag: `git push origin vX.Y.Z`
6. GitHub Actions builds and creates release

## Questions and Help

### Getting Help

- **Documentation** - Check this guide and [DEVELOPMENT.md](DEVELOPMENT.md)
- **Issues** - Search existing issues or create a new one
- **Discussions** - Use GitHub Discussions for questions
- **Pull Requests** - Ask questions in PR comments

### Maintainer Response Times

- Issues: Usually within 1-3 days
- PRs: Usually within 3-7 days
- Critical bugs: Within 24 hours when possible

## Recognition

Contributors are recognized in:
- Git commit history
- Release notes
- Project documentation
- GitHub contributors page

Thank you for contributing to Mimir!
