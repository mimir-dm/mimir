---
id: v0-5-database-schema-design
level: task
title: "v0.5 Database Schema Design"
short_code: "MIMIR-T-0357"
created_at: 2026-01-19T22:06:59.358792+00:00
updated_at: 2026-01-20T02:58:55.228219+00:00
parent: MIMIR-I-0041
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: MIMIR-I-0041
---

# v0.5 Database Schema Design

## Parent Initiative
[[MIMIR-I-0041]] - Mimir v0.5 Architecture Rewrite

## Objective
Define the complete information model for Mimir v0.5, starting from ontology (what exists and how it relates) through to concrete schema and type definitions.

## Acceptance Criteria

## Acceptance Criteria
- [ ] Complete ontology defining all entities and relationships
- [ ] DDL for all tables with proper constraints
- [ ] Rust types for all entities and value objects
- [ ] FTS5 virtual tables for full-text search
- [ ] Indexes for common query patterns

---

# Part 1: Ontology

## Domain Overview

Mimir is a **campaign management tool for D&D 5e Dungeon Masters**. The core workflow is:
1. DM creates a **Campaign** (the overarching story/world)
2. Campaign contains **Modules** (discrete adventures/chapters)
3. Modules contain preparation materials: **Documents**, **Maps**, **Encounters**
4. DM populates encounters with **Monsters** and **Items** from the **Catalog**
5. DM tracks **Characters** (both PCs and NPCs) across the campaign
6. During play, DM uses **Maps** with **Tokens** to run encounters

## Entity Categories

### 1. Campaign Entities (User-Created, Mutable)
These are the things the DM creates and modifies.

### 2. Catalog Entities (Imported, Read-Only)  
These are 5e reference data imported from source books.

### 3. Placement Entities (References + State)
These link catalog entities into campaign context with runtime state.

---

## Campaign Entities

### Campaign
The top-level container for all DM content.

| Attribute | Type | Description |
|-----------|------|-------------|
| id | ID | Unique identifier |
| name | String | Campaign name |
| description | Text? | Optional description |
| created_at | Timestamp | When created |
| updated_at | Timestamp | Last modified |
| archived_at | Timestamp? | Soft delete |

**Relationships:**
- has many → Module
- has many → Document (campaign-level)
- has many → Character
- has many → Map (campaign-level)
- has many ↔ CatalogSource (via CampaignSource) — **Rule Set**

---

### CampaignSource
Join table defining which source books are allowed in a campaign (the "rule set").

| Attribute | Type | Description |
|-----------|------|-------------|
| id | ID | |
| campaign_id | FK | Parent campaign |
| source_code | String | CatalogSource code ('PHB', 'XGE', etc.) |

**Purpose:**
- Defines "what books are in play" for this campaign
- Constrains character creation options for players
- Filters catalog searches to relevant content
- Documents requirements when sharing campaigns

**Constraint:** (campaign_id, source_code) is unique

---

### Module
A discrete adventure or chapter within a campaign.

| Attribute | Type | Description |
|-----------|------|-------------|
| id | ID | Unique identifier |
| campaign_id | FK | Parent campaign |
| name | String | Module name |
| module_number | Integer | Ordering (1, 2, 3...) |
| description | Text? | Optional description |
| created_at | Timestamp | |
| updated_at | Timestamp | |

**Relationships:**
- belongs to → Campaign
- has many → Document (module-level)
- has many → Map (module-level)
- has many → ModuleMonster (encounter building)
- has many → ModuleNpc (NPC assignments)

**Constraints:**
- (campaign_id, module_number) is unique

---

### Document
Freeform markdown content for notes, handouts, world-building.

| Attribute | Type | Description |
|-----------|------|-------------|
| id | ID | Unique identifier |
| campaign_id | FK | Parent campaign |
| module_id | FK? | Optional parent module |
| title | String | Document title |
| content | Text | Markdown content (stored in DB) |
| document_type | String? | Category: 'play_notes', 'handout', 'lore', etc. |
| created_at | Timestamp | |
| updated_at | Timestamp | |

**Relationships:**
- belongs to → Campaign
- optionally belongs to → Module

**Key Decision:** Content stored in database, not filesystem. Enables ACID, search, export.

---

### Character
A PC or NPC in the campaign. Unified model with `is_npc` flag.

| Attribute | Type | Description |
|-----------|------|-------------|
| id | ID | Unique identifier |
| campaign_id | FK | Parent campaign |
| name | String | Character name |
| is_npc | Boolean | true=NPC, false=PC |
| player_name | String? | For PCs, the player's name |

**Identity:**
| Attribute | Type | Description |
|-----------|------|-------------|
| race | String? | Race name |
| background | String? | Background name |
| alignment | String? | Alignment |
| level | Integer | Total character level |
| classes | ClassLevel[] | Multiclass support (JSON) |

**Abilities:**
| Attribute | Type | Description |
|-----------|------|-------------|
| strength | Integer | Ability score (default 10) |
| dexterity | Integer | |
| constitution | Integer | |
| intelligence | Integer | |
| wisdom | Integer | |
| charisma | Integer | |

**Combat:**
| Attribute | Type | Description |
|-----------|------|-------------|
| max_hp | Integer? | Maximum hit points |
| current_hp | Integer? | Current hit points |
| temp_hp | Integer? | Temporary hit points |
| armor_class | Integer? | AC |
| speed | Integer | Movement speed in feet |
| initiative_bonus | Integer? | Initiative modifier |

