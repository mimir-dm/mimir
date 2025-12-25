---
id: create-losgeometryservice-for-wall
level: task
title: "Create LosGeometryService for wall and portal CRUD"
short_code: "MIMIR-T-0229"
created_at: 2025-12-25T16:41:50.187988+00:00
updated_at: 2025-12-25T16:41:50.187988+00:00
parent: MIMIR-I-0028
blocked_by: []
archived: true

tags:
  - "#task"
  - "#phase/todo"


exit_criteria_met: false
strategy_id: NULL
initiative_id: MIMIR-I-0028
---

# Create LosGeometryService for wall and portal CRUD

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[MIMIR-I-0028]]

## Objective **[REQUIRED]**

Create LosGeometryService for CRUD operations on walls and portals, including bulk import from UVTT files.

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

## Acceptance Criteria **[REQUIRED]**

- [ ] `get_walls_for_map(map_id)` returns all wall polylines
- [ ] `get_portals_for_map(map_id)` returns all portals with state
- [ ] `toggle_portal(id)` flips is_closed and returns new state
- [ ] `import_from_uvtt(map_id, uvtt)` creates walls and portals in transaction
- [ ] `delete_walls_for_map(map_id)` clears all walls
- [ ] Integration tests verify CRUD operations
- [ ] Service registered in mod.rs exports

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

**Location:** `crates/mimir-dm-core/src/services/los_geometry_service.rs`

```rust
pub struct LosGeometryService;

impl LosGeometryService {
    // Walls
    pub fn create_wall(conn: &Connection, map_id: i32, points: Vec<Point>) -> Result<LosWall>;
    pub fn get_walls_for_map(conn: &Connection, map_id: i32) -> Result<Vec<LosWall>>;
    pub fn delete_walls_for_map(conn: &Connection, map_id: i32) -> Result<()>;

    // Portals
    pub fn create_portal(conn: &Connection, map_id: i32, portal: NewPortal) -> Result<LosPortal>;
    pub fn get_portals_for_map(conn: &Connection, map_id: i32) -> Result<Vec<LosPortal>>;
    pub fn toggle_portal(conn: &Connection, id: i32) -> Result<bool>;
    
    // Bulk import
    pub fn import_from_uvtt(conn: &Connection, map_id: i32, uvtt: &UvttFile) -> Result<LosImportResult>;
}

pub struct LosImportResult {
    pub walls_created: usize,
    pub portals_created: usize,
}
```

### Dependencies
Depends on: MIMIR-T-0227 (UvttFile type), MIMIR-T-0228 (database tables)

### Risk Considerations
JSON serialization of points arrays must be consistent

## Status Updates **[REQUIRED]**

*To be added during implementation*