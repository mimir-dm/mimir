---
id: replace-fog-token-buttons-with-new
level: task
title: "Replace Fog/Token buttons with new toggle UI and ambient dropdown"
short_code: "MIMIR-T-0435"
created_at: 2026-01-26T02:35:46.080119+00:00
updated_at: 2026-01-26T14:12:17.661363+00:00
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

# Replace Fog/Token buttons with new toggle UI and ambient dropdown

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[MIMIR-I-0047]]

## Objective **[REQUIRED]**

Replace the confusing Fog/Token mode buttons with clear toggle UI: separate Fog toggle and Token LOS toggle, plus ambient light dropdown (Bright/Dim/Dark). When Fog is ON, Token LOS is forced ON.

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

- [ ] Fog toggle: OFF = map fully visible, ON = map masked by party vision
- [ ] Token LOS toggle: OFF = all tokens visible, ON = tokens need LOS to be seen
- [ ] When Fog ON, Token LOS toggle disabled and forced ON
- [ ] Ambient light dropdown: Bright / Dim / Dark
- [ ] Old Fog/Token buttons removed from UI
- [ ] State persists during session (no need to persist to backend)

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

### Completed
- Replaced Fog/Token button group with two independent toggles:
  - **Fog** toggle: When ON, hides map outside party vision
  - **LOS** toggle: When ON, filters tokens by line of sight
- Added visual indicator when LOS is forced on (fog enabled)
- Reordered toolbar: Fog toggle, LOS toggle, separator, Ambient dropdown, Debug button
- Added new refs: `fogEnabled`, `tokenLosEnabled`
- `tokenOnlyLos` computed for backward compatibility with IPC
- Added toggle functions with proper state management
- Added CSS styles for toggle buttons with labels