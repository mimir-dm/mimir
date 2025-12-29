---
id: create-uvttimportpreview-modal
level: task
title: "Create UvttImportPreview modal with metadata display"
short_code: "MIMIR-T-0232"
created_at: 2025-12-25T16:41:50.504724+00:00
updated_at: 2025-12-25T16:41:50.504724+00:00
parent: MIMIR-I-0028
blocked_by: []
archived: true

tags:
  - "#task"
  - "#phase/todo"


exit_criteria_met: false
strategy_id: NULL
initiative_id: MIMIR-I-0028
---

# Create UvttImportPreview modal with metadata display

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[MIMIR-I-0028]]

## Objective **[REQUIRED]**

Create UvttImportPreview modal that displays map preview, detected metadata (grid size, wall count, portal count, lights), and import options before committing the import.

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

## Acceptance Criteria **[REQUIRED]**

- [ ] Modal displays map image thumbnail from UVTT
- [ ] Shows grid dimensions (e.g., "35 x 20 squares")
- [ ] Shows wall segment count from line_of_sight
- [ ] Shows portal count from portals array
- [ ] Shows light source count from lights array
- [ ] Checkbox for "Import LOS geometry" (default: checked)
- [ ] Checkbox for "Import lights" (default: checked)
- [ ] Name input field (defaults to filename without extension)
- [ ] Import button calls `import_uvtt` command
- [ ] Cancel button closes modal without action
- [ ] Loading state during import with progress feedback

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

## Implementation Notes

### Technical Approach

**File:** `frontend/src/features/campaigns/components/StageLanding/UvttImportPreview.vue`

**Props:**
```typescript
interface Props {
  file: File
  campaignId: number
  moduleId?: number
}
```

**Client-side parsing:**
Parse UVTT JSON in browser to extract preview data without sending to backend:
```typescript
async function parseUvttPreview(file: File): Promise<UvttPreview> {
  const text = await file.text()
  const uvtt = JSON.parse(text)
  return {
    gridSize: uvtt.resolution.map_size,
    wallCount: uvtt.line_of_sight.length,
    portalCount: uvtt.portals.length,
    lightCount: uvtt.lights.length,
    imageDataUrl: uvtt.image  // Already data URL
  }
}
```

**UX (from Foundry research):**
- Show parsing stages during import (image → walls → lights)
- Progress indicator for 3-10 second imports

### Dependencies
Depends on: MIMIR-T-0230 (import_uvtt command), MIMIR-T-0231 (modal trigger)

### Risk Considerations
Large UVTT files may cause browser slowdown during JSON parse

## Status Updates **[REQUIRED]**

*To be added during implementation*