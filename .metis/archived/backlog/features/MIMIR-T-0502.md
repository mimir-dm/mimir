---
id: expand-magic-variant-items-during
level: task
title: "Expand magic variant items during import"
short_code: "MIMIR-T-0502"
created_at: 2026-01-31T04:25:02.897950+00:00
updated_at: 2026-01-31T14:34:47.733635+00:00
parent: 
blocked_by: []
archived: true

tags:
  - "#task"
  - "#feature"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: NULL
---

# Expand magic variant items during import

## Objective

During 5etools data import, expand Generic Variant (GV) magic items into concrete item entries by combining each variant template with its matching base items. For example, "+1 Weapon" (GV) + "Shortsword" (base) = "+1 Shortsword" in the items table.

Currently searching for "+1 Shortsword" returns nothing because 5etools stores this as a Generic Variant template in `magicvariants.json`, not as a concrete item. The expansion that 5etools does client-side needs to happen at import time.

## Backlog Details

- **Type**: Feature
- **Priority**: P1 - High (users can't find/add magic weapons, armor, or ammo to character inventories)
- **Effort**: M

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] Import collector reads `magicvariants.json` and collects `magicvariant` entries
- [ ] For each GV entry, expand against matching base items using `requires`/`excludes` rules
- [ ] Expanded items get the inherited name (e.g., `namePrefix` "+" + base name = "+1 Shortsword")
- [ ] Expanded items inherit `source`, `rarity`, `bonusWeapon`/`bonusAc`, `entries` from the `inherits` block
- [ ] Expanded items retain base item properties (`dmg1`, `dmgType`, `weight`, `weaponCategory`, `property`, etc.)
- [ ] Searching "Shortsword" in the equipment dialog returns both the base and +1/+2/+3 variants
- [ ] Existing base items and regular magic items are unaffected
- [ ] Re-importing handles duplicates gracefully (upsert or skip)

## Analysis

### 5etools Data Structure

**`magicvariants.json`** contains 214 GV templates under the `magicvariant` key. Each template has:

```json
{
  "name": "+1 Weapon",
  "type": "GV|DMG",
  "requires": [{ "weapon": true }],
  "excludes": { "net": true },
  "inherits": {
    "namePrefix": "+1 ",
    "source": "DMG",
    "rarity": "uncommon",
    "bonusWeapon": "+1",
    "entries": ["You have a {=bonusWeapon} bonus to attack and damage rolls..."]
  }
}
```

**`items-base.json`** contains 196 base items under `baseitem`. Each has boolean flags like `weapon: true`, `armor: true`, and type codes like `"M"`, `"R"`, `"HA"`.

### Matching Rules

- `requires` is an array of objects; a base item matches if ANY requirement object matches
- Each requirement object: ALL key-value pairs must match (e.g., `{ "weapon": true }` matches any item with `weapon: true`)
- `excludes` is an object; a base item is excluded if ANY key matches a truthy property
- Type codes have source suffixes (e.g., `"M|XPHB"`, `"AF|DMG"`) that must be matched literally

### Scale

- 214 variant templates x 196 base items = ~5,921 expanded items
- Manageable for SQLite; comparable to the existing ~2,500 regular items

### Key Files

- `crates/mimir-core/src/import/collector.rs:254-256` — add `magicvariants.json` collection
- `crates/mimir-core/src/import/service.rs:878-903` — `import_item()` + new expansion logic
- `crates/mimir-core/src/import/service.rs:1509-1511` — entity type mappings
- `crates/mimir-core/src/catalog/item.rs` — may need GV-specific deserialization struct
- `crates/mimir-core/src/dal/catalog/item.rs` — insert expanded items

### Additional Bug: item_type Column Corruption (MIMIR-T-0503)

Tracked separately. The `type` field uses `TYPE|SOURCE` format and we store the whole string. Must be fixed before or alongside this task since `requires` matching depends on type codes.

### Deep Dive: Naming Rules

All 214 variants have an `inherits` block. Naming uses three fields:
- **`namePrefix`** (161 variants): e.g., `"+1 "` → "+1 Shortsword"
- **`nameSuffix`** (53 variants): e.g., `" of Slaying"` → "Arrow of Slaying"
- **`nameRemove`** (1 variant): Barding removes `" Armor"` from base name

Formula: `namePrefix + baseItem.name.replace(nameRemove, "") + nameSuffix`

Every variant has at least one of prefix or suffix — no ambiguous naming.

### Deep Dive: Requires Matching

The `requires` array uses these key patterns against base item fields:

