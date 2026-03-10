---
id: weapon-detection-uses-keyword
level: task
title: "Weapon detection uses keyword matching, misses many common weapons"
short_code: "MIMIR-T-0541"
created_at: 2026-03-09T18:59:14.048826+00:00
updated_at: 2026-03-10T18:28:22.977544+00:00
parent: 
blocked_by: []
archived: true

tags:
  - "#task"
  - "#bug"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: NULL
---

# Weapon detection uses keyword matching, misses many common weapons

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Objective

Fix weapon detection in CharacterStatsTab so the attacks section shows all equipped weapons, not just ones matching a hardcoded keyword list.

## Bug Details

### Type
- [x] Bug - Production issue that needs fixing

### Priority
- [x] P2 - Medium (nice to have)

### Impact Assessment
- **Affected Users**: Any character using a weapon not in the keyword list
- **Reproduction Steps**: 
  1. Create a character with a Rapier equipped
  2. Open the character sheet, Character tab
  3. Look at the Attacks section — Rapier does not appear
- **Expected vs Actual**: Expected: Rapier shows with attack/damage stats. Actual: Rapier is missing from attacks entirely.

### Root Cause

`CharacterStatsTab.vue` detects weapons by checking if `item_name.toLowerCase()` contains one of these substrings: `sword`, `axe`, `bow`, `dagger`, `mace`, `staff`, `crossbow`, `spear`, `hammer`.

**Missing weapons** (common SRD/PHB weapons not matched):
- Rapier, Scimitar, Flail, Glaive, Halberd, Javelin, Lance, Morningstar, Pike, Trident, War Pick, Whip, Blowgun, Dart, Sling, Net

That's 16 out of ~37 PHB weapons — nearly half are invisible in the attacks section.

### Recommended Fix

Instead of keyword matching on `item_name`, use the item's catalog data to determine if it's a weapon. The `get_item_by_name` invoke command returns a `type` field — weapons have `type: "M"` (melee) or `type: "R"` (ranged). Alternatively, check for the presence of `dmg1` or `weapon: true` in the item data.

This would require either:
1. **Enriching inventory at load time** — when CharacterSheetView loads inventory, also fetch item details and tag weapons, OR
2. **Using a comprehensive weapon name list** from the catalog instead of a hardcoded substring list

Option 1 is more robust and future-proof (handles homebrew weapons too).

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] All SRD/PHB simple and martial weapons appear in the attacks section when equipped
- [ ] Homebrew weapons also appear in attacks when equipped
- [ ] Attack bonus and damage calculations are correct for all weapon types
- [ ] Regression test added for Rapier (and other previously-missing weapons)

## Location

- **File**: `src/features/characters/components/sheet/CharacterStatsTab.vue`
- **Discovered by**: MIMIR-I-0056 automated testing initiative (T-0537)

## Status Updates

### Session 1 (2026-03-10)
- Added `isWeapon()` function to `characterUtils.ts` with a complete `WEAPON_BASE_NAMES` set covering all 37 PHB/SRD weapons
- `isWeapon()` matches exact names, prefix variants ("Longsword +1"), and suffix variants ("Flame Tongue Longsword")
- Replaced keyword substring matching in `CharacterStatsTab.vue` with `isWeapon()` call
- Expanded `getWeaponDamage()` to cover all newly-detected weapons: Flail, Glaive, Halberd, Lance, Morningstar, Pike, Trident, War Pick, Greatclub, Handaxe, Javelin, Mace, Club, Light Hammer, Sickle, Whip, Dart, Sling, Blowgun, Net
- Expanded `isRanged()` to include Blowgun and Net
- Added 114 tests in `src/utils/__tests__/characterUtils.test.ts` (was 80+, now covers `isWeapon` with all weapon categories + magic variants + non-weapon rejection)
- All 1466 tests pass across 59 files

### Session 2 (2026-03-10) — Dynamic catalog-driven detection
- Made weapon detection dynamic via catalog query:
  - **Rust DAL**: `list_weapon_names()` — `SELECT DISTINCT lower(name) FROM items WHERE item_type IN ('M', 'R')`
  - **Rust service + Tauri command**: `list_weapon_names`
  - **Frontend composable**: `useWeaponRegistry` — loads weapon names from catalog once, caches in a `Set`, falls back to static PHB list until loaded
  - **Component**: `CharacterStatsTab.vue` uses `useWeaponRegistry` with `onMounted` load
- New rulesets/homebrew items with `item_type = "M"/"R"` will automatically appear as weapons
- All 1466 frontend tests pass, Rust compiles cleanly

**Files modified:**
- `crates/mimir-core/src/dal/catalog/item.rs` — added `list_weapon_names()`
- `crates/mimir-core/src/services/catalog/item.rs` — added service method
- `crates/mimir/src/commands/catalog/items.rs` — added Tauri command
- `crates/mimir/src/main.rs` — registered command
- `crates/mimir/frontend/src/composables/useWeaponRegistry.ts` — new composable
- `crates/mimir/frontend/src/utils/characterUtils.ts` — added `WEAPON_BASE_NAMES`, `isWeapon()`, expanded `getWeaponDamage()`, `isRanged()`
- `crates/mimir/frontend/src/features/characters/components/sheet/CharacterStatsTab.vue` — uses registry
- `crates/mimir/frontend/src/utils/__tests__/characterUtils.test.ts` — expanded tests