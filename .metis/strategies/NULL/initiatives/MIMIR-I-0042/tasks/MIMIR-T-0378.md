---
id: 5etools-import-service-with-field
level: task
title: "5etools import service with field extraction"
short_code: "MIMIR-T-0378"
created_at: 2026-01-20T02:44:02.198814+00:00
updated_at: 2026-01-20T02:44:02.198814+00:00
parent: MIMIR-I-0042
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/todo"


exit_criteria_met: false
strategy_id: NULL
initiative_id: MIMIR-I-0042
---

# 5etools import service with field extraction

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[MIMIR-I-0042]]

## Objective

Implement the 5etools import service in `mimir-services` that reads 5etools JSON files, extracts indexed fields, and populates catalog tables. Handle per-source transactions with fail-and-continue error handling.

## Acceptance Criteria

- [ ] Create `CatalogImportService` in mimir-services crate
- [ ] Parse 5etools JSON files using typify-generated types
- [ ] Extract indexed columns per entity type (see field extraction rules)
- [ ] Store full JSON blob in `data` column
- [ ] Wrap each source book in a transaction
- [ ] On error: rollback book transaction, log error, continue to next book
- [ ] Populate join tables (spell_classes, spell_subclasses, item_attunement_classes)
- [ ] Populate FTS table with flattened entries
- [ ] Return import summary (X books imported, Y failed, Z entities total)

## Service Interface

```rust
pub struct CatalogImportService {
    conn: SqliteConnection,
    app_data_dir: PathBuf,
}

pub struct ImportResult {
    pub sources_imported: Vec<String>,
    pub sources_failed: Vec<(String, String)>,  // (source, error)
    pub entity_counts: HashMap<String, usize>,
}

impl CatalogImportService {
    pub fn new(conn: SqliteConnection, app_data_dir: PathBuf) -> Self;
    
    /// Import all 5etools data from a directory
    pub fn import_from_directory(&mut self, path: &Path) -> Result<ImportResult>;
    
    /// Import a single source file
    pub fn import_source(&mut self, path: &Path) -> Result<()>;
}
```

## Field Extraction Rules

| Entity | Field | Extraction |
|--------|-------|------------|
| Monster | `creature_type` | `type.type` or `type` if string |
| Monster | `size` | `size[0]` (first element) |
| Monster | `cr` | `cr.cr` or `cr` if string |
| Spell | `concentration` | `duration[].concentration == true` |
| Spell | `ritual` | `meta.ritual == true` |
| Item | `type` | Direct field (single char code) |
| Item | `rarity` | Direct field |

## Error Handling

```rust
// Per-source transaction pattern
for source_file in source_files {
    let tx = conn.begin_transaction()?;
    match import_source_file(&tx, &source_file) {
        Ok(count) => {
            tx.commit()?;
            result.sources_imported.push(source_file.name);
            result.entity_counts.insert(source_file.name, count);
        }
        Err(e) => {
            tx.rollback()?;
            tracing::error!("Failed to import {}: {}", source_file.name, e);
            result.sources_failed.push((source_file.name, e.to_string()));
            // Continue to next source
        }
    }
}
```

## Implementation Notes

### File Discovery

5etools data is organized by type:
- `bestiary/bestiary-*.json` - Monster files by source
- `spells/spells-*.json` - Spell files by source  
- `items.json`, `items-base.json` - Item files
- `class/class-*.json` - Class files with embedded subclasses
- `races.json` - All races
- `backgrounds.json` - All backgrounds
- `feats.json` - All feats

### Spell-Class Mapping

Spell-class relationships come from generated lookup file:
- `data/generated/gendata-spell-source-lookup.json`
- Maps spell name+source to list of classes/subclasses

### Dependencies

- MIMIR-T-0367 (typify-generated types)
- MIMIR-T-0370 through MIMIR-T-0376 (all tables)
- MIMIR-T-0377 (FTS table and entry flattener)

## Status Updates **[REQUIRED]**

*To be added during implementation*