**Resources:**
| Attribute | Type | Description |
|-----------|------|-------------|
| hit_dice_remaining | Integer? | Remaining hit dice |
| death_save_successes | Integer | 0-3 |
| death_save_failures | Integer | 0-3 |

**Currency:**
| Attribute | Type | Description |
|-----------|------|-------------|
| copper | Integer | CP |
| silver | Integer | SP |
| electrum | Integer | EP |
| gold | Integer | GP |
| platinum | Integer | PP |

**Personality:**
| Attribute | Type | Description |
|-----------|------|-------------|
| personality_traits | Text? | |
| ideals | Text? | |
| bonds | Text? | |
| flaws | Text? | |
| backstory | Text? | |

**NPC-Specific:**
| Attribute | Type | Description |
|-----------|------|-------------|
| npc_role | String? | 'quest_giver', 'merchant', 'antagonist', etc. |
| npc_location | String? | Where they're found |
| npc_faction | String? | Faction affiliation |
| npc_notes | Text? | DM notes |

**Relationships:**
- belongs to → Campaign
- has many → CharacterInventory
- has many → CharacterProficiency
- has many → CharacterSpell (for spellcasters)

---

### CharacterInventory
Items held by a character.

| Attribute | Type | Description |
|-----------|------|-------------|
| id | ID | |
| character_id | FK | Owner |
| item_name | String | Item name (may reference catalog) |
| item_source | String? | Source book code |
| quantity | Integer | Stack count |
| equipped | Boolean | Currently equipped? |
| attuned | Boolean | Attuned (for magic items) |
| notes | Text? | Custom notes |

---

### CharacterProficiency
Skills, saves, tools, languages a character is proficient in.

| Attribute | Type | Description |
|-----------|------|-------------|
| id | ID | |
| character_id | FK | Owner |
| proficiency_type | Enum | 'skill', 'save', 'armor', 'weapon', 'tool', 'language' |
| name | String | The proficiency name |
| expertise | Boolean | Double proficiency bonus? |

**Constraint:** (character_id, proficiency_type, name) is unique

---

### CharacterSpell
Spells known/prepared by a spellcasting character.

| Attribute | Type | Description |
|-----------|------|-------------|
| id | ID | |
| character_id | FK | Owner |
| spell_name | String | Spell name |
| spell_source | String | Source book code |
| prepared | Boolean | Currently prepared? |
| source_class | String? | Which class grants this spell |

---

### Map
A tactical map for encounter play.

| Attribute | Type | Description |
|-----------|------|-------------|
| id | ID | |
| campaign_id | FK | Parent campaign |
| module_id | FK? | Optional parent module |
| name | String | Map name |

**Image/Source:**
| Attribute | Type | Description |
|-----------|------|-------------|
| uvtt_path | String | Path to .uvtt file |
| image_path | String | Extracted map image |
| width_px | Integer | Image width |
| height_px | Integer | Image height |

**Grid:**
| Attribute | Type | Description |
|-----------|------|-------------|
| grid_type | Enum | 'square', 'hex', 'none' |
| grid_size_px | Integer | Pixels per grid cell |
| grid_offset_x | Integer | Grid origin X |
| grid_offset_y | Integer | Grid origin Y |

**Lighting:**
| Attribute | Type | Description |
|-----------|------|-------------|
| ambient_light | Enum | 'bright', 'dim', 'darkness' |
| fog_enabled | Boolean | Fog of war active? |
| los_walls | Wall[] | Line-of-sight walls (JSON from UVTT) |

**Relationships:**
- belongs to → Campaign
- optionally belongs to → Module
- has many → TokenPlacement
- has many → MapTrap
- has many → FogArea
- has many → LightSource

---

### TokenPlacement
An instance of a token placed on a map. The actual token image comes from the linked Monster's `token_image_path`.

| Attribute | Type | Description |
|-----------|------|-------------|
| id | ID | |
| map_id | FK | Parent map |
| name | String | Display name (can override monster name) |
| token_type | Enum | 'monster', 'npc', 'marker' |
| size | Enum | 'tiny', 'small', 'medium', 'large', 'huge', 'gargantuan' |
| x | Float | Grid position X |
| y | Float | Grid position Y |
| visible_to_players | Boolean | Player visibility |
| color | String? | Fallback color if no image (hex) |

**Links:**
| Attribute | Type | Description |
|-----------|------|-------------|
| monster_id | FK? | Link to catalog monster (provides token image) |
| character_id | FK? | Link to campaign NPC |

**Note:** Token images are stored on Monster entities (`token_image_path`). Characters don't have tokens yet. When placing a monster, we use its associated token image.

---

### MapTrap
A trap placed on a map.

| Attribute | Type | Description |
|-----------|------|-------------|
| id | ID | |
| map_id | FK | Parent map |
| trap_name | String | Trap name (references catalog) |
| trap_source | String | Source book code |
| x | Float | Position X |
| y | Float | Position Y |
| width | Integer | Size in grid squares |
| height | Integer | Size in grid squares |
| triggered | Boolean | Has it been triggered? |
| visible_to_players | Boolean | Player visibility |
| notes | Text? | DM notes |

---

### FogArea
Revealed/hidden areas on a map.

| Attribute | Type | Description |
|-----------|------|-------------|
| id | ID | |
| map_id | FK | Parent map |
| x | Integer | Grid position |
| y | Integer | Grid position |
| width | Integer | Grid squares |
| height | Integer | Grid squares |

