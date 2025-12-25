---
id: convert-png-maps-to-uvtt-format-on
level: task
title: "Convert PNG maps to UVTT format on grid configuration save"
short_code: "MIMIR-T-0238"
created_at: 2025-12-25T16:42:05.417131+00:00
updated_at: 2025-12-25T16:42:05.417131+00:00
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

# Convert PNG maps to UVTT format on grid configuration save

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[MIMIR-I-0028]]

## Objective **[REQUIRED]**

Convert existing PNG maps to minimal UVTT format internally when grid configuration is saved, enabling unified map handling.

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

- [ ] When grid config saved, pixels_per_grid calculated and stored
- [ ] Grid origin stored in grid_origin_x, grid_origin_y
- [ ] Existing PNG maps work unchanged (backwards compatible)
- [ ] Maps with pixels_per_grid are treated as "UVTT-ready"
- [ ] Future LOS editing can add walls to PNG-origin maps
- [ ] Migration script updates existing maps with calculated values

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

**Update:** `update_map_grid` command in maps.rs

```rust
pub async fn update_map_grid(
    state: State<'_, AppState>,
    map_id: i32,
    grid_type: GridType,
    grid_size_px: i32,
    grid_offset_x: i32,
    grid_offset_y: i32,
) -> Result<Map, Error> {
    // Calculate UVTT-compatible values
    let pixels_per_grid = grid_size_px;  // Direct mapping
    let grid_origin_x = grid_offset_x as f32 / grid_size_px as f32;
    let grid_origin_y = grid_offset_y as f32 / grid_size_px as f32;
    
    // Update map with both legacy and UVTT fields
    MapService::update_map(&conn, map_id, MapUpdate {
        grid_type: Some(grid_type),
        grid_size_px: Some(grid_size_px),
        grid_offset_x: Some(grid_offset_x),
        grid_offset_y: Some(grid_offset_y),
        pixels_per_grid: Some(pixels_per_grid),
        grid_origin_x: Some(grid_origin_x),
        grid_origin_y: Some(grid_origin_y),
        ..Default::default()
    })
}
```

### Dependencies
None - uses existing grid columns

### Risk Considerations
Must maintain backwards compatibility with existing grid_size_px usage

## Status Updates **[REQUIRED]**

*To be added during implementation*