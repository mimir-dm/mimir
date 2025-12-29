---
id: create-uvtt-wrapper-from-png-after
level: task
title: "Create UVTT wrapper from PNG after grid configuration"
short_code: "MIMIR-T-0241"
created_at: 2025-12-25T17:02:32.221449+00:00
updated_at: 2025-12-29T03:36:43.327697+00:00
parent: MIMIR-I-0028
blocked_by: []
archived: true

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: MIMIR-I-0028
---

# Create UVTT wrapper from PNG after grid configuration

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[MIMIR-I-0028]]

## Objective **[REQUIRED]**

When user uploads PNG and configures grid, generate a UVTT wrapper file so all maps use unified UVTT format.

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

- [ ] PNG upload stores temp image until grid configured
- [ ] Grid config modal captures: grid_size, offset_x, offset_y (square only)
- [ ] On save, creates UVTT wrapper with image + resolution
- [ ] UVTT saved to data directory, map record points to it
- [ ] Original PNG deleted after UVTT created
- [ ] Generated UVTT has empty walls/portals/lights arrays

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

**UVTT wrapper generation:**
```rust
pub fn create_uvtt_from_png(
    image_bytes: &[u8],
    grid_size_px: i32,
    offset_x: i32,
    offset_y: i32,
    width_px: i32,
    height_px: i32,
) -> Result<UvttFile> {
    let grid_width = width_px / grid_size_px;
    let grid_height = height_px / grid_size_px;
    
    Ok(UvttFile {
        format: 0.3,
        resolution: UvttResolution {
            map_size: Point { x: grid_width as f32, y: grid_height as f32 },
            map_origin: Point { 
                x: offset_x as f32 / grid_size_px as f32,
                y: offset_y as f32 / grid_size_px as f32 
            },
            pixels_per_grid: grid_size_px,
        },
        line_of_sight: vec![],
        portals: vec![],
        lights: vec![],
        environment: UvttEnvironment::default(),
        image: base64_encode(image_bytes),
    })
}
```

**Flow:**
1. User uploads PNG → store in temp location
2. User configures grid in MapGridConfigModal
3. On save → create_uvtt_from_png() → save .uvtt → create map record

### Dependencies
Depends on: MIMIR-T-0227 (UVTT types), MIMIR-T-0240 (simplified schema)

### Risk Considerations
Large images may create large UVTT files (base64 bloat)

## Status Updates **[REQUIRED]**

*To be added during implementation*