---

### LightSource
A light on a map.

| Attribute | Type | Description |
|-----------|------|-------------|
| id | ID | |
| map_id | FK | Parent map |
| x | Float | Position |
| y | Float | Position |
| light_type | Enum | 'bright', 'dim' |
| radius_ft | Integer | Light radius |
| color | String? | Light color (hex) |

---

## Placement Entities

These link catalog entities into modules for encounter building.

### ModuleMonster
A monster assigned to a module for encounter planning.

| Attribute | Type | Description |
|-----------|------|-------------|
| id | ID | |
| module_id | FK | Parent module |
| monster_name | String | Monster name (catalog reference) |
| monster_source | String | Source book code |
| quantity | Integer | How many |
| encounter_tag | String? | Grouping tag ('room1', 'boss', etc.) |
| display_name | String? | Override name ('Goblin Chief') |
| notes | Text? | DM notes |

**Constraint:** (module_id, monster_name, monster_source, encounter_tag) is unique

---

### ModuleNpc
An NPC assigned to appear in a module.

| Attribute | Type | Description |
|-----------|------|-------------|
| id | ID | |
| module_id | FK | Parent module |
| character_id | FK | The NPC character |
| role | String? | Role in this module |
| encounter_tag | String? | Grouping tag |
| notes | Text? | DM notes |

**Constraint:** (module_id, character_id) is unique

---

## Catalog Entities

Read-only reference data imported from 5e source books. All catalog entities follow a common pattern:
- **Indexed columns** for search/filter queries
- **data** column containing the full JSON blob
- **source** references CatalogSource.code

### CatalogSource
Tracks which source books are imported and enabled.

| Attribute | Type | Description |
|-----------|------|-------------|
| id | ID | |
| code | String | Short code ('PHB', 'MM', 'DMG') |
| name | String | Full name |
| source_type | Enum | 'core', 'supplement', 'adventure', 'homebrew' |
| enabled | Boolean | User can disable sources globally |
| imported_at | Timestamp | When imported |

**Constraint:** code is unique

---

### Monster
5e monster/creature stat block.

| Indexed | Type | Description |
|---------|------|-------------|
| name | String | Monster name |
| source | String | Source book code |
| size | Enum | Creature size |
| creature_type | String | 'dragon', 'undead', 'humanoid', etc. |
| cr | String | Challenge rating ('1/4', '1', '10') |
| cr_numeric | Float | Numeric CR for range filtering |
| hp | Integer | Hit points |
| ac | Integer | Armor class |
| token_image_path | String? | Path to token image on disk |
| data | JSON | Complete monster object |

---

### Item
5e item (weapons, armor, magic items, adventuring gear).

| Indexed | Type | Description |
|---------|------|-------------|
| name | String | Item name |
| source | String | Source book |
| item_type | String | 'Weapon', 'Armor', 'Potion', 'Wondrous Item', etc. |
| rarity | String? | 'Common', 'Uncommon', 'Rare', 'Very Rare', 'Legendary' |
| value_cp | Integer? | Value in copper pieces |
| weight | Float? | Weight in pounds |
| requires_attunement | Boolean | Requires attunement? |
| data | JSON | Complete item object |

---

### Spell
5e spell.

| Indexed | Type | Description |
|---------|------|-------------|
| name | String | Spell name |
| source | String | Source book |
| level | Integer | 0-9 (0 = cantrip) |
| school | String | 'Evocation', 'Abjuration', 'Necromancy', etc. |
| classes | String | Comma-separated class list |
| ritual | Boolean | Can be cast as ritual? |
| concentration | Boolean | Requires concentration? |
| casting_time | String | |
| range | String | |
| duration | String | |
| data | JSON | Complete spell object |

---

### Class
5e character class with subclasses.

| Indexed | Type | Description |
|---------|------|-------------|
| name | String | Class name |
| source | String | Source book |
| hit_die | Integer | d6, d8, d10, d12 |
| primary_ability | String | Primary ability score |
| saves | String | Comma-separated saving throws |
| data | JSON | Complete class with features and subclasses |

---

### Race
5e character race with subraces.

| Indexed | Type | Description |
|---------|------|-------------|
| name | String | Race name |
| source | String | Source book |
| size | String | 'Small', 'Medium', etc. |
| speed | Integer | Base walking speed |
| data | JSON | Complete race with traits and subraces |

---

### Background
5e character background.

| Indexed | Type | Description |
|---------|------|-------------|
| name | String | Background name |
| source | String | Source book |
| skill_proficiencies | String | Comma-separated skills |
| data | JSON | Complete background |

---

### Feat
5e feat.

| Indexed | Type | Description |
|---------|------|-------------|
| name | String | Feat name |
| source | String | Source book |
| prerequisite | String? | Prerequisite text |
| data | JSON | Complete feat |

---

### Trap
5e trap or environmental hazard.

| Indexed | Type | Description |
|---------|------|-------------|
| name | String | Trap name |
| source | String | Source book |
| trap_type | String | 'mechanical', 'magical' |
| category | String | 'Trap', 'Hazard' |
| severity | String | 'setback', 'dangerous', 'deadly' |
| data | JSON | Complete trap |

---

### Action
Combat actions (Dash, Dodge, Help, etc.).

