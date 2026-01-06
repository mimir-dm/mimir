# Class Schema Reference

Complete field reference for class entries in the extracted 5etools data format.

## File Location
- Path: `{book}/class/{source}.json`
- Features: `{book}/class/features-{source}.json`
- Subclass Features: `{book}/class/subclass-features-{source}.json`
- Fluff: `{book}/class/fluff-{source}.json`
- Subclass Fluff: `{book}/class/subclass-fluff-{source}.json`

## JSON Structure
```json
{
  "class": [
    {
      // class objects
    }
  ],
  "subclass": [
    {
      // subclass objects
    }
  ]
}
```

## Core Fields

### Required Fields
- `name` (string) - Class name (e.g., "Fighter", "Wizard")
- `source` (string) - Source book abbreviation (PHB, XGE, etc.)
- `hd` (object) - Hit dice specification
- `proficiency` (array) - Proficiency bonus by level
- `classFeatures` (array) - List of class features by level
- `startingProficiencies` (object) - Initial proficiencies

### Optional Core Fields
- `page` (number) - Page number in source book
- `srd` (boolean) - Available in System Reference Document
- `basicRules` (boolean) - Available in basic rules
- `hasFluff` (boolean) - Has associated fluff/lore text
- `hasFluffImages` (boolean) - Has associated images
- `subclassTitle` (string) - Name for subclasses (e.g., "Martial Archetype" for Fighter)

## Hit Dice (`hd`)
Object defining hit points:

```json
{
  "number": 1,     // Number of hit dice
  "faces": 10      // Die size (d6, d8, d10, d12)
}
```

## Spellcasting Fields

### Caster Types
- `casterProgression` (string) - Type of spellcaster:
  - `"full"` - Full caster (Wizard, Cleric, etc.)
  - `"1/2"` - Half caster (Paladin, Ranger)
  - `"1/3"` - Third caster (Eldritch Knight, Arcane Trickster)
  - `"pact"` - Pact Magic (Warlock)
  - `null` - Non-caster

### Spell Progression
- `cantripProgression` (array) - Number of cantrips known per level
- `spellsKnownProgression` (array) - Spells known per level (for known casters)
- `preparedSpells` (string) - Formula for prepared spells
- `spellcastingAbility` (string) - Ability score for spellcasting ("int", "wis", "cha")

## Class Features (`classFeatures`)
Array of feature references or objects:

```json
[
  "Fighting Style|Fighter||1",  // Simple reference
  {
    "classFeature": "Action Surge|Fighter||2",
    "tableDisplayName": "Action Surge (one use)"
  },
  {
    "classFeature": "Martial Archetype|Fighter||3",
    "gainSubclassFeature": true  // Indicates subclass choice
  }
]
```

Feature reference format: `"Feature Name|Class||Level|Source"`

## Starting Proficiencies (`startingProficiencies`)
Object defining initial proficiencies:

```json
{
  "armor": ["light", "medium", "shields"],
  "weapons": ["simple", "martial"],
  "tools": ["Thieves' tools"],
  "toolProficiencies": [
    {
      "choose": {
        "from": ["Gaming set", "Musical instrument"],
        "count": 1
      }
    }
  ],
  "skills": [
    {
      "choose": {
        "from": ["Acrobatics", "Athletics", "History", "Insight", "Religion", "Stealth"],
        "count": 2
      }
    }
  ]
}
```

## Multiclassing (`multiclassing`)
Requirements and benefits for multiclassing:

```json
{
  "requirements": {
    "str": 13,  // Minimum ability scores
    "or": {
      "dex": 13
    }
  },
  "proficienciesGained": {
    "armor": ["light", "medium", "shields"],
    "weapons": ["simple", "martial"]
  }
}
```

## Class Table Groups (`classTableGroups`)
Arrays defining class progression tables:

```json
[
  {
    "title": "Spell Slots per Spell Level",
    "colLabels": ["1st", "2nd", "3rd", "4th", "5th", "6th", "7th", "8th", "9th"],
    "rowsSpellProgression": [
      [2, 0, 0, 0, 0, 0, 0, 0, 0],  // Level 1
      [3, 0, 0, 0, 0, 0, 0, 0, 0],  // Level 2
      // ... etc
    ]
  }
]
```

## Starting Equipment (`startingEquipment`)
Default equipment and choices:

```json
{
  "additionalFromBackground": true,
  "default": [
    "(a) {@item leather armor|phb} or (b) {@item scale mail|phb}",
    "(a) two {@item shortsword|phb|shortswords} or (b) two {@filter simple melee weapons|items|source=phb|category=basic|type=simple weapon;melee weapon=}",
    "(a) a {@item dungeoneer's pack|phb} or (b) an {@item explorer's pack|phb}",
    "A {@item longbow|phb} and a {@item quiver|phb} of 20 {@item arrow|phb|arrows}"
  ],
  "goldAlternative": "{@dice 5d4 × 10} gp"
}
```

