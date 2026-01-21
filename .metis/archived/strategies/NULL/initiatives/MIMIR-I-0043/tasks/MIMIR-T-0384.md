---
id: migration-character-table-with
level: task
title: "Migration: Character table with ability scores"
short_code: "MIMIR-T-0384"
created_at: 2026-01-20T21:49:42.165268+00:00
updated_at: 2026-01-21T00:59:45.054612+00:00
parent: MIMIR-I-0043
blocked_by: []
archived: true

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: MIMIR-I-0043
---

# Migration: Character table with ability scores

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[MIMIR-I-0043]]

## Objective

Create the Character table for PCs and NPCs with base ability scores, currency, and personality fields. Derived values (AC, HP, etc.) calculated at view time.

## Schema

```sql
CREATE TABLE characters (
    id INTEGER PRIMARY KEY,
    campaign_id INTEGER NOT NULL REFERENCES campaigns(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    is_npc INTEGER NOT NULL DEFAULT 0,
    player_name TEXT,
    -- Base ability scores
    strength INTEGER NOT NULL DEFAULT 10,
    dexterity INTEGER NOT NULL DEFAULT 10,
    constitution INTEGER NOT NULL DEFAULT 10,
    intelligence INTEGER NOT NULL DEFAULT 10,
    wisdom INTEGER NOT NULL DEFAULT 10,
    charisma INTEGER NOT NULL DEFAULT 10,
    -- Currency
    cp INTEGER NOT NULL DEFAULT 0,
    sp INTEGER NOT NULL DEFAULT 0,
    ep INTEGER NOT NULL DEFAULT 0,
    gp INTEGER NOT NULL DEFAULT 0,
    pp INTEGER NOT NULL DEFAULT 0,
    -- Personality
    traits TEXT,
    ideals TEXT,
    bonds TEXT,
    flaws TEXT,
    -- NPC fields
    role TEXT,
    location TEXT,
    faction TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_characters_campaign ON characters(campaign_id);
CREATE INDEX idx_characters_is_npc ON characters(is_npc);
```

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

## Acceptance Criteria

- [ ] Create migration with up.sql and down.sql
- [ ] Run diesel migration and update schema.rs
- [ ] Create Rust models: Character, NewCharacter
- [ ] Create DAL functions: insert, get, list_by_campaign, list_npcs, list_pcs, update, delete
- [ ] Add tests for CRUD operations
- [ ] Verify CASCADE delete (campaign deletion removes characters)

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

### Session 2026-01-20
- Created migration 012_characters with up.sql and down.sql
- Used TEXT primary key (UUID) consistent with other campaign tables
- Created Character, NewCharacter, UpdateCharacter models with builder patterns
- Added helper methods for ability modifiers (using div_euclid for proper D&D floor division)
- Added total_gold_value() method for currency calculations
- Created DAL with CRUD operations plus specialized queries (list_pcs, list_npcs, list_npcs_by_location, list_npcs_by_faction)
- All 405 tests passing

Files created:
- migrations/012_characters/up.sql
- migrations/012_characters/down.sql
- src/models/campaign/character.rs
- src/dal/campaign/character.rs