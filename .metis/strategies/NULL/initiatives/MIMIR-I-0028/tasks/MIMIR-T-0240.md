---
id: simplify-maps-table-to-uvtt-file
level: task
title: "Simplify maps table to UVTT file reference"
short_code: "MIMIR-T-0240"
created_at: 2025-12-25T17:02:32.139735+00:00
updated_at: 2025-12-25T17:02:32.139735+00:00
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

# Simplify maps table to UVTT file reference

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[MIMIR-I-0028]]

## Objective **[REQUIRED]**

Refactor maps table to minimal schema where UVTT file is source of truth for all map data (grid, dimensions, LOS, lights).

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

- [ ] Maps table reduced to: id, campaign_id, module_id, name, file_path, timestamps
- [ ] Hex grid support removed (square only, matches UVTT)
- [ ] file_path points to .uvtt file (campaigns/{id}/maps/ or modules/{id}/maps/)
- [ ] All grid/dimension data derived from parsing UVTT on load
- [ ] Migration handles existing maps (create UVTT wrappers)
- [ ] MapService returns parsed UVTT data with map
- [ ] Backwards compatible with existing functionality

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

**New minimal schema:**
```sql
CREATE TABLE maps (
  id INTEGER PRIMARY KEY,
  campaign_id INTEGER NOT NULL REFERENCES campaigns(id),
  module_id INTEGER REFERENCES modules(id),
  name TEXT NOT NULL,
  file_path TEXT NOT NULL,  -- Path to .uvtt file
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);
```

**Storage paths:**
- Campaign maps: `{data_dir}/campaigns/{campaign_id}/maps/{uuid}.uvtt`
- Module maps: `{data_dir}/modules/{module_id}/maps/{uuid}.uvtt`

**Migration strategy:**
1. For each existing map with image_path:
   - Read image file
   - Create minimal UVTT wrapper with existing grid config
   - Save to appropriate path (campaign or module)
   - Update file_path to new .uvtt
2. Drop old columns (grid_*, width_*, etc.)

**MapService changes:**
```rust
pub struct MapWithData {
    pub map: Map,           // DB record
    pub uvtt: UvttFile,     // Parsed file
}

impl MapService {
    pub fn get_map_with_data(id: i32) -> Result<MapWithData> {
        let map = self.get_map(id)?;
        let uvtt = UvttService::parse_file(&map.file_path)?;
        Ok(MapWithData { map, uvtt })
    }
}
```

### Dependencies
Depends on: MIMIR-T-0227 (UVTT parser)

### Risk Considerations
Migration must handle existing maps without data loss

## Status Updates **[REQUIRED]**

*To be added during implementation*