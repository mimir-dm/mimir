---
id: migrate-simple-modals-to-appmodal
level: task
title: "Migrate simple modals to AppModal (Batch 1)"
short_code: "MIMIR-T-0248"
created_at: 2025-12-29T15:13:20.863099+00:00
updated_at: 2025-12-29T15:26:58.069599+00:00
parent: MIMIR-I-0029
blocked_by: []
archived: true

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: MIMIR-I-0029
---

# Migrate simple modals to AppModal (Batch 1)

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[MIMIR-I-0029]]

## Objective **[REQUIRED]**

Migrate simple modals to use AppModal component as proof of concept.

**Components:**
- `CreateModuleModal.vue`
- `ModuleListView.vue` (inline create modal)

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria **[REQUIRED]**

- [x] CreateModuleModal uses `<AppModal>` wrapper
- [x] ModuleListView inline modal uses `<AppModal>` wrapper
- [x] Scoped `.modal-overlay` CSS removed from both
- [x] Open/close behavior works correctly (via AppModal)
- [x] Overlay click closes modal (via AppModal closeOnOverlay)
- [x] Escape key closes modal (via AppModal closeOnEscape)
- [x] Visual appearance unchanged (uses modals.css)

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

### 2025-12-29: Completed
Migrated 2 simple modals to AppModal:

1. **CreateModuleModal.vue**
   - Replaced inline modal HTML with `<AppModal>` wrapper
   - Used slots for body content and footer buttons
   - Removed all scoped styles (component had none)

2. **ModuleListView.vue**
   - Replaced inline modal with `<AppModal>` component
   - Added AppModal import
   - Removed ~60 lines of scoped modal CSS (.modal-overlay, .modal-content, .form-group, .modal-actions)
   - Kept .btn-secondary styles (still needed for buttons)

Both modals now use centralized modals.css via AppModal, with automatic:
- Escape key handling
- Overlay click to close
- Body scroll lock
- Proper z-index stacking via Teleport