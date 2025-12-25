---
id: add-los-debug-overlay-to
level: task
title: "Add LOS debug overlay to DmMapViewer"
short_code: "MIMIR-T-0235"
created_at: 2025-12-25T16:42:05.097171+00:00
updated_at: 2025-12-25T16:42:05.097171+00:00
parent: MIMIR-I-0028
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/todo"


exit_criteria_met: false
strategy_id: NULL
initiative_id: MIMIR-I-0028
---

# Add LOS debug overlay to DmMapViewer

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[MIMIR-I-0028]]

## Objective **[REQUIRED]**

Add optional LOS debug overlay to DmMapViewer that visualizes wall geometry and portal states for debugging and verification.

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

## Acceptance Criteria **[REQUIRED]**

- [ ] Toggle button in DM toolbar shows/hides LOS overlay
- [ ] Walls rendered as semi-transparent colored lines
- [ ] Portals rendered with different color (open vs closed)
- [ ] Closed portals show solid line, open portals show dashed
- [ ] Wall endpoints visible as small circles
- [ ] Overlay scales correctly with zoom
- [ ] Overlay hidden by default, persists across map changes

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

**File:** `frontend/src/components/LosDebugOverlay.vue`

**Color scheme (from Foundry research):**
- Walls: Yellow/amber semi-transparent
- Closed portals: Red
- Open portals: Green dashed

**Component structure:**
```vue
<template>
  <svg class="los-debug-overlay" v-if="visible">
    <!-- Walls -->
    <polyline
      v-for="wall in walls"
      :points="wall.points.map(p => `${p.x},${p.y}`).join(' ')"
      stroke="rgba(251, 191, 36, 0.6)"
      stroke-width="2"
      fill="none"
    />
    <!-- Portals -->
    <line
      v-for="portal in portals"
      :class="{ 'portal-closed': portal.is_closed }"
      ...
    />
  </svg>
</template>
```

**Integration:** Add to DmMapViewer toolbar alongside fog toggle

### Dependencies
Depends on: MIMIR-T-0240 (UVTT file parsed for walls/portals)

### Risk Considerations
SVG performance with many walls; may need canvas fallback

## Status Updates **[REQUIRED]**

*To be added during implementation*