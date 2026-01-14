---
id: vue-component-unit-test-coverage
level: task
title: "Vue Component Unit Test Coverage"
short_code: "MIMIR-T-0339"
created_at: 2026-01-14T01:50:56.150964+00:00
updated_at: 2026-01-14T14:33:07.235054+00:00
parent: MIMIR-I-0039
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: MIMIR-I-0039
---

# Vue Component Unit Test Coverage

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[MIMIR-I-0039]]

## Objective

Add Vitest unit tests for key Vue components in the frontend, focusing on interactive components with business logic.

## Scope

**Target: `crates/mimir-dm/frontend/src/`**

Existing infrastructure:
- Vitest configured in `vitest.config.ts`
- Test setup with Tauri mocks in `src/test/setup.ts`
- Mock utilities in `src/test/utils/mockTauri.ts`

**Priority Components to Test:**
1. Forms with validation logic
2. List components with filtering/sorting
3. Modal/dialog components
4. Components with complex state management
5. Reusable UI components with props

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [x] Identify top 10 components needing tests (by complexity/usage)
- [x] Add unit tests for at least 5 priority components
- [x] Tests cover props handling, events, and user interactions
- [x] Tests use existing mock utilities (`createMockModule`, etc.)
- [x] Tests follow Vue Testing Library best practices
- [x] All tests pass with `npm run test`
- [x] Document component testing patterns

## Implementation Notes

### Technical Approach

**Test Pattern:**
```typescript
import { mount } from '@vue/test-utils'
import { describe, it, expect, vi } from 'vitest'
import CampaignList from './CampaignList.vue'
import { createMockCampaign } from '@/test/utils/mockTauri'

describe('CampaignList', () => {
  it('renders campaigns', () => {
    const campaigns = [createMockCampaign(), createMockCampaign()]
    const wrapper = mount(CampaignList, {
      props: { campaigns }
    })
    expect(wrapper.findAll('[data-testid="campaign-item"]')).toHaveLength(2)
  })

  it('emits select event on click', async () => {
    // ...
  })
})
```

**Component Categories:**
- **Views**: CampaignView, ModuleView, etc.
- **Forms**: CampaignForm, ModuleForm, CharacterForm
- **Lists**: CampaignList, ModuleList, CharacterList
- **Dialogs**: ConfirmDialog, CreateDialog, etc.

### Key Files
- `vitest.config.ts` - Test configuration
- `src/test/setup.ts` - Global test setup
- `src/test/utils/mockTauri.ts` - Mock factories

### Risk Considerations
- Vue 3 composition API testing patterns
- Pinia store mocking in component tests
- Router mock setup for view components

### Testing with angreal

Run Vue component tests:
```bash
# Run frontend tests
angreal test unit --ui

# Run in watch mode during development
angreal test unit --ui --watch

# Check coverage for components
angreal test coverage --ui --open
```

Coverage reports output to `crates/mimir-dm/frontend/coverage/index.html`

## Status Updates **[REQUIRED]**

### Session 2026-01-14

**Components Identified for Testing (Top 10):**
1. **EmptyState** - Simple UI component with variant icons
2. **MultiSelectFilter** - Interactive dropdown with search/selection
3. **AppModal** - Core modal with props, events, accessibility
4. **CatalogTable** - Complex table with filtering, sorting
5. **LoadingSpinner** - Simple UI component
6. **ThemeSelector** - Theme toggle with Pinia store
7. **CampaignSelector** - Complex dropdown with store integration
8. **CreateDocumentModal** - Form with validation
9. **ToolConfirmation** - Dialog component
10. **Panel** - Layout component

**Tests Created (6 components, 92 new tests):**

| Component | Tests | Coverage |
|-----------|-------|----------|
| EmptyState | 12 | Variants, title, description, action slot |
| MultiSelectFilter | 17 | Dropdown, selection, search, clear/select all |
| AppModal | 26 | Visibility, sizes, close behavior, slots, a11y |
| LoadingSpinner | 7 | Spinner, message prop |
| CatalogTable | 18 | Rendering, empty state, sorting, filters, cells |
| ThemeSelector | 12 | Themes list, selection, store integration |

**Test Files Created:**
- `src/shared/components/ui/__tests__/EmptyState.test.ts`
- `src/shared/components/ui/__tests__/MultiSelectFilter.test.ts`
- `src/shared/components/ui/__tests__/LoadingSpinner.test.ts`
- `src/shared/components/ui/__tests__/ThemeSelector.test.ts`
- `src/shared/components/catalog/__tests__/CatalogTable.test.ts`
- `src/components/shared/__tests__/AppModal.test.ts`

**Test Results:**
```
Test Files  8 passed (8)
Tests       134 passed (134)
```

**Component Testing Patterns Documented:**

### 1. Simple Presentational Components (EmptyState, LoadingSpinner)
```typescript
// Test props rendering, conditional content, slots
it('renders title correctly', () => {
  const wrapper = mount(Component, { props: { title: 'Test' } })
  expect(wrapper.find('.title').text()).toBe('Test')
})
```

### 2. Interactive Components (MultiSelectFilter)
```typescript
// Test user interactions, emitted events
it('emits update:modelValue when option toggled', async () => {
  const wrapper = mount(Component, { props: defaultProps })
  await wrapper.find('input').setValue(true)
  expect(wrapper.emitted('update:modelValue')).toBeTruthy()
})
```

### 3. Modal/Dialog Components (AppModal)
```typescript
// Stub Teleport, test visibility, close behaviors
const wrapper = mount(AppModal, {
  props: { visible: true, title: 'Test' },
  global: { stubs: { Teleport: true } }
})
```

### 4. Store-Connected Components (ThemeSelector)
```typescript
// Mock the store module
vi.mock('../../../../stores/theme', () => ({
  useThemeStore: () => mockThemeStore
}))
```

### 5. Complex Table Components (CatalogTable)
```typescript
// Mock child filter components, test sorting, filtering
vi.mock('../filters/SelectFilter.vue', () => ({
  default: { template: '<div class="mock"></div>' }
}))
```

**Key Patterns:**
- Use `@vue/test-utils` mount with global stubs for router
- Mock Tauri API via setup.ts
- Test accessibility (aria attributes, labels)
- Test edge cases (empty data, missing props)