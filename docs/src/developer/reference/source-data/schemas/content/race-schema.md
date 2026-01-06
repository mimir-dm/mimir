# Race Schema Reference

Complete field reference for race and subrace entries in the extracted 5etools data format.

## File Location
- Path: `{book}/races/{source}.json`
- Fluff: `{book}/races/fluff-{source}.json`
- Images: `{book}/img/races/{source}/`

## JSON Structure
```json
{
  "race": [
    {
      // race objects
    }
  ],
  "subrace": [
    {
      // subrace objects
    }
  ]
}
```

## Core Fields

### Required Fields
- `name` (string) - Race name (e.g., "Dwarf", "Elf", "Human")
- `source` (string) - Source book abbreviation (PHB, VGM, etc.)
- `size` (array) - Size categories (typically ["M"] for Medium)
- `speed` (number/object) - Movement speed(s)
- `entries` (array) - Racial trait descriptions

### Optional Core Fields
- `page` (number) - Page number in source book
- `srd` (boolean) - Available in System Reference Document
- `basicRules` (boolean) - Available in basic rules
- `hasFluff` (boolean) - Has associated lore text
- `hasFluffImages` (boolean) - Has associated artwork
- `soundClip` (object) - Audio pronunciation reference

## Ability Scores (`ability`)
Array of ability score modifications:

```json
// Fixed bonuses
[
  {
    "str": 2,
    "con": 1
  }
]

// Choice-based bonuses
[
  {
    "choose": {
      "from": ["str", "dex", "con", "int", "wis", "cha"],
      "count": 2,
      "amount": 1
    }
  }
]

// Mixed bonuses
[
  {
    "cha": 2,
    "choose": {
      "from": ["str", "dex", "con", "int", "wis"],
      "count": 1,
      "amount": 1
    }
  }
]
```

## Age (`age`)
Maturity and lifespan information:

```json
{
  "mature": 18,      // Age of maturity
  "max": 100         // Maximum typical age
}
```

## Size and Physical Traits

### Size Categories
- `"T"` - Tiny
- `"S"` - Small
- `"M"` - Medium
- `"L"` - Large
- `"V"` - Varies (with choice)

### Height and Weight (`heightAndWeight`)
```json
{
  "baseHeight": 56,    // Base height in inches
  "heightMod": "2d10", // Height modifier roll
  "baseWeight": 110,   // Base weight in pounds
  "weightMod": "2d4"   // Weight multiplier
}
```

## Speed (`speed`)
Movement capabilities:

```json
// Simple speed
30

// Complex speed with multiple types
{
  "walk": 30,
  "fly": 50,
  "swim": 30,
  "climb": 30,
  "burrow": 20
}

// Conditional speed
{
  "walk": 25,
  "condition": "(35 ft. in light armor)"
}
```

## Languages (`languageProficiencies`)
Known and learnable languages:

```json
[
  {
    "common": true,
    "dwarvish": true
  }
]

// With choices
[
  {
    "common": true,
    "anyStandard": 1  // Choose 1 standard language
  }
]
```

## Racial Traits (`entries`)
Array of trait descriptions:

```json
[
  {
    "type": "entries",
    "name": "Darkvision",
    "entries": [
      "You can see in dim light within 60 feet..."
    ]
  },
  {
    "type": "entries", 
    "name": "Dwarven Resilience",
    "entries": [
      "You have advantage on saving throws against poison..."
    ]
  }
]
```

## Special Abilities

### Damage Resistance (`resist`)
```json
[
  "poison",
  {
    "resist": ["fire"],
    "condition": "from spells"
  }
]
```

### Damage Immunity (`immune`)
```json
[
  "poison",
  "disease"
]
```

### Condition Immunities (`conditionImmune`)
```json
[
  "poisoned",
  "charmed"
]
```

### Senses (`senses`)
```json
[
  {
    "type": "darkvision",
    "range": 60
  },
  {
    "type": "tremorsense",
    "range": 30
  }
]
```

## Skill and Tool Proficiencies

### Skill Proficiencies (`skillProficiencies`)
```json
[
  {
    "perception": true,
    "stealth": true
  }
]

// With expertise
[
  {
    "perception": true,
    "expertise": true
  }
]
```

### Tool Proficiencies (`toolProficiencies`)
```json
[
  {
    "smith's tools": true,
    "brewer's supplies": true,
    "mason's tools": true
  }
]
```

