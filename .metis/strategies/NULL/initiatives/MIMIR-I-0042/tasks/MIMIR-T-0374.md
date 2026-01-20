---
id: join-tables-spell-classes-spell
level: task
title: "Join tables: spell_classes, spell_subclasses, item_attunement_classes"
short_code: "MIMIR-T-0374"
created_at: 2026-01-20T02:43:49.169208+00:00
updated_at: 2026-01-20T02:43:49.169208+00:00
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

# Join tables: spell_classes, spell_subclasses, item_attunement_classes

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[MIMIR-I-0042]]

## Objective

Create Diesel migration for many-to-many relationship tables between spells/items and classes/subclasses.

## Acceptance Criteria

- [ ] Create `spell_classes` join table linking spells to classes
- [ ] Create `spell_subclasses` join table linking spells to subclass spell lists
- [ ] Create `item_attunement_classes` join table for class-specific attunement
- [ ] All tables have proper foreign key references
- [ ] Indexes on frequently-queried columns
- [ ] Diesel schema.rs generated and compiles

## SQL Schema

```sql
-- Spells available to each class
CREATE TABLE spell_classes (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    spell_id INTEGER NOT NULL REFERENCES spells(id) ON DELETE CASCADE,
    class_name TEXT NOT NULL,
    source TEXT NOT NULL REFERENCES catalog_sources(code),
    UNIQUE(spell_id, class_name, source)
);
CREATE INDEX idx_spell_classes_spell ON spell_classes(spell_id);
CREATE INDEX idx_spell_classes_class ON spell_classes(class_name);

-- Subclass-specific spell lists (e.g., Arcane Trickster, Eldritch Knight)
CREATE TABLE spell_subclasses (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    spell_id INTEGER NOT NULL REFERENCES spells(id) ON DELETE CASCADE,
    subclass_name TEXT NOT NULL,
    class_name TEXT NOT NULL,
    source TEXT NOT NULL REFERENCES catalog_sources(code),
    UNIQUE(spell_id, subclass_name, class_name, source)
);
CREATE INDEX idx_spell_subclasses_spell ON spell_subclasses(spell_id);
CREATE INDEX idx_spell_subclasses_subclass ON spell_subclasses(subclass_name, class_name);

-- Items requiring attunement by specific class
CREATE TABLE item_attunement_classes (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    item_id INTEGER NOT NULL REFERENCES items(id) ON DELETE CASCADE,
    class_name TEXT NOT NULL,
    UNIQUE(item_id, class_name)
);
CREATE INDEX idx_item_attunement_item ON item_attunement_classes(item_id);
CREATE INDEX idx_item_attunement_class ON item_attunement_classes(class_name);
```

## Implementation Notes

### Data Sources

Spell-class mappings come from generated lookup files in 5etools, not embedded in spell data:
- `data/generated/gendata-spell-source-lookup.json` - Maps spells to source classes
- Class/subclass definitions include `additionalSpells` arrays

### Dependencies

- MIMIR-T-0373 (Spell table must exist first)
- MIMIR-T-0372 (Item table must exist first)

## Status Updates **[REQUIRED]**

*To be added during implementation*