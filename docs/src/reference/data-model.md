# Data Model

The hierarchy of entities in Mimir and how they relate to each other.

## Entity Hierarchy

```
Campaign
├── Documents (campaign-level)
├── Maps (campaign-level)
├── Characters (PCs and NPCs)
│   ├── Classes (multiclass support)
│   ├── Inventory
│   ├── Spells
│   ├── Proficiencies
│   ├── Feats
│   └── Features
├── Modules
│   ├── Documents (module-level)
│   ├── Maps (module-level)
│   ├── Module Monsters (catalog references)
│   ├── Module NPCs
│   └── Token Placements
├── Homebrew Items
├── Homebrew Monsters
├── Homebrew Spells
└── Campaign Sources (enabled books)
```

## Campaigns

The top-level container. A campaign represents a complete D&D game — the world, the story, the characters.

| Field | Description |
|-------|-------------|
| `id` | UUID, auto-generated |
| `name` | Campaign name (required) |
| `description` | Optional notes |
| `archived_at` | Soft-delete timestamp (null = active) |
| `created_at` | Creation timestamp |
| `updated_at` | Last modification timestamp |

Archiving a campaign hides it from the default list but preserves all data. Deleting a campaign is permanent and cascades to all children.

When you create a campaign, Mimir automatically generates 11 template documents (Campaign Pitch, Starting Scenario, World Primer, Character Guidelines, Table Expectations, Character Integration, Campaign Bible, Safety Tools, House Rules, Player Secrets, Faction Overview).

## Characters

Both player characters (PCs) and non-player characters (NPCs) use the same data model. The `is_npc` flag distinguishes them.

| Field | Description |
|-------|-------------|
| `id` | UUID |
| `campaign_id` | Parent campaign |
| `name` | Character name (required) |
| `is_npc` | Integer: 0 = PC, 1 = NPC |
| `player_name` | Real player name (PCs only) |
| `race_name` | Race name (references catalog) |
| `race_source` | Source book for the race |
| `background_name` | Background name (references catalog) |
| `background_source` | Source book for the background |
| `strength` | Strength score (default 10) |
| `dexterity` | Dexterity score (default 10) |
| `constitution` | Constitution score (default 10) |
| `intelligence` | Intelligence score (default 10) |
| `wisdom` | Wisdom score (default 10) |
| `charisma` | Charisma score (default 10) |
| `cp`, `sp`, `ep`, `gp`, `pp` | Currency in 5 denominations |
| `traits` | Personality traits |
| `ideals` | Character ideals |
| `bonds` | Character bonds |
| `flaws` | Character flaws |
| `role` | NPC role (NPC only) |
| `location` | NPC location (NPC only) |
| `faction` | NPC faction (NPC only) |
| `created_at`, `updated_at` | Timestamps |

### Character Classes

Characters support multiclassing. Each class entry tracks:

| Field | Description |
|-------|-------------|
| `character_id` | Parent character |
| `class_name` | Class name (e.g., "Fighter") |
| `class_source` | Source book for the class |
| `level` | Levels in this class |
| `subclass_name` | Subclass (e.g., "Champion") |
| `subclass_source` | Source book for the subclass |
| `starting_class` | Whether this is the character's first class (for multiclass proficiency rules) |

A level 5 Fighter / level 3 Wizard has two entries.

### Character Inventory

| Field | Description |
|-------|-------------|
| `item_name` | Item name (may reference catalog) |
| `item_source` | Source book for the item |
| `quantity` | Number of items |
| `equipped` | Whether the item is worn/held (integer: 0/1) |
| `attuned` | Whether the item is attuned (integer: 0/1, max 3 per character) |
| `notes` | Optional notes about the item |

### Character Spells

| Field | Description |
|-------|-------------|
| `spell_name` | Spell name (references catalog) |
| `spell_source` | Source book for the spell |
| `prepared` | Whether currently prepared (integer: 0/1) |
| `source_class` | Which class grants this spell |

### Character Proficiencies

| Field | Description |
|-------|-------------|
| `proficiency_type` | skill, tool, language, or saving_throw |
| `name` | Proficiency name (e.g., "Perception") |
| `expertise` | Whether the character has expertise (integer: 0/1) |

### Character Feats

