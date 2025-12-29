---
id: unify-maps-rs-and-maps-v2-rs-into
level: task
title: "Unify maps.rs and maps_v2.rs into single module"
short_code: "MIMIR-T-0244"
created_at: 2025-12-29T14:45:57.334097+00:00
updated_at: 2025-12-29T15:06:24.172760+00:00
parent: 
blocked_by: []
archived: true

tags:
  - "#task"
  - "#tech-debt"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: NULL
---

# Unify maps.rs and maps_v2.rs into single module

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[Parent Initiative]]

## Objective **[REQUIRED]**

Consolidate `maps.rs` and `maps_v2.rs` into a single unified map module. The v2 file was created for UVTT upload support as a breaking change, but having both files creates confusion and maintenance burden.

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
- **Current Problems**: Two separate map modules (maps.rs, maps_v2.rs) with overlapping functionality. Confusing which to use, duplicated code patterns.
- **Benefits of Fixing**: Single source of truth for map handling, cleaner codebase, easier maintenance.
- **Risk Assessment**: Low - primarily code organization, not behavioral changes.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria **[REQUIRED]**

- [ ] Single maps.rs file with all map functionality
- [ ] maps_v2.rs deleted
- [ ] Dead code removed (old upload_map, update_map, get_uvtt_map_image)
- [ ] Frontend updated to use renamed functions
- [ ] All tests pass

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

**Step 1: Move UVTT types to maps.rs**
- UvttFile, UvttResolution, UvttPoint, UvttPortal, UvttLight, UvttEnvironment, UvttSummary
- UploadMapRequestV2 → UploadMapRequest (replace old)
- UploadMapResponseV2

**Step 2: Move used functions to maps.rs**
- upload_map_v2 → upload_map (rename)
- get_uvtt_map

**Step 3: Remove dead code from maps.rs**
- Old upload_map function
- update_map function (unused)

**Step 4: Delete maps_v2.rs**

**Step 5: Update mod.rs**
- Remove maps_v2 module

**Step 6: Update frontend**
- MapUploadModal.vue: upload_map_v2 → upload_map

### Dependencies
{Other tasks or systems this depends on}

### Risk Considerations
{Technical risks and mitigation strategies}

## Status Updates **[REQUIRED]**

### 2025-12-29: Completed

**Changes made:**
- Consolidated UVTT types (UvttFile, UvttResolution, UvttPoint, UvttPortal, UvttLight, UvttEnvironment, UvttSummary) into maps.rs
- Consolidated request/response types (UploadMapRequest, UploadMapResponse) into maps.rs
- Moved upload_map_v2 → upload_map (renamed)
- Moved get_uvtt_map to maps.rs
- Removed dead code: old upload_map, update_map, process_map_image, get_uvtt_map_image
- Deleted maps_v2.rs
- Updated mod.rs to remove maps_v2 module
- Updated main.rs to remove v2 command registrations
- Updated MapUploadModal.vue to use upload_map instead of upload_map_v2
- All 5 map tests pass