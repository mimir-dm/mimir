---
id: migration-characterinventory
level: task
title: "Migration: CharacterInventory, CharacterProficiency, CharacterSpell tables"
short_code: "MIMIR-T-0386"
created_at: 2026-01-20T21:49:43.093534+00:00
updated_at: 2026-01-21T01:07:27.306551+00:00
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

# Migration: CharacterInventory, CharacterProficiency, CharacterSpell tables

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[MIMIR-I-0043]]

## Objective

Create CharacterInventory, CharacterProficiency, and CharacterSpell tables.

## Schema

```sql
CREATE TABLE character_inventory (
    id INTEGER PRIMARY KEY,
    character_id INTEGER NOT NULL REFERENCES characters(id) ON DELETE CASCADE,
    item_name TEXT NOT NULL,
    item_source TEXT NOT NULL,
    quantity INTEGER NOT NULL DEFAULT 1,
    equipped INTEGER NOT NULL DEFAULT 0,
    attuned INTEGER NOT NULL DEFAULT 0,
    notes TEXT
);

CREATE INDEX idx_character_inventory_character ON character_inventory(character_id);

CREATE TABLE character_proficiencies (
    id INTEGER PRIMARY KEY,
    character_id INTEGER NOT NULL REFERENCES characters(id) ON DELETE CASCADE,
    proficiency_type TEXT NOT NULL,  -- 'skill', 'save', 'tool', 'weapon', 'armor', 'language'
    name TEXT NOT NULL,
    expertise INTEGER NOT NULL DEFAULT 0
);

CREATE INDEX idx_character_proficiencies_character ON character_proficiencies(character_id);

CREATE TABLE character_spells (
    id INTEGER PRIMARY KEY,
    character_id INTEGER NOT NULL REFERENCES characters(id) ON DELETE CASCADE,
    spell_name TEXT NOT NULL,
    spell_source TEXT NOT NULL,
    source_class TEXT NOT NULL,
    equipped INTEGER NOT NULL DEFAULT 0  -- prepared/chosen
);

CREATE INDEX idx_character_spells_character ON character_spells(character_id);
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
- [ ] Create Rust models for all three tables
- [ ] Create DAL functions for all three tables
- [ ] Add tests for CRUD operations
- [ ] Verify CASCADE delete (character deletion removes all related data)

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
- Created migration 014_character_details with up.sql and down.sql
- Used TEXT primary key (UUID) consistent with other campaign tables
- Created CharacterInventory model with equipped/attuned tracking
- Created CharacterProficiency model with ProficiencyType enum (skill, save, tool, weapon, armor, language) and expertise support
- Created CharacterSpell model with prepared status and source_class tracking
- Created comprehensive DAL functions including:
  - list_equipped_items, list_attuned_items, count_attuned_items
  - list_skill_proficiencies, list_save_proficiencies, list_languages, list_expertise
  - list_prepared_spells, list_spells_by_class, character_knows_spell
- All 468 tests passing

Files created:
- migrations/014_character_details/up.sql
- migrations/014_character_details/down.sql
- src/models/campaign/character_inventory.rs
- src/models/campaign/character_proficiency.rs
- src/models/campaign/character_spell.rs
- src/dal/campaign/character_inventory.rs
- src/dal/campaign/character_proficiency.rs
- src/dal/campaign/character_spell.rs