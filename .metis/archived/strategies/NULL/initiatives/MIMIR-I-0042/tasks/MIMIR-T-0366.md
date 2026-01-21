---
id: vendor-5etools-json-schemas-from
level: task
title: "Vendor 5etools JSON schemas from brew-fast"
short_code: "MIMIR-T-0366"
created_at: 2026-01-20T02:43:34.232913+00:00
updated_at: 2026-01-20T03:06:23.209462+00:00
parent: MIMIR-I-0042
blocked_by: []
archived: true

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: MIMIR-I-0042
---

# Vendor 5etools JSON schemas from brew-fast

## Parent Initiative

[[MIMIR-I-0042]] - v0.5 Catalog Implementation

## Objective

Download and vendor all 5etools JSON schemas from the `5etools-utils` repository's `brew-fast` directory. These schemas define the structure of 5etools data and will be used by typify to generate Rust types.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] All 20+ entity schemas downloaded from `https://github.com/TheGiddyLimit/5etools-utils/tree/master/schema/brew-fast`
- [ ] Schemas stored in `crates/mimir-core/schema/5etools/`
- [ ] Shared schemas included: `entry.json`, `util.json`
- [ ] Entity schemas included:
  - bestiary.json, items.json, spells.json, class.json, races.json
  - backgrounds.json, feats.json, trapshazards.json, actions.json
  - conditionsdiseases.json, languages.json, deities.json, vehicles.json
  - objects.json, rewards.json, optionalfeatures.json, psionics.json
  - variantrules.json, cultsboons.json, tables.json
- [ ] Download script or documentation for refreshing schemas

## Implementation Notes

### Source
- Repository: `https://github.com/TheGiddyLimit/5etools-utils`
- Path: `schema/brew-fast/`
- These are pre-processed schemas with `$$merge` resolved (ready for typify)

### Directory Structure
```
crates/mimir-core/schema/
└── 5etools/
    ├── entry.json          # Shared entry types
    ├── util.json           # Shared utility types
    ├── bestiary.json       # Monster schema
    ├── items.json          # Item schema
    ├── spells.json         # Spell schema
    └── ... (17 more)
```

### Notes
- Some schemas are in subdirectories (bestiary/, class/, spells/) - flatten or preserve structure as needed
- Pin to a specific commit for reproducibility

## Status Updates

### 2026-01-19: Completed
- Created `crates/mimir-core/schema/5etools/` directory with full brew-fast structure
- Created `download-schemas.sh` script - shallow clones repo, copies brew-fast, cleans up
- Downloaded **99 schemas** from brew-fast including:
  - Core entity schemas: bestiary, items, spells, class, races, feats, backgrounds, etc.
  - Fluff schemas: 14 files for flavor text content
  - Foundry VTT schemas: 14 files (platform integration, may not need)
  - Utility schemas: entry.json, util.json, util-copy, util-time, etc.
  - Generated data: `gendata-spell-source-lookup.json` (spell-class mappings for join tables)
  - Extras: magicvariants, legendarygroups, loot, names, decks, etc.
- Directory structure preserved (bestiary/, class/, spells/, adventure/, book/, generated/)
- Schemas pinned to master branch (can update COMMIT variable for reproducibility)