| Pattern | Count | Example |
|---|---|---|
| `{ "weapon": true }` | 22 | +1 Weapon → all weapons |
| `{ "armor": true }` | 22 | +1 Armor → all armor |
| `{ "type": "A" }` | 222 | +1 Ammunition → ammo by type code |
| `{ "sword": true }` | 25 | Sword of Wounding → swords only |
| `{ "bow": true }` | 4 | Bow of Conflagration → bows only |
| `{ "axe": true }` | 2 | Berserker Axe → axes only |
| `{ "weaponCategory": "simple" }` | 35 | Dazzling Weapon → simple weapons |
| `{ "name": "X", "source": "Y" }` | 74 | Berserker Axe (XPHB) → specific items |
| `{ "scfType": "arcane", "type": "SCF" }` | 32 | Imbued Wood → arcane focuses |
| `{ "property": "A|XPHB", "weaponCategory": "simple" }` | 4 | Repeating Shot → simple ammo weapons |
| `{ "dmgType": "S", "sword": true }` | 2 | Sword of Sharpness → slashing swords |

**Matching logic**: A base item matches a variant if ANY `requires` entry matches. Within a single entry, ALL key-value pairs must match.

**Base item flags available**: `weapon`, `armor`, `sword`, `axe`, `bow`, `crossbow`, `spear`, `polearm`, `arrow`, `bolt`, `net`, `dagger`, `mace`, `hammer`, `lance`, `rapier`, `staff`, `club`, `firearm`, `bulletFirearm`, `cellEnergy`, `needleBlowgun`, `bulletSling`

### Deep Dive: Excludes

Only ~20 variants have `excludes`. Patterns:
- `{ "net": true }` — most +N Weapons exclude nets
- `{ "name": "Hide Armor" }` — Adamantine Armor excludes hide
- `{ "property": ["2H", "2H|XPHB"] }` — Armblade excludes two-handed
- `{ "cellEnergy": true, "bulletFirearm": true }` — Arrow of Slaying excludes modern ammo
- `{ "name": ["Crystal", "Orb"] }` — Imbued Wood variants exclude crystal/orb

Exclude values can be `true` (boolean check), a string (exact name match), or an array (any-of match).

### Deep Dive: Data Blob Construction

For each expanded item, merge base item JSON + variant `inherits` block:
1. Start with full base item JSON
2. Overlay all `inherits` fields (source, rarity, bonusWeapon, entries, etc.)
3. Apply naming formula to `name`
4. Add `_variantOf` and `_baseItem` provenance fields
5. Skip inherits-only metadata: `namePrefix`, `nameSuffix`, `nameRemove`, `reprintedAs`, `lootTables`

Result example — "+1 Shortsword": base Shortsword properties (dmg1, dmgType, weight, weaponCategory) + variant properties (source=DMG, rarity=uncommon, bonusWeapon=+1, entries).

### Deep Dive: Source Filtering

Variants come from DMG (46), XDMG (82), BMT (18), ERLW (18), etc. Base items from PHB (74), XPHB (77), DMG (15).

**Decision needed**: When importing source X, should we:
- (a) Only expand variants whose `inherits.source` matches X? (Means PHB-only import gets no magic weapons)
- (b) Always expand all variants against available base items? (Simpler, more useful)
- (c) Expand variants if EITHER the variant source OR base item source is in the import set?

**Recommendation**: Option (b) — import `magicvariants.json` unconditionally (not filtered by source), then expand against whatever base items are already imported. The expanded items inherit source from the variant's `inherits.source`, so they'll be correctly attributed. This matches how 5etools itself handles it — variants are always available.

### Scale (revised)

- 214 variant templates
- 196 base items (but many don't match most variants)
- ~5,921 expanded items (confirmed via simulation)
- DB currently has ~2,500 items, so this roughly triples the count — still well within SQLite's comfort zone

## Status Updates

### Session 1
- Implemented full magic variant expansion pipeline
- **collector.rs**: Added `magicvariants.json` to root file collection
- **service.rs collect_entities_from_memory**: Fixed entity mappings — `baseitem` now collected as own type, `magicvariant` mapped to correct file path
- **service.rs import_single_entity**: Added `baseitem` → import as item, `magicvariant` → skip (expanded separately)
- **service.rs**: Added `expand_and_import_magic_variants()` method with full requires/excludes matching
- **service.rs**: Added `expand_magic_variants_from_memory()` for tarball path — global pass after all sources
- **service.rs**: Added `expand_magic_variants_from_disk()` for disk path — global pass after all sources
- **Helper functions**: `base_item_matches_field()`, `base_item_excluded()`, `build_expanded_item()`
- **Tests**: 8 new unit tests for matching, exclusion, and item building — all passing
- **Key design decision**: Expansion is global (not per-source) since variants (DMG) need base items (PHB)
- All 19 import service tests pass; no regressions in 741 passing tests
- T-0503 (strip source suffix from item_type) completed as prerequisite
- **Next**: Need to test with actual data import to verify end-to-end