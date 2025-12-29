---
id: add-database-migration-for-los
level: task
title: "Add database migration for los_walls and los_portals tables"
short_code: "MIMIR-T-0228"
created_at: 2025-12-25T16:41:50.087017+00:00
updated_at: 2025-12-25T16:41:50.087017+00:00
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

# Add database migration for los_walls and los_portals tables

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[MIMIR-I-0028]]

## Objective **[REQUIRED]**

Add database migration for LOS geometry storage (walls and portals tables) with breaking changes to maps table for UVTT resolution fields.

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

## Acceptance Criteria **[REQUIRED]**

- [ ] Migration 040_add_los_geometry creates los_walls table
- [ ] Migration creates los_portals table with is_closed boolean
- [ ] Maps table has pixels_per_grid, grid_origin_x, grid_origin_y columns
- [ ] CASCADE delete removes LOS data when map deleted
- [ ] Indexes on map_id for both tables
- [ ] Rust models in `models/campaign/los.rs` match schema
- [ ] Down migration cleanly reverts changes

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

**Migration:** `040_add_los_geometry/up.sql`

```sql
-- Wall polylines (from UVTT line_of_sight)
CREATE TABLE los_walls (
  id INTEGER PRIMARY KEY,
  map_id INTEGER NOT NULL REFERENCES maps(id) ON DELETE CASCADE,
  points TEXT NOT NULL,  -- JSON: [{x: 5.0, y: 3.0}, ...]
  created_at TEXT NOT NULL DEFAULT (datetime('now'))
);
CREATE INDEX idx_los_walls_map ON los_walls(map_id);

-- Doors/windows (from UVTT portals)
CREATE TABLE los_portals (
  id INTEGER PRIMARY KEY,
  map_id INTEGER NOT NULL REFERENCES maps(id) ON DELETE CASCADE,
  position_x REAL NOT NULL,
  position_y REAL NOT NULL,
  bounds TEXT NOT NULL,  -- JSON array of points
  rotation REAL DEFAULT 0,
  is_closed BOOLEAN DEFAULT 1,
  created_at TEXT NOT NULL DEFAULT (datetime('now')),
  updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);
CREATE INDEX idx_los_portals_map ON los_portals(map_id);

-- Maps table additions for UVTT resolution
ALTER TABLE maps ADD COLUMN pixels_per_grid INTEGER;
ALTER TABLE maps ADD COLUMN grid_origin_x REAL DEFAULT 0;
ALTER TABLE maps ADD COLUMN grid_origin_y REAL DEFAULT 0;
```

**Models:** `crates/mimir-dm-core/src/models/campaign/los.rs`

### Dependencies
Depends on: MIMIR-T-0227 (types reference UvttPoint)

### Risk Considerations
Breaking change - existing maps won't have pixels_per_grid (nullable)

## Status Updates **[REQUIRED]**

*To be added during implementation*