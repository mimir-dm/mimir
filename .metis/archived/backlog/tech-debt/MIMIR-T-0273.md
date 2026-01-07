---
id: standardize-button-usage-across
level: task
title: "Standardize button usage across views"
short_code: "MIMIR-T-0273"
created_at: 2026-01-03T02:58:32.206350+00:00
updated_at: 2026-01-03T03:08:47.704668+00:00
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

# Standardize button usage across views

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[Parent Initiative]]

## Objective **[REQUIRED]**

Replace custom scoped button styles with global `.btn` classes from `buttons.css` for UI consistency.

## Backlog Item Details **[CONDITIONAL: Backlog Item]**

{Delete this section when task is assigned to an initiative}

### Type
- [ ] Bug - Production issue that needs fixing
- [ ] Feature - New functionality or enhancement  
- [x] Tech Debt - Code improvement or refactoring
- [ ] Chore - Maintenance or setup work

### Priority
- [ ] P0 - Critical (blocks users/revenue)
- [x] P1 - High (important for user experience)
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
- **Current Problems**: Views define custom `.btn-action`, `.btn-pdf`, `.btn-play` in scoped styles instead of using global button system. Creates visual inconsistency and duplicated CSS.
- **Benefits of Fixing**: Unified button appearance, easier theme updates, reduced CSS duplication
- **Risk Assessment**: Low risk - no new functionality, visual refinement only

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria **[REQUIRED]**

- [ ] `CharacterListView.vue` uses global `.btn` classes instead of `.btn-action`
- [ ] `ModulesTable.vue` uses global `.btn` classes instead of `.btn-play`, `.btn-pdf`
- [ ] `ModuleListView.vue` uses global `.btn` classes
- [ ] All buttons have consistent hover/active states across views
- [ ] Scoped button CSS removed from affected views

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
1. Replace `<button class="btn-action">` with `<button class="btn btn-ghost btn-sm">`
2. Replace custom color buttons with appropriate variants (`.btn-primary`, `.btn-danger`, etc.)
3. Remove scoped `.btn-action`, `.btn-play`, `.btn-pdf` styles from components

### Files to Modify
- `crates/mimir-dm/frontend/src/features/characters/views/CharacterListView.vue`
- `crates/mimir-dm/frontend/src/features/campaigns/components/StageLanding/ModulesTable.vue`
- `crates/mimir-dm/frontend/src/features/modules/views/ModuleListView.vue`

### Reference
Global button system: `src/assets/styles/components/buttons.css`
- Variants: `.btn-primary`, `.btn-secondary`, `.btn-ghost`, `.btn-danger`, `.btn-warning`
- Sizes: `.btn-xs`, `.btn-sm`, `.btn-lg`, `.btn-xl`

## Status Updates **[REQUIRED]**

*To be added during implementation*