---
id: frontend-organizational-debt
level: initiative
title: "Frontend Organizational Debt Cleanup"
short_code: "MIMIR-I-0049"
created_at: 2026-01-28T03:41:58.927047+00:00
updated_at: 2026-01-28T03:46:56.124190+00:00
parent: MIMIR-V-0001
blocked_by: []
archived: false

tags:
  - "#initiative"
  - "#phase/design"


exit_criteria_met: false
estimated_complexity: M
strategy_id: NULL
initiative_id: frontend-organizational-debt
---

# Frontend Organizational Debt Cleanup Initiative

## Context

The Mimir frontend codebase (262 Vue/TypeScript files) exhibits a semi-modular architecture with clear feature boundaries but inconsistent internal organization. While the feature-based structure (`src/features/`) is solid, the codebase suffers from scattered component placement, ambiguous service vs. composable boundaries, and fragmented utilities.

**Current Structure:**
```
src/
├── app/           # Main app, router
├── components/    # Shared UI (INCONSISTENT - mixed patterns)
├── composables/   # Global composables (FRAGMENTED)
├── features/      # Feature modules (GOOD - consistent internal structure)
├── services/      # Global services (INCOMPLETE)
├── shared/        # Truly shared utilities
├── stores/        # Pinia stores (GOOD)
├── types/         # TypeScript types (GOOD)
├── utils/         # Global utilities (NEARLY EMPTY)
├── views/         # Page-level views
└── constants/     # Global constants
```

## Goals & Non-Goals

**Goals:**
- Establish consistent patterns for component, composable, and service placement
- Reduce cognitive load when navigating the codebase
- Improve discoverability of reusable code
- Standardize import paths using `@/` aliases
- Create clear boundaries between feature and shared code

**Non-Goals:**
- Rewriting existing functionality (pure reorganization)
- Changing the feature-based architecture (it works well)
- Adding new features during reorganization
- Modifying store or type organization (already consistent)

## Identified Issues

### Issue 1: Modal Components Scattered (High Severity)

Modal/dialog components are spread across 5+ locations:
- `src/components/` - Root level (BookManagementModal, CampaignManagementModal, CreateDocumentModal)
- `src/components/print/` - 8 print-related dialogs
- `src/components/campaigns/` - 3 campaign dialogs
- `src/features/campaigns/components/dashboard/` - AddCharacterModal
- `src/features/characters/components/levelup/` - LevelUpDialog

**Impact:** Hard to find, update, or maintain UI consistency.

### Issue 2: Composables Split Between Locations (High Severity)

- `src/composables/` - 10 global composables (useTokens, useFog, etc.)
- `src/features/{feature}/composables/` - Feature-specific (varies by feature)
- `src/shared/composables/` - 3 shared (useApiCall, useDataEvents, usePagination)

**Impact:** Unclear where to add new composables. Related composables split across directories.

### Issue 3: Services Underutilized (High Severity)

Only 3 global services exist (DocumentService, ModuleService, PrintService). Most features implement service-like logic in composables instead. No clear pattern for when to create a service vs. composable.

**Impact:** Business logic mixed into components, inconsistent data access patterns.

### Issue 4: Utils Fragmented Across 3 Locations (Medium Severity)

- `src/utils/` - Only 1 file (characterUtils.ts - 505 lines)
- `src/shared/utils/` - 6 files (api.ts, dataEvents.ts, debounce.ts, etc.)
- `src/features/sources/utils/` - textFormatting.ts, renderers/

**Impact:** Hard to discover and reuse utilities.

### Issue 5: Formatter Explosion (Medium Severity)

21 separate formatter files in `src/features/sources/formatters/` with similar patterns. Could be consolidated into a factory pattern or fewer files with grouped formatters.

### Issue 6: Inconsistent Import Paths (Medium Severity)

Mix of `@/` aliases and deep relative paths:
```typescript
// Inconsistent
import { CharacterCard } from '../../../components/characters'
import { PrintService } from '../../services/PrintService'

// Should be
import { CharacterCard } from '@/components/characters'
import { PrintService } from '@/services/PrintService'
```

**Impact:** Brittle paths that break on reorganization.

### Issue 7: Deeply Nested Components (Low Severity)

Some components are deeply nested (3+ levels):
- `src/features/characters/components/levelup/steps/` - 10 step components
- `src/features/sources/components/search/` - 24 table components

## What's Working Well

- **Feature-based structure** - Features are well-separated
- **Consistent feature internals** - Each feature follows `components/`, `composables/`, `views/` pattern
- **Type organization** - Centralized in `src/types/`
- **Store organization** - Clean `src/stores/` directory
- **Alias imports** - `@/` prefix widely used where applied

## Detailed Design

### Target Organization Pattern

```
src/
├── app/                      # Keep as-is
├── components/               # REORGANIZE: Shared components only
│   ├── dialogs/              # All shared dialogs/modals
│   ├── map/                  # Map-related shared components
│   ├── tokens/               # Token rendering
│   └── print/                # Print components
├── composables/              # CONSOLIDATE: All shared composables
│   ├── map/                  # Map-related (useFog, useTokens, etc.)
│   ├── windows/              # Window management
│   └── api/                  # API-related (from shared/)
├── services/                 # EXPAND: More services, clear patterns
├── shared/                   # REDUCE: Move most to appropriate location
│   └── utils/                # Keep generic utilities
├── features/                 # Keep structure, standardize internals
│   └── {feature}/
│       ├── components/       # Feature-specific components only
│       ├── composables/      # Feature-specific composables only
│       ├── services/         # Feature-specific services (optional)
│       └── views/            # Feature views
├── stores/                   # Keep as-is
├── types/                    # Keep as-is
└── utils/                    # REMOVE: Move to shared/utils
```

### Decision Criteria

**When to put in `src/components/`:**
- Used by 2+ features
- Generic UI patterns (modals, buttons, inputs)

**When to put in `src/composables/`:**
- Used by 2+ features
- Generic Vue composition patterns

**When to put in feature directory:**
- Only used by that feature
- Tightly coupled to feature's domain logic

**When to create a service:**
- Complex business logic
- Data transformation/API calls
- Stateless, reusable operations

**When to create a composable:**
- Vue-specific reactivity needed
- Component lifecycle hooks
- Shared reactive state

## Alternatives Considered

### Alternative 1: Domain-Driven Organization
Reorganize by domain (maps, characters, campaigns) instead of by type (components, services).

**Rejected because:** The current feature-based structure already accomplishes this. Would require massive reorganization.

### Alternative 2: Flat Structure
Remove all nesting, put everything in top-level directories with naming conventions.

**Rejected because:** Would lose the benefits of feature isolation. Harder to enforce boundaries.

### Alternative 3: No Change
Leave organization as-is and document the inconsistencies.

**Rejected because:** Debt will continue to accumulate. New developers will struggle with discoverability.

## Implementation Plan

### Phase 1: Shared Components Consolidation
- Consolidate scattered modals into `src/components/dialogs/`
- Move misplaced shared components
- Update imports

### Phase 2: Composables Consolidation
- Merge `src/shared/composables/` into `src/composables/`
- Organize by subdomain (map/, api/, windows/)
- Update imports

### Phase 3: Utils Consolidation
- Move `src/utils/characterUtils.ts` to appropriate location
- Consolidate `src/shared/utils/` organization
- Remove empty directories

### Phase 4: Import Path Standardization
- Convert all relative imports to `@/` aliases
- Verify no broken imports
- Add linting rule to enforce

### Phase 5: Formatter Consolidation (Optional)
- Analyze 21 formatter files for patterns
- Consolidate into factory or grouped modules
- Update consumers