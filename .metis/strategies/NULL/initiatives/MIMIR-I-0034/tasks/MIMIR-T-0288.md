---
id: moduleplayview-prep-play-mode-split
level: task
title: "ModulePlayView Prep/Play Mode Split"
short_code: "MIMIR-T-0288"
created_at: 2026-01-03T13:57:42.828692+00:00
updated_at: 2026-01-03T13:57:42.828692+00:00
parent: MIMIR-I-0034
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/todo"


exit_criteria_met: false
strategy_id: NULL
initiative_id: MIMIR-I-0034
---

# ModulePlayView Prep/Play Mode Split

## Overview

Split the 67KB ModulePlayView into focused Prep and Play modes.

## Problem

ModulePlayView handles too many concerns:
- Document editing
- Encounter building
- Monster selection
- Map management
- Initiative tracking
- Player display controls
- Session management

This creates cognitive overload during actual play sessions.

## Chosen Approach: Option C - Prep is Board, Play is Dedicated

**Mental Model:** "I prep on the board, I run in play mode"

- **ModuleBoardView** (existing) → Prep: documents, encounters, monsters, maps
- **ModulePlayView** (refactor) → Play-only: map, quick stats, notes, player display

## Implementation Plan

### Phase 1: Extract Composables
Extract reusable logic from ModulePlayView into composables:

| Composable | Responsibility |
|------------|----------------|
| `useModuleMonsters.ts` | Load monsters, formatting functions (15+ funcs) |
| `useModuleMaps.ts` | Load maps, send to player display |
| `useSessionNotes.ts` | Auto-save notes to file |

### Phase 2: Streamline ModulePlayView
Remove prep concerns, focus on play:

**Remove:**
- Document tabs and viewer (access from board)
- Full encounter browser (just show active)
- Complex sidebar

**Keep/Enhance:**
- Map viewer (make default, more prominent)
- Monster quick-stats panel (slide-in)
- Session notes (always visible bottom bar)
- Player display controls

**Add:**
- "Back to Prep" button
- Encounter selector dropdown (not full browser)

### Phase 3: Update ModuleBoardView
Add "Start Session" button to launch play mode.

## Files to Modify

| File | Action |
|------|--------|
| `composables/useModuleMonsters.ts` | Create |
| `composables/useModuleMaps.ts` | Create |
| `composables/useSessionNotes.ts` | Create |
| `views/ModulePlayView.vue` | Refactor (reduce ~50%) |
| `views/ModuleBoardView.vue` | Add "Start Session" button |

## Acceptance Criteria

- [ ] Monster formatting extracted to composable
- [ ] Map loading/display extracted to composable
- [ ] Session notes extracted to composable
- [ ] ModulePlayView reduced to <1200 lines
- [ ] Play mode defaults to map view
- [ ] Document tabs removed from play mode
- [ ] "Back to Prep" navigation works
- [ ] Type-check passes

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[Parent Initiative]]

## Objective **[REQUIRED]**

{Clear statement of what this task accomplishes}

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

- [ ] {Specific, testable requirement 1}
- [ ] {Specific, testable requirement 2}
- [ ] {Specific, testable requirement 3}

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

{Keep for technical tasks, delete for non-technical. Technical details, approach, or important considerations}

### Technical Approach
{How this will be implemented}

### Dependencies
{Other tasks or systems this depends on}

### Risk Considerations
{Technical risks and mitigation strategies}

## Status Updates **[REQUIRED]**

*To be added during implementation*