| Indexed | Type | Description |
|---------|------|-------------|
| name | String | Action name |
| source | String | Source book |
| action_type | String? | Action category |
| data | JSON | Complete action |

---

### Condition
Status conditions (Blinded, Charmed, Frightened, etc.).

| Indexed | Type | Description |
|---------|------|-------------|
| name | String | Condition name |
| source | String | Source book |
| data | JSON | Complete condition with effects |

---

### Language
Languages spoken in the game world.

| Indexed | Type | Description |
|---------|------|-------------|
| name | String | Language name |
| source | String | Source book |
| language_type | String | 'Standard', 'Exotic', 'Secret' |
| script | String? | Script used |
| data | JSON | Complete language |

---

### Deity
Gods and divine beings.

| Indexed | Type | Description |
|---------|------|-------------|
| name | String | Deity name |
| source | String | Source book |
| pantheon | String | Pantheon (Forgotten Realms, Greek, etc.) |
| alignment | String | Deity alignment |
| domains | String | Comma-separated cleric domains |
| data | JSON | Complete deity |

---

### Vehicle
Vehicles (ships, siege equipment, etc.).

| Indexed | Type | Description |
|---------|------|-------------|
| name | String | Vehicle name |
| source | String | Source book |
| vehicle_type | String | 'Ship', 'Land', 'Air', etc. |
| data | JSON | Complete vehicle with stats |

---

### Object
Mundane and magical objects (not items - things like doors, chests).

| Indexed | Type | Description |
|---------|------|-------------|
| name | String | Object name |
| source | String | Source book |
| object_type | String? | Object category |
| data | JSON | Complete object |

---

### Reward
Supernatural gifts, blessings, boons.

| Indexed | Type | Description |
|---------|------|-------------|
| name | String | Reward name |
| source | String | Source book |
| reward_type | String | 'Blessing', 'Boon', 'Charm', etc. |
| data | JSON | Complete reward |

---

### OptionalFeature
Optional class features, fighting styles, invocations, etc.

| Indexed | Type | Description |
|---------|------|-------------|
| name | String | Feature name |
| source | String | Source book |
| feature_type | String | 'Fighting Style', 'Eldritch Invocation', 'Metamagic', etc. |
| prerequisite | String? | Prerequisite text |
| data | JSON | Complete feature |

---

### Psionic
Psionic powers and disciplines.

| Indexed | Type | Description |
|---------|------|-------------|
| name | String | Power name |
| source | String | Source book |
| psionic_type | String | 'Talent', 'Discipline' |
| order | String? | Psionic order |
| data | JSON | Complete psionic |

---

### VariantRule
Variant and optional rules.

| Indexed | Type | Description |
|---------|------|-------------|
| name | String | Rule name |
| source | String | Source book |
| rule_type | String? | Rule category |
| data | JSON | Complete rule text |

---

### Cult
Cults, boons, and demonic gifts.

| Indexed | Type | Description |
|---------|------|-------------|
| name | String | Cult/boon name |
| source | String | Source book |
| data | JSON | Complete cult with boons |

---

### Table
Random tables (treasure, encounters, names, etc.).

| Indexed | Type | Description |
|---------|------|-------------|
| name | String | Table name |
| source | String | Source book |
| table_type | String? | Table category |
| data | JSON | Complete table with rows |

---

## Value Objects

These are embedded types, not separate entities.

### ClassLevel
Represents one class in a multiclass character.

| Attribute | Type | Description |
|-----------|------|-------------|
| class | String | Class name |
| level | Integer | Levels in this class |
| subclass | String? | Subclass name |

---

### Feature
A feat or feature on a character.

| Attribute | Type | Description |
|-----------|------|-------------|
| name | String | Feature name |
| source | String | Source book |
| feature_type | Enum | 'feat', 'class_feature', 'racial_trait', 'background_feature' |
| description | String? | Description text |

---

### Wall
Line-of-sight wall segment on a map.

| Attribute | Type | Description |
|-----------|------|-------------|
| start | Point | Start coordinate |
| end | Point | End coordinate |

---

### Point
2D coordinate.

| Attribute | Type | Description |
|-----------|------|-------------|
| x | Float | X coordinate |
| y | Float | Y coordinate |

---

## Entity Relationship Summary

```
Campaign (1)
├── CampaignSource (N) ──references──> CatalogSource  [RULE SET]
├── Module (N)
│   ├── Document (N)
│   ├── Map (N)
│   │   ├── TokenPlacement (N) ──references──> Monster (token image)
│   │   ├── MapTrap (N) ──references──> Trap (Catalog)
│   │   ├── FogArea (N)
│   │   └── LightSource (N)
│   ├── ModuleMonster (N) ──references──> Monster (Catalog)
│   └── ModuleNpc (N) ──references──> Character
├── Document (N) [campaign-level]
├── Map (N) [campaign-level]
└── Character (N)
    ├── CharacterInventory (N) ──references──> Item (Catalog)
    ├── CharacterProficiency (N)
    └── CharacterSpell (N) ──references──> Spell (Catalog)

Catalog (read-only, 20 entity types)
├── CatalogSource (N)
├── Monster (N)        ── has token_image_path
├── Item (N)
├── Spell (N)
├── Class (N)
├── Race (N)
├── Background (N)
├── Feat (N)
├── Trap (N)
├── Action (N)
├── Condition (N)
├── Language (N)
├── Deity (N)
├── Vehicle (N)
├── Object (N)
├── Reward (N)
├── OptionalFeature (N)
├── Psionic (N)
├── VariantRule (N)
├── Cult (N)
└── Table (N)
```