| Field | Description |
|-------|-------------|
| `feat_name` | Feat name |
| `feat_source` | Source book for the feat |
| `source_type` | How the feat was acquired (e.g., class feature, ASI) |

### Character Features

| Field | Description |
|-------|-------------|
| `feature_type` | Type of feature (class, subclass, racial, etc.) |
| `feature_name` | Feature name |
| `feature_source` | Source book |
| `source_class` | Which class grants the feature |

### Character Sources

Optional per-character source filtering, separate from campaign sources.

| Field | Description |
|-------|-------------|
| `character_id` | Parent character |
| `source_code` | Enabled source book code |

## Modules

Modules are adventure containers within a campaign — a dungeon, a town, a story arc.

| Field | Description |
|-------|-------------|
| `id` | UUID |
| `campaign_id` | Parent campaign |
| `name` | Module name (required) |
| `description` | Optional description |
| `module_number` | Display order |
| `created_at`, `updated_at` | Timestamps |

### Module Monsters

References to catalog monsters (or homebrew monsters) used in this module's encounters.

| Field | Description |
|-------|-------------|
| `module_id` | Parent module |
| `monster_name` | Catalog monster name |
| `monster_source` | Source book for the monster |
| `homebrew_monster_id` | Alternative: reference to a homebrew monster |
| `display_name` | Optional custom name (e.g., "Goblin Boss Grix") |
| `quantity` | Number of creatures |
| `notes` | DM notes about this monster's role |

### Module NPCs

NPCs created specifically for this module (distinct from campaign-level characters).

| Field | Description |
|-------|-------------|
| `name` | NPC name |
| `role` | Role in the module |
| `description` | Background and behavior |
| `appearance` | Physical description |
| `personality` | Personality traits |
| `motivation` | What drives this NPC |
| `secrets` | Hidden information about the NPC |
| `stat_block` | Optional JSON stat block |
| `token_asset_id` | Optional custom token art |

## Documents

Flexible text content at either the campaign or module level.

| Field | Description |
|-------|-------------|
| `id` | UUID |
| `campaign_id` | Parent campaign |
| `module_id` | Parent module (null for campaign-level docs) |
| `title` | Document title |
| `doc_type` | Type identifier (e.g., `campaign_pitch`, `world_primer`, `character_guidelines`, `user_document`) |
| `content` | Full text content |
| `sort_order` | Display ordering |
| `created_at`, `updated_at` | Timestamps |

A document belongs to a campaign, and optionally to a specific module within that campaign.

## Maps

Battle maps with optional grid, wall, and lighting data.

| Field | Description |
|-------|-------------|
| `id` | UUID |
| `campaign_id` | Parent campaign |
| `module_id` | Optional parent module |
| `name` | Map name |
| `description` | Optional description |
| `sort_order` | Display ordering |
| `uvtt_asset_id` | Reference to the uploaded map asset |
| `lighting_mode` | bright, dim, or dark |
| `fog_enabled` | Whether fog of war is active (integer: 0/1) |
| `created_at`, `updated_at` | Timestamps |

### Map Overlays

Maps support several overlay types:

**Light Sources** (`light_sources` table) — Dynamic lights with grid position, bright/dim radius, color, and active state.

**Fog Revealed Areas** (`fog_revealed_areas` table) — Rectangles tracking which map areas have been revealed to players (x, y, width, height).

**Points of Interest** (`map_pois` table) — Named markers with descriptions, icons, colors, and visibility at specific grid positions.

**Traps** (`map_traps` table) — Trap markers with name, description, trigger/effect descriptions, detection DC, triggered state, and visibility.

**Token Placements** (`token_placements` table) — Monster and NPC positions on the map.

| Field | Description |
|-------|-------------|
| `map_id` | Parent map |
| `module_monster_id` | Reference to module monster (or null) |
| `module_npc_id` | Reference to module NPC (or null) |
| `grid_x`, `grid_y` | Grid position |
| `label` | Optional display label |
| `faction_color` | Token border color |
| `hidden` | Whether hidden from players (integer: 0/1) |
| `vision_bright_ft` | Bright vision range in feet |
| `vision_dim_ft` | Dim vision range in feet |
| `vision_dark_ft` | Darkvision range in feet |
| `light_radius_ft` | Light emitted by this token |
| `created_at` | Placement timestamp |

