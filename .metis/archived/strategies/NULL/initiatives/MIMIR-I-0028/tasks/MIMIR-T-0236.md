---
id: add-portal-toggle-command-and
level: task
title: "Add portal toggle command and state sync"
short_code: "MIMIR-T-0236"
created_at: 2025-12-25T16:42:05.204236+00:00
updated_at: 2025-12-25T16:42:05.204236+00:00
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

# Add portal toggle command and state sync

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[MIMIR-I-0028]]

## Objective **[REQUIRED]**

Add Tauri command to toggle portal open/closed state and sync state changes to PlayerDisplayWindow via IPC.

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

- [ ] `toggle_portal(portal_id)` command flips is_closed state
- [ ] Command returns new portal state for UI update
- [ ] Portal state change emits IPC event to PlayerDisplayWindow
- [ ] PlayerDisplayWindow updates vision calculation on portal change
- [ ] Database updated_at timestamp updated on toggle
- [ ] Bulk toggle command for multiple portals (optional)

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

**Backend:** `crates/mimir-dm/src/commands/campaign/los.rs`

```rust
#[tauri::command]
pub async fn toggle_portal(
    state: State<'_, AppState>,
    portal_id: i32,
) -> Result<LosPortal, Error> {
    let conn = state.db.lock()?;
    let new_state = LosGeometryService::toggle_portal(&conn, portal_id)?;
    
    // Emit IPC event
    state.app_handle.emit_all("portal-toggled", PortalToggleEvent {
        portal_id,
        is_closed: new_state,
    })?;
    
    Ok(portal)
}
```

**Frontend IPC listener:**
```typescript
listen('portal-toggled', (event) => {
  // Invalidate vision cache
  visionService.invalidateCache()
  // Re-render fog
  renderFog()
})
```

### Dependencies
Depends on: MIMIR-T-0229, MIMIR-T-0233, MIMIR-T-0234

### Risk Considerations
Rapid toggling could cause render thrashing; debounce if needed

## Status Updates **[REQUIRED]**

*To be added during implementation*