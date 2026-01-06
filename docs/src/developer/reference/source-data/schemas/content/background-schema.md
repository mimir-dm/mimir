# Background Schema Reference

Complete field reference for background entries in the extracted 5etools data format.

## File Location
- Path: `{book}/backgrounds/{source}.json`
- Fluff: `{book}/backgrounds/fluff-{source}.json`
- Images: `{book}/img/backgrounds/{source}/`

## JSON Structure
```json
{
  "background": [
    {
      // background objects
    }
  ]
}
```

## Core Fields

### Required Fields
- `name` (string) - Background name (e.g., "Acolyte", "Criminal", "Sage")
- `source` (string) - Source book abbreviation (PHB, SCAG, etc.)
- `entries` (array) - Background description and features
- `skillProficiencies` (array) - Granted skill proficiencies

### Optional Core Fields
- `page` (number) - Page number in source book
- `srd` (boolean) - Available in System Reference Document
- `basicRules` (boolean) - Available in basic rules
- `hasFluff` (boolean) - Has associated lore text
- `startingEquipment` (array) - Initial equipment grants
- `languageProficiencies` (array) - Language grants
- `toolProficiencies` (array) - Tool proficiency grants

## Skill Proficiencies (`skillProficiencies`)
Array of skill grants:

```json
[
  {
    "insight": true,
    "religion": true
  }
]

// With choices
[
  {
    "choose": {
      "from": ["deception", "sleight of hand", "stealth"],
      "count": 2
    }
  }
]

// Mixed fixed and choice
[
  {
    "history": true,
    "choose": {
      "from": ["arcana", "nature", "religion"],
      "count": 1
    }
  }
]
```

## Language Proficiencies (`languageProficiencies`)
Language grants and choices:

```json
[
  {
    "anyStandard": 2  // Choose 2 standard languages
  }
]

// Specific languages
[
  {
    "elvish": true,
    "anyStandard": 1
  }
]

// Exotic languages
[
  {
    "anyExotic": 1,
    "anyStandard": 1
  }
]
```

## Tool Proficiencies (`toolProficiencies`)
Tool and gaming set proficiencies:

```json
[
  {
    "thieves' tools": true
  }
]

// With choices
[
  {
    "choose": {
      "from": ["gaming set", "musical instrument"],
      "count": 1
    }
  }
]

// Multiple tools
[
  {
    "navigator's tools": true,
    "vehicles (water)": true
  }
]
```

## Starting Equipment (`startingEquipment`)
Initial equipment and wealth:

```json
[
  {
    "a": [
      "holy symbol|phb"
    ],
    "b": [
      "prayer book|phb",
      "prayer wheel|phb"
    ]
  },
  {
    "_": [
      "incense|phb|5",
      "vestments|phb",
      "common clothes|phb",
      "pouch|phb"
    ]
  },
  {
    "value": 1500  // 15 gp in copper pieces
  }
]
```

## Background Entries (`entries`)
Array containing proficiencies, equipment, and features:

```json
[
  {
    "type": "list",
    "style": "list-hang-notitle",
    "items": [
      {
        "type": "item",
        "name": "Skill Proficiencies:",
        "entry": "{@skill Insight}, {@skill Religion}"
      },
      {
        "type": "item",
        "name": "Languages:",
        "entry": "Two of your choice"
      },
      {
        "type": "item",
        "name": "Equipment:",
        "entry": "A {@item holy symbol|phb}..."
      }
    ]
  },
  {
    "type": "entries",
    "name": "Feature: Shelter of the Faithful",
    "data": {
      "isFeature": true
    },
    "entries": [
      "As an acolyte, you command the respect..."
    ]
  }
]
```

## Background Features
Each background includes a unique feature:

```json
{
  "type": "entries",
  "name": "Feature: Criminal Contact",
  "data": {
    "isFeature": true
  },
  "entries": [
    "You have a reliable and trustworthy contact..."
  ]
}
```

## Suggested Characteristics
Backgrounds include personality traits, ideals, bonds, and flaws:

