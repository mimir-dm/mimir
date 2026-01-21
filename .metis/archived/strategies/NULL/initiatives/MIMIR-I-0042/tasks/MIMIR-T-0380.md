---
id: lift-5etools-import-core-from
level: task
title: "Lift 5etools import core from splitter into mimir-core"
short_code: "MIMIR-T-0380"
created_at: 2026-01-20T18:39:37.853657+00:00
updated_at: 2026-01-20T19:08:00.195917+00:00
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

# Lift 5etools import core from splitter into mimir-core

## Parent Initiative

[[MIMIR-I-0042]] - v0.5 Catalog Implementation

## Objective

Extract the reusable import logic from `mimir-5etools-splitter` into `mimir-core/src/import/` module. This enables direct import from 5etools zip/directory without the intermediate archive step.

## Background

The current flow requires:
1. Run `mimir-5etools-splitter` to create per-book tar.gz archives
2. Upload archives through Tauri UI
3. Extract and import to database

The new flow will be:
1. User points to 5etools zip/directory
2. Direct import to SQLite (selecting which sources to include)

Analysis of the splitter shows **75-80% of the code is archive-agnostic** and can be lifted directly.

## Modules to Lift

### 1. parser.rs (100% reusable)
- `load_all_books()` - reads `data/books.json` to discover sources
- `get_matching_files()` - finds entity files per source
- `file_exists_for_source()` - checks for `{prefix}-{source}.json` pattern

### 2. filter.rs (100% reusable)
- `SourceFilter` trait - filters JSON arrays by source field
- Multi-pattern matching: direct `source`, `inheritsFrom`, `sources[]`
- `get_all_sources()` - recursive source discovery

### 3. srd_filter.rs (100% reusable)
- `check_srd_status()` - detects SRD content via `srd` and `basicRules` fields
- `transform_item_for_srd()` - handles SRD renaming
- `SrdStatus` enum and `SrdItem` struct

### 4. collector.rs patterns (adapt for DB output)
- `collect_filtered_generic()` - base collection pattern
- Class feature extraction with source filtering
- Race inheritance merging (subrace + parent abilities)
- Item variant expansion (DMG magic items)
- Spell-class association from `spells/sources.json`

## File Structure

```
crates/mimir-core/src/import/
├── mod.rs              # Module exports
├── discovery.rs        # Book/source discovery (from parser.rs)
├── filter.rs           # SourceFilter trait (from filter.rs)
├── srd.rs              # SRD detection (from srd_filter.rs)
├── collector.rs        # Generic entity collection
└── entities/           # Per-entity collection logic
    ├── mod.rs
    ├── monster.rs
    ├── spell.rs
    ├── item.rs
    ├── class.rs
    └── ... (other types)
```

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [x] `mimir-core/src/import/discovery.rs` - loads books.json, finds source files
- [x] `mimir-core/src/import/filter.rs` - SourceFilter trait working
- [x] `mimir-core/src/import/srd.rs` - SRD detection and transformation
- [x] `mimir-core/src/import/collector.rs` - generic collection pattern
- [x] Unit tests (26 tests covering all import modules)
- [x] Can discover all sources from a 5etools directory (`discover_available_sources`)
- [x] Can filter entities by source correctly (`collect_source_entities`)

## Source Files (for reference)

From `mimir-dm-bu/mimir-5etools-splitter/src/`:
- `parser.rs` - book discovery
- `filter.rs` - source filtering
- `srd_filter.rs` - SRD detection
- `srd_collector.rs` - SRD aggregation
- `collector.rs` - entity collection patterns

## Status Updates

**2026-01-20 (Session 2)**: Added collector module completing the import pipeline:
- `collector.rs` - Generic entity collection patterns
  - `CollectedEntities` - container for source-filtered content
  - `CollectedSrdContent` - container for SRD content
  - `collect_source_entities()` - main entry point for per-book collection
  - `collect_srd_content()` - extracts all SRD content
  - `collect_all_of_type()` - get all entities of a type regardless of source
  - `filter_srd_only()` - filter already-collected entities to SRD only

All 148 tests passing (122 catalog + 26 import).

**2026-01-20 (Session 1)**: Created import module with three core components:
- `discovery.rs` - Book/source discovery, file finding (from parser.rs)
- `filter.rs` - SourceFilter trait, multi-pattern matching (from filter.rs)
- `srd.rs` - SRD detection and transformation (from srd_filter.rs)