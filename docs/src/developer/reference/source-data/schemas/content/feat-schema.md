# Feat Schema Reference

Complete field reference for feat entries in the extracted 5etools data format.

## File Location
- Path: `{book}/feats/{source}.json`

## JSON Structure
```json
{
  "feat": [
    {
      // feat objects
    }
  ]
}
```

## Core Fields

### Required Fields
- `name` (string) - Feat name (e.g., "Alert", "Lucky", "Sharpshooter")
- `source` (string) - Source book abbreviation (PHB, XGE, TCE, etc.)
- `entries` (array) - Feat description and benefits

### Optional Core Fields
- `page` (number) - Page number in source book
- `srd` (boolean) - Available in System Reference Document
- `prerequisite` (array) - Requirements to take the feat
- `ability` (array) - Ability score improvements
- `skillProficiencies` (array) - Skill proficiency grants
- `languageProficiencies` (array) - Language grants
- `toolProficiencies` (array) - Tool proficiency grants
- `weaponProficiencies` (array) - Weapon proficiency grants
- `armorProficiencies` (array) - Armor proficiency grants
- `savingThrowProficiencies` (array) - Saving throw proficiency grants
- `expertise` (array) - Expertise grants
- `resist` (array) - Damage resistances
- `immune` (array) - Damage/condition immunities
- `senses` (array) - Special senses (darkvision, etc.)
- `additionalSpells` (array) - Spells granted by the feat
- `otherSources` (array) - Additional source references

## Prerequisites (`prerequisite`)
Requirements that must be met to take the feat:

```json
[
  {
    "ability": [
      {
        "str": 13
      }
    ]
  }
]

// Multiple requirements (AND)
[
  {
    "ability": [
      {
        "str": 13
      }
    ]
  },
  {
    "proficiency": [
      {
        "armor": "heavy"
      }
    ]
  }
]

// Spellcasting requirement
[
  {
    "spellcasting": true
  }
]

// Level requirement
[
  {
    "level": 4
  }
]

// Race requirement
[
  {
    "race": [
      {
        "name": "elf"
      }
    ]
  }
]
```

## Ability Score Improvements (`ability`)
Ability score increases granted by the feat:

```json
// Fixed increase
[
  {
    "cha": 1
  }
]

// Choice of increases
[
  {
    "choose": {
      "from": ["str", "dex", "con", "int", "wis", "cha"],
      "count": 1,
      "amount": 1
    }
  }
]

// Multiple increases
[
  {
    "str": 1,
    "con": 1
  }
]

// Mixed fixed and choice
[
  {
    "int": 1,
    "choose": {
      "from": ["str", "dex", "con", "wis", "cha"],
      "count": 1,
      "amount": 1
    }
  }
]
```

## Proficiency Grants

### Skill Proficiencies
```json
[
  {
    "perception": true
  }
]

// With choices
[
  {
    "choose": {
      "from": ["arcana", "history", "investigation", "nature", "religion"],
      "count": 1
    }
  }
]
```

### Weapon/Armor Proficiencies
```json
{
  "weaponProficiencies": [
    {
      "firearms": true
    }
  ],
  "armorProficiencies": [
    {
      "light": true,
      "medium": true
    }
  ]
}
```

### Saving Throw Proficiencies
```json
[
  {
    "con": true
  }
]
```

## Expertise (`expertise`)
Double proficiency bonus for specific skills:

```json
[
  {
    "choose": {
      "from": ["investigation", "perception"],
      "count": 1
    }
  }
]
```

## Granted Spells (`additionalSpells`)
Spells learned through the feat:

```json
[
  {
    "innate": {
      "_": {
        "daily": {
          "1": ["misty step"]
        }
      }
    }
  }
]

// Expanded spell list
[
  {
    "expanded": {
      "s1": ["shield", "mage armor"],
      "s2": ["mirror image", "misty step"],
      "s3": ["counterspell", "dispel magic"]
    }
  }
]

// Known spells
[
  {
    "known": {
      "1": ["detect magic", "disguise self"]
    }
  }
]
```

## Special Senses (`senses`)
```json
[
  {
    "darkvision": 60
  }
]

// Multiple senses
[
  {
    "darkvision": 120,
    "blindsight": 10
  }
]
```