## Subclass Structure

### Core Subclass Fields
- `name` (string) - Subclass name
- `shortName` (string) - Abbreviated name
- `source` (string) - Source book
- `className` (string) - Parent class name
- `classSource` (string) - Parent class source
- `page` (number) - Page reference
- `subclassFeatures` (array) - Feature progression

### Subclass Features
Array of feature grants by level:

```json
[
  "Champion|Fighter|Champion|PHB|3",
  "Remarkable Athlete|Fighter|Champion|PHB|7",
  "Additional Fighting Style|Fighter|Champion|PHB|10",
  "Superior Critical|Fighter|Champion|PHB|15",
  "Survivor|Fighter|Champion|PHB|18"
]
```

## Optional Feature Progression
Some classes have optional features (Fighting Styles, Eldritch Invocations, etc.):

```json
{
  "optionalfeatureProgression": [
    {
      "name": "Fighting Style",
      "featureType": ["FS:F"],
      "progression": {
        "1": 1  // Level: count
      }
    }
  ]
}
```

## Associated Files

### Class Features File
Contains detailed feature descriptions:

```json
{
  "classFeature": [
    {
      "name": "Second Wind",
      "source": "PHB",
      "className": "Fighter",
      "classSource": "PHB",
      "level": 1,
      "entries": [
        "You have a limited well of stamina..."
      ]
    }
  ]
}
```

### Fluff Files and Linkage

The fluff system separates flavor text from mechanical data. Classes link to fluff through name/source matching:

#### Class Fluff File (`fluff-{source}.json`)
Contains lore, descriptions, and flavor text:

```json
{
  "classFluff": [
    {
      "name": "Fighter",        // Must match class name
      "source": "PHB",          // Must match class source
      "entries": [              // Flavor text entries
        {
          "type": "quote",
          "entries": ["A master of martial combat..."],
          "by": "Tordek, dwarf fighter"
        },
        "Fighters learn the basics of all combat styles...",
        {
          "type": "entries",
          "name": "Creating a Fighter",
          "entries": [...]
        }
      ],
      "images": [               // Associated artwork
        {
          "type": "image",
          "href": {
            "type": "internal",
            "path": "classes/PHB/Fighter.webp"
          }
        }
      ]
    }
  ]
}
```

#### Subclass Fluff File (`subclass-fluff-{source}.json`)
Contains subclass-specific lore:

```json
{
  "subclassFluff": [
    {
      "name": "Champion",         // Subclass name
      "shortName": "Champion",    
      "source": "PHB",
      "className": "Fighter",     // Parent class
      "classSource": "PHB",       // Parent class source
      "entries": [
        "The archetypal Champion focuses on the development of raw physical power..."
      ],
      "images": [...]
    }
  ]
}
```

#### Fluff Linkage System

1. **Automatic Linking**: Classes with `"hasFluff": true` automatically link to fluff entries with matching:
   - `name` field (exact match)
   - `source` field (exact match)

2. **Image References**: The `"hasFluffImages": true` flag indicates associated artwork exists in the fluff file

3. **Subclass Linkage**: Subclasses link through:
   - `name` + `shortName` (subclass identifiers)
   - `className` + `classSource` (parent class reference)
   - `source` (subclass source book)

4. **File Organization**:
   - Main class fluff: `{book}/class/fluff-{source}.json`
   - Subclass fluff: `{book}/class/subclass-fluff-{source}.json`
   - Images: `{book}/img/classes/{source}/{name}.webp`

## Example Class Entry

```json
{
  "name": "Fighter",
  "source": "PHB",
  "page": 70,
  "srd": true,
  "basicRules": true,
  "hd": {
    "number": 1,
    "faces": 10
  },
  "proficiency": [2, 2, 2, 2, 3, 3, 3, 3, 4, 4, 4, 4, 5, 5, 5, 5, 6, 6, 6, 6],
  "startingProficiencies": {
    "armor": ["light", "medium", "heavy", "shields"],
    "weapons": ["simple", "martial"],
    "skills": [
      {
        "choose": {
          "from": ["Acrobatics", "Animal Handling", "Athletics", "History", "Insight", "Intimidation", "Perception", "Survival"],
          "count": 2
        }
      }
    ]
  },
  "multiclassing": {
    "requirements": {
      "or": [
        {"str": 13},
        {"dex": 13}
      ]
    },
    "proficienciesGained": {
      "armor": ["light", "medium", "shields"],
      "weapons": ["simple", "martial"]
    }
  },
  "classFeatures": [
    "Fighting Style|Fighter||1",
    "Second Wind|Fighter||1",
    "Action Surge|Fighter||2",
    {
      "classFeature": "Martial Archetype|Fighter||3",
      "gainSubclassFeature": true
    }
  ],
  "subclassTitle": "Martial Archetype"
}
```