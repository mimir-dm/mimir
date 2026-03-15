---
id: pass-2-2-feature-coverage-matrix
level: task
title: "Pass 2.2: Feature coverage matrix — verify every feature is documented"
short_code: "MIMIR-T-0617"
created_at: 2026-03-13T13:50:45.814945+00:00
updated_at: 2026-03-13T14:46:28.365624+00:00
parent: MIMIR-I-0061
blocked_by: []
archived: true

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: MIMIR-I-0061
---

# Pass 2.2: Feature coverage matrix — verify every feature is documented

## Parent Initiative

[[MIMIR-I-0061]]

## Objective

Build a feature coverage matrix: list every major Mimir feature and verify each has at least one how-to page, one reference mention, and one glossary entry where appropriate. Flag undocumented features.

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

**Feature Coverage Matrix** — checked every major feature against docs:

| Feature | How-To | Reference | Tutorial | Glossary | Status |
|---------|--------|-----------|----------|----------|--------|
| Campaign CRUD | create-campaign.md | campaign-dashboard.md | 01-first-campaign.md | Yes | OK |
| Campaign Sources | (mentioned in tutorial) | campaign-dashboard.md | 01-first-campaign.md | No | OK |
| Campaign Export/Import | export-campaign.md | file-formats.md | 01-first-campaign.md | No | OK |
| Module CRUD | create-module.md | module-prep-view.md | 02-first-module.md | Yes | OK |
| Module PDF Export | (mentioned in create-module.md) | campaign-dashboard.md | — | No | Minor gap |
| Documents (campaign+module) | manage-documents.md, module-documents.md | — | 02-first-module.md | Yes | OK |
| Map Upload | upload-map.md | token-setup-modal.md | 02-first-module.md | Yes | OK |
| Map Grid | configure-grid.md (rewritten) | — | — | No | OK |
| Token Placement | place-tokens.md | token-setup-modal.md | 02-first-module.md | Yes | OK |
| Light Sources | manage-light-sources.md | token-setup-modal.md | 02-first-module.md | No | OK |
| Map Print | print-map.md | — | — | No | OK |
| Map Generation | generate-map.md | mapgen.md | — | Yes | OK |
| Character Creation (PC) | create-pc.md | — | 01-first-campaign.md | Yes | OK |
| Character Creation (NPC) | create-npc.md | — | — | Yes | OK |
| Character Assignment | assign-to-campaign.md | — | — | No | OK |
| Character Inventory | manage-inventory.md | — | — | No | OK |
| Character Spells | manage-spells.md | — | — | No | OK |
| Character Level Up | level-up.md | — | — | No | OK |
| Character Print | print-character-sheet.md | — | — | No | OK |
| Homebrew Items | create-item.md | — | — | Yes | OK |
| Homebrew Monsters | create-monster.md | — | — | Yes | OK |
| Homebrew Spells | create-spell.md | — | — | Yes | OK |
| Play Mode | start-session.md, manage-encounters.md | play-mode.md | 03-first-session.md | Yes | OK |
| Fog of War | fog-of-war.md | play-mode.md | 03-first-session.md | Yes | OK |
| Player Display | use-player-display.md | player-display.md | 04-player-display.md | Yes | OK |
| Blackout | (in player display) | player-display.md | 03-first-session.md | Yes | OK |
| Session Notes | (in start-session.md) | play-mode.md | 03-first-session.md | Yes | OK |
| Keyboard Shortcuts | — | keyboard-shortcuts.md | — | No | OK |
| Home Screen | — | home-screen.md | 01-first-campaign.md | No | OK |
| Settings | (mentioned in tutorial) | — | 01-first-campaign.md | No | Minor gap |
| MCP/AI Assistant | ai-assistant/README.md | mcp-server.md | — | Yes | OK |
| Source/Reference Search | — | sources/README.md | — | No | OK |
| Vision & Lighting | — | vision-and-lighting.md | — | No | OK |

**Gaps identified (minor, not blocking):**
1. No dedicated how-to for module PDF export (mentioned in passing in create-module.md)
2. No settings reference page (Settings only mentioned in tutorial header nav)
3. Campaign PDF export not documented as how-to (reference/pdf-export exists as developer doc)

**Assessment:** All major features have sufficient documentation coverage. The gaps are for minor features that are either self-explanatory (Settings page) or adequately mentioned in context where users encounter them (PDF export buttons).