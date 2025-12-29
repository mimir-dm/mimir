---
id: integrate-los-visibility-into
level: task
title: "Integrate LOS visibility into PlayerDisplayWindow fog rendering"
short_code: "MIMIR-T-0234"
created_at: 2025-12-25T16:42:04.997366+00:00
updated_at: 2025-12-29T03:36:42.926167+00:00
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

# Integrate LOS visibility into PlayerDisplayWindow fog rendering

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[MIMIR-I-0028]]

## Objective **[REQUIRED]**

Integrate LOS visibility polygons into PlayerDisplayWindow fog rendering so players only see areas within their tokens' line of sight.

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

- [ ] Fog respects LOS walls when map has LOS geometry
- [ ] Vision polygon clips fog reveal to visible areas only
- [ ] Combined party vision shows union of all PC token vision
- [ ] Explored areas (grey fog) remain visible but dimmed
- [ ] Currently visible areas (in LOS) fully clear
- [ ] Smooth fog edge rendering (no jagged polygon edges)
- [ ] Falls back to current behavior when map has no LOS data

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

**File:** `frontend/src/components/PlayerDisplayWindow.vue`

**Three-state fog model (from Foundry research):**
1. **Unexplored (black):** Never revealed
2. **Explored (grey):** Previously seen, shows terrain
3. **Visible (clear):** Currently in LOS

**Rendering approach:**
```typescript
function renderFogWithLOS(ctx: CanvasRenderingContext2D) {
  // 1. Draw revealed areas (from fog_revealed_areas)
  drawExploredFog(ctx, revealedAreas)
  
  // 2. Calculate party vision polygon
  const partyVision = visionService.combinePartyVision(
    pcTokens.map(t => visionService.calculateVisibility(t.position, t.visionRange, walls, portals))
  )
  
  // 3. Clip to vision polygon for "currently visible"
  ctx.save()
  ctx.beginPath()
  partyVision.points.forEach(p => ctx.lineTo(p.x, p.y))
  ctx.closePath()
  ctx.clip()
  ctx.globalCompositeOperation = 'destination-out'
  ctx.fill()
  ctx.restore()
}
```

### Dependencies
Depends on: MIMIR-T-0233 (VisionService)

### Risk Considerations
Performance with many tokens; may need caching/memoization

## Status Updates **[REQUIRED]**

*To be added during implementation*