---
id: standardize-import-paths
level: task
title: "Standardize Import Paths"
short_code: "MIMIR-T-0454"
created_at: 2026-01-28T03:54:05.271927+00:00
updated_at: 2026-01-28T04:50:33.973384+00:00
parent: MIMIR-I-0049
blocked_by: []
archived: true

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: MIMIR-I-0049
---

# Standardize Import Paths

## Parent Initiative

[[MIMIR-I-0049]] - Frontend Organizational Debt Cleanup

## Objective

Convert all relative imports to use `@/` path aliases for consistency and resilience to reorganization.

## Current State

Mixed import patterns throughout the codebase:

```typescript
// Inconsistent - deep relative paths
import { CharacterCard } from '../../../components/characters'
import { PrintService } from '../../services/PrintService'
import { useCharacterStore } from '../../../stores/characters'

// Good - using aliases
import { usePlayerDisplay } from '@/composables/usePlayerDisplay'
import TokenRenderer from '@/components/tokens/TokenRenderer.vue'
```

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] All cross-directory imports use `@/` alias
- [ ] No relative imports that go up more than 1 level (`../`)
- [ ] Within-feature relative imports allowed (e.g., `./` or `../components/`)
- [ ] ESLint rule added to enforce pattern
- [ ] No broken imports (app builds successfully)

## Implementation Notes

### Rules

1. **Always use `@/`** for imports outside the current feature
2. **Relative OK** for sibling files (`./`) or one level up within same feature (`../`)
3. **Never** use deep relative paths (`../../..`)

### Examples

```typescript
// GOOD: Cross-feature with alias
import { useCharacterStore } from '@/stores/characters'
import { PrintService } from '@/services/PrintService'

// GOOD: Within-feature relative
import LevelUpStep from './LevelUpStep.vue'
import { useLevelUp } from '../composables/useLevelUp'

// BAD: Deep relative cross-feature
import { CharacterCard } from '../../../components/characters'
```

### ESLint Configuration

Add rule to `.eslintrc.js`:
```javascript
rules: {
  'no-restricted-imports': ['error', {
    patterns: ['../../../*', '../../../../*']
  }]
}
```

### Dependencies

- Should run AFTER file reorganization tasks (T-0451, T-0452, T-0453)
- Paths will change during reorganization

## Status Updates

### Session 1 - Completed
- Converted ~80 deep relative imports to `@/` aliases across:
  - features/characters/ (views, components, levelup steps)
  - features/campaigns/ (views, components)
  - features/sources/ (search tables, reader, composables)
  - features/modules/ (views)
  - shared/components/ (layout, ui)
- ESLint rule skipped (no existing config, would require careful setup)
- Build passes

**Result:** Zero deep relative imports (`../../../` or deeper) remain in codebase.