### Weapon/Armor Proficiencies (`weaponProficiencies`, `armorProficiencies`)
```json
{
  "weaponProficiencies": [
    {
      "battleaxe|phb": true,
      "handaxe|phb": true,
      "throwing hammer|phb": true,
      "warhammer|phb": true
    }
  ]
}
```

## Trait Tags (`traitTags`)
Standardized trait categories for filtering:

- `"Amphibious"` - Can breathe air and water
- `"Armor Proficiency"` - Grants armor proficiencies
- `"Damage Resistance"` - Has damage resistances
- `"Darkvision"` - Has darkvision
- `"Improved Resting"` - Modified rest mechanics
- `"Monstrous Race"` - Non-standard player race
- `"Natural Armor"` - Has natural armor
- `"Natural Weapon"` - Has natural weapons
- `"Powerful Build"` - Counts as larger size
- `"Skill Proficiency"` - Grants skill proficiencies
- `"Spellcasting"` - Has innate spellcasting
- `"Tool Proficiency"` - Grants tool proficiencies
- `"Weapon Proficiency"` - Grants weapon proficiencies

## Subrace Structure

### Core Subrace Fields
```json
{
  "name": "Hill Dwarf",
  "source": "PHB",
  "raceName": "Dwarf",        // Parent race
  "raceSource": "PHB",         // Parent race source
  "ability": [
    {
      "wis": 1
    }
  ],
  "entries": [
    {
      "type": "entries",
      "name": "Dwarven Toughness",
      "entries": [
        "Your hit point maximum increases by 1..."
      ]
    }
  ]
}
```

### Subrace Overrides
Subraces can override or add to base race features:
- Additional ability scores
- Modified speed
- Extra proficiencies
- New traits
- Different size

## Fluff Linkage

### Race Fluff (`fluff-{source}.json`)
```json
{
  "raceFluff": [
    {
      "name": "Dwarf",           // Must match race name
      "source": "PHB",           // Must match race source
      "entries": [
        {
          "type": "quote",
          "entries": ["..."],
          "by": "Bruenor Battlehammer"
        },
        "Kingdoms rich in ancient grandeur...",
        {
          "type": "entries",
          "name": "Short and Stout",
          "entries": ["Bold and hardy..."]
        }
      ],
      "images": [
        {
          "type": "image",
          "href": {
            "type": "internal",
            "path": "races/PHB/Dwarf.webp"
          }
        }
      ]
    }
  ]
}
```

## Example Race Entry

```json
{
  "name": "Dwarf",
  "source": "PHB",
  "page": 18,
  "srd": true,
  "basicRules": true,
  "size": ["M"],
  "speed": 25,
  "ability": [
    {
      "con": 2
    }
  ],
  "age": {
    "mature": 50,
    "max": 350
  },
  "darkvision": 60,
  "resist": ["poison"],
  "weaponProficiencies": [
    {
      "battleaxe|phb": true,
      "handaxe|phb": true,
      "light hammer|phb": true,
      "warhammer|phb": true
    }
  ],
  "toolProficiencies": [
    {
      "choose": {
        "from": [
          "smith's tools|phb",
          "brewer's supplies|phb",
          "mason's tools|phb"
        ],
        "count": 1
      }
    }
  ],
  "languageProficiencies": [
    {
      "common": true,
      "dwarvish": true
    }
  ],
  "traitTags": [
    "Damage Resistance",
    "Darkvision",
    "Tool Proficiency",
    "Weapon Proficiency"
  ],
  "heightAndWeight": {
    "baseHeight": 44,
    "heightMod": "2d4",
    "baseWeight": 115,
    "weightMod": "2d6"
  },
  "entries": [
    {
      "name": "Age",
      "type": "entries",
      "entries": [
        "Dwarves mature at the same rate as humans..."
      ]
    },
    {
      "name": "Size",
      "type": "entries",
      "entries": [
        "Dwarves stand between 4 and 5 feet tall..."
      ]
    },
    {
      "name": "Darkvision",
      "entries": [
        "Accustomed to life underground..."
      ]
    },
    {
      "name": "Dwarven Resilience",
      "entries": [
        "You have advantage on saving throws against poison..."
      ]
    }
  ],
  "hasFluff": true,
  "hasFluffImages": true
}
```