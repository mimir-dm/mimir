---
id: implement-raycasting-visibility
level: task
title: "Implement raycasting visibility polygon algorithm in VisionService"
short_code: "MIMIR-T-0233"
created_at: 2025-12-25T16:42:04.895851+00:00
updated_at: 2025-12-29T03:36:42.796619+00:00
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

# Implement raycasting visibility polygon algorithm in VisionService

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[MIMIR-I-0028]]

## Objective **[REQUIRED]**

Implement raycasting algorithm in VisionService that calculates visibility polygons from token positions against LOS wall geometry.

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

- [ ] `calculateVisibility(origin, range, walls, portals)` returns visibility polygon
- [ ] Rays cast to all wall segment endpoints
- [ ] Ray-wall intersection correctly blocks vision
- [ ] Closed portals block vision, open portals allow through
- [ ] `combinePartyVision(polygons)` unions multiple visibility polygons
- [ ] Performance: <16ms for typical map (50 walls, 5 tokens)
- [ ] Unit tests with sample wall configurations

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

**File:** `frontend/src/services/VisionService.ts`

**Algorithm (2D Raycasting):**
1. Collect all wall segment endpoints
2. For each endpoint, cast ray from origin through endpoint
3. Cast additional rays ±0.0001 radians to catch corners
4. Find closest wall intersection for each ray
5. Sort intersections by angle
6. Connect intersections to form visibility polygon

```typescript
interface WallSegment {
  start: Point
  end: Point
}

interface VisibilityPolygon {
  points: Point[]
}

function calculateVisibility(
  origin: Point,
  range: number,
  walls: WallSegment[],
  portals: Portal[]
): VisibilityPolygon

function rayWallIntersection(
  rayOrigin: Point,
  rayDirection: Point,
  wallStart: Point,
  wallEnd: Point
): Point | null
```

**Reference:** Red Blob Games visibility algorithm

### Dependencies
Depends on: MIMIR-T-0240 (UVTT file as source of truth)

### Risk Considerations
Complex wall configurations may produce degenerate polygons

## Status Updates **[REQUIRED]**

*To be added during implementation*