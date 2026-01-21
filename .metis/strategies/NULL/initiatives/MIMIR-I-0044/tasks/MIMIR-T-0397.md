---
id: update-characters-schema-and
level: task
title: "Update characters schema and implement CharacterService"
short_code: "MIMIR-T-0397"
created_at: 2026-01-21T03:02:30.849052+00:00
updated_at: 2026-01-21T13:56:32.863046+00:00
parent: MIMIR-I-0044
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


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

## Acceptance Criteria

## Acceptance Criteria

### Schema Updates
- [x] Add `race_name TEXT` column to characters table
- [x] Add `race_source TEXT` column to characters table
- [x] Add `background_name TEXT` column to characters table
- [x] Add `background_source TEXT` column to characters table
- [x] Update `Character` model with new fields
- [x] Update `NewCharacter` with builder methods (with_race, with_background)
- [x] Update `schema.rs` with new columns

### CharacterService
- [x] `CharacterService` struct with stateful connection pattern
- [x] `create()` - create character with race/background/ability scores
- [x] `list_for_campaign()` - list characters in a campaign
- [x] `list_pcs()` / `list_npcs()` - filter by character type
- [x] `get()` - get character by ID
- [x] `update()` - update character attributes
- [x] `delete()` - delete character
- [x] `add_to_inventory()` - add item to character inventory
- [x] `remove_from_inventory()` - remove item from inventory
- [x] `get_inventory()` / `get_equipped_items()` / `get_attuned_items()`
- [x] `update_inventory_item()` - update quantity/equipped/attuned
- [x] Unit tests for all operations (18 tests)

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