---

# Part 2: Schema Design

### campaigns
```sql
CREATE TABLE campaigns (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    archived_at TEXT
);
```

### modules
```sql
CREATE TABLE modules (
    id INTEGER PRIMARY KEY,
    campaign_id INTEGER NOT NULL REFERENCES campaigns(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    module_number INTEGER NOT NULL,
    description TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(campaign_id, module_number)
);
```

### documents
```sql
CREATE TABLE documents (
    id INTEGER PRIMARY KEY,
    campaign_id INTEGER NOT NULL REFERENCES campaigns(id) ON DELETE CASCADE,
    module_id INTEGER REFERENCES modules(id) ON DELETE CASCADE,
    title TEXT NOT NULL,
    content TEXT NOT NULL DEFAULT '',
    document_type TEXT,  -- 'play_notes', 'handout', 'world_building', etc.
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Full-text search on documents
CREATE VIRTUAL TABLE documents_fts USING fts5(
    title, content, 
    content='documents', 
    content_rowid='id'
);
```

### characters (unified PC/NPC, no versioning)
```sql
CREATE TABLE characters (
    id INTEGER PRIMARY KEY,
    campaign_id INTEGER REFERENCES campaigns(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    is_npc INTEGER NOT NULL DEFAULT 1,
    player_name TEXT,  -- For PCs, just store the player's name directly
    
    -- Core stats
    race TEXT,
    level INTEGER DEFAULT 1,
    background TEXT,
    alignment TEXT,
    
    -- Multiclass support (JSON array of ClassLevel)
    -- Format: [{"class": "Fighter", "level": 5}, {"class": "Wizard", "level": 2}]
    classes TEXT NOT NULL DEFAULT '[]',
    
    -- Feats and Features (JSON array of Feature)
    -- Format: [{"name": "Great Weapon Master", "source": "PHB", "description": "..."}, ...]
    feats TEXT NOT NULL DEFAULT '[]',
    
    -- Abilities
    strength INTEGER DEFAULT 10,
    dexterity INTEGER DEFAULT 10,
    constitution INTEGER DEFAULT 10,
    intelligence INTEGER DEFAULT 10,
    wisdom INTEGER DEFAULT 10,
    charisma INTEGER DEFAULT 10,
    
    -- Combat
    max_hp INTEGER,
    current_hp INTEGER,
    armor_class INTEGER,
    speed INTEGER DEFAULT 30,
    
    -- Currency
    copper INTEGER DEFAULT 0,
    silver INTEGER DEFAULT 0,
    electrum INTEGER DEFAULT 0,
    gold INTEGER DEFAULT 0,
    platinum INTEGER DEFAULT 0,
    
    -- Personality
    personality_traits TEXT,
    ideals TEXT,
    bonds TEXT,
    flaws TEXT,
    backstory TEXT,
    
    -- NPC-specific
    npc_role TEXT,        -- 'quest_giver', 'merchant', 'antagonist', etc.
    npc_location TEXT,
    npc_faction TEXT,
    npc_notes TEXT,
    
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);
```

### Rust Types for Character JSON Fields

```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct ClassLevel {
    pub class: String,      // "Fighter", "Wizard", etc.
    pub level: i32,
    pub subclass: Option<String>,  // "Champion", "Evocation", etc.
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Feature {
    pub name: String,
    pub source: String,     // "PHB", "XGE", etc.
    pub feature_type: FeatureType,
    pub description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum FeatureType {
    Feat,
    ClassFeature,
    RacialTrait,
    BackgroundFeature,
}
```

### character_inventory
```sql
CREATE TABLE character_inventory (
    id INTEGER PRIMARY KEY,
    character_id INTEGER NOT NULL REFERENCES characters(id) ON DELETE CASCADE,
    item_name TEXT NOT NULL,
    item_source TEXT,
    quantity INTEGER DEFAULT 1,
    equipped INTEGER DEFAULT 0,
    notes TEXT
);
```

### character_proficiencies
```sql
CREATE TABLE character_proficiencies (
    id INTEGER PRIMARY KEY,
    character_id INTEGER NOT NULL REFERENCES characters(id) ON DELETE CASCADE,
    proficiency_type TEXT NOT NULL,  -- 'skill', 'save', 'armor', 'weapon', 'tool', 'language'
    name TEXT NOT NULL,
    UNIQUE(character_id, proficiency_type, name)
);
```

### maps (UVTT format storage)
```sql
CREATE TABLE maps (
    id INTEGER PRIMARY KEY,
    campaign_id INTEGER NOT NULL REFERENCES campaigns(id) ON DELETE CASCADE,
    module_id INTEGER REFERENCES modules(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    
    -- UVTT file storage
    uvtt_path TEXT NOT NULL,  -- Path to .uvtt file on filesystem
    
    -- Extracted/cached from UVTT for quick access
    image_path TEXT NOT NULL,  -- Extracted map image
    width_px INTEGER NOT NULL,
    height_px INTEGER NOT NULL,
    
    -- Grid (from UVTT or overridden)
    grid_type TEXT DEFAULT 'square',  -- 'square', 'hex', 'none'
    grid_size_px INTEGER DEFAULT 70,
    grid_offset_x INTEGER DEFAULT 0,
    grid_offset_y INTEGER DEFAULT 0,
    
    -- Line of Sight walls (JSON array from UVTT)
    -- Format: [{"start": {"x": 0, "y": 0}, "end": {"x": 100, "y": 0}}, ...]
    los_walls TEXT NOT NULL DEFAULT '[]',
    
    -- Lighting
    ambient_light TEXT DEFAULT 'bright',  -- 'bright', 'dim', 'darkness'
    fog_enabled INTEGER DEFAULT 0,
    
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);
```