## Feat Categories
Common feat types and their typical structure:

### Ability Score Improvement
```json
{
  "name": "Ability Score Improvement",
  "source": "PHB",
  "page": 165,
  "ability": [
    {
      "choose": {
        "from": ["str", "dex", "con", "int", "wis", "cha"],
        "count": 2,
        "amount": 1
      }
    }
  ],
  "entries": [
    "You can increase one ability score of your choice by 2, or you can increase two ability scores of your choice by 1. As normal, you can't increase an ability score above 20 using this feature."
  ]
}
```

### Combat Feats
```json
{
  "name": "Great Weapon Master",
  "source": "PHB",
  "page": 167,
  "entries": [
    "You've learned to put the weight of a weapon to your advantage, letting its momentum empower your strikes. You gain the following benefits:",
    {
      "type": "list",
      "items": [
        "On your turn, when you score a critical hit with a melee weapon or reduce a creature to 0 hit points with one, you can make one melee weapon attack as a bonus action.",
        "Before you make a melee attack with a heavy weapon that you are proficient with, you can choose to take a -5 penalty to the attack roll. If the attack hits, you add +10 to the attack's damage."
      ]
    }
  ]
}
```

### Magic Feats
```json
{
  "name": "Magic Initiate",
  "source": "PHB",
  "page": 168,
  "entries": [
    "Choose a class: bard, cleric, druid, sorcerer, warlock, or wizard. You learn two cantrips of your choice from that class's spell list.",
    "In addition, choose one 1st-level spell from that same list. You learn that spell and can cast it at its lowest level. Once you cast it, you must finish a long rest before you can cast it again using this feat.",
    "Your spellcasting ability for these spells depends on the class you chose: Charisma for bard, sorcerer, or warlock; Wisdom for cleric or druid; or Intelligence for wizard."
  ]
}
```

### Racial Feats
```json
{
  "name": "Elven Accuracy",
  "source": "XGE",
  "page": 74,
  "prerequisite": [
    {
      "race": [
        {
          "name": "elf",
          "subrace": "half-elf"
        }
      ]
    }
  ],
  "ability": [
    {
      "choose": {
        "from": ["dex", "int", "wis", "cha"],
        "count": 1,
        "amount": 1
      }
    }
  ],
  "entries": [
    "The accuracy of elves is legendary...",
    "Whenever you have advantage on an attack roll using Dexterity, Intelligence, Wisdom, or Charisma, you can reroll one of the dice once."
  ]
}
```

## Example Feat Entry

```json
{
  "name": "Alert",
  "source": "PHB",
  "page": 165,
  "entries": [
    "Always on the lookout for danger, you gain the following benefits:",
    {
      "type": "list",
      "items": [
        "You gain a +5 bonus to initiative.",
        "You can't be surprised while you are conscious.",
        "Other creatures don't gain advantage on attack rolls against you as a result of being unseen by you."
      ]
    }
  ]
}
```

## Complex Feat Example

```json
{
  "name": "Fey Touched",
  "source": "TCE",
  "page": 79,
  "ability": [
    {
      "choose": {
        "from": ["int", "wis", "cha"],
        "count": 1,
        "amount": 1
      }
    }
  ],
  "additionalSpells": [
    {
      "innate": {
        "_": {
          "daily": {
            "1": [
              "misty step",
              {
                "choose": "level=1|school=E;D"
              }
            ]
          }
        }
      }
    }
  ],
  "entries": [
    "Your exposure to the Feywild's magic has changed you, granting you the following benefits:",
    {
      "type": "list",
      "items": [
        "Increase your Intelligence, Wisdom, or Charisma score by 1, to a maximum of 20.",
        "You learn the {@spell misty step} spell and one 1st-level spell of your choice. The 1st-level spell must be from the divination or enchantment school of magic. You can cast each of these spells without expending a spell slot. Once you cast either of these spells in this way, you can't cast that spell in this way again until you finish a long rest. You can also cast these spells using spell slots you have of the appropriate level. The spells' spellcasting ability is the ability increased by this feat."
      ]
    }
  ]
}
```