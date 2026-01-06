# Class Schema Documentation

This document defines the JSON schema for character classes and subclasses in the D&D 5etools format.

## Overview

The class schema represents all character classes and their subclasses, including progression, features, and abilities. Class data is split across multiple files to separate mechanics from descriptions and lore.

## File Structure

Class content is organized into five distinct files within the `class/` directory:

```
class/
├── {source}.json                    # Main class and subclass definitions
├── features-{source}.json           # Class feature descriptions
├── subclass-features-{source}.json  # Subclass feature descriptions
├── fluff-{source}.json              # Class lore and flavor text
└── subclass-fluff-{source}.json     # Subclass lore and flavor text
```

## Main Class File (`{source}.json`)

Contains the mechanical definitions for classes and subclasses.

### Root Structure

```json
{
  "class": [
    // Class objects
  ],
  "subclass": [
    // Subclass objects
  ]
}
```

## Class Object

Every class contains these core fields:

```json
{
  "name": "string",              // Class name (e.g., "Cleric")
  "source": "string",            // Source book abbreviation
  "page": "number",              // Page number in source book
  "hd": {                        // Hit dice
    "number": 1,
    "faces": 8                   // d8 hit die
  },
  "proficiency": ["str", "wis"], // Saving throw proficiencies
  "classFeatures": [             // Class progression
    // Feature references or objects
  ],
  "startingProficiencies": {     // Starting proficiencies
    // Proficiency details
  },
  "startingEquipment": {         // Starting equipment options
    // Equipment details
  },
  "multiclassing": {             // Multiclassing requirements
    // Requirements and proficiencies
  }
}
```

### Hit Dice

```json
"hd": {
  "number": 1,      // Number of hit dice (always 1 for single class)
  "faces": 6        // Die size (6 for d6, 8 for d8, etc.)
}
```

### Proficiencies

```json
"proficiency": ["con", "cha"]  // Saving throw proficiencies
```

Ability abbreviations: `str`, `dex`, `con`, `int`, `wis`, `cha`

### Starting Proficiencies

```json
"startingProficiencies": {
  "armor": ["light", "medium", "shield"],
  "weapons": ["simple", "martial"],
  "tools": ["Herbalism kit"],
  "toolProficiencies": [{
    "anyArtisansTool": 1
  }],
  "skills": [{
    "choose": {
      "count": 2,
      "from": ["Arcana", "History", "Insight", "Medicine", "Religion"]
    }
  }],
  "languages": [{
    "anyStandard": 1
  }]
}
```

### Class Features Array

The `classFeatures` array defines the progression of features by level:

```json
"classFeatures": [
  "Spellcasting|Cleric||1",           // Simple string reference
  {
    "classFeature": "Divine Domain|Cleric||1",
    "gainSubclassFeature": true       // Marks subclass feature level
  },
  {
    "classFeature": "Channel Divinity|Cleric||2",
    "tableDisplayName": "Channel Divinity (1/rest)"  // Override table display
  },
  "Ability Score Improvement|Cleric||4",
  "Destroy Undead (CR 1/2)|Cleric||5"
]
```

### Feature Reference Format

Features use a pipe-delimited string format:
```
"Feature Name|Class Name|Source|Subclass|Level|Additional Source"
```

Examples:
- `"Spellcasting|Cleric||1"` - Basic feature
- `"Divine Domain|Cleric||1"` - Subclass choice feature
- `"Channel Divinity: Harness Divine Power|Cleric||2|TCE"` - Optional feature from another source

### Spellcasting Classes

Classes with spellcasting have additional fields:

```json
{
  "casterProgression": "full",        // "full", "1/2", "1/3", "pact"
  "cantripProgression": [3, 3, 3, 4, 4, ...],  // Cantrips known by level
  "spellsKnownProgression": [2, 3, 4, ...],    // Spells known (for known casters)
  "spellcastingAbility": "wis",                // Spellcasting ability
  "preparedSpells": "<$level$> + <$wis_mod$>"  // Prepared spell formula
}
```

