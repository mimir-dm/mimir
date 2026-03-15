---
id: pass-3-1-adversarial-review
level: task
title: "Pass 3.1: Adversarial review — tutorials (all 4 pages)"
short_code: "MIMIR-T-0619"
created_at: 2026-03-13T13:50:47.997108+00:00
updated_at: 2026-03-13T15:14:45.724041+00:00
parent: MIMIR-I-0061
blocked_by: []
archived: true

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: MIMIR-I-0061
---

# Pass 3.1: Adversarial review — tutorials (all 4 pages)

## Parent Initiative

[[MIMIR-I-0061]]

## Objective

**Adversarial review** of all 4 tutorials. Assume every claim is wrong and try to disprove it by checking source code. For every step instruction, open the corresponding Vue component and verify the step is possible. Log every claim checked and the source file that confirms or refutes it. Fix any errors found.

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

### 2026-03-13
Adversarially verified 23 claims across all 4 tutorials against source code.

**Results: 20 CONFIRMED, 3 REFUTED (all fixed)**

**Fixed issues:**
1. **Phantom "Sync" and "Push View" buttons** — Tutorial 3, Tutorial 4, play-mode.md, player-display.md, use-player-display.md all referenced auto-sync toggle and Push View button that don't exist in the codebase. Removed all references (6 files fixed).
2. **"Grid button in canvas controls"** — Tutorial 2 Quick Reference table still referenced this phantom feature. Replaced with accurate "Delete token" entry.
3. **Missing PDF button in dashboard header** — Tutorial 1 only mentioned Sources and Export Archive, but the dashboard actually has 3 buttons: Sources, PDF, Export Archive. Fixed tutorial and reference page.
4. **Right-click simplification** — Tutorial 3 said "right-click to toggle visibility". Actually right-click opens a context menu with visibility toggle (and other options). Updated to be accurate.

**Confirmed claims (sample):**
- Header nav items: skull, campaign selector, Characters, Reference, Settings ✓ (AppHeader.vue)
- 5 dashboard tabs ✓ (useDashboardState.ts)
- Create module dialog fields and types ✓ (CreateModuleModal.vue)
- Map upload accepts PNG/JPG/WebP/UVTT ✓ (MapUploadModal.vue)
- Token options: Size/Color/Visible ✓ (TokenPalette.vue)
- Light sources: Torch 20/40, Lantern 30/60, Candle 5/10 ✓ (light_source.rs presets)
- Play Mode header: Back to Prep, Module Name, PLAY MODE, Player Display, Blackout, End Session ✓
- LOS Toggle Fog/Token modes ✓ (DmMapViewer.vue)
- Debug button, Ambient Light Bright/Dim/Dark ✓
- F11 fullscreen on player display ✓ (PlayerDisplayWindow.vue)