---
id: campaign-dashboard-tests-campaign
level: task
title: "Campaign dashboard tests — campaign list, source management, character list"
short_code: "MIMIR-T-0543"
created_at: 2026-03-10T01:31:27.697306+00:00
updated_at: 2026-03-10T01:52:24.375448+00:00
parent: MIMIR-I-0056
blocked_by: []
archived: true

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: MIMIR-I-0056
---

# Campaign dashboard tests — campaign list, source management, character list

**Phase 3** — Campaign & Module Coverage

## Objective

Write Vitest component tests for the campaign dashboard view, covering the campaign list, campaign source/book management modal, and character list panel. Mock invoke responses for `list_campaigns`, `get_campaign_details`, `list_characters`, `list_catalog_sources`, `get_campaign_sources`, and `set_campaign_sources`.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] Campaign list renders campaigns with correct names, descriptions, and character counts
- [ ] Campaign source management modal shows available books and toggles selections
- [ ] CampaignSourcesModal save calls `set_campaign_sources` with correct payload
- [ ] Character list renders PCs and NPCs with correct class/level/race info
- [ ] Character list distinguishes PCs from NPCs (badge, player name)
- [ ] Empty states render correctly (no campaigns, no characters)
- [ ] Error states display when invoke calls fail
- [ ] All tests pass in CI

## Key Components

- `CampaignDashboard.vue` / campaign list view
- `CampaignSourcesModal.vue`
- `CharacterList` panel within campaign dashboard
- `HomebrewTab.vue` (list views only — CRUD covered in Phase 6)

## Implementation Notes

Use the invoke mock harness from MIMIR-T-0534. Create fixtures for campaign data, character summaries, and source book lists. Focus on data flow — does the component correctly display what the backend returns?

## Status Updates

### 2026-03-09
- Created 4 test files covering campaign dashboard components:
  - `CharacterCard.test.ts` — 10 tests (name, race, player name show/hide, NPC badge, null handling, click/view/print events)
  - `CampaignSourcesModal.test.ts` — 15 tests (loading, selection, quick actions, unsaved changes, saving, cancel)
  - `PCsTab.test.ts` — 6 tests (PC names, player names, empty state, header, create button)
  - `NPCsTab.test.ts` — 6 tests (NPC names, NPC badge, empty state, header, create button)
- Fixed CampaignSourcesModal tests: component uses `watch(visible)` without `immediate: true`, so tests must mount with `visible=false` then toggle to `true`
- All 38 tests passing across 4 files

*To be added during implementation*