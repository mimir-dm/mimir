---
id: add-module-campaign-export-options
level: task
title: "Add module/campaign export options dialog"
short_code: "MIMIR-T-0258"
created_at: 2025-12-29T16:21:10.218837+00:00
updated_at: 2025-12-29T20:20:05.516581+00:00
parent: MIMIR-I-0027
blocked_by: []
archived: true

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: MIMIR-I-0027
---

# Add module/campaign export options dialog

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[MIMIR-I-0027]]

## Objective

Create an export options dialog for module/campaign views that offers Reference Document and Physical Play Kit as separate export options.

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

## Acceptance Criteria

### Module Export Dialog (`ModuleExportDialog.vue`)
- [ ] Reference Document section with checkboxes:
  - [ ] Documents (default: on)
  - [ ] Monsters (default: on)
  - [ ] NPCs (default: off) - pulls from campaign's NPC list
  - [ ] Map Previews (default: on)
- [ ] Physical Play Kit section with checkboxes:
  - [ ] Tiled Maps at 1"=5ft scale (default: off)
  - [ ] Token Cutouts (default: off)
- [ ] Shows estimated page count
- [ ] Outputs single combined PDF

### Campaign Export Dialog (`CampaignExportDialog.vue`)  
- [ ] Reference Document section with checkboxes:
  - [ ] Campaign Documents (default: on)
  - [ ] Module Content - docs + monsters only (default: on)
  - [ ] NPCs (default: off) - all campaign NPCs, printed once
  - [ ] Map Previews (default: on)
- [ ] Physical Play Kit section with checkboxes:
  - [ ] Tiled Maps at 1"=5ft scale (default: off)
  - [ ] Token Cutouts (default: off)
- [ ] Shows estimated page count
- [ ] Outputs single combined PDF

### Integration
- [ ] Update `ModulesTable.vue` PDF button to open ModuleExportDialog
- [ ] Update `DocumentSidebar.vue` Export button to open CampaignExportDialog
- [ ] NPCs checkbox independent at each level (campaign selection doesn't affect module defaults)

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