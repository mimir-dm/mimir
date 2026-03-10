---
id: create-rust-integration-test
level: task
title: "Create Rust integration test harness with SRD-seeded test database"
short_code: "MIMIR-T-0535"
created_at: 2026-03-09T14:25:11.158516+00:00
updated_at: 2026-03-10T01:17:31.395505+00:00
parent: MIMIR-I-0056
blocked_by: []
archived: true

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: MIMIR-I-0056
---

# Create Rust integration test harness with SRD-seeded test database

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[MIMIR-I-0056]]

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

## Acceptance Criteria

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

### 2026-03-09: Complete

**Files created:**
- `crates/mimir-core/tests/common/mod.rs` — Shared test harness with `setup_srd_db()` and `setup_srd_db_with_extra_sources()`. Loads all SRD fixture JSON files, dynamically discovers all required source codes, seeds classes/subclasses/features/backgrounds/races/items/spells/monsters.
- `crates/mimir-core/tests/srd_smoke_test.rs` — 11 smoke tests verifying:
  - All 12 SRD classes present and queryable
  - 4 key subclasses (Champion, Thief, Evocation, Life Domain)
  - Class features spanning multiple levels
  - Subclass features including children (Fast Hands, Improved Critical)
  - Backgrounds (Acolyte, Soldier)
  - Races (Human, Elf)
  - Spells (Fireball level 3, Fire Bolt cantrip, 20+ total)
  - Items (Longsword)
  - Monsters (Goblin, 10+ total)
  - JSON data blob validity for all classes and spells
  - entity_to_json roundtrip correctness

**Key design decisions:**
- Source codes extracted dynamically from fixture data (handles PHB, MM, DMG, XDMG without hardcoding)
- Uses in-memory SQLite (`:memory:`) for speed — no cleanup needed
- Reuses existing DAL insert functions rather than raw SQL
- Strips metadata fields (id, fluff, className etc.) from data blobs before insertion to match production format

**Results:** 11/11 smoke tests pass. All existing mimir-core tests unaffected.