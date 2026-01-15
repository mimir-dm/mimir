---
id: class-type-migration
level: task
title: "Class Type Migration"
short_code: "MIMIR-T-0345"
created_at: 2026-01-14T15:49:22.399445+00:00
updated_at: 2026-01-15T01:52:27.146229+00:00
parent: MIMIR-I-0037
blocked_by: [MIMIR-T-0343]
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: MIMIR-I-0037
---

# Class Type Migration

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[MIMIR-I-0037]]

## Objective

Migrate the class model (`class.rs`) from 9 `serde_json::Value` fields to fully typed structs, making it the highest-impact type migration in the initiative.

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

## Scope

**Target:** `crates/mimir-dm-core/src/models/catalog/class.rs`

The class model has the most Value fields (9) of any catalog type, making this the highest-impact migration.

## Acceptance Criteria

## Acceptance Criteria

- [x] Replace `hd: Option<Value>` with `HitDice` struct
- [x] Replace `proficiency: Option<Value>` with `Vec<String>` for saving throws
- [x] Replace `class_features: Option<Value>` with `Vec<ClassFeatureRef>`
- [x] Replace `starting_proficiencies: Option<Value>` with `StartingProficiencies` struct
- [x] Replace `multiclassing: Option<Value>` with `Multiclassing` struct (typed requirements)
- [x] Replace `class_table_groups: Option<Value>` with `Vec<ClassTableGroup>`
- [x] Replace `starting_equipment: Option<Value>` with `StartingEquipment` struct
- [x] Replace `optionalfeature_progression: Option<Value>` with `Vec<OptionalFeatureProgression>`
- [x] Replace `subclass_features: Option<Value>` with `Vec<String>`
- [x] Add deserialization tests for all 9 field types
- [x] All existing tests continue to pass (465 tests)

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

### 2026-01-14: Completed

**All type migrations completed and tested:**

1. **hd** → `HitDice` struct (already existed, now used)
2. **proficiency** → `Vec<String>` for saving throw abbreviations
3. **class_features** → `Vec<ClassFeatureRef>` with Simple/Object variants
4. **starting_proficiencies** → `StartingProficiencies` struct (already existed, now used)
5. **multiclassing** → `Multiclassing` with typed `MulticlassingRequirements`
6. **class_table_groups** → `Vec<ClassTableGroup>` with typed structure
7. **starting_equipment** → `StartingEquipment` struct (already existed, enhanced with defaultData)
8. **optionalfeature_progression** → `Vec<OptionalFeatureProgression>` with Array/Object progression variants
9. **subclass_features** → `Vec<String>` (subclass feature references)

**Additional changes:**
- Updated `From<&Class>` impl for ClassSummary to use typed fields
- Updated `From<&Class>` impl for NewCatalogClass to use typed fields
- Fixed `level_up.rs` service to use typed HitDice and Multiclassing
- Fixed `reference_service.rs` to use typed HitDice
- Added 11 deserialization tests covering all new types
- All 465 tests pass

**Ready for review and transition to completed.**