```json
{
  "type": "entries",
  "name": "Suggested Characteristics",
  "entries": [
    "Description of how the background shapes character...",
    {
      "type": "table",
      "colLabels": ["d8", "Personality Trait"],
      "colStyles": ["col-1 text-center", "col-11"],
      "rows": [
        ["1", "I idolize a particular hero..."],
        ["2", "I can find common ground..."],
        // ... up to 8
      ]
    },
    {
      "type": "table",
      "colLabels": ["d6", "Ideal"],
      "colStyles": ["col-1 text-center", "col-11"],
      "rows": [
        ["1", "**Tradition.** The ancient traditions..."],
        ["2", "**Charity.** I always try to help..."],
        // ... up to 6
      ]
    },
    {
      "type": "table",
      "colLabels": ["d6", "Bond"],
      "colStyles": ["col-1 text-center", "col-11"],
      "rows": [
        ["1", "I would die to recover an ancient relic..."],
        ["2", "I will someday get revenge..."],
        // ... up to 6
      ]
    },
    {
      "type": "table",
      "colLabels": ["d6", "Flaw"],
      "colStyles": ["col-1 text-center", "col-11"],
      "rows": [
        ["1", "I judge others harshly..."],
        ["2", "I put too much trust in those..."],
        // ... up to 6
      ]
    }
  ]
}
```

## Background Variants
Some backgrounds have variant options:

```json
{
  "name": "Pirate",
  "source": "PHB",
  "page": 139,
  "entries": [
    {
      "type": "entries",
      "name": "Variant: Pirate",
      "entries": [
        "You can use this as a variant of the {@background sailor|phb}..."
      ]
    }
  ],
  "_copy": {
    "name": "Sailor",
    "source": "PHB",
    "_mod": {
      "entries": {
        "mode": "insertArr",
        "index": 2,
        "items": {
          "type": "entries",
          "name": "Variant Feature: Bad Reputation",
          "entries": ["..."]
        }
      }
    }
  }
}
```

## Fluff Linkage

### Background Fluff (`fluff-{source}.json`)
```json
{
  "backgroundFluff": [
    {
      "name": "Acolyte",        // Must match background name
      "source": "PHB",          // Must match background source
      "entries": [
        "You have spent your life in the service of a temple...",
        {
          "type": "entries",
          "name": "Life in the Temple",
          "entries": [
            "Temples vary widely..."
          ]
        }
      ]
    }
  ]
}
```

## Custom Backgrounds
Rules for creating custom backgrounds:

```json
{
  "type": "entries",
  "name": "Customizing a Background",
  "entries": [
    "You might want to tweak some of the features...",
    {
      "type": "list",
      "items": [
        "Choose any two skills",
        "Choose a total of two tool proficiencies or languages",
        "Choose equipment worth 10-25 gp",
        "Define a feature with your DM"
      ]
    }
  ]
}
```

## Example Background Entry

```json
{
  "name": "Criminal",
  "source": "PHB",
  "page": 129,
  "srd": true,
  "basicRules": true,
  "skillProficiencies": [
    {
      "deception": true,
      "stealth": true
    }
  ],
  "toolProficiencies": [
    {
      "thieves' tools": true,
      "choose": {
        "from": ["gaming set"],
        "count": 1
      }
    }
  ],
  "startingEquipment": [
    {
      "_": [
        "crowbar|phb",
        "common clothes|phb|dark",
        "pouch|phb"
      ]
    },
    {
      "value": 1500  // 15 gp
    }
  ],
  "entries": [
    {
      "type": "list",
      "style": "list-hang-notitle",
      "items": [
        {
          "type": "item",
          "name": "Skill Proficiencies:",
          "entry": "{@skill Deception}, {@skill Stealth}"
        },
        {
          "type": "item",
          "name": "Tool Proficiencies:",
          "entry": "One type of gaming set, {@item thieves' tools|phb}"
        },
        {
          "type": "item",
          "name": "Equipment:",
          "entry": "A {@item crowbar|phb}, a set of dark {@item common clothes|phb}..."
        }
      ]
    },
    {
      "type": "entries",
      "name": "Feature: Criminal Contact",
      "data": {
        "isFeature": true
      },
      "entries": [
        "You have a reliable and trustworthy contact who acts as your liaison..."
      ]
    },
    {
      "type": "entries",
      "name": "Criminal Specialties",
      "entries": [
        "There are many kinds of criminals...",
        {
          "type": "table",
          "colLabels": ["d8", "Specialty"],
          "colStyles": ["col-1 text-center", "col-11"],
          "rows": [
            ["1", "Blackmailer"],
            ["2", "Burglar"],
            ["3", "Enforcer"],
            ["4", "Fence"],
            ["5", "Highway robber"],
            ["6", "Hired killer"],
            ["7", "Pickpocket"],
            ["8", "Smuggler"]
          ]
        }
      ]
    }
  ],
  "hasFluff": true
}
```