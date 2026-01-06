# Archive Structure

This document defines the complete directory and file structure for Mimir source data archives. Each archive represents a single D&D sourcebook with all its content organized by type.

> The structure of our archives are *HEAVILY* influenced by 5e tools, we've simply attempted to split them up by book in an effort to ensure that individuals can load data they're arguably entitled to; because they have a physical copy of a book. 

## Archive Root Structure

```
{book-abbreviation}/
├── metadata.json                 # Book metadata and configuration
├── book/                        # Book structure and content
├── adventure/                   # Adventure-specific content
├── spells/                      # Spell definitions
├── items/                       # Equipment and magic items
├── bestiary/                    # Creatures and monsters
├── class/                       # Character classes and features
├── races/                       # Player races
├── backgrounds/                 # Character backgrounds
├── feats/                       # Character feats
├── optionalfeatures/            # Optional class features
├── variantrules/                # Optional and variant rules
├── rewards/                     # Rewards and treasures
├── objects/                     # Environmental objects
├── traps/                       # Traps
├── hazards/                     # Environmental hazards
├── vehicles/                    # Vehicles and mounts
├── actions/                     # Game actions
├── languages/                   # Languages
├── conditions/                  # Status conditions
├── diseases/                    # Diseases and afflictions
├── tables/                      # Reference tables
├── deities/                     # Deities and pantheons
├── cults/                       # Cults and organizations
├── boons/                       # Supernatural boons
└── img/                         # Images and artwork
```

## Directory Details

### `/metadata.json` (Required)
Book identification and configuration
```json
{
  "source": "PHB",
  "name": "Player's Handbook",
  "abbreviation": "PHB",
  "publishedDate": "2014-08-19",
  "version": "1.0.0"
}
```

### `/book/` (Required)
Contains the book's table of contents and structural information
- `book-{source}.json` - Main book structure with chapters and sections

### `/spells/`
Spell definitions and descriptions
- `spells-{source}.json` - Spell mechanics and rules
- `fluff-spells-{source}.json` - Spell lore and descriptions

### `/items/`
All equipment including mundane and magical items
- `items-{source}.json` - Item statistics and properties
- `fluff-items-{source}.json` - Item descriptions and history

### `/bestiary/`
Monsters, NPCs, and creatures
- `bestiary-{source}.json` - Creature statistics and abilities
- `fluff-bestiary-{source}.json` - Creature lore and ecology
- `legendaryGroup-{source}.json` - Lair actions and regional effects

### `/class/`
Character classes and subclasses
- `{source}.json` - Class mechanics and progression
- `features-{source}.json` - Class feature descriptions
- `subclass-features-{source}.json` - Subclass feature descriptions
- `fluff-{source}.json` - Class lore and descriptions
- `subclass-fluff-{source}.json` - Subclass lore

### `/races/`
Player character races and subraces
- `races-{source}.json` - Racial traits and abilities
- `fluff-races-{source}.json` - Racial history and culture

### `/backgrounds/`
Character background options
- `backgrounds-{source}.json` - Background features and proficiencies
- `fluff-backgrounds-{source}.json` - Background stories

### `/feats/`
Character feat options
- `feats-{source}.json` - Feat mechanics and prerequisites
- `fluff-feats-{source}.json` - Feat descriptions

### `/variantrules/`
Optional and variant game rules
- `variantrules-{source}.json` - Rule modifications and options

### `/rewards/`
Non-item rewards and boons
- `rewards-{source}.json` - Supernatural gifts, blessings, charms

### `/objects/`
Environmental and interactive objects
- `objects-{source}.json` - Object properties and interactions

### `/traps/`
Traps and environmental hazards
- `traps-{source}.json` - Trap mechanics and effects

### `/vehicles/`
Vehicles, mounts, and ships
- `vehicles-{source}.json` - Vehicle statistics
- `fluff-vehicles-{source}.json` - Vehicle descriptions

### `/actions/`
Standard and special actions
- `actions-{source}.json` - Action definitions and rules

### `/languages/`
Language definitions
- `languages-{source}.json` - Language properties and scripts

### `/conditions/`
Status conditions and effects
- `conditions-{source}.json` - Condition rules and effects

### `/diseases/`
Diseases and afflictions
- `diseases-{source}.json` - Disease mechanics and cures

### `/tables/`
Reference and random tables
- `tables-{source}.json` - Lookup and generation tables

### `/deities/`
Gods and pantheons
- `deities-{source}.json` - Deity information and domains

