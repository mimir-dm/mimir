---
id: archiveservice-unit-tests
level: task
title: "ArchiveService unit tests"
short_code: "MIMIR-T-0527"
created_at: 2026-03-08T22:48:30.015671+00:00
updated_at: 2026-03-09T01:27:23.448143+00:00
parent: MIMIR-I-0055
blocked_by: []
archived: true

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: MIMIR-I-0055
---

# ArchiveService unit tests

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[MIMIR-I-0055]]

## Objective
Add tests to `crates/mimir-core/src/services/archive.rs` covering export/import round-trip, manifest verification, and error handling.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria
- [x] Export produces valid tar.gz with manifest.json + data.json
- [x] Manifest counts match entities (modules, documents, characters)
- [x] Import round-trip preserves all data (campaign, modules, documents, characters)
- [x] Import generates new UUIDs (no collision with originals)
- [x] Import with name override works
- [x] Import name collision auto-increments suffix
- [x] Export/import of empty campaign works
- [x] Nonexistent campaign export returns error
- [x] Invalid archive import returns error
- [x] All tests pass — 16 total (14 new + 2 existing)

## Implementation Notes
- File: `crates/mimir-core/src/services/archive.rs` (existing `#[cfg(test)]` module has 2 tests)
- Expand existing test module with ~13 new tests
- Uses tempfile for tar.gz output, `setup_test_db()` for in-memory DB

## Status Updates
- 2026-03-08: Added 14 new tests to archive.rs. All 16 pass (14 new + 2 existing).
  - Export: file creation, archive structure (manifest+data), empty campaign, nonexistent campaign error
  - Preview: counts, catalog references, nonexistent file error
  - Import: full round-trip, new UUID generation, name override, name collision auto-increment, invalid archive error
  - Preservation: document content, module-document associations, empty round-trip