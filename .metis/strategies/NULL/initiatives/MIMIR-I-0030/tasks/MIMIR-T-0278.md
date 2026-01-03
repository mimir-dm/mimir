---
id: rewrite-core-module-overview-md
level: task
title: "Rewrite core module_overview.md template"
short_code: "MIMIR-T-0278"
created_at: 2026-01-03T03:56:54.814226+00:00
updated_at: 2026-01-03T13:23:19.747629+00:00
parent: MIMIR-I-0030
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: MIMIR-I-0030
---

# Rewrite core module_overview.md template

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[MIMIR-I-0030]]

## Objective

Rewrite the core `module_overview.md` template to produce complete, runnable adventures instead of just planning documents. Use Frost Architect Module 04 as the reference implementation.

**New template structure:**

**FRONT MATTER (extractable as cards):**
1. **Monsters & Enemies** - stat blocks, tactics (→ monster cards)
2. **Key NPCs** - role, description, motivation (→ NPC cards)
3. **Magic Items & Rewards** - properties, effects (→ item cards)

**MODULE CONTENT:**
4. Overview (pitch, theme, tone, estimated time)
5. Locations (hub + challenge sites)
6. Critical Path (must/should/could happen)
7. Information Architecture (clues with multiple sources)
8. **Adventure Content** (scenes with read-aloud, links to front matter)
9. Puzzles & Traps
10. DM Notes (pacing, tone, scaling)
11. Connections (from/to other modules)
12. Post-Module Notes (checklist, continuity)

Front matter enables cross-linking (`[[Monster Name]]`) and card extraction for physical play.

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

- [ ] Template includes Adventure Content section with scene structure
- [ ] Template includes read-aloud text guidance with blockquote format
- [ ] Template includes Encounters section with stat block format
- [ ] Template includes Puzzles section with solution/hints format
- [ ] Template includes Rewards section
- [ ] Template includes DM Notes (pacing, tone, scaling guidance)
- [ ] Template includes Post-Module Notes checklist
- [ ] Session Breakdown removed or converted to "Estimated Play Time"
- [ ] Works for both AI generation and human authoring

## Files to Modify

- `docs/src/campaign-framework/06-templates/templates/module_overview.md`

## Reference

- Frost Architect Module 04: `/Users/dstorey/Documents/Mimir Campaigns/Frost Architect/modules/module_04/module-overview.md`

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