---
id: unified-fts-table-catalog-fts-with
level: task
title: "Unified FTS table (catalog_fts) with entry flattener"
short_code: "MIMIR-T-0377"
created_at: 2026-01-20T02:44:01.809958+00:00
updated_at: 2026-01-20T20:50:00.676251+00:00
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

# Unified FTS table (catalog_fts) with entry flattener

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[MIMIR-I-0042]]

## Objective

Create unified FTS5 virtual table for full-text search across all catalog entities, with separate indexing for rules content vs fluff content. Implement entry flattener utility to extract searchable text from 5etools nested entry structures.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [x] Create `catalog_fts` FTS5 virtual table
- [x] Implement `flatten_entries()` function to extract text from 5etools entry arrays
- [x] Handle all entry types: string, object with `entries`, `{@tag}` references
- [x] Index rules content (name + entries) separately from fluff
- [x] Index fluff content (name + fluff entries) separately
- [x] Search can filter by `content_type` ('rules' | 'fluff')
- [x] Search can filter by `entity_type` (monster, spell, item, etc.)

## SQL Schema

```sql
-- Unified full-text search table
CREATE VIRTUAL TABLE catalog_fts USING fts5(
    entity_type,    -- 'monster', 'spell', 'item', etc.
    entity_id,      -- References the entity's primary key (unindexed)
    content_type,   -- 'rules' | 'fluff'
    name,           -- Entity name (always indexed)
    text_content,   -- Flattened entries text
    tokenize='porter unicode61'
);
```

## Entry Flattener Implementation

```rust
/// Flattens 5etools entry arrays into searchable plain text
pub fn flatten_entries(entries: &[serde_json::Value]) -> String {
    let mut result = Vec::new();
    for entry in entries {
        flatten_entry_recursive(entry, &mut result);
    }
    result.join(" ")
}

fn flatten_entry_recursive(entry: &Value, output: &mut Vec<String>) {
    match entry {
        Value::String(s) => {
            // Strip {@tag ...} markers, keep inner text
            output.push(strip_5etools_tags(s));
        }
        Value::Object(obj) => {
            // Handle entry objects with nested entries
            if let Some(entries) = obj.get("entries") {
                if let Value::Array(arr) = entries {
                    for e in arr {
                        flatten_entry_recursive(e, output);
                    }
                }
            }
            // Handle name/title fields
            if let Some(Value::String(name)) = obj.get("name") {
                output.push(name.clone());
            }
        }
        Value::Array(arr) => {
            for e in arr {
                flatten_entry_recursive(e, output);
            }
        }
        _ => {}
    }
}

/// Strips 5etools tag markers like {@spell fireball} -> "fireball"
fn strip_5etools_tags(s: &str) -> String {
    // Regex: \{@\w+\s+([^}]+)\} -> capture group 1
    TAG_REGEX.replace_all(s, "$1").to_string()
}
```

## Implementation Notes

### Content Types

- **rules**: Mechanical content - stats, abilities, effects, descriptions
  - Source: `entries` array in entity JSON
  - Example: Monster abilities, spell effects, item properties

- **fluff**: Flavor/lore content - background, history, descriptions
  - Source: Separate `*Fluff.json` files or `fluff` property
  - Example: Monster lore, item history, deity myths

### 5etools Entry Types

Common entry object types to handle:
- `"entries"` - Nested entry array
- `"table"` - Table data
- `"list"` - Bulleted list
- `"quote"` - Quoted text
- `"inset"` - Sidebar/callout
- Plain strings with `{@tag}` references

### Dependencies

- All entity tables must exist (MIMIR-T-0370 through MIMIR-T-0376)

## Status Updates **[REQUIRED]**

### 2026-01-20: Complete

**Migration 008_catalog_fts created and applied:**
- Created FTS5 virtual table with: entity_type, entity_id (UNINDEXED), content_type, name, text_content
- Uses porter stemming and unicode61 tokenizer for high-quality search

**Entry Flattener implemented (`src/fts/entry_flattener.rs`):**
- `flatten_entries()` - Flattens 5etools entry arrays into searchable plain text
- `strip_5etools_tags()` - Strips `{@tag}` markers, keeping display text
- Handles all entry types: strings, nested objects, lists, quotes, tables
- Uses `once_cell` + `regex` for efficient tag stripping
- 20 comprehensive tests

**FTS Search functions implemented (`src/fts/search.rs`):**
- `index_entity()` - Add entity to FTS index
- `remove_entity_from_index()` - Remove entity from FTS index
- `clear_entity_type_from_index()` - Clear all entities of a type
- `clear_index()` - Clear entire FTS index
- `search()` - Full-text search with ranking
- `search_by_entity_type()` - Filter by entity type (monster, spell, etc.)
- `search_by_content_type()` - Filter by rules vs fluff content
- `search_filtered()` - Combined entity type + content type filters
- `count_indexed()`, `count_indexed_by_type()` - Index statistics
- Supports FTS5 features: phrase search, boolean operators, porter stemming

**Tests: 291 passing** (up from 264)