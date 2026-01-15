---
id: race-type-migration
level: task
title: "Race Type Migration"
short_code: "MIMIR-T-0346"
created_at: 2026-01-14T15:49:22.506975+00:00
updated_at: 2026-01-15T02:02:39.595167+00:00
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

# Race Type Migration

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[MIMIR-I-0037]]

## Objective

Migrate the race model (`race.rs`) from 4 `serde_json::Value` fields to fully typed structs, handling the polymorphic speed, lineage, and physical characteristics fields.

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

**Target:** `crates/mimir-dm-core/src/models/catalog/race.rs`

The race model has 4 Value fields with polymorphic behavior that needs proper typing.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] Replace `speed: Option<Value>` with `RaceSpeed` (number or SpeedObject)
- [ ] Replace `lineage: Option<Value>` with `Lineage` enum (bool or source string)
- [ ] Replace `height_and_weight: Option<Value>` with `HeightAndWeight` struct
- [ ] Handle additional speed-related Value fields if present
- [ ] Add deserialization tests for polymorphic race fields
- [ ] All existing tests continue to pass

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

### Session 2026-01-15

**Completed Race Type Migration**

1. **Analyzed race.rs Value fields** - Found 4 fields: `speed` and `lineage` and `height_and_weight` in Race, `speed` in Subrace

2. **Examined actual JSON data** - Reviewed PHB and AAG race data to understand polymorphic patterns:
   - `speed`: Can be number (30) or object with walk/fly/swim/climb/burrow keys
   - Speed values can be numbers or `true` (meaning "equal to walk speed")
   - `lineage`: Can be boolean flag or source string like "VRGR"
   - `heightAndWeight`: Structured with baseHeight, baseWeight, heightMod, weightMod

3. **Added new types to types.rs**:
   - `RaceSpeed` enum (Number or Object variants) with `walk_speed()` helper
   - `RaceSpeedObject` struct with optional movement types
   - Helper methods on `SpeedValue` (`as_number()`, `is_equal_to_walk()`)
   - `Lineage` enum (Flag or Source variants)
   - `HeightAndWeight` struct

4. **Updated race.rs**:
   - Changed `speed: Option<Value>` to `speed: Option<RaceSpeed>` (Race and Subrace)
   - Changed `lineage: Option<Value>` to `lineage: Option<Lineage>`
   - Changed `height_and_weight: Option<Value>` to `height_and_weight: Option<HeightAndWeight>`
   - Updated `From<&Race> for RaceSummary` to use typed speed
   - Updated `From<&Subrace> for RaceSummary` to use typed speed

5. **Fixed dependent services**:
   - `creation.rs`: Updated speed extraction to use `race.speed.as_ref().map(|s| s.walk_speed()).unwrap_or(30)`
   - `reference_service.rs`: Updated speed formatting to use typed RaceSpeed

6. **Added 9 deserialization tests**:
   - test_parse_race_speed_number
   - test_parse_race_speed_object
   - test_parse_race_speed_with_swim_true
   - test_parse_lineage_string
   - test_parse_lineage_bool
   - test_parse_height_and_weight
   - test_parse_minimal_race
   - test_race_summary_from_race
   - test_subrace_speed

**Results**: All 9 new tests pass, all 465 existing tests continue to pass.