---
id: pass-3-4-adversarial-review
level: task
title: "Pass 3.4: Adversarial review — reference pages (UI, shortcuts, formats, glossary)"
short_code: "MIMIR-T-0622"
created_at: 2026-03-13T13:50:56.907161+00:00
updated_at: 2026-03-13T15:32:27.259467+00:00
parent: MIMIR-I-0061
blocked_by: []
archived: true

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: MIMIR-I-0061
---

# Pass 3.4: Adversarial review — reference pages (UI, shortcuts, formats, glossary)

## Parent Initiative

[[MIMIR-I-0061]]

## Objective

**Adversarial review** of all reference pages: 6 UI reference pages, keyboard shortcuts, file formats, vision-and-lighting, and glossary. Verify every keyboard shortcut by finding its keydown handler. Verify every UI element described exists in its component. Verify every file format entry. Verify every glossary definition. Fix any errors found.

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

## Status Updates

### Session 1 (2026-03-13)

**Pages reviewed:** All reference pages — 6 UI references, keyboard shortcuts, file formats, vision-and-lighting, glossary, MCP server, mapgen — 13 pages total.

**Fixes applied:**

**Keyboard Shortcuts (keyboard-shortcuts.md):**
1. Removed phantom `Cmd/Ctrl + ,` → Open Settings shortcut (no keydown handler exists; settings opens via gear icon click only).
2. Fixed `0` description from "Reset view (fit map)" to "Reset view (1:1 zoom, centered)" (resets to zoom=1, doesn't calculate fit-to-viewport).
3. Editor shortcuts (Cmd+B/I/Z) are Tiptap StarterKit defaults — kept as documented since they do work.

**Play Mode Reference (play-mode.md):**
4. "LOS Toggle — Switch between Fog and Token modes" → Two separate entries: "Fog — Toggle fog of war" and "LOS — Toggle token line of sight" (two independent buttons, not a single toggle).
5. "Debug — Visualize line of sight" → "Debug — Toggle debug overlays (vision ranges, walls)" (matches actual tooltip).
6. "Session Notes" → "Play Notes" (actual UI label).
7. Fixed module entry: "module's table row" → "select module, click Play in module header".

**Campaign Dashboard (campaign-dashboard.md):**
8. "Assign PC" → "Add Existing" (actual button label).

**Token Setup Modal (token-setup-modal.md):**
9. "Click any map card" → "Click the Place Tokens button on a map card" (3 places).
10. "Token Inventory (Right)" → "Placed Tokens (Right)" (actual heading).

**Module Prep View (module-prep-view.md):**
11. "Click to open Token Setup" → "Click the Place Tokens button to open Token Setup".
12. Actions table: same fix for Token Setup entry.
13. "Print" button → "PDF" (actual button text in module header).

**Player Display (player-display.md):**
14. Renamed "Fog of War Modes" → "Fog of War Controls" with "Fog" and "LOS" as independent toggles.

**Vision & Lighting (vision-and-lighting.md):**
15. Same Fog/LOS mode rename — independent toggles, not named modes.

**Glossary (glossary.md):**
16. "Grid — Configurable size and offset" → "Auto-detected from UVTT files; defaults to 70 pixels per square for image files" (no user-facing grid config UI).
17. "Session Notes" → "Play Notes" (actual UI label).
18. "Viewport — Can be synced between DM and player display" → "DM and player display each have independent viewports" (no sync feature exists).
19. Fixed alphabetical ordering: moved G section before H section.

**MCP Server (mcp-server.md):**
20. "70 tools" → "71 tools" (actual count: 10+8+6+13+8+3+8+5+5+5=71).

**No issues found in:** file-formats.md, home-screen.md, mapgen.md.