## Homebrew Content

Custom content scoped to a campaign. Three types:

### Homebrew Items

| Field | Description |
|-------|-------------|
| `name` | Item name |
| `item_type` | weapon, armor, potion, etc. |
| `rarity` | common through legendary |
| `data` | Full item data as JSON |
| `cloned_from_name` | Original catalog item name (if cloned) |
| `cloned_from_source` | Original catalog item source (if cloned) |

### Homebrew Monsters

| Field | Description |
|-------|-------------|
| `name` | Monster name |
| `cr` | Challenge rating |
| `creature_type` | beast, fiend, undead, etc. |
| `size` | Tiny through Gargantuan |
| `data` | Full stat block as JSON |
| `cloned_from_name` | Original catalog monster name (if cloned) |
| `cloned_from_source` | Original catalog monster source (if cloned) |

### Homebrew Spells

| Field | Description |
|-------|-------------|
| `name` | Spell name |
| `level` | Spell level (0 for cantrips) |
| `school` | Spell school |
| `data` | Full spell data as JSON |
| `cloned_from_name` | Original catalog spell name (if cloned) |
| `cloned_from_source` | Original catalog spell source (if cloned) |

## Campaign Sources

Controls which D&D source books are active for a campaign.

| Field | Description |
|-------|-------------|
| `campaign_id` | Campaign this applies to |
| `source_code` | Source book abbreviation (e.g., "PHB", "MM", "XGE") |

Catalog searches filter results by the campaign's enabled sources.

## Campaign Assets

Uploaded files (map images, token art) stored as binary blobs.

| Field | Description |
|-------|-------------|
| `id` | UUID |
| `campaign_id` | Parent campaign (optional) |
| `module_id` | Parent module (optional) |
| `filename` | Original filename |
| `description` | Optional description |
| `mime_type` | File MIME type |
| `blob_path` | Storage path relative to assets directory |
| `file_size` | File size in bytes |
| `uploaded_at` | Upload timestamp |

Assets are stored in the `assets/` directory within the application data folder.

## Catalog Data

Read-only D&D 5e reference data imported from external sources. Key tables:

| Table | Contents |
|-------|----------|
| `monsters` | Creature stat blocks with CR, type, size, token image path, full JSON |
| `items` | Equipment and magic items with type, rarity |
| `spells` | Spells with level, school, ritual/concentration flags |
| `races` | Playable races with traits |
| `classes` | Classes with feature data |
| `subclasses` | Subclasses linked to parent classes |
| `class_features` | Class features by level |
| `subclass_features` | Subclass features by level |
| `backgrounds` | Backgrounds with proficiencies |
| `feats` | Feats with prerequisites |
| `conditions` | Status conditions (blinded, charmed, etc.) |
| `languages` | Languages with type classification |
| `deities` | Deities with pantheon grouping |
| `diseases` | Diseases and afflictions |
| `hazards` | Environmental hazards |
| `traps` | Trap definitions with tier |
| `senses` | Sense types |
| `skills` | Skills with associated ability |
| `objects` | Interactive objects |
| `vehicles` | Vehicles with type |
| `rewards` | Rewards with type |
| `psionics` | Psionic abilities |
| `optional_features` | Optional class features (invocations, maneuvers, etc.) |
| `variant_rules` | Variant and optional rules |
| `actions` | Standard actions |
| `cults` | Cults and supernatural boons |
| `books` | Source book metadata with cover images |
| `catalog_sources` | Source book registry with enabled/disabled state |
| `spell_classes` | Join table: which classes have access to which spells |
| `spell_subclasses` | Join table: subclass spell lists |
| `item_attunement_classes` | Join table: class-restricted attunement |
| `catalog_tables` | Reference tables from source books |

A full-text search index (`catalog_fts`, an FTS5 virtual table) provides fast search across all catalog entity types.

See [The Catalog System](../explanation/catalog-system.md) for how this data is used.

## See Also

- [Campaigns vs Modules](../explanation/campaign-vs-module.md) — Why the hierarchy exists
- [The Homebrew System](../explanation/homebrew-system.md) — Creating custom content
- [Architecture Overview](../explanation/architecture-overview.md) — How data flows through the system
