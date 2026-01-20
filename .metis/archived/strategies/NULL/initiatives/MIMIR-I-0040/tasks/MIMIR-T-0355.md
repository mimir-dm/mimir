---
id: remove-file-path-usage-from
level: task
title: "Remove file_path Usage from Character Versions"
short_code: "MIMIR-T-0355"
created_at: 2026-01-19T21:27:11.068772+00:00
updated_at: 2026-01-19T21:27:11.068772+00:00
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

# Remove file_path Usage from Character Versions

## Parent Initiative

[[MIMIR-I-0040]] - Database-Only Document Storage

## Objective

Remove file_path usage from character version operations. The `character_data` column already stores the JSON content, so file_path is redundant.

## Acceptance Criteria

## Acceptance Criteria

- [ ] Character version creation no longer writes JSON files to disk
- [ ] Character version reads use `character_data` column only
- [ ] No file I/O in character version operations
- [ ] `file_path` field removed from CharacterVersion model
- [ ] Character-related tests pass

## Implementation Notes

### Key Changes
The `character_versions` table has:
- `file_path` - currently stores path to JSON file (being removed)
- `character_data` - stores the actual JSON content (keep using this)

After schema migration drops `file_path`, update code to:
- Remove any file writes when creating character versions
- Remove any file reads when loading character versions
- Remove `file_path` from model structs

### Files to Modify
- `crates/mimir-dm-core/src/models/character/` (character version models)
- `crates/mimir-dm-core/src/services/` (character-related services)
- `crates/mimir-dm-core/src/dal/` (character version repository)

### Dependencies
- MIMIR-T-0351 (Schema Migration) - drops the column

## Status Updates

*To be added during implementation*