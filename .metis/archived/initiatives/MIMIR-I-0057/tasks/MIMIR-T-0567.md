---
id: frontend-homebrew-monsters-in
level: task
title: "Frontend: Homebrew monsters in modules and token placement"
short_code: "MIMIR-T-0567"
created_at: 2026-03-11T14:49:24.411861+00:00
updated_at: 2026-03-11T20:29:29.014235+00:00
parent: MIMIR-I-0057
blocked_by: []
archived: true

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: MIMIR-I-0057
---

# Frontend: Homebrew monsters in modules and token placement

## Parent Initiative

[[MIMIR-I-0057]]

## Objective

Update the Vue frontend to allow adding homebrew monsters to modules, display them in monster lists with a visual indicator, and make them available in the token placement picker on maps.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] Module monster list displays homebrew monsters with a visual "Homebrew" badge/indicator
- [ ] "Add Monster" dialog has a tab or toggle to search homebrew monsters alongside catalog
- [ ] Homebrew monsters show name, CR, and creature type in the list (parsed from JSON blob)
- [ ] Token placement picker includes homebrew module monsters
- [ ] Removing a homebrew monster from a module works (uses existing `remove_module_monster`)
- [ ] Token stat popover shows correct stats for homebrew monster tokens

## Implementation Notes

### Technical Approach

**Module monster list** (`frontend/src/components/modules/`):
- Update the monster list component to handle both `monster_source` and `homebrew_monster_id` response fields
- Display a "Homebrew" badge when `homebrew_monster_id` is present
- Show name from homebrew data instead of `monster_name` column

**Add monster dialog**:
- Add a toggle/tab: "Catalog" | "Homebrew"
- Catalog tab: existing monster search (unchanged)
- Homebrew tab: list campaign's homebrew monsters, with search/filter
- On selection, call `add_module_monster` with `homebrew_monster_id` instead of `monster_name`

**Token placement** (`frontend/src/components/maps/`):
- Token type picker already lists module monsters — ensure homebrew ones appear
- Token stat display reads from the enriched token data (backend handles resolution)

### Dependencies
- MIMIR-T-0564 (migration)
- MIMIR-T-0565 (DAL/services)
- MIMIR-T-0566 (Tauri commands returning homebrew data)

### Key Files
- `crates/mimir/frontend/src/components/modules/` — module monster components
- `crates/mimir/frontend/src/components/maps/` — token placement
- `crates/mimir/frontend/src/stores/` — relevant Pinia stores

## Status Updates

### Completed
- ✅ `ModuleMonsters.vue`: Updated `ModuleMonster` interface with nullable `monster_name`/`monster_source` and new `homebrew_monster_id`
- ✅ `ModuleMonsters.vue`: Search now combines catalog + homebrew results, homebrew shown first with "HB" badge
- ✅ `ModuleMonsters.vue`: `addMonster()` sends `homebrew_monster_id` for homebrew, `monster_name`/`monster_source` for catalog
- ✅ `ModuleMonsters.vue`: Tagged monster list shows "HB" badge for homebrew monsters, handles null `monster_name`
- ✅ `ModuleMonsters.vue`: `viewMonster()` handles homebrew monsters (uses `monster_data` directly if available)
- ✅ `useModuleMonsters.ts`: Updated `MonsterWithData` interface, added `getMonsterDisplayName()` and `isHomebrewMonster()` helpers
- ✅ `TokenPalette.vue`: Updated `ModuleMonsterWithData` interface, handles homebrew monsters in quick-select and token config
- ✅ `DangersList.vue`: Updated `EncounterGroup` interface, shows "HB" badge, handles null `monster_name`
- ✅ `ModulesTab.vue`: Monster edit modal handles null `monster_name` gracefully
- ✅ `HomebrewMonstersSubTab.vue`: Delete check now uses `homebrew_monster_id` (not just legacy `monster_source === 'HB'`)
- ✅ TypeScript check passes (vue-tsc --noEmit), frontend builds, all tests pass