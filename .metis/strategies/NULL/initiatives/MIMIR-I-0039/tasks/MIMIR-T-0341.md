---
id: frontend-service-layer-test
level: task
title: "Frontend Service Layer Test Coverage"
short_code: "MIMIR-T-0341"
created_at: 2026-01-14T01:50:56.345280+00:00
updated_at: 2026-01-14T14:25:04.847124+00:00
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

# Frontend Service Layer Test Coverage

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[MIMIR-I-0039]]

## Objective

Extend the frontend service layer test coverage following the pattern established by `ModuleService.test.ts`, covering all service classes.

## Scope

**Target: `crates/mimir-dm/frontend/src/services/`**

Existing example: `ModuleService.test.ts` (333 lines) provides a comprehensive testing pattern.

**Services to Test:**
1. CampaignService - Campaign CRUD operations
2. CharacterService - Character management
3. DocumentService - Document operations
4. SessionService - Session/chat management
5. Any other service classes

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [x] Audit all service files in `src/services/`
- [x] Identify services without test coverage
- [x] Add tests following `ModuleService.test.ts` pattern
- [x] Test all CRUD operations (get, list, create, update, delete)
- [x] Test caching behavior where applicable
- [x] Test error handling for Tauri command failures
- [x] All tests use mock utilities from `mockTauri.ts`
- [x] Tests pass with `npm run test`

## Implementation Notes

### Technical Approach

**Follow ModuleService Pattern:**
```typescript
import { describe, it, expect, vi, beforeEach } from 'vitest'
import { CampaignService } from './CampaignService'
import { mockInvoke, createMockCampaign } from '@/test/utils/mockTauri'

describe('CampaignService', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    CampaignService.clearCache() // if caching exists
  })

  describe('get', () => {
    it('returns campaign by id', async () => {
      const mockCampaign = createMockCampaign({ id: 1 })
      mockInvoke.mockResolvedValueOnce(mockCampaign)
      
      const result = await CampaignService.get(1)
      
      expect(result).toEqual(mockCampaign)
      expect(mockInvoke).toHaveBeenCalledWith('get_campaign', { id: 1 })
    })
  })

  describe('list', () => { /* ... */ })
  describe('create', () => { /* ... */ })
  describe('update', () => { /* ... */ })
  describe('delete', () => { /* ... */ })
})
```

**Key Test Areas:**
1. Tauri command invocation with correct parameters
2. Response transformation/mapping
3. Cache invalidation on mutations
4. Error propagation

### Reference Files
- `src/services/ModuleService.test.ts` - Pattern to follow
- `src/test/utils/mockTauri.ts` - Mock utilities

### Risk Considerations
- Services may have different caching strategies
- Some services may have complex response types
- Need to add mock factories for missing types

### Testing with angreal

Run frontend service tests:
```bash
# Run frontend tests
angreal test unit --ui

# Run in watch mode during development
angreal test unit --ui --watch

# Check coverage for services
angreal test coverage --ui --open
```

Coverage reports output to `crates/mimir-dm/frontend/coverage/index.html`

## Status Updates **[REQUIRED]**

### Session 1 - 2026-01-14

**Completed:**

1. **Audited service files in `src/services/`:**
   - `boardConfigService.ts` - Board configuration and stage management
   - `DocumentService.ts` - Document CRUD operations (already has tests)
   - `ModuleService.ts` - Module management (already has tests)
   - `PrintService.ts` - PDF generation and export

2. **Created `boardConfigService.test.ts` (34 tests):**
   - `fetchBoardConfig` - async fetch from backend with response transformation
   - `cacheBoard` - caching configurations
   - `getBoardConfig` - retrieving cached configs
   - `getStageDefinition` - finding stages by key
   - `getStageDocuments` - documents with metadata for stages
   - `canTransition` - validating stage transitions
   - `getNextStage` - getting next stage in progression
   - `calculateStageCompletion` - completion percentage calculation
   - Document metadata for all 14 known document types

3. **Created `PrintService.test.ts` (30 tests):**
   - `listTemplates` - list available print templates
   - `generateCharacterSheet` - character sheet PDF generation
   - `generateCharacterExport` - composable character export
   - `exportCampaignDocument` - single document export
   - `exportCampaignDocuments` - campaign-wide export
   - `exportModuleDocuments` - module export
   - `printMap` - map printing with options
   - `pdfToBlob` - base64 to Blob conversion
   - `createPdfUrl` - object URL creation
   - `savePdf` - file system save with dialog
   - `openPdf` - browser window opening
   - `printPdf` - iframe print dialog trigger
   - Error handling for network and API failures

**Test Results:**
- All 298 frontend tests passing
- 64 new tests added (34 + 30)
- No regressions in existing tests

**Files Created:**
- `src/services/__tests__/boardConfigService.test.ts`
- `src/services/__tests__/PrintService.test.ts`