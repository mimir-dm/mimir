---
id: create-uvtt-types-and-parser-in
level: task
title: "Create UVTT types and parser in mimir-dm-core"
short_code: "MIMIR-T-0227"
created_at: 2025-12-25T16:41:49.988476+00:00
updated_at: 2025-12-29T03:36:28.158909+00:00
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

# Create UVTT types and parser in mimir-dm-core

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[MIMIR-I-0028]]

## Objective **[REQUIRED]**

Create Rust types and parser for Universal VTT (.dd2vtt) format in mimir-dm-core, enabling import of Dungeondraft maps with LOS geometry.

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

- [ ] `UvttFile` struct parses all UVTT format 0.3 fields
- [ ] `UvttResolution` correctly extracts map_size, map_origin, pixels_per_grid
- [ ] `UvttPoint` handles fractional grid coordinates (f32)
- [ ] `UvttPortal` includes position, bounds, rotation, closed state
- [ ] `parse_uvtt()` decodes base64 image data to raw bytes
- [ ] `validate_uvtt()` returns errors for missing required fields
- [ ] Unit tests pass with sample UVTT JSON (without image)

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

**Location:** `crates/mimir-dm-core/src/services/uvtt_service.rs`

**UVTT Format (from sample file):**
```json
{
  "format": 0.3,
  "resolution": { "map_origin": {x,y}, "map_size": {x,y}, "pixels_per_grid": 54 },
  "line_of_sight": [[{x,y}, {x,y}, ...]],
  "objects_line_of_sight": [],
  "portals": [{ "position": {x,y}, "bounds": [...], "rotation": 0, "closed": true }],
  "lights": [{ "position": {x,y}, "range": 5, "intensity": 1.0, "color": "ffaa00" }],
  "environment": { "baked_lighting": true, "ambient_light": "ffffffff" },
  "image": "data:image/png;base64,..."
}
```

**Types to create:**
- `UvttFile` - Top-level with all fields
- `UvttResolution` - Grid dimensions
- `UvttPoint` - x/y as f32
- `UvttPortal` - Door/window geometry
- `UvttLight` - Light source data
- `UvttEnvironment` - Ambient settings

**Functions:**
- `parse_uvtt(data: &[u8]) -> Result<UvttFile>`
- `extract_image(uvtt: &UvttFile) -> Result<Vec<u8>>`
- `validate_uvtt(uvtt: &UvttFile) -> Result<()>`

### Dependencies
None - foundational task

### Risk Considerations
Base64 image decoding may fail on malformed files

## Status Updates **[REQUIRED]**

*To be added during implementation*