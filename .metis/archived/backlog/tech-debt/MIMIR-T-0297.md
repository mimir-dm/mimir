---
id: replace-serde-json-value-with
level: task
title: "Replace serde_json::Value with typed structs in catalog models"
short_code: "MIMIR-T-0297"
created_at: 2026-01-03T15:08:53.804301+00:00
updated_at: 2026-01-07T10:26:24.685202+00:00
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

# Replace serde_json::Value with typed structs in catalog models

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[Parent Initiative]]

## Objective

Replace 166 instances of `serde_json::Value` in catalog models with properly typed Rust structs to gain compile-time type safety.

## Backlog Item Details

### Type
- [x] Tech Debt - Code improvement or refactoring

### Priority
- [x] P3 - Low (when time permits)

### Technical Debt Impact
- **Current Problems**: Catalog models use loose JSON types instead of typed structures. No compile-time guarantees on data shape, runtime errors possible when accessing fields, IDE autocomplete doesn't work.
- **Benefits of Fixing**: Full type safety, compile-time validation, better IDE support, safer refactoring.
- **Risk Assessment**: Low risk of not addressing - application works fine, but maintenance burden increases over time.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] All `serde_json::Value` fields in `models/catalog/` have typed alternatives
- [ ] Importers deserialize into typed structs
- [ ] All existing tests pass
- [ ] No runtime behavior changes

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
1. Audit each `serde_json::Value` field in catalog models
2. Create typed structs matching 5etools JSON schema
3. Update importers to deserialize into typed structs
4. Update services that consume this data

### Files Affected
- All files in `crates/mimir-dm-core/src/models/catalog/`

### Effort Estimate
1-2 weeks due to volume and testing requirements

### Risk Considerations
- Schema variations in 5etools data may require flexible typing
- Some fields may genuinely need dynamic JSON (custom content)

## Status Updates **[REQUIRED]**

*To be added during implementation*