---
id: add-vision-configuration-to-token
level: task
title: "Add vision configuration to token setup UI"
short_code: "MIMIR-T-0220"
created_at: 2025-12-22T14:40:42.637339+00:00
updated_at: 2025-12-22T15:12:15.848432+00:00
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

# Add vision configuration to token setup UI

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[MIMIR-I-0026]]

## Objective **[REQUIRED]**

Add vision configuration controls to the token setup/edit UI, allowing DMs to assign vision types (normal, darkvision, blindsight, etc.) and vision ranges to tokens. This enables the lighting system to calculate what each token can see based on ambient light and light sources.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria **[REQUIRED]**

- [ ] Token setup modal includes a "Vision" section with vision type dropdown
- [ ] Vision types available: Normal, Darkvision, Blindsight, Tremorsense, Truesight, Devil's Sight
- [ ] Vision range input field appears for relevant vision types (darkvision, blindsight, etc.)
- [ ] Common presets available (e.g., "Darkvision 60ft", "Darkvision 120ft")
- [ ] Monster tokens auto-populate vision from catalog_monsters data when available
- [ ] Character tokens can have vision configured manually
- [ ] Vision settings persist when token is saved
- [ ] Vision settings load correctly when editing existing tokens

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

### Technical Approach
- Extend TokenSetupModal.vue with a collapsible "Vision" section
- Add VisionType enum matching backend: 'normal', 'darkvision', 'blindsight', 'tremorsense', 'truesight', 'devils_sight'
- Create dropdown for vision type selection with human-readable labels
- Conditionally show vision range input when type != 'normal'
- Add preset buttons for common configurations (Darkvision 60ft, Darkvision 120ft, etc.)
- Parse monster senses from full_monster_json to auto-populate when monster_id is set
- Update CreateToken and UpdateToken API payloads to include vision_type and vision_range_ft

### Dependencies
- T-0215: Schema and models must include vision fields on tokens
- T-0216: LightSourceService for any token-attached light handling

### Risk Considerations
- Monster JSON parsing may have inconsistent formats across sources
- Need fallback to manual entry if auto-detection fails

## Status Updates **[REQUIRED]**

*To be added during implementation*