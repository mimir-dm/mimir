---
id: update-tokenplacement-rust-model
level: task
title: "Update TokenPlacement Rust model with vision fields"
short_code: "MIMIR-T-0430"
created_at: 2026-01-26T02:35:35.119285+00:00
updated_at: 2026-01-26T02:44:44.513487+00:00
parent: MIMIR-I-0047
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: MIMIR-I-0047
---

# Update TokenPlacement Rust model with vision fields

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[MIMIR-I-0047]]

## Objective **[REQUIRED]**

Update the Rust `TokenPlacement` model and related structs to include the new vision fields. This allows the backend to read/write vision settings from the database.

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

- [ ] `TokenPlacement` struct has `vision_bright_ft: Option<i32>`, `vision_dim_ft: Option<i32>`, `vision_dark_ft: i32`, `light_radius_ft: i32`
- [ ] `NewTokenPlacement` has corresponding fields for inserts
- [ ] `UpdateTokenPlacement` has corresponding `Option` fields for partial updates
- [ ] `TokenSummary` includes the vision fields in responses
- [ ] Existing token list/get commands return vision data
- [ ] TypeScript types updated in `api.ts`

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

Update `crates/mimir-core/src/models/campaign/token_placement.rs`:

```rust
pub struct TokenPlacement {
    // ... existing fields ...
    pub vision_bright_ft: Option<i32>,
    pub vision_dim_ft: Option<i32>,
    pub vision_dark_ft: i32,
    pub light_radius_ft: i32,
}
```

Also update frontend types in `frontend/src/types/api.ts`.

### Dependencies
- MIMIR-T-0429 (database migration must exist first)

### Risk Considerations
Low risk - adding fields with Option/defaults won't break existing code.

## Status Updates **[REQUIRED]**

*To be added during implementation*