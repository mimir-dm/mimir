---
id: update-mapuploadmodal-to-accept
level: task
title: "Update MapUploadModal to accept .dd2vtt files"
short_code: "MIMIR-T-0231"
created_at: 2025-12-25T16:41:50.396471+00:00
updated_at: 2025-12-25T16:41:50.396471+00:00
parent: MIMIR-I-0028
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/todo"


exit_criteria_met: false
strategy_id: NULL
initiative_id: MIMIR-I-0028
---

# Update MapUploadModal to accept .dd2vtt files

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[MIMIR-I-0028]]

## Objective **[REQUIRED]**

Update MapUploadModal to detect and accept .dd2vtt files, routing UVTT files to preview flow instead of direct upload.

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

## Acceptance Criteria **[REQUIRED]**

- [ ] File input accepts .dd2vtt and .uvtt extensions
- [ ] Drop zone shows both image and UVTT format hints
- [ ] File type detection routes PNG/JPG to existing flow
- [ ] UVTT files emit event to open UvttImportPreview modal
- [ ] Upload progress indicator works for both file types
- [ ] Error message for unsupported file types

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

**File:** `frontend/src/features/campaigns/components/StageLanding/MapUploadModal.vue`

**Changes:**
1. Update file input accept: `.png,.jpg,.jpeg,.webp,.dd2vtt,.uvtt`
2. Add format detection function:
```typescript
function getFileType(file: File): 'image' | 'uvtt' | null {
  const ext = file.name.split('.').pop()?.toLowerCase()
  if (['dd2vtt', 'uvtt'].includes(ext)) return 'uvtt'
  if (['png', 'jpg', 'jpeg', 'webp'].includes(ext)) return 'image'
  return null
}
```
3. Route UVTT to new emit: `@uvtt-selected="(file) => emit('uvttSelected', file)"`
4. Update drop zone UI to show format hints

**UX Pattern (from Foundry research):**
- Clear format distinction in UI
- UVTT badge shows "with LOS data"

### Dependencies
None - standalone UI change

### Risk Considerations
Must not break existing PNG upload flow

## Status Updates **[REQUIRED]**

*To be added during implementation*