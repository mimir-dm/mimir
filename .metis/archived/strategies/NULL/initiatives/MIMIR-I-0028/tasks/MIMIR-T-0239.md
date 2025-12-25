---
id: add-uvtt-fields-to-maps-table
level: task
title: "Add UVTT fields to maps table migration"
short_code: "MIMIR-T-0239"
created_at: 2025-12-25T16:58:22.051473+00:00
updated_at: 2025-12-25T16:58:22.051473+00:00
parent: MIMIR-I-0028
blocked_by: []
archived: true

tags:
  - "#task"
  - "#phase/todo"


exit_criteria_met: false
strategy_id: NULL
initiative_id: MIMIR-I-0028
---

# Add UVTT fields to maps table migration

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[MIMIR-I-0028]]

## Objective **[REQUIRED]**

Add UVTT-related columns to maps table for storing grid resolution and LOS geometry as JSON blob.

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

## Acceptance Criteria **[REQUIRED]**

- [ ] Migration adds los_data TEXT column for LOS geometry JSON
- [ ] Map model updated with los_data field
- [ ] Down migration removes column cleanly

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

**Migration:** `040_add_los_data/up.sql`

```sql
ALTER TABLE maps ADD COLUMN los_data TEXT;
```

**Model update:** `models/campaign/maps.rs`
```rust
pub struct Map {
    // existing fields (grid_size_px, grid_offset_x, grid_offset_y already exist)
    pub los_data: Option<String>,  // JSON blob
}
```

**On UVTT import, populate existing grid fields:**
- `grid_size_px` ← `resolution.pixels_per_grid`
- `grid_offset_x` ← `resolution.map_origin.x * pixels_per_grid`
- `grid_offset_y` ← `resolution.map_origin.y * pixels_per_grid`

**los_data JSON structure:**
```json
{
  "walls": [[{x, y}, {x, y}, ...]],
  "portals": [{ "position": {x, y}, "bounds": [...], "closed": true }]
}
```

### Dependencies
Depends on: MIMIR-T-0227 (defines JSON structure)

### Risk Considerations
Nullable columns maintain backwards compatibility

## Status Updates **[REQUIRED]**

*To be added during implementation*