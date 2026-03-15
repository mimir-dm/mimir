---
id: pass-3-3-adversarial-review-how-to
level: task
title: "Pass 3.3: Adversarial review — how-to guides (characters, play mode, homebrew, AI assistant)"
short_code: "MIMIR-T-0621"
created_at: 2026-03-13T13:50:50.427595+00:00
updated_at: 2026-03-13T15:27:50.744098+00:00
parent: MIMIR-I-0061
blocked_by: []
archived: true

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: MIMIR-I-0061
---

# Pass 3.3: Adversarial review — how-to guides (characters, play mode, homebrew, AI assistant)

## Parent Initiative

[[MIMIR-I-0061]]

## Objective

**Adversarial review** of all how-to guides for characters, play mode, homebrew, and AI assistant. Pay special attention to newly written pages (manage-spells, manage-inventory, level-up, homebrew section, AI assistant) which haven't been through a prior audit. Every claim must be verified against code. Fix any errors found.

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

**Pages reviewed:** All how-to pages for characters (7), play mode (4), homebrew (4), and AI assistant (1) — 16 pages total.

**Fixes applied:**

**Characters:**
1. **create-pc.md** — "Click + New Character" → "Click Create Character" (actual button text). Removed "Select Player Character" step (type selection hidden when entering from Characters nav).
2. **create-npc.md** — Changed entry path from "Characters header → + New Character → Select NPC" to "Campaign dashboard → NPCs tab → Create NPC" (NPCs are created from campaign dashboard, not Characters header).
3. **assign-to-campaign.md** — "Add Character" → "Add Existing" (actual button). "Assign" → "Add to Campaign" (actual button). Removed "Save" from character list flow (assignment happens on dropdown change, no Save button). Fixed NPC assignment to "Create NPC" instead of "Add NPC → Select or create".
4. **level-up.md** — "Level Up on the character sheet" → "Level Up button on the character card" (button is on list cards, not sheet). "up to 8 steps" → "relevant steps, skipped automatically" (most steps are conditional). "Ability Score Improvement" → "Ability Score" (actual step label).
5. **print-character-sheet.md** — "Click Print or Export PDF" → "Click Print PDF" (actual button text).
6. **manage-inventory.md** — "Equipment tab has three sub-tabs" → "Inventory & Equipment dialog has three tabs" (it's a modal dialog, not a sub-tab). Added "Click + Add Item" step (the open button). Renamed "Sub-Tab" headings to "Tab".

**Play Mode:**
7. **start-session.md** — "Find your module in the modules table, Click the Play button in the module's row" → "Select a module, Click the Play button in the module header". "Session notes" → "Play notes" (actual label).
8. **fog-of-war.md** — Rewrote vision modes section: was describing a single "LOS" switch toggling between "Fog Mode" and "Token Mode" — actual UI has two independent buttons ("Fog" and "LOS"). Fixed "Switch to Token mode" tip.
9. **manage-encounters.md** — "Right-click a token to toggle visibility" → "Right-click a token and select the visibility option from the context menu" (it's a menu item, not direct toggle).

**AI Assistant:**
10. **ai-assistant/README.md** — "40+ tools" → "70+ tools" (actual count is 71). Added missing "Homebrew Management" tool category.

**Other:**
11. **characters/README.md** — Added missing links to manage-inventory, manage-spells, and level-up pages.

**No issues found in:** manage-spells.md, homebrew/README.md, create-item.md, create-monster.md, create-spell.md, play-mode/use-player-display.md (already fixed in prior tasks).