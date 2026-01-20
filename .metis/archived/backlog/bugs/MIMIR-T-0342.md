---
id: fix-broken-moduleservice-frontend
level: task
title: "Fix Broken ModuleService Frontend Tests"
short_code: "MIMIR-T-0342"
created_at: 2026-01-14T02:30:21.896977+00:00
updated_at: 2026-01-14T13:18:19.763432+00:00
parent: 
blocked_by: []
archived: true

tags:
  - "#task"
  - "#bug"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: NULL
---

# Fix Broken ModuleService Frontend Tests

## Objective

Fix the 7 failing tests in `ModuleService.test.ts` that are out of sync with the current API implementation.

## Backlog Item Details

### Type
- [x] Bug - Production issue that needs fixing

### Priority
- [x] P2 - Medium (nice to have)

### Impact Assessment
- **Affected Users**: Developers running frontend tests
- **Reproduction Steps**: 
  1. Run `angreal test unit --ui` or `npm test` in frontend directory
  2. Observe 7 test failures in ModuleService.test.ts
- **Expected vs Actual**: Tests expect old API parameter names (camelCase), but service now uses snake_case with request wrapper

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [x] All 7 failing ModuleService tests pass
- [x] `angreal test unit --ui` runs without failures
- [x] Frontend coverage can be measured after fixes

## Implementation Notes

### Technical Approach

The tests are failing because the ModuleService API parameters changed:
- **Old**: `{ campaignId: 1, status: 'active' }`
- **New**: `{ request: { campaign_id: 1, new_stage: 'active' } }`

Failing tests need updates in:
- `src/services/__tests__/ModuleService.test.ts`

Specific fixes needed:
1. `list` - Change `campaignId` to `request.campaign_id`
2. `create` - Add `request` wrapper, add `expected_sessions` field
3. `update` - Add `request` wrapper with optional fields
4. `updateStatus` - Rename to `transitionStage`, update params
5. `transitionStage` - Update param names from camelCase
6. `initializeDocuments` - Fix mock response structure
7. `incrementSessionCount` - Change `moduleId` to `module_id`

### Testing with angreal

```bash
# Run frontend tests after fixes
angreal test unit --ui

# Run coverage after tests pass
angreal test coverage --ui --open
```

## Status Updates

### Session 2026-01-14

**Problem:** 7 tests in `ModuleService.test.ts` were failing due to API parameter format changes.

**Root Cause:** The service implementation was updated to use:
- Snake_case parameter names (`campaign_id`, `module_id`, `new_stage`)
- Request wrapper objects (`{ request: { ... } }`)

But tests still expected the old format with camelCase and flat parameters.

**Fixes Applied:**

| Test | Old Format | New Format |
|------|------------|------------|
| `list` | `{ campaignId: 1 }` | `{ request: { campaign_id: 1 } }` |
| `create` | `{ name, campaign_id, module_type }` | `{ request: { campaign_id, name, module_type, expected_sessions: 4 } }` |
| `update` | `{ id, name }` | `{ id, request: { name, expected_sessions, actual_sessions } }` |
| `updateStatus` | `update_module_status` | Uses `transition_module_stage` internally |
| `transitionStage` | `{ moduleId, newStage }` | `{ request: { module_id, new_stage } }` |
| `initializeDocuments` | `{ moduleId }` | Gets module/campaign first, then `{ request: { module_id, campaign_directory } }` |
| `incrementSessionCount` | `{ moduleId }` | `{ module_id }` |

**Test Results:**
```
✓ src/services/__tests__/ModuleService.test.ts (21 tests)
✓ src/services/__tests__/DocumentService.test.ts (21 tests)

Test Files  2 passed (2)
Tests       42 passed (42)
```

**Files Modified:**
- `src/services/__tests__/ModuleService.test.ts` - Updated 7 test expectations