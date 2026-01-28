---
id: update-usetokenvision-to-sync-with
level: task
title: "Update useTokenVision to sync with backend"
short_code: "MIMIR-T-0434"
created_at: 2026-01-26T02:35:40.709952+00:00
updated_at: 2026-01-26T02:52:29.057497+00:00
parent: MIMIR-I-0047
blocked_by: []
archived: true

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: MIMIR-I-0047
---

# Update useTokenVision to sync with backend

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[MIMIR-I-0047]]

## Objective **[REQUIRED]**

Update `useTokenVision.ts` to load vision settings from backend on init and save changes via the `update_token_vision` command. Keep the preset system for UI convenience.

## Backlog Item Details **[CONDITIONAL: Backlog Item]**

{Delete this section when task is assigned to an initiative}

### Type
- [ ] Bug - Production issue that needs fixing
- [ ] Feature - New functionality or enhancement  
- [ ] Tech Debt - Code improvement or refactoring
- [ ] Chore - Maintenance or setup work

### Priority
- [ ] P0 - Critical (blocks users/revenue)
- [ ] P1 - High (important for user experience)
- [ ] P2 - Medium (nice to have)
- [ ] P3 - Low (when time permits)

### Impact Assessment **[CONDITIONAL: Bug]**
- **Affected Users**: {Number/percentage of users affected}
- **Reproduction Steps**: 
  1. {Step 1}
  2. {Step 2}
  3. {Step 3}
- **Expected vs Actual**: {What should happen vs what happens}

### Business Justification **[CONDITIONAL: Feature]**
- **User Value**: {Why users need this}
- **Business Value**: {Impact on metrics/revenue}
- **Effort Estimate**: {Rough size - S/M/L/XL}

### Technical Debt Impact **[CONDITIONAL: Tech Debt]**
- **Current Problems**: {What's difficult/slow/buggy now}
- **Benefits of Fixing**: {What improves after refactoring}
- **Risk Assessment**: {Risks of not addressing this}

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria **[REQUIRED]**

- [ ] Vision settings loaded from token data on component mount
- [ ] `setVisionSettings()` calls backend `update_token_vision` command
- [ ] `applyPreset()` maps preset to field values and saves to backend
- [ ] Presets still work: Human, Human+Torch, Darkvision 60/120, etc.
- [ ] Settings persist across page refreshes (stored in database)

## Test Cases **[CONDITIONAL: Testing Task]**

{Delete unless this is a testing task}

### Test Case 1: {Test Case Name}
- **Test ID**: TC-001
- **Preconditions**: {What must be true before testing}
- **Steps**: 
  1. {Step 1}
  2. {Step 2}
  3. {Step 3}
- **Expected Results**: {What should happen}
- **Actual Results**: {To be filled during execution}
- **Status**: {Pass/Fail/Blocked}

### Test Case 2: {Test Case Name}
- **Test ID**: TC-002
- **Preconditions**: {What must be true before testing}
- **Steps**: 
  1. {Step 1}
  2. {Step 2}
- **Expected Results**: {What should happen}
- **Actual Results**: {To be filled during execution}
- **Status**: {Pass/Fail/Blocked}

## Documentation Sections **[CONDITIONAL: Documentation Task]**

{Delete unless this is a documentation task}

### User Guide Content
- **Feature Description**: {What this feature does and why it's useful}
- **Prerequisites**: {What users need before using this feature}
- **Step-by-Step Instructions**:
  1. {Step 1 with screenshots/examples}
  2. {Step 2 with screenshots/examples}
  3. {Step 3 with screenshots/examples}

### Troubleshooting Guide
- **Common Issue 1**: {Problem description and solution}
- **Common Issue 2**: {Problem description and solution}
- **Error Messages**: {List of error messages and what they mean}

### API Documentation **[CONDITIONAL: API Documentation]**
- **Endpoint**: {API endpoint description}
- **Parameters**: {Required and optional parameters}
- **Example Request**: {Code example}
- **Example Response**: {Expected response format}

## Implementation Notes **[CONDITIONAL: Technical Task]**

{Keep for technical tasks, delete for non-technical. Technical details, approach, or important considerations}

### Technical Approach
{How this will be implemented}

### Dependencies
{Other tasks or systems this depends on}

### Risk Considerations
{Technical risks and mitigation strategies}

## Status Updates **[REQUIRED]**

### Session 1 - Completed
- Completely rewrote `useTokenVision.ts` to sync with backend:
  - Removed local state - now reads settings directly from Token object
  - Added `updateVisionSettings()` that calls backend `update_token_vision` command
  - Presets map to the 4 vision fields and save via backend
  - `findMatchingPreset()` detects if current settings match a preset
- Updated `TokenVisionMenu.vue` to work with new API:
  - Now accepts full `Token` prop instead of `tokenId`/`tokenName`
  - Uses async backend calls for all updates
  - Emits `updated` event with new Token object
- Fixed `DmMapViewer.vue` integration:
  - Updated visionMenu state to store Token object
  - Implemented vision calculation inline (no longer needs composable)
  - Fixed onVisionUpdated to update tokens array with new values
- Fixed SQL migration syntax error (missing comma)