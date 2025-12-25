---
id: add-import-uvtt-tauri-command-for
level: task
title: "Add import_uvtt Tauri command for UVTT file processing"
short_code: "MIMIR-T-0230"
created_at: 2025-12-25T16:41:50.289830+00:00
updated_at: 2025-12-25T16:41:50.289830+00:00
parent: MIMIR-I-0028
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/todo"


exit_criteria_met: false
strategy_id: NULL
initiative_id: MIMIR-I-0028
---

# Add import_uvtt Tauri command for UVTT file processing

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[MIMIR-I-0028]]

## Objective **[REQUIRED]**

Add Tauri command `import_uvtt` that handles end-to-end UVTT file import: parsing, image extraction, map creation, and LOS geometry import.

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

## Acceptance Criteria **[REQUIRED]**

- [ ] `import_uvtt` command accepts file bytes, campaign_id, optional module_id, name
- [ ] UVTT file validated before saving
- [ ] Campaign maps saved to `campaigns/{id}/maps/`
- [ ] Module maps saved to `modules/{id}/maps/`
- [ ] Directories created if they don't exist
- [ ] Minimal map record created (just location + file_path)
- [ ] TypeScript bindings generated for frontend

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

## Implementation Notes

### Technical Approach

**Location:** `crates/mimir-dm/src/commands/campaign/maps.rs`

```rust
#[tauri::command]
pub async fn import_uvtt(
    state: State<'_, AppState>,
    campaign_id: i32,
    module_id: Option<i32>,
    name: String,
    file_data: Vec<u8>,
    import_los: bool,
    import_lights: bool,
) -> Result<Map, Error>
```

**Steps:**
1. Validate UVTT file structure
2. Determine storage path:
   - Campaign map: `{data_dir}/campaigns/{campaign_id}/maps/{uuid}.uvtt`
   - Module map: `{data_dir}/modules/{module_id}/maps/{uuid}.uvtt`
3. Create directories if needed, save .uvtt file
4. Create minimal Map record (id, campaign_id, module_id, name, file_path)
5. Return Map

### Dependencies
Depends on: MIMIR-T-0227 (parser), MIMIR-T-0240 (simplified schema)

### Risk Considerations
Large UVTT files (5MB+) may need streaming or chunked processing

## Status Updates **[REQUIRED]**

*To be added during implementation*