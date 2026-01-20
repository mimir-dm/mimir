---
id: remaining-entity-type-migrations
level: task
title: "Remaining Entity Type Migrations"
short_code: "MIMIR-T-0347"
created_at: 2026-01-14T15:49:22.610803+00:00
updated_at: 2026-01-15T02:21:07.362970+00:00
parent: MIMIR-I-0037
blocked_by: [MIMIR-T-0343]
archived: true

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: MIMIR-I-0037
---

# Remaining Entity Type Migrations

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[MIMIR-I-0037]]

## Objective

Complete type migrations for remaining catalog entities: background, spell, and optionalfeature models, addressing their `serde_json::Value` fields.

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

**Target:** `crates/mimir-dm-core/src/models/catalog/`
- `background.rs`
- `spell.rs`
- `optionalfeature.rs`

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] Background: Replace `starting_equipment: Option<Value>` with `Vec<StartingEquipmentEntry>`
- [ ] Spell: Replace `MaterialComponent.consume: Option<Value>` with `ConsumeValue` enum
- [ ] OptionalFeature: Replace `additional_spells: Option<Value>` with existing `AdditionalSpells` type
- [ ] Audit each file for any other untyped Value fields
- [ ] Add deserialization tests for migrated fields
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

**Completed Remaining Entity Type Migrations**

1. **Analyzed all three target files**:
   - `background.rs`: `starting_equipment: Vec<serde_json::Value>` on line 16
   - `spell.rs`: `consume: Option<serde_json::Value>` in MaterialComponent::Object
   - `optionalfeature.rs`: `additional_spells: Option<Vec<serde_json::Value>>`

2. **Examined actual JSON data** to understand polymorphic patterns:
   - `startingEquipment`: Array of choice groups with `_` for default items, `a`/`b`/`c` for alternatives
   - Items can be strings ("common clothes|phb") or objects with item/special/displayName/quantity/containsValue
   - `consume`: Can be boolean `true` or string `"optional"`
   - `additionalSpells`: Already has existing `AdditionalSpells` type in types.rs

3. **Added new types to types.rs**:
   - `StartingEquipmentEntry` enum (wrapping ChoiceGroup)
   - `StartingEquipmentChoiceGroup` struct with default_items, a, b, c fields
   - `StartingEquipmentItem` enum (ItemRef or Object variants)
   - `StartingEquipmentItemObject` struct with item, special, display_name, quantity, contains_value
   - `ConsumeValue` enum (Flag or Text variants) with helper methods

4. **Updated model files**:
   - `background.rs`: Changed `Vec<serde_json::Value>` to `Vec<StartingEquipmentEntry>`
   - `spell.rs`: Changed `Option<serde_json::Value>` to `Option<ConsumeValue>`
   - `optionalfeature.rs`: Changed `Option<Vec<serde_json::Value>>` to `Option<Vec<AdditionalSpells>>`

5. **Added 12 deserialization tests**:
   - Background: test_parse_starting_equipment_with_items, test_parse_starting_equipment_with_choices, test_parse_empty_starting_equipment
   - Spell: test_parse_consume_bool_true, test_parse_consume_optional, test_parse_material_component_no_consume, test_parse_material_component_text, test_parse_material_component_bool
   - OptionalFeature: test_parse_additional_spells_innate, test_parse_optional_feature_no_additional_spells, test_optional_feature_summary_grants_spells, test_optional_feature_summary_no_spells

**Results**: All 12 new tests pass, all 465 existing tests continue to pass.