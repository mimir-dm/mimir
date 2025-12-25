---
id: render-light-source-radii-on-dm
level: task
title: "Render light source radii on DM and player displays"
short_code: "MIMIR-T-0218"
created_at: 2025-12-22T14:40:31.990446+00:00
updated_at: 2025-12-22T15:06:56.209305+00:00
parent: MIMIR-I-0026
blocked_by: []
archived: true

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: MIMIR-I-0026
---

# Render light source radii on DM and player displays

## Parent Initiative
[[MIMIR-I-0026]] - Vision and Lighting System

## Objective
Render visual indicators for light source radii (bright/dim circles) on both DM and player displays. Light sources should show concentric circles for bright light (inner) and dim light (outer), with optional color tinting.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria **[REQUIRED]**

- [ ] LightSourceRenderer component renders bright radius as inner circle
- [ ] LightSourceRenderer component renders dim radius as outer ring  
- [ ] Light radii convert from feet to pixels using grid size (1 square = 5 feet)
- [ ] Active lights show with full opacity, inactive lights are dimmed (DM only)
- [ ] Light color tinting is applied when color is set
- [ ] DM display shows all lights (active and inactive)
- [ ] Player display only shows active lights
- [ ] Lights attached to tokens follow token position
- [ ] Rendering performs smoothly during pan/zoom

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

*To be added during implementation*