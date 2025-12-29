---
id: calculate-composite-party-vision
level: task
title: "Calculate composite party vision for player display"
short_code: "MIMIR-T-0221"
created_at: 2025-12-22T14:40:42.746177+00:00
updated_at: 2025-12-22T14:40:42.746177+00:00
parent: MIMIR-I-0026
blocked_by: []
archived: true

tags:
  - "#task"
  - "#phase/todo"


exit_criteria_met: false
strategy_id: NULL
initiative_id: MIMIR-I-0026
---

# Calculate composite party vision for player display

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[MIMIR-I-0026]]

## Objective **[REQUIRED]**

Implement logic to calculate composite party vision for the player display. This determines what players can collectively see based on their tokens' positions, vision types, vision ranges, and the current lighting conditions. The result is used to render appropriate visibility on the player display.

## Acceptance Criteria

## Acceptance Criteria **[REQUIRED]**

- [ ] Function calculates visible areas for each player-visible token based on vision type
- [ ] Darkvision tokens can see in dim light as if bright, and darkness as if dim (within range)
- [ ] Blindsight/Tremorsense tokens have full visibility within their range regardless of light
- [ ] Truesight tokens see through magical darkness within range
- [ ] Devil's Sight tokens treat magical darkness as normal darkness
- [ ] Normal vision tokens only see in bright light and dim light (with disadvantage noted)
- [ ] Composite vision unions all player token visible areas
- [ ] Light sources extend visible areas appropriately (bright radius = full vision, dim radius = reduced)
- [ ] Ambient light level affects baseline visibility
- [ ] Vision calculation runs efficiently (sub-100ms for typical maps)

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

### Technical Approach
Create a `useVisionCalculation` composable that:

1. **Input Data**:
   - Player-visible tokens with their positions, vision types, and ranges
   - Light sources with positions and radii
   - Map ambient light level
   - Grid size (for feet-to-pixel conversion)

2. **Per-Token Visibility Calculation**:
   ```typescript
   interface VisibleArea {
     tokenId: number
     brightVisionRadius: number  // pixels
     dimVisionRadius: number     // pixels
     fullVisionRadius: number    // for blindsight/truesight
   }
   ```

3. **Light Level Zones**:
   - Start with ambient light as baseline
   - Add bright/dim zones around each light source
   - Calculate which zones each token can see into based on vision type

4. **Composite Union**:
   - Merge all player token visible areas
   - Return as array of circles/polygons for rendering

### Dependencies
- T-0215: Token vision fields and light source data
- T-0218: Light source positions and radii
- Map grid_size_px for feet-to-pixel conversion

### Risk Considerations
- Complex geometry calculations may need optimization
- Consider caching results and only recalculating on token/light changes
- May need simplified circular approximations rather than true line-of-sight

## Status Updates **[REQUIRED]**

*To be added during implementation*