### Class Table Data

Optional table data for class progression display:

```json
"classTableGroups": [
  {
    "colLabels": ["{@filter 1st|spells|level=1|class=Cleric}"],
    "rows": [
      [2],  // Level 1
      [3],  // Level 2
      // ...
    ]
  }
]
```

### Multiclassing

```json
"multiclassing": {
  "requirements": {
    "wis": 13
  },
  "proficienciesGained": {
    "armor": ["light", "medium", "shield"],
    "weapons": ["simple"]
  }
}
```

## Subclass Object

Subclasses are defined separately and linked to their parent class:

```json
{
  "name": "Knowledge Domain",
  "shortName": "Knowledge",
  "source": "PHB",
  "page": 59,
  "className": "Cleric",
  "classSource": "PHB",
  "subclassFeatures": [
    "Knowledge Domain|Cleric||Knowledge||1",
    "Channel Divinity: Knowledge of the Ages|Cleric||Knowledge||2",
    "Channel Divinity: Read Thoughts|Cleric||Knowledge||6",
    "Potent Spellcasting|Cleric||Knowledge||8",
    "Visions of the Past|Cleric||Knowledge||17"
  ],
  "subclassSpells": [
    "command",
    "identify",
    // ...
  ]
}
```

### Subclass Features Array

Similar to class features but includes subclass identifier:
```
"Feature Name|Class|Class Source|Subclass|Subclass Source|Level"
```

## Class Features File (`features-{source}.json`)

Contains detailed descriptions of class features.

### Root Structure

```json
{
  "classFeature": [
    // Feature description objects
  ]
}
```

### Class Feature Object

```json
{
  "name": "Divine Domain",
  "source": "PHB",
  "page": 56,
  "className": "Cleric",
  "classSource": "PHB",
  "level": 1,
  "entries": [
    "Choose one domain related to your deity...",
    {
      "type": "entries",
      "name": "Domain Spells",
      "entries": [
        "Each domain has a list of spells..."
      ]
    }
  ],
  "srd": true,
  "basicRules": true
}
```

## Subclass Features File (`subclass-features-{source}.json`)

Contains detailed descriptions of subclass-specific features.

### Root Structure

```json
{
  "subclassFeature": [
    // Subclass feature objects
  ]
}
```

### Subclass Feature Object

```json
{
  "name": "Knowledge Domain",
  "source": "PHB",
  "page": 59,
  "className": "Cleric",
  "classSource": "PHB",
  "subclassShortName": "Knowledge",
  "subclassSource": "PHB",
  "level": 1,
  "entries": [
    "The gods of knowledge value learning...",
    {
      "type": "table",
      "caption": "Knowledge Domain Spells",
      "colLabels": ["Cleric Level", "Spells"],
      "rows": [
        ["1st", "{@spell command}, {@spell identify}"],
        ["3rd", "{@spell augury}, {@spell suggestion}"]
      ]
    },
    {
      "type": "refSubclassFeature",
      "subclassFeature": "Blessings of Knowledge|Cleric||Knowledge||1"
    }
  ]
}
```

## Class Fluff File (`fluff-{source}.json`)

Contains narrative descriptions and lore for classes.

### Root Structure

```json
{
  "classFluff": [
    // Class fluff objects
  ]
}
```

### Class Fluff Object

```json
{
  "name": "Paladin",
  "source": "PHB",
  "page": 82,
  "entries": [
    {
      "type": "section",
      "name": "Paladin",
      "entries": [
        "Clad in plate armor that gleams...",
        {
          "type": "entries",
          "name": "The Cause of Righteousness",
          "entries": [
            "A paladin swears to uphold justice..."
          ]
        },
        {
          "type": "entries",
          "name": "Creating a Paladin",
          "entries": [
            "The most important aspect...",
            {
              "type": "entries",
              "name": "Quick Build",
              "entries": [
                "You can make a paladin quickly..."
              ]
            }
          ]
        }
      ]
    }
  ]
}
```