### `/hazards/`
Environmental hazards separate from traps
- `{source}.json` - Hazard mechanics and effects

### `/adventure/`
Adventure-specific content and narrative
- `adventure-{source}.json` - Adventure content and structure

### `/optionalfeatures/`
Optional class features and alternatives
- `{source}.json` - Optional feature mechanics

### `/cults/`
Cults and organizations
- `{source}.json` - Cult information and mechanics

### `/boons/`
Supernatural boons and epic boons
- `{source}.json` - Boon mechanics and descriptions

### `/img/`
Images and artwork organized by type
```
img/
├── covers/                # Book cover art
│   └── {source}.webp
├── book/                  # Interior book art
│   └── {source}/
│       ├── chapter-1.webp
│       └── diagram-1.webp
├── bestiary/              # Creature artwork
│   └── {source}/
│       └── {creature-name}.webp
├── items/                 # Item illustrations
│   └── {source}/
│       └── {item-name}.webp
└── symbols/               # Symbols and icons
    └── {source}/
        └── {symbol-name}.webp
```

## File Naming Conventions

### JSON Files
- **Primary content**: `{type}-{source}.json` or `{source}.json`
- **Fluff content**: `fluff-{source}.json` or `fluff-{type}-{source}.json`
- **Book content**: `book-{source}.json`
- **Adventure content**: `adventure-{source}.json`

Note: Some directories like `class/`, `items/`, `races/` use `{source}.json` for primary content, while others like `spells/`, `bestiary/` use `{type}-{source}.json`

### Source Codes
Standard abbreviations for source books:
- `phb` - Player's Handbook
- `mm` - Monster Manual
- `dmg` - Dungeon Master's Guide
- `xge` - Xanathar's Guide to Everything
- `tce` - Tasha's Cauldron of Everything
- `vgm` - Volo's Guide to Monsters
- `mtf` - Mordenkainen's Tome of Foes

### Image Files
- Format: WebP (`.webp`) for optimal compression
- Naming: Lowercase, hyphenated names matching content
- Resolution: Variable based on use case

## Content Presence

Not all directories are present in every archive. Content depends on the book type:

### Core Rulebooks (PHB)
- ✅ Classes, Races, Backgrounds, Feats
- ✅ Spells, Equipment
- ✅ Conditions, Rules
- ❌ Bestiary (limited)

### Monster Books (MM, VGM)
- ✅ Bestiary
- ✅ Legendary Groups
- ❌ Classes, Spells (limited)

### Adventure Books
- ✅ Bestiary (new creatures)
- ✅ Items (adventure-specific)
- ✅ Spells (if new)
- ✅ Maps and locations

### Setting Books
- ✅ Races, Backgrounds
- ✅ Deities
- ✅ Setting-specific content

## Required vs Optional

### Always Required
- `metadata.json` - Book identification
- `book/` directory - Book structure
- At least one content directory

### Content-Specific Requirements
- If spells exist → `spells/` directory required
- If creatures exist → `bestiary/` directory required
- If items exist → `items/` directory required

### Always Optional
- Fluff files (enhance but not required)
- Image directories
- Specialized content (psionics, recipes, etc.)

## Validation Rules

1. **Structure Validation**
   - Archive must be valid tar.gz
   - Root directory must match book abbreviation
   - metadata.json must be present and valid

2. **Content Validation**
   - All JSON files must be valid JSON
   - Required fields must be present
   - Cross-references must use correct format

3. **Naming Validation**
   - Files must follow naming conventions
   - Source codes must be consistent
   - No spaces in filenames

## Example: PHB Archive Structure

```
phb/
├── metadata.json
├── book/
│   └── book-phb.json
├── class/
│   ├── phb.json
│   ├── features-phb.json
│   ├── subclass-features-phb.json
│   ├── fluff-phb.json
│   └── subclass-fluff-phb.json
├── races/
│   ├── phb.json
│   └── fluff-phb.json
├── backgrounds/
│   ├── phb.json
│   └── fluff-phb.json
├── feats/
│   └── phb.json
├── spells/
│   ├── spells-phb.json
│   └── fluff-spells-phb.json
├── items/
│   ├── phb.json
│   └── fluff-phb.json
├── conditions/
│   └── phb.json
├── variantrules/
│   └── phb.json
└── img/
    └── covers/
        └── phb.webp
```

This structure ensures Mimir can reliably locate and render all content types while maintaining flexibility for different book types and optional content.