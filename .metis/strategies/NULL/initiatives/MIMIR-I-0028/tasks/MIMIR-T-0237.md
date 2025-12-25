---
id: create-doorinteractionoverlay-for
level: task
title: "Create DoorInteractionOverlay for play mode portal toggling"
short_code: "MIMIR-T-0237"
created_at: 2025-12-25T16:42:05.308092+00:00
updated_at: 2025-12-25T16:42:05.308092+00:00
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

# Create DoorInteractionOverlay for play mode portal toggling

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[MIMIR-I-0028]]

## Objective **[REQUIRED]**

Create DoorInteractionOverlay component for DmMapViewer that allows DM to click doors to toggle open/closed state during play.

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

- [ ] Doors display clickable icon at portal position
- [ ] Closed doors show closed-door icon (blue per Foundry pattern)
- [ ] Open doors show open-door icon (green per Foundry pattern)
- [ ] Hover state highlights door with subtle glow
- [ ] Click toggles portal state in Vue reactive state
- [ ] UI updates immediately on toggle (optimistic update)
- [ ] Works correctly at all zoom levels
- [ ] Overlay only visible in play mode, not edit mode

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

**File:** `frontend/src/components/DoorInteractionOverlay.vue`

**UX Pattern (from Foundry research):**
- Door icons: Blue (closed), Green (open), Red (locked - future)
- Players can click to open/close if unlocked
- Only DM can unlock (right-click context menu - future)

**Component:**
```vue
<template>
  <div class="door-overlay">
    <button
      v-for="portal in portals"
      :key="portal.id"
      :style="portalPosition(portal)"
      :class="{ 'door-closed': portal.is_closed, 'door-open': !portal.is_closed }"
      @click="toggleDoor(portal.id)"
    >
      <DoorIcon :closed="portal.is_closed" />
    </button>
  </div>
</template>
```

**Icon sizing:** Scale inversely with zoom to maintain consistent click target

### Dependencies
Depends on: MIMIR-T-0233 (VisionService for portal state)

### Risk Considerations
Click targets may be small on dense maps; ensure minimum 24px hit area

## Status Updates **[REQUIRED]**

*To be added during implementation*