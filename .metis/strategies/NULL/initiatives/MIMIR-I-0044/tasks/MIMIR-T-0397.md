---
id: update-characters-schema-and
level: task
title: "Update characters schema and implement CharacterService"
short_code: "MIMIR-T-0397"
created_at: 2026-01-21T03:02:30.849052+00:00
updated_at: 2026-01-21T03:02:30.849052+00:00
parent: MIMIR-I-0044
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/todo"


exit_criteria_met: false
strategy_id: NULL
initiative_id: MIMIR-I-0044
---

# Update characters schema and implement CharacterService

## Parent Initiative

[[MIMIR-I-0044]] - Implement Service Layer for mimir-core

## Objective

Update the characters table schema to include race and background catalog references, then implement `CharacterService` for managing player characters and NPCs within campaigns.

## Acceptance Criteria

### Schema Updates
- [ ] Add `race_name TEXT` column to characters table
- [ ] Add `race_source TEXT` column to characters table
- [ ] Add `background_name TEXT` column to characters table
- [ ] Add `background_source TEXT` column to characters table
- [ ] Update `Character` model with new fields
- [ ] Update `NewCharacter` with builder methods
- [ ] Run `diesel print-schema` to regenerate `schema.rs`

### CharacterService
- [ ] `CharacterService` struct with stateful connection pattern
- [ ] `create()` - create character with race/background/class
- [ ] `list_for_campaign()` - list characters in a campaign
- [ ] `get()` - get character by ID with full details
- [ ] `update()` - update character attributes
- [ ] `delete()` - delete character and related data
- [ ] `add_to_inventory()` - add item to character inventory
- [ ] `remove_from_inventory()` - remove item from inventory
- [ ] Unit tests for all operations

## Implementation Notes

### Schema Update (Edit Existing Migration)

Edit the existing characters migration to add columns:

```sql
CREATE TABLE characters (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    campaign_id INTEGER NOT NULL REFERENCES campaigns(id),
    name TEXT NOT NULL,
    is_npc INTEGER NOT NULL DEFAULT 0,
    level INTEGER NOT NULL DEFAULT 1,
    -- New columns
    race_name TEXT,
    race_source TEXT,
    background_name TEXT,
    background_source TEXT,
    -- Existing columns
    data TEXT NOT NULL DEFAULT '{}',
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);
```

### Model Updates

```rust
// src/models/campaign/character.rs

#[derive(Queryable, Selectable, Debug)]
#[diesel(table_name = characters)]
pub struct Character {
    pub id: i32,
    pub campaign_id: i32,
    pub name: String,
    pub is_npc: bool,
    pub level: i32,
    pub race_name: Option<String>,
    pub race_source: Option<String>,
    pub background_name: Option<String>,
    pub background_source: Option<String>,
    pub data: String,
    pub created_at: String,
    pub updated_at: String,
}

impl NewCharacter {
    pub fn with_race(mut self, name: &str, source: &str) -> Self {
        self.race_name = Some(name.to_string());
        self.race_source = Some(source.to_string());
        self
    }
    
    pub fn with_background(mut self, name: &str, source: &str) -> Self {
        self.background_name = Some(name.to_string());
        self.background_source = Some(source.to_string());
        self
    }
}
```

### CharacterService API

```rust
pub struct CharacterService<'a> {
    conn: &'a mut SqliteConnection,
}

pub struct CreateCharacter {
    pub campaign_id: i32,
    pub name: String,
    pub is_npc: bool,
    pub race_name: Option<String>,
    pub race_source: Option<String>,
    pub background_name: Option<String>,
    pub background_source: Option<String>,
    pub class_name: Option<String>,
    pub class_source: Option<String>,
}

impl<'a> CharacterService<'a> {
    pub fn new(conn: &'a mut SqliteConnection) -> Self;
    
    pub fn create(&mut self, input: CreateCharacter) -> ServiceResult<Character>;
    pub fn list_for_campaign(&mut self, campaign_id: i32) -> ServiceResult<Vec<Character>>;
    pub fn get(&mut self, id: i32) -> ServiceResult<Option<Character>>;
    pub fn update(&mut self, id: i32, input: UpdateCharacter) -> ServiceResult<Character>;
    pub fn delete(&mut self, id: i32) -> ServiceResult<()>;
    
    // Inventory management
    pub fn add_to_inventory(&mut self, character_id: i32, item: AddInventoryItem) -> ServiceResult<()>;
    pub fn remove_from_inventory(&mut self, character_id: i32, inventory_id: i32) -> ServiceResult<()>;
    pub fn get_inventory(&mut self, character_id: i32) -> ServiceResult<Vec<InventoryItem>>;
}
```

### Dependencies

- MIMIR-T-0390 (ServiceError type)
- MIMIR-T-0392 (RaceService, BackgroundService for lookups)
- Existing `dal::character` module
- Related tables: `character_classes`, `character_inventory`, `character_spells`

## Status Updates

*To be added during implementation*