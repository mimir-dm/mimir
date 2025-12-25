---
id: render-dim-light-and-darkness
level: task
title: "Render dim light and darkness overlays with vision integration"
short_code: "MIMIR-T-0222"
created_at: 2025-12-22T14:40:42.860653+00:00
updated_at: 2025-12-22T14:40:42.860653+00:00
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

# Render dim light and darkness overlays with vision integration

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[MIMIR-I-0026]]

## Objective **[REQUIRED]**

Render visual overlays on both DM and player displays that communicate lighting conditions. On the DM display, show all light levels for reference. On the player display, integrate lighting with vision calculation to show what players can actually see - areas in darkness they cannot see into appear as fog, dim areas have a subtle overlay, and bright areas are fully visible.

## Acceptance Criteria

## Acceptance Criteria **[REQUIRED]**

- [ ] DM display shows dim light areas with semi-transparent yellow/amber overlay
- [ ] DM display shows darkness areas with semi-transparent dark overlay  
- [ ] DM display shows light source radii as reference circles (bright inner, dim outer)
- [ ] Player display integrates fog of war with vision/lighting
- [ ] Areas in darkness that no player token can see into appear as fog (black)
- [ ] Areas in dim light appear with subtle overlay indicating reduced visibility
- [ ] Areas in bright light (or seen with darkvision) appear normally
- [ ] Overlays update in real-time as tokens move
- [ ] Overlays update when light sources are added/removed/toggled
- [ ] Performance remains smooth during pan/zoom operations

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

**DM Display (DmMapViewer.vue)**:
1. Add lighting overlay layer above map but below tokens
2. Render ambient darkness as base layer (if ambient != 'bright')
3. Cut out bright circles around light sources
4. Render dim light rings (between bright and dim radii) with amber tint
5. Use CSS mix-blend-mode or canvas compositing for proper overlay effects

**Player Display (PlayerDisplayWindow.vue)**:
1. Integrate with existing fog of war system
2. Replace simple revealed/unrevealed with light-level-aware visibility
3. Use composite vision from T-0221 to determine what's visible
4. Render layers:
   - Base: Full black fog
   - Cut out: Areas player tokens can see (based on vision + lighting)
   - Overlay: Dim light tint on dim-visible areas
   
**Rendering Strategy**:
```typescript
// Canvas layer order (bottom to top):
// 1. Map image
// 2. Lighting overlay (dim/dark tints)
// 3. Light source indicators (DM only)
// 4. Fog of war mask (player only)
// 5. Tokens
// 6. UI elements
```

### Dependencies
- T-0218: Light source rendering (bright/dim circles)
- T-0221: Composite party vision calculation
- Existing fog of war implementation

### Risk Considerations
- Canvas compositing can be complex with multiple overlapping circles
- May need to use off-screen canvas for fog calculation
- Performance critical: consider requestAnimationFrame batching
- Different browsers may render blend modes slightly differently

## Status Updates **[REQUIRED]**

*To be added during implementation*