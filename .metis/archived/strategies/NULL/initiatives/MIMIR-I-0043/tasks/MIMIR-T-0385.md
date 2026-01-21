---
id: migration-characterclass
level: task
title: "Migration: CharacterClass, CharacterFeat tables"
short_code: "MIMIR-T-0385"
created_at: 2026-01-20T21:49:42.639123+00:00
updated_at: 2026-01-21T01:02:50.930407+00:00
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

# Migration: CharacterClass, CharacterFeat tables

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[MIMIR-I-0043]]

## Objective

Create CharacterClass and CharacterFeat tables for tracking character class levels and selected feats.

## Schema

```sql
CREATE TABLE character_classes (
    id INTEGER PRIMARY KEY,
    character_id INTEGER NOT NULL REFERENCES characters(id) ON DELETE CASCADE,
    class_name TEXT NOT NULL,
    class_source TEXT NOT NULL,
    level INTEGER NOT NULL DEFAULT 1,
    subclass_name TEXT,
    subclass_source TEXT,
    starting_class INTEGER NOT NULL DEFAULT 0
);

CREATE INDEX idx_character_classes_character ON character_classes(character_id);

CREATE TABLE character_feats (
    id INTEGER PRIMARY KEY,
    character_id INTEGER NOT NULL REFERENCES characters(id) ON DELETE CASCADE,
    feat_name TEXT NOT NULL,
    feat_source TEXT NOT NULL,
    source_type TEXT NOT NULL DEFAULT 'asi'  -- 'asi', 'race', 'class', 'bonus'
);

CREATE INDEX idx_character_feats_character ON character_feats(character_id);
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
- [ ] Create Rust models: CharacterClass, NewCharacterClass, CharacterFeat, NewCharacterFeat
- [ ] Create DAL functions for both tables
- [ ] Add tests for CRUD operations
- [ ] Verify CASCADE delete (character deletion removes classes/feats)

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
- Created migration 013_character_classes_feats with up.sql and down.sql
- Used TEXT primary key (UUID) consistent with other campaign tables
- Created CharacterClass, NewCharacterClass, UpdateCharacterClass models with builder patterns
- Created CharacterFeat, NewCharacterFeat models with FeatSourceType enum (asi, race, class, bonus)
- Created DAL functions for both tables including:
  - list_character_classes, get_starting_class, get_total_level
  - list_character_feats, list_feats_by_source_type, character_has_feat
- All 429 tests passing

Files created:
- migrations/013_character_classes_feats/up.sql
- migrations/013_character_classes_feats/down.sql
- src/models/campaign/character_class.rs
- src/models/campaign/character_feat.rs
- src/dal/campaign/character_class.rs
- src/dal/campaign/character_feat.rs