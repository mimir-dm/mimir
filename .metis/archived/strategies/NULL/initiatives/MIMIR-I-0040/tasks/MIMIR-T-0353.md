---
id: update-moduleservice-for-db-backed
level: task
title: "Update ModuleService for DB-Backed Markdown Storage"
short_code: "MIMIR-T-0353"
created_at: 2026-01-19T21:27:10.569668+00:00
updated_at: 2026-01-19T21:27:10.569668+00:00
parent: MIMIR-I-0040
blocked_by: []
archived: true

tags:
  - "#task"
  - "#phase/todo"


exit_criteria_met: false
strategy_id: NULL
initiative_id: MIMIR-I-0040
---

# Update ModuleService for DB-Backed Markdown Storage

## Parent Initiative

[[MIMIR-I-0040]] - Database-Only Document Storage

## Objective

Update ModuleService to store module overview markdown content in the database instead of the filesystem.

## Acceptance Criteria

## Acceptance Criteria

- [ ] Module overview content stored in DB when creating modules
- [ ] Module overview reads from DB
- [ ] Module creation no longer writes overview.md file
- [ ] Operations wrapped in transactions
- [ ] Existing module tests pass or are updated

## Implementation Notes

### Key Changes
ModuleService creates module overview documents via DocumentService. Most changes should be inherited from MIMIR-T-0352, but need to verify:
- `create_module()` flow uses DocumentService correctly
- Any direct file operations for module markdown are removed

### Files to Modify
- `crates/mimir-dm-core/src/services/module_service.rs`

### Methods to Review
- `create_module()` - verify it delegates to DocumentService
- `delete_module()` - verify cleanup is handled correctly
- Any methods that read/write module overview directly

### Dependencies
- MIMIR-T-0351 (Schema Migration)
- MIMIR-T-0352 (DocumentService Update) - may handle most of this

## Status Updates

*To be added during implementation*