---
id: pinia-store-test-coverage
level: task
title: "Pinia Store Test Coverage"
short_code: "MIMIR-T-0340"
created_at: 2026-01-14T01:50:56.244090+00:00
updated_at: 2026-01-14T14:33:07.322865+00:00
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

# Pinia Store Test Coverage

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[MIMIR-I-0039]]

## Objective

Add unit tests for Pinia stores in the frontend, testing state management, actions, and getters.

## Scope

**Target: `crates/mimir-dm/frontend/src/stores/`**

Pinia stores manage application state. Each store needs tests for:
- Initial state
- Actions (mutations and async operations)
- Getters (computed values)
- Error handling

**Priority Stores:**
1. Campaign store - Core entity management
2. Module store - Module state and relationships
3. Character store - Character management
4. UI store - Application-wide UI state

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [x] Identify all Pinia stores in the codebase
- [x] Add tests for at least 3 core stores
- [x] Test initial state values
- [x] Test all actions including async operations
- [x] Test getters with various state configurations
- [x] Test error handling in async actions
- [x] Tests use mocked Tauri `invoke` calls
- [x] Document store testing patterns

## Implementation Notes

### Technical Approach

**Test Pattern:**
```typescript
import { setActivePinia, createPinia } from 'pinia'
import { describe, it, expect, beforeEach, vi } from 'vitest'
import { useCampaignStore } from './campaignStore'
import { mockInvoke, createMockCampaign } from '@/test/utils/mockTauri'

describe('campaignStore', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.clearAllMocks()
  })

  it('has correct initial state', () => {
    const store = useCampaignStore()
    expect(store.campaigns).toEqual([])
    expect(store.loading).toBe(false)
  })

  it('fetches campaigns', async () => {
    const mockCampaigns = [createMockCampaign()]
    mockInvoke.mockResolvedValueOnce(mockCampaigns)
    
    const store = useCampaignStore()
    await store.fetchCampaigns()
    
    expect(store.campaigns).toEqual(mockCampaigns)
  })

  it('handles fetch error', async () => {
    mockInvoke.mockRejectedValueOnce(new Error('Network error'))
    
    const store = useCampaignStore()
    await store.fetchCampaigns()
    
    expect(store.error).toBe('Network error')
  })
})
```

**Key Test Scenarios:**
1. Initial state correctness
2. Successful CRUD operations
3. Loading states during async operations
4. Error state management
5. Getter computations

### Key Files
- `src/stores/` - All Pinia store definitions
- `src/test/utils/mockTauri.ts` - Mock utilities

### Risk Considerations
- Async action testing complexity
- Mocking Tauri invoke correctly
- Store dependencies on other stores

### Testing with angreal

Run Pinia store tests:
```bash
# Run frontend tests
angreal test unit --ui

# Run in watch mode during development
angreal test unit --ui --watch

# Check coverage for stores
angreal test coverage --ui --open
```

Coverage reports output to `crates/mimir-dm/frontend/coverage/index.html`

## Status Updates **[REQUIRED]**

### Session 2026-01-14

**Pinia Stores Identified:**
1. `campaigns.ts` - Campaign CRUD, archive/unarchive, import/export
2. `characters.ts` - Character management, versions, spells, inventory
3. `theme.ts` - Theme loading, localStorage sync, cross-window sync
4. `players.ts` - Player management
5. `sharedContext.ts` - Shared application context
6. `appSettings.ts` - Application settings
7. `chat/` - Chat-related stores (messages, session, tokens, todos, tool-confirmations)

**Tests Created (3 stores, 100 new tests):**

| Store | Tests | Coverage |
|-------|-------|----------|
| `campaigns.ts` | 30 | Initial state, fetchCampaigns, getCampaign, createCampaign, updateStatus, archive/unarchive, delete, import/export |
| `theme.ts` | 22 | Initial state, loadThemes, applyTheme, setTheme, initThemeSync, cleanup, all theme values |
| `characters.ts` | 48 | Initial state, computed (characterCount, getCharacterById, level, proficiency), fetch, create, HP, delete, versions, spells, rest, inventory, currency, equipped |

**Test Files Created:**
- `src/stores/__tests__/campaigns.test.ts` - 30 tests
- `src/stores/__tests__/theme.test.ts` - 22 tests
- `src/stores/__tests__/characters.test.ts` - 48 tests

**Test Results:**
```
Test Files  11 passed (11)
Tests       234 passed (234)
```

**Store Testing Patterns Documented:**

### 1. Setup Pattern
```typescript
import { setActivePinia, createPinia } from 'pinia'
import { useStore } from '../store'
import { invoke } from '@tauri-apps/api/core'

const mockInvoke = vi.mocked(invoke)

beforeEach(() => {
  setActivePinia(createPinia())
  vi.clearAllMocks()
})
```

### 2. Testing Initial State
```typescript
it('has correct initial state', () => {
  const store = useStore()
  expect(store.items).toEqual([])
  expect(store.loading).toBe(false)
  expect(store.error).toBeNull()
})
```

### 3. Testing Async Actions
```typescript
it('fetches items successfully', async () => {
  mockInvoke.mockResolvedValueOnce({ success: true, data: mockItems })
  
  const store = useStore()
  await store.fetchItems()
  
  expect(mockInvoke).toHaveBeenCalledWith('list_items')
  expect(store.items).toEqual(mockItems)
})
```

### 4. Testing Error Handling
```typescript
it('handles API error', async () => {
  mockInvoke.mockResolvedValueOnce({ success: false, error: 'Failed' })
  
  const store = useStore()
  await store.fetchItems()
  
  expect(store.error).toBe('Failed')
})

it('handles exception', async () => {
  mockInvoke.mockRejectedValueOnce(new Error('Network error'))
  
  const store = useStore()
  await expect(store.fetchItems()).rejects.toThrow()
  expect(store.error).toBe('Network error')
})
```

### 5. Testing Computed Properties
```typescript
it('returns correct computed value', () => {
  const store = useStore()
  store.items = [item1, item2, item3]
  
  expect(store.itemCount).toBe(3)
  expect(store.getItemById(2)).toEqual(item2)
})
```

### 6. Mocking localStorage
```typescript
let localStorageMock: Record<string, string> = {}

beforeEach(() => {
  vi.spyOn(Storage.prototype, 'getItem').mockImplementation(
    (key) => localStorageMock[key] || null
  )
  vi.spyOn(Storage.prototype, 'setItem').mockImplementation(
    (key, value) => { localStorageMock[key] = value }
  )
})
```

### 7. Mocking Tauri Events
```typescript
vi.mock('@tauri-apps/api/event', () => ({
  emit: vi.fn(),
  listen: vi.fn()
}))

const mockListen = vi.mocked(listen)
mockListen.mockResolvedValueOnce(vi.fn()) // Returns unlisten function
```

**Key Learnings:**
- Use `vi.mocked()` for type-safe mock access
- Create helper factories for mock data consistency
- Test loading states during async operations
- Test state mutations after successful operations
- Test cleanup and side effects