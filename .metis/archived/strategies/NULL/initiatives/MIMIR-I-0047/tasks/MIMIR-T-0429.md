---
id: add-vision-fields-to-token
level: task
title: "Add vision fields to token_placements table"
short_code: "MIMIR-T-0429"
created_at: 2026-01-26T02:35:34.996336+00:00
updated_at: 2026-01-26T02:44:37.771598+00:00
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

# Add vision fields to token_placements table

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[MIMIR-I-0047]]

## Objective **[REQUIRED]**

Create database migration to add vision and light fields to the `token_placements` table. This enables storing per-token vision settings (bright/dim/dark vision ranges and light source radius).

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

- [ ] Migration `018_token_vision_fields/up.sql` adds `vision_bright_ft INTEGER` (nullable)
- [ ] Migration adds `vision_dim_ft INTEGER` (nullable)  
- [ ] Migration adds `vision_dark_ft INTEGER NOT NULL DEFAULT 0`
- [ ] Migration adds `light_radius_ft INTEGER NOT NULL DEFAULT 0`
- [ ] Down migration removes all four columns
- [ ] Migration runs successfully on existing database

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

Create migration files:
- `crates/mimir-core/migrations/018_token_vision_fields/up.sql`
- `crates/mimir-core/migrations/018_token_vision_fields/down.sql`

```sql
-- up.sql
ALTER TABLE token_placements ADD COLUMN vision_bright_ft INTEGER;
ALTER TABLE token_placements ADD COLUMN vision_dim_ft INTEGER;
ALTER TABLE token_placements ADD COLUMN vision_dark_ft INTEGER NOT NULL DEFAULT 0;
ALTER TABLE token_placements ADD COLUMN light_radius_ft INTEGER NOT NULL DEFAULT 0;
```

### Dependencies
None - this is the first task in the initiative.

### Risk Considerations
Low risk - adding nullable columns with defaults won't affect existing data.

## Status Updates **[REQUIRED]**

*To be added during implementation*