### map_traps (traps placed on maps)
```sql
CREATE TABLE map_traps (
    id INTEGER PRIMARY KEY,
    map_id INTEGER NOT NULL REFERENCES maps(id) ON DELETE CASCADE,
    trap_name TEXT NOT NULL,
    trap_source TEXT NOT NULL,
    x REAL NOT NULL,
    y REAL NOT NULL,
    width INTEGER DEFAULT 1,  -- Grid squares
    height INTEGER DEFAULT 1,
    triggered INTEGER DEFAULT 0,
    visible_to_players INTEGER DEFAULT 0,
    notes TEXT
);
```

### tokens
```sql
CREATE TABLE tokens (
    id INTEGER PRIMARY KEY,
    map_id INTEGER NOT NULL REFERENCES maps(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    token_type TEXT NOT NULL,  -- 'monster', 'pc', 'npc', 'trap', 'marker'
    size TEXT DEFAULT 'medium',  -- 'tiny', 'small', 'medium', 'large', 'huge', 'gargantuan'
    x REAL NOT NULL,
    y REAL NOT NULL,
    visible_to_players INTEGER DEFAULT 1,
    color TEXT,  -- hex color fallback
    image_path TEXT,  -- optional token image
    
    -- Links to entities
    monster_id INTEGER REFERENCES catalog_monsters(id),
    character_id INTEGER REFERENCES characters(id),
    
    -- Vision
    vision_type TEXT DEFAULT 'normal',
    vision_range_ft INTEGER
);
```

### fog_areas
```sql
CREATE TABLE fog_areas (
    id INTEGER PRIMARY KEY,
    map_id INTEGER NOT NULL REFERENCES maps(id) ON DELETE CASCADE,
    x INTEGER NOT NULL,
    y INTEGER NOT NULL,
    width INTEGER NOT NULL,
    height INTEGER NOT NULL
);
```

### light_sources
```sql
CREATE TABLE light_sources (
    id INTEGER PRIMARY KEY,
    map_id INTEGER NOT NULL REFERENCES maps(id) ON DELETE CASCADE,
    x REAL NOT NULL,
    y REAL NOT NULL,
    light_type TEXT NOT NULL,  -- 'bright', 'dim'
    radius_ft INTEGER NOT NULL,
    color TEXT
);
```

### module_monsters
```sql
CREATE TABLE module_monsters (
    id INTEGER PRIMARY KEY,
    module_id INTEGER NOT NULL REFERENCES modules(id) ON DELETE CASCADE,
    monster_name TEXT NOT NULL,
    monster_source TEXT NOT NULL,
    quantity INTEGER DEFAULT 1,
    encounter_tag TEXT,
    display_name TEXT,
    notes TEXT,
    UNIQUE(module_id, monster_name, monster_source, encounter_tag)
);
```

### module_items
```sql
CREATE TABLE module_items (
    id INTEGER PRIMARY KEY,
    module_id INTEGER NOT NULL REFERENCES modules(id) ON DELETE CASCADE,
    item_name TEXT NOT NULL,
    item_source TEXT NOT NULL,
    quantity INTEGER DEFAULT 1,
    location TEXT,
    notes TEXT
);
```

### module_npcs
```sql
CREATE TABLE module_npcs (
    id INTEGER PRIMARY KEY,
    module_id INTEGER NOT NULL REFERENCES modules(id) ON DELETE CASCADE,
    character_id INTEGER NOT NULL REFERENCES characters(id) ON DELETE CASCADE,
    role TEXT,
    encounter_tag TEXT,
    notes TEXT,
    UNIQUE(module_id, character_id)
);
```

### Source Management

```sql
-- Available source books (imported content)
CREATE TABLE catalog_sources (
    id INTEGER PRIMARY KEY,
    code TEXT NOT NULL UNIQUE,      -- 'PHB', 'MM', 'DMG', 'XGE', etc.
    name TEXT NOT NULL,             -- 'Player''s Handbook'
    source_type TEXT NOT NULL,      -- 'core', 'supplement', 'adventure', 'homebrew'
    enabled INTEGER DEFAULT 1,      -- User can disable sources
    imported_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Default sources to enable when no filter specified
-- If empty, all enabled sources are used
```

### Catalog Tables (read-only, populated from 5e data)