## Subclass Fluff File (`subclass-fluff-{source}.json`)

Contains narrative descriptions for subclasses.

### Root Structure

```json
{
  "subclassFluff": [
    // Subclass fluff objects
  ]
}
```

### Subclass Fluff Object

```json
{
  "name": "Path of the Berserker",
  "source": "PHB",
  "className": "Barbarian",
  "classSource": "PHB",
  "shortName": "Berserker",
  "entries": [
    {
      "type": "section",
      "entries": [
        "For some barbarians, rage is a means to an end..."
      ]
    }
  ]
}
```

## Special Fields

### Optional Class Features (TCE)

Features from optional sources include their source:
```json
"Cantrip Versatility|Cleric||4|TCE"
```

### Sidekick Classes

Simplified classes for sidekicks have `isSidekick: true`

### Prestige Classes

Prestige classes may have additional requirements:
```json
"requirements": {
  "level": 5,
  "feat": ["Spellcasting"],
  "special": "Must have made friendly contact with a dragon"
}
```

## Entry Types

Class and feature descriptions use various entry types:

### Tables

```json
{
  "type": "table",
  "caption": "Domain Spells",
  "colLabels": ["Level", "Spells"],
  "colStyles": ["col-2 text-center", "col-10"],
  "rows": [
    ["1st", "{@spell bless}, {@spell cure wounds}"]
  ]
}
```

### Lists

```json
{
  "type": "list",
  "items": [
    "Proficiency with Wisdom saving throws",
    "Proficiency with Charisma saving throws"
  ]
}
```

### Options

```json
{
  "type": "options",
  "entries": [
    {
      "type": "entries",
      "name": "Option 1",
      "entries": ["Description..."]
    }
  ]
}
```

### References

```json
{
  "type": "refSubclassFeature",
  "subclassFeature": "Feature Name|Class||Subclass||Level"
}
```

## Formatting Tags

Text entries support D&D formatting tags:

- `{@spell fireball}` - Spell references
- `{@condition blinded}` - Condition references
- `{@dice 1d6}` - Dice expressions
- `{@damage 2d8}` - Damage expressions
- `{@filter cleric spells|spells|class=cleric}` - Filtered links
- `{@book chapter 10|PHB|10}` - Book references
- `{@deity Oghma}` - Deity references
- `{@skill Perception}` - Skill references

## Complete Example: Cleric Class Structure

### Main Class Definition (`phb.json`)
```json
{
  "class": [{
    "name": "Cleric",
    "source": "PHB",
    "page": 56,
    "hd": {"number": 1, "faces": 8},
    "proficiency": ["wis", "cha"],
    "casterProgression": "full",
    "cantripProgression": [3, 3, 3, 4, 4, ...],
    "classFeatures": [
      "Spellcasting|Cleric||1",
      {
        "classFeature": "Divine Domain|Cleric||1",
        "gainSubclassFeature": true
      }
    ]
  }]
}
```

### Feature Description (`features-phb.json`)
```json
{
  "classFeature": [{
    "name": "Divine Domain",
    "className": "Cleric",
    "level": 1,
    "entries": [
      "Choose one domain related to your deity..."
    ]
  }]
}
```

### Class Lore (`fluff-phb.json`)
```json
{
  "classFluff": [{
    "name": "Cleric",
    "entries": [
      {
        "type": "section",
        "entries": [
          "Arms and eyes upraised toward the sun..."
        ]
      }
    ]
  }]
}
```

## Notes

- Feature references must match exactly between files
- The separation allows for modular loading and cross-referencing
- Optional features from other sources maintain compatibility
- All text supports formatting tags for rich display
- Arrays are used for all content types, never direct objects