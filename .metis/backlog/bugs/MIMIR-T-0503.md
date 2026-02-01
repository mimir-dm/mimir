---
id: strip-source-suffix-from-item-type
level: task
title: "Strip source suffix from item_type during import"
short_code: "MIMIR-T-0503"
created_at: 2026-01-31T04:26:18.529624+00:00
updated_at: 2026-01-31T13:43:41.992387+00:00
parent: 
blocked_by: []
archived: false

tags:
  - "#task"
  - "#bug"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: NULL
---

# Strip source suffix from item_type during import

## Objective

The `item_type` column in the items table stores the raw 5etools `type` field which uses a `TYPE|SOURCE` format (e.g., `"M|XPHB"`, `"AF|DMG"`, `"GV|DMG"`). The source suffix should be stripped so the column contains only the type code (`"M"`, `"AF"`, `"GV"`). This causes filtering by item type to break since queries expect clean type codes.

## Backlog Details

- **Type**: Bug
- **Priority**: P2 - Medium
- **Effort**: S

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `import_item()` strips the `|SOURCE` suffix from the type field before inserting
- [ ] After re-import, `item_type` values are clean codes: `M`, `R`, `HA`, `LA`, `MA`, `S`, `A`, `AF`, `GV`, etc.
- [ ] `type_name()` and `get_item_type_name()` still resolve correctly
- [ ] `ItemFilter` type queries work with clean codes
- [ ] Existing item type display in the UI is unaffected

## Implementation Notes

- Single-line fix in `import_item()` at `crates/mimir-core/src/import/service.rs:878-903`: split on `|` and take the first part
- Requires re-import to fix existing data
- Should be done before or alongside MIMIR-T-0502 (magic variant expansion) since the GV type matching depends on clean type codes

## Status Updates

*To be added during implementation*