```sql
CREATE TABLE catalog_monsters (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    source TEXT NOT NULL REFERENCES catalog_sources(code),
    size TEXT,
    creature_type TEXT,
    alignment TEXT,
    cr TEXT,
    cr_numeric REAL,
    hp INTEGER,
    ac INTEGER,
    data TEXT NOT NULL,  -- Full JSON blob
    UNIQUE(name, source)
);

CREATE TABLE catalog_items (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    source TEXT NOT NULL REFERENCES catalog_sources(code),
    item_type TEXT,
    rarity TEXT,
    value_cp INTEGER,
    weight REAL,
    requires_attunement INTEGER DEFAULT 0,
    data TEXT NOT NULL,
    UNIQUE(name, source)
);

CREATE TABLE catalog_spells (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    source TEXT NOT NULL REFERENCES catalog_sources(code),
    level INTEGER,
    school TEXT,
    casting_time TEXT,
    range TEXT,
    components TEXT,
    duration TEXT,
    classes TEXT,  -- comma-separated
    ritual INTEGER DEFAULT 0,
    concentration INTEGER DEFAULT 0,
    data TEXT NOT NULL,
    UNIQUE(name, source)
);

CREATE TABLE catalog_traps (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    source TEXT NOT NULL REFERENCES catalog_sources(code),
    trap_type TEXT,       -- 'mechanical', 'magical'
    category TEXT,        -- 'Trap', 'Hazard'
    trigger TEXT,
    severity TEXT,        -- 'setback', 'dangerous', 'deadly'
    data TEXT NOT NULL,
    UNIQUE(name, source)
);

CREATE TABLE catalog_classes (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    source TEXT NOT NULL REFERENCES catalog_sources(code),
    hit_die INTEGER,
    primary_ability TEXT,
    saves TEXT,           -- comma-separated
    data TEXT NOT NULL,
    UNIQUE(name, source)
);

CREATE TABLE catalog_races (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    source TEXT NOT NULL REFERENCES catalog_sources(code),
    size TEXT,
    speed INTEGER,
    ability_bonuses TEXT, -- JSON
    data TEXT NOT NULL,
    UNIQUE(name, source)
);

CREATE TABLE catalog_feats (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    source TEXT NOT NULL REFERENCES catalog_sources(code),
    prerequisite TEXT,
    data TEXT NOT NULL,
    UNIQUE(name, source)
);

CREATE TABLE catalog_backgrounds (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    source TEXT NOT NULL REFERENCES catalog_sources(code),
    skill_proficiencies TEXT,
    data TEXT NOT NULL,
    UNIQUE(name, source)
);

-- FTS for catalog search
CREATE VIRTUAL TABLE catalog_monsters_fts USING fts5(
    name, creature_type, source,
    content='catalog_monsters',
    content_rowid='id'
);

CREATE VIRTUAL TABLE catalog_items_fts USING fts5(
    name, item_type, rarity,
    content='catalog_items',
    content_rowid='id'
);

CREATE VIRTUAL TABLE catalog_spells_fts USING fts5(
    name, school, classes,
    content='catalog_spells',
    content_rowid='id'
);
```

## Catalog Entity Rust Types

These types represent the `data` JSON blob stored in each catalog table.

### Sources

```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct CatalogSource {
    pub code: String,           // "PHB", "MM", "DMG"
    pub name: String,           // "Player's Handbook"
    pub source_type: SourceType,
    pub enabled: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum SourceType {
    Core,           // PHB, MM, DMG
    Supplement,     // XGE, TCE, etc.
    Adventure,      // CoS, ToA, etc.
    Homebrew,       // User-created content
}
```

### Monsters

```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct Monster {
    pub name: String,
    pub source: String,
    pub size: CreatureSize,
    pub creature_type: String,
    pub subtype: Option<String>,
    pub alignment: String,
    pub ac: ArmorClass,
    pub hp: HitPoints,
    pub speed: Speed,
    pub abilities: AbilityScores,
    pub saves: Option<HashMap<String, i32>>,
    pub skills: Option<HashMap<String, i32>>,
    pub damage_resistances: Vec<String>,
    pub damage_immunities: Vec<String>,
    pub condition_immunities: Vec<String>,
    pub senses: Vec<String>,
    pub languages: Vec<String>,
    pub cr: ChallengeRating,
    pub traits: Vec<Trait>,
    pub actions: Vec<Action>,
    pub legendary_actions: Option<Vec<Action>>,
    pub lair_actions: Option<Vec<Action>>,
    pub reactions: Option<Vec<Action>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChallengeRating {
    pub cr: String,         // "1/4", "1", "10"
    pub xp: i32,
}
```

### Items

```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct Item {
    pub name: String,
    pub source: String,
    pub item_type: ItemType,
    pub rarity: Option<Rarity>,
    pub value: Option<Currency>,
    pub weight: Option<f32>,
    pub requires_attunement: bool,
    pub attunement_requirement: Option<String>,
    pub properties: Vec<String>,
    pub damage: Option<Damage>,
    pub ac: Option<i32>,
    pub description: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ItemType {
    Weapon, Armor, Potion, Ring, Rod, Scroll, Staff, 
    Wand, WondrousItem, AdventuringGear, Tool, Mount, Vehicle,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Rarity {
    Common, Uncommon, Rare, VeryRare, Legendary, Artifact,
}
```

### Spells

```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct Spell {
    pub name: String,
    pub source: String,
    pub level: i32,              // 0 for cantrips
    pub school: SpellSchool,
    pub casting_time: String,
    pub range: String,
    pub components: Components,
    pub duration: String,
    pub concentration: bool,
    pub ritual: bool,
    pub classes: Vec<String>,
    pub description: String,
    pub higher_levels: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum SpellSchool {
    Abjuration, Conjuration, Divination, Enchantment,
    Evocation, Illusion, Necromancy, Transmutation,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Components {
    pub verbal: bool,
    pub somatic: bool,
    pub material: Option<String>,
    pub material_cost: Option<Currency>,
    pub consumed: bool,
}
```

### Traps

