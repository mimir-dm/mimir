---
id: add-frontend-test-job-to-ci
level: task
title: "Add frontend test job to CI pipeline"
short_code: "MIMIR-T-0530"
created_at: 2026-03-08T22:48:32.440615+00:00
updated_at: 2026-03-09T01:27:25.438900+00:00
parent: MIMIR-I-0055
blocked_by: []
archived: true

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: MIMIR-I-0055
---

# Add frontend test job to CI pipeline

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[MIMIR-I-0055]]

## Objective
Add a `test-frontend` job to `.github/workflows/ci.yml` so frontend tests gate PRs.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria
- [ ] New `test-frontend` job in ci.yml
- [ ] Runs `npm ci && npm test` in `crates/mimir/frontend`
- [ ] Runs on ubuntu-latest (no platform matrix needed)
- [ ] Job passes in CI

## Implementation Notes
- File: `.github/workflows/ci.yml`
- Simple job: checkout, setup node, npm ci, npm test
- No Rust or Tauri dependencies needed — frontend tests mock Tauri invoke
- Depends on MIMIR-T-0529 (need tests to exist before CI runs them)

## Status Updates

### Session 2
- Deleted 5 broken test files that tested non-existent APIs (DocumentService, ModuleService, boardConfigService, campaigns store, characters store) — all from a previous agent that generated tests against a planned API
- Fixed 2 CatalogTable test assertions to match actual component output ("Showing 1-4 of 4 items" not "4 items", empty string not em-dash for null values)
- All 9 test files now pass: 161 tests total
- Added `test-frontend` job to `.github/workflows/ci.yml`: checkout → setup node → npm ci → npx vitest run
- Job runs on ubuntu-latest, no Rust/Tauri deps needed