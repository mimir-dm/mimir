---
id: add-uvtt-maps-to-dev-seeder
level: task
title: "Add UVTT maps to dev seeder"
short_code: "MIMIR-T-0242"
created_at: 2025-12-25T18:01:56.209723+00:00
updated_at: 2025-12-29T03:36:43.686260+00:00
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

# Add UVTT maps to dev seeder

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[MIMIR-I-0028]]

## Objective **[REQUIRED]**

Add sample UVTT maps to dev seeder for testing LOS, portals, and vision rendering during development.

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

- [ ] Dev seeder creates sample UVTT map with walls
- [ ] Dev seeder creates sample UVTT map with portals (doors)
- [ ] Maps saved to correct locations (campaign/maps or module/maps)
- [ ] Sample tokens placed for vision testing
- [ ] Works with `cargo run -- seed-dev`

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

**Location:** `crates/mimir-dm/src/commands/dev/seed_dev_data.rs`

**Sample maps to create:**

1. **Tavern map** (campaign-level)
   - Simple room with 4 walls
   - 2 doors (portals)
   - Placed in `{data_dir}/campaigns/{id}/maps/tavern.uvtt`

2. **Dungeon map** (module-level)  
   - Multiple rooms with corridors
   - Several walls and doors
   - Placed in `{data_dir}/modules/{id}/maps/dungeon.uvtt`

**UVTT generation:**
```rust
fn create_sample_uvtt(walls: Vec<Vec<Point>>, portals: Vec<Portal>) -> UvttFile {
    UvttFile {
        format: 0.3,
        resolution: UvttResolution {
            map_size: Point { x: 20.0, y: 15.0 },
            pixels_per_grid: 70,
            map_origin: Point { x: 0.0, y: 0.0 },
        },
        line_of_sight: walls,
        portals,
        lights: vec![],
        environment: UvttEnvironment::default(),
        image: generate_simple_grid_image(20, 15, 70),
    }
}
```

### Dependencies
Depends on: MIMIR-T-0227 (UVTT types), MIMIR-T-0240 (storage paths)

### Risk Considerations
Need simple image generation or embed a test PNG

## Status Updates **[REQUIRED]**

*To be added during implementation*