```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct Trap {
    pub name: String,
    pub source: String,
    pub category: TrapCategory,
    pub trap_type: TrapType,
    pub trigger: String,
    pub effect: String,
    pub countermeasures: String,
    pub severity: TrapSeverity,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum TrapCategory { Trap, Hazard }

#[derive(Debug, Serialize, Deserialize)]
pub enum TrapType { Mechanical, Magical }

#[derive(Debug, Serialize, Deserialize)]
pub enum TrapSeverity { Setback, Dangerous, Deadly }
```

### Classes

```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct Class {
    pub name: String,
    pub source: String,
    pub hit_die: i32,
    pub primary_ability: Vec<String>,
    pub saving_throws: Vec<String>,
    pub armor_proficiencies: Vec<String>,
    pub weapon_proficiencies: Vec<String>,
    pub tool_proficiencies: Vec<String>,
    pub skill_choices: SkillChoice,
    pub starting_equipment: Vec<String>,
    pub features: Vec<ClassFeature>,
    pub subclasses: Vec<Subclass>,
    pub spellcasting: Option<SpellcastingInfo>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ClassFeature {
    pub name: String,
    pub level: i32,
    pub description: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Subclass {
    pub name: String,
    pub source: String,
    pub features: Vec<ClassFeature>,
}
```

### Races

```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct Race {
    pub name: String,
    pub source: String,
    pub size: CreatureSize,
    pub speed: i32,
    pub ability_bonuses: HashMap<String, i32>,
    pub traits: Vec<RacialTrait>,
    pub languages: Vec<String>,
    pub subraces: Vec<Subrace>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RacialTrait {
    pub name: String,
    pub description: String,
}
```

### Feats

```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct Feat {
    pub name: String,
    pub source: String,
    pub prerequisite: Option<String>,
    pub ability_increase: Option<HashMap<String, i32>>,
    pub description: String,
}
```

### Backgrounds

```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct Background {
    pub name: String,
    pub source: String,
    pub skill_proficiencies: Vec<String>,
    pub tool_proficiencies: Vec<String>,
    pub languages: i32,
    pub equipment: Vec<String>,
    pub feature_name: String,
    pub feature_description: String,
}
```

## Indexes

```sql
-- Campaign/Module hierarchy
CREATE INDEX idx_modules_campaign ON modules(campaign_id);
CREATE INDEX idx_documents_campaign ON documents(campaign_id);
CREATE INDEX idx_documents_module ON documents(module_id);
CREATE INDEX idx_characters_campaign ON characters(campaign_id);
CREATE INDEX idx_characters_is_npc ON characters(is_npc);

-- Maps
CREATE INDEX idx_maps_campaign ON maps(campaign_id);
CREATE INDEX idx_maps_module ON maps(module_id);
CREATE INDEX idx_tokens_map ON tokens(map_id);
CREATE INDEX idx_map_traps_map ON map_traps(map_id);
CREATE INDEX idx_fog_areas_map ON fog_areas(map_id);
CREATE INDEX idx_light_sources_map ON light_sources(map_id);

-- Module content
CREATE INDEX idx_module_monsters_module ON module_monsters(module_id);
CREATE INDEX idx_module_monsters_encounter ON module_monsters(encounter_tag);
CREATE INDEX idx_module_items_module ON module_items(module_id);
CREATE INDEX idx_module_npcs_module ON module_npcs(module_id);

-- Catalog
CREATE INDEX idx_catalog_sources_enabled ON catalog_sources(enabled);
CREATE INDEX idx_catalog_monsters_cr ON catalog_monsters(cr_numeric);
CREATE INDEX idx_catalog_monsters_type ON catalog_monsters(creature_type);
CREATE INDEX idx_catalog_monsters_source ON catalog_monsters(source);
CREATE INDEX idx_catalog_items_type ON catalog_items(item_type);
CREATE INDEX idx_catalog_items_rarity ON catalog_items(rarity);
CREATE INDEX idx_catalog_items_source ON catalog_items(source);
CREATE INDEX idx_catalog_spells_level ON catalog_spells(level);
CREATE INDEX idx_catalog_spells_school ON catalog_spells(school);
CREATE INDEX idx_catalog_spells_source ON catalog_spells(source);
CREATE INDEX idx_catalog_traps_source ON catalog_traps(source);
CREATE INDEX idx_catalog_classes_source ON catalog_classes(source);
CREATE INDEX idx_catalog_races_source ON catalog_races(source);
CREATE INDEX idx_catalog_feats_source ON catalog_feats(source);
```

## Key Differences from Current Schema

| Aspect | Current | v0.5 |
|--------|---------|------|
| Documents | file_path to filesystem | content column in DB |
| Characters | JSON blob + versions table | Normalized columns + typed JSON for classes/feats |
| Players | Separate entity | Dropped (player_name on character) |
| Sessions | First-class entity | Removed (just documents) |
| Workflow status | On campaigns/modules | Removed |
| Document requirements | Per-stage tracking | Removed |
| Multiclass | In character YAML blob | Typed JSON array (ClassLevel[]) |
| Feats/Features | In character YAML blob | Typed JSON array (Feature[]) |
| Maps | Simple image path | UVTT file + extracted data |
| Traps | In modules | On maps (map_traps table) |
| Sources | Implicit | Explicit catalog_sources with enable/disable |
| Catalog | Monsters, items, spells | + traps, classes, races, feats, backgrounds |

## Progress

*To be updated during implementation*