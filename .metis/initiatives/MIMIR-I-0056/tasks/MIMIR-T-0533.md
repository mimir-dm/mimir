---
id: extract-srd-test-fixtures-from
level: task
title: "Extract SRD test fixtures from production database"
short_code: "MIMIR-T-0533"
created_at: 2026-03-09T14:25:08.460304+00:00
updated_at: 2026-03-10T01:17:29.655354+00:00
parent: MIMIR-I-0056
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: MIMIR-I-0056
---

# Extract SRD test fixtures from production database

## Parent Initiative

[[MIMIR-I-0056]] — Automated GUI testing to catch regressions and validate data flow

## Objective

Extract SRD (System Reference Document) data from the production database and organize it into test fixture files that both frontend (Vitest) and backend (cargo test) can consume. This is the foundation task — all other testing tasks depend on having realistic, committed fixture data.

SRD content is published under the OGL and is safe to commit to the repo.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] SRD class data extracted as JSON fixtures: Fighter, Rogue, Wizard, Cleric, Ranger, Paladin, Bard, Barbarian, Druid, Monk, Sorcerer, Warlock
- [ ] SRD subclass data: Champion (Fighter), Thief (Rogue), Evocation (Wizard), Life (Cleric) — at minimum
- [ ] SRD class features extracted for the above classes (all levels)
- [ ] SRD subclass features extracted (including child features like Fast Hands, Improved Critical)
- [ ] SRD background data: Acolyte, Criminal, Folk Hero, Noble, Sage, Soldier
- [ ] SRD race data: Human, Elf, Dwarf, Halfling, Dragonborn, Gnome, Half-Elf, Half-Orc, Tiefling
- [ ] SRD item data: common weapons (longsword, shortsword, dagger, longbow), common armor (leather, chain mail, plate, shield), and a few magic items
- [ ] SRD spell data: representative sample across levels and schools (cantrips through 9th level, ~20-30 spells)
- [ ] SRD monster data: representative sample across CRs (Goblin, Wolf, Skeleton, Owlbear, Adult Red Dragon, ~10-15 monsters)
- [ ] Frontend fixture files at `frontend/__tests__/fixtures/` as TypeScript exports wrapping the JSON
- [ ] Rust fixture files at `crates/mimir-core/tests/fixtures/` as JSON files loadable by test harness
- [ ] Test character fixtures: a Fighter 5/Champion with SRD equipment, a Rogue 3/Thief, a Wizard 5/Evocation, a multiclass Fighter 3/Rogue 2
- [ ] Homebrew item fixtures: 2-3 homebrew items with proper 5etools-format data blobs for testing HB source paths

## Implementation Notes

### Data Sources
- **Production DB**: `~/Library/Application Support/com.mimir.app/data/mimir.db`
- **Dev DB**: `~/Library/Application Support/com.mimir.app/dev/data/mimir.db`
- Tables: `classes`, `subclasses`, `class_features`, `subclass_features`, `backgrounds`, `races`, `items`, `spells`, `monsters`
- All SRD content has `srd: true` in the 5etools data blob, or `basicRules: true`

### SRD Identification
Query pattern to find SRD content:
```sql
SELECT name, source, data FROM classes WHERE source = 'PHB' AND (data LIKE '%"srd":true%' OR data LIKE '%"basicRules":true%');
```

### File Organization
```
frontend/__tests__/fixtures/
  classes.ts          -- SRD class data blobs (as entity_to_json would return)
  subclasses.ts       -- SRD subclass data
  classFeatures.ts    -- SRD class features
  subclassFeatures.ts -- SRD subclass features
  backgrounds.ts      -- SRD backgrounds
  races.ts            -- SRD races
  items.ts            -- SRD items
  spells.ts           -- SRD spells
  monsters.ts         -- SRD monsters
  characters.ts       -- Test character data (CharacterResponse format)
  homebrew.ts         -- Homebrew item fixtures

crates/mimir-core/tests/fixtures/
  srd_classes.json
  srd_subclasses.json
  srd_class_features.json
  srd_subclass_features.json
  srd_backgrounds.json
  srd_races.json
  srd_items.json
  srd_spells.json
  srd_monsters.json
  test_characters.json
  homebrew_items.json
```

### Frontend Fixture Format
Fixtures should match the shape returned by `entity_to_json` (merged metadata + parsed data blob), since that's what the components receive via `invoke`. Example:
```typescript
export const fighterClass = {
  id: 1,
  name: "Fighter",
  source: "PHB",
  // ... all fields from the 5etools data blob merged in
  hd: { number: 1, faces: 10 },
  classFeatures: ["Fighting Style|Fighter||1", ...],
  startingProficiencies: { ... },
  proficiency: [{ str: true }, { con: true }],
  // etc.
}
```

### Rust Fixture Format
JSON files that can be loaded and inserted into a test database. Should include the raw `data` column value (stringified JSON) plus the table columns (`name`, `source`, `class_name`, etc.).

## Status Updates

### 2026-03-09: Extraction Complete

**Extraction script**: `scripts/extract-srd-fixtures.py` — reads production DB, produces both TS and JSON fixtures.

**Results**:
| Category | Count | Notes |
|---|---|---|
| Classes | 12 | All SRD classes (Barbarian through Wizard) |
| Subclasses | 12 | All SRD subclasses (Champion, Thief, Evocation, Life, etc.) |
| Class Features | 170 | Full level progression for all 12 classes |
| Subclass Features | 89 | Including child features (Fast Hands, Improved Critical, etc.) |
| Backgrounds | 7 | Acolyte, Criminal, Folk Hero, Noble, Sage, Soldier, Variant Criminal |
| Races | 17 | 9 base + 8 subraces (High Elf, Hill Dwarf, Lightfoot Halfling, etc.) |
| Items | 44 | Weapons, armor, adventuring gear, and magic items |
| Spells | 43 | Cantrips through 9th level across all schools |
| Monsters | 17 | Goblin through Ancient Red Dragon, multiple CRs |
| Test Characters | 4 | Fighter 5/Champion, Rogue 3/Thief, Wizard 5/Evocation, Fighter 3/Rogue 2 multiclass |
| Homebrew Items | 3 | Blade of Testing, Amulet of Test Protection, Potion of Test Healing |

**Output files**:
- Frontend: `frontend/__tests__/fixtures/` — 11 TS files + index.ts (30k lines total)
- Rust: `crates/mimir-core/tests/fixtures/` — 11 JSON files (30k lines total)
- TS files compile cleanly, JSON validates correctly

**Format**: Frontend fixtures use `entity_to_json` format (id/name/source merged into data blob). Named exports for individual items. Rust fixtures are raw JSON arrays.