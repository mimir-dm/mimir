# Bestiary Schema Documentation

This document defines the JSON schema for monster and creature data in the D&D 5etools format.

## Overview

The bestiary schema represents all creatures and monsters in D&D including NPCs, beasts, dragons, aberrations, and other creature types. Monsters are stored in JSON files with the root key `"monster"` containing an array of creature objects.

## File Location
- Path: `{book}/bestiary/bestiary-{source}.json`
- Fluff: `{book}/bestiary/fluff-bestiary-{source}.json`

## Root Structure

```json
{
  "monster": [
    {
      // Monster objects
    }
  ]
}
```

## Core Monster Object

Every monster object contains these required fields:

```json
{
  "name": "string",           // Monster name
  "source": "string",         // Source book abbreviation (MM, VGM, etc.)
  "page": "number",           // Page number in source book
  "size": ["string"],         // Size category array
  "type": "string|object",    // Creature type
  "alignment": ["string"],    // Alignment array
  "ac": ["number|object"],    // Armor class array
  "hp": "object",             // Hit points
  "speed": "object",          // Movement speeds
  "str": "number",            // Strength score
  "dex": "number",            // Dexterity score
  "con": "number",            // Constitution score
  "int": "number",            // Intelligence score
  "wis": "number",            // Wisdom score
  "cha": "number"             // Charisma score
}
```

## Size Categories

The `size` field is an array containing one or more size codes:

- `T` - Tiny
- `S` - Small
- `M` - Medium
- `L` - Large
- `H` - Huge
- `G` - Gargantuan

Example: `["M"]` for Medium, `["L", "H"]` for Large or Huge

## Creature Type

The `type` field can be a string or complex object:

### Simple Type
```json
"type": "humanoid"
```

### Complex Type with Tags
```json
"type": {
  "type": "humanoid",
  "tags": ["goblinoid"]
}
```

### Swarm Type
```json
"type": {
  "type": "swarm",
  "swarmSize": "T",
  "tags": ["beast"]
}
```

### Creature Type Values
- `aberration`
- `beast`
- `celestial`
- `construct`
- `dragon`
- `elemental`
- `fey`
- `fiend`
- `giant`
- `humanoid`
- `monstrosity`
- `ooze`
- `plant`
- `undead`

## Alignment

The `alignment` field is an array of alignment objects or codes:

```json
// Single alignment
["L", "E"]  // Lawful Evil

// Multiple alignments
["L", "NX", "C", "E"]  // Lawful/Neutral/Chaotic Evil

// Special alignments
["U"]  // Unaligned
["A"]  // Any alignment
["N"]  // Neutral

// Complex alignment
[{
  "alignment": ["C", "E"],
  "chance": 50
}]
```

### Alignment Codes
- `L` - Lawful
- `N` - Neutral  
- `C` - Chaotic
- `G` - Good
- `E` - Evil
- `U` - Unaligned
- `A` - Any
- `NX` - Neutral (on law/chaos axis)
- `NY` - Neutral (on good/evil axis)

## Armor Class

The `ac` field is an array of AC values or objects:

```json
// Simple AC
[15]

// AC with armor type
[{
  "ac": 15,
  "from": ["natural armor"]
}]

// Multiple AC values
[
  {
    "ac": 15,
    "from": ["leather armor", "shield"]
  },
  {
    "ac": 18,
    "from": ["plate armor"],
    "condition": "with armor"
  }
]

// AC with bonus
[{
  "ac": 13,
  "from": ["{@spell mage armor}"],
  "condition": "with {@spell mage armor}"
}]
```

## Hit Points

The `hp` field defines creature health:

```json
{
  "average": 45,
  "formula": "7d8 + 14"
}

// Special HP
{
  "special": "equal to the summoner's hit point maximum"
}
```

## Speed

The `speed` field defines movement types and distances:

```json
{
  "walk": 30
}

// Multiple movement types
{
  "walk": 30,
  "fly": 60,
  "swim": 40,
  "climb": 30,
  "burrow": 20
}

// Conditional speeds
{
  "walk": 30,
  "fly": {
    "number": 60,
    "condition": "(hover)"
  }
}

// Alternative forms
{
  "walk": 30,
  "alternate": {
    "walk": [
      {
        "number": 40,
        "condition": "in wolf form"
      }
    ]
  }
}
```

## Ability Scores

Six required ability scores (1-30):

```json
{
  "str": 18,
  "dex": 14,
  "con": 16,
  "int": 10,
  "wis": 12,
  "cha": 8
}
```

## Saving Throws

The `save` field defines proficient saving throws:

```json
{
  "str": "+7",
  "con": "+10",
  "wis": "+6",
  "cha": "+5"
}
```

## Skills

The `skill` field defines skill proficiencies:

```json
{
  "perception": "+4",
  "stealth": "+6",
  "athletics": "+7",
  "deception": "+5"
}

// With special modifiers
{
  "perception": {
    "mod": "+9",
    "condition": "while in dim light or darkness"
  }
}
```

## Damage Interactions

### Damage Vulnerabilities
```json
"vulnerable": ["fire", "radiant"]
```

### Damage Resistances
```json
"resist": [
  "cold",
  "necrotic",
  {
    "resist": ["bludgeoning", "piercing", "slashing"],
    "note": "from nonmagical attacks"
  }
]
```

### Damage Immunities
```json
"immune": [
  "poison",
  "psychic",
  {
    "immune": ["bludgeoning", "piercing", "slashing"],
    "note": "from nonmagical attacks that aren't silvered"
  }
]
```

### Condition Immunities
```json
"conditionImmune": ["charmed", "frightened", "paralyzed", "poisoned"]
```

## Senses

The `senses` field lists special senses:

```json
[
  "darkvision 60 ft.",
  "blindsight 30 ft.",
  "truesight 120 ft.",
  "tremorsense 60 ft."
]

// With passive Perception
[
  "darkvision 60 ft.",
  "passive Perception 14"
]
```

## Languages

The `languages` field lists known languages:

```json
["Common", "Goblin"]

// With telepathy
["Common", "telepathy 120 ft."]

// Special cases
["—"]  // No languages
["all", "telepathy 120 ft."]
```

## Challenge Rating

The `cr` field defines difficulty and XP:

```json
// Standard CR
"cr": "5"

// Fractional CR
"cr": "1/4"
"cr": "1/2"

// Zero CR
"cr": "0"

// Complex CR
"cr": {
  "cr": "13",
  "lair": "14"
}

// Unknown CR
"cr": "Unknown"
```

## Traits

The `trait` field is an array of special abilities:

```json
[
  {
    "name": "Keen Smell",
    "entries": [
      "The wolf has advantage on Wisdom (Perception) checks that rely on smell."
    ]
  },
  {
    "name": "Pack Tactics",
    "entries": [
      "The wolf has advantage on an attack roll against a creature if at least one of the wolf's allies is within 5 feet of the creature and the ally isn't {@condition incapacitated}."
    ]
  }
]
```

## Spellcasting

The `spellcasting` field defines spellcasting abilities:

```json
[
  {
    "name": "Spellcasting",
    "headerEntries": [
      "The mage is a 9th-level spellcaster. Its spellcasting ability is Intelligence (spell save {@dc 14}, {@hit 6} to hit with spell attacks). The mage has the following wizard spells prepared:"
    ],
    "spells": {
      "0": {
        "spells": ["{@spell fire bolt}", "{@spell light}", "{@spell mage hand}", "{@spell prestidigitation}"]
      },
      "1": {
        "slots": 4,
        "spells": ["{@spell detect magic}", "{@spell mage armor}", "{@spell magic missile}", "{@spell shield}"]
      },
      "2": {
        "slots": 3,
        "spells": ["{@spell misty step}", "{@spell suggestion}"]
      },
      "3": {
        "slots": 3,
        "spells": ["{@spell counterspell}", "{@spell fireball}", "{@spell fly}"]
      },
      "4": {
        "slots": 3,
        "spells": ["{@spell greater invisibility}", "{@spell ice storm}"]
      },
      "5": {
        "slots": 1,
        "spells": ["{@spell cone of cold}"]
      }
    },
    "ability": "int",
    "type": "spellcasting"
  }
]

// Innate Spellcasting
[
  {
    "name": "Innate Spellcasting",
    "headerEntries": [
      "The deva's spellcasting ability is Charisma (spell save {@dc 17}). The deva can innately cast the following spells, requiring only verbal components:"
    ],
    "will": [
      "{@spell detect evil and good}",
      "{@spell detect thoughts}"
    ],
    "daily": {
      "1": ["{@spell commune}", "{@spell raise dead}"],
      "3": ["{@spell cure wounds}", "{@spell lesser restoration}"]
    },
    "ability": "cha",
    "type": "innate"
  }
]
```

## Actions

The `action` field defines standard actions:

```json
[
  {
    "name": "Multiattack",
    "entries": [
      "The creature makes two attacks: one with its bite and one with its claws."
    ]
  },
  {
    "name": "Bite",
    "entries": [
      "{@atk mw} {@hit 7} to hit, reach 5 ft., one target. {@h}11 ({@damage 2d6 + 4}) piercing damage."
    ]
  },
  {
    "name": "Fire Breath {@recharge 5}",
    "entries": [
      "The dragon exhales fire in a 15-foot cone. Each creature in that area must make a {@dc 13} Dexterity saving throw, taking 24 ({@damage 7d6}) fire damage on a failed save, or half as much damage on a successful one."
    ]
  }
]
```

## Bonus Actions

The `bonus` field defines bonus actions:

```json
[
  {
    "name": "Nimble Escape",
    "entries": [
      "The goblin can take the Disengage or Hide action as a bonus action on each of its turns."
    ]
  }
]
```

## Reactions

The `reaction` field defines reactions:

```json
[
  {
    "name": "Parry",
    "entries": [
      "The noble adds 2 to its AC against one melee attack that would hit it. To do so, the noble must see the attacker and be wielding a melee weapon."
    ]
  }
]
```

## Legendary Actions

The `legendary` field defines legendary actions:

```json
[
  {
    "name": "Detect",
    "entries": [
      "The dragon makes a Wisdom (Perception) check."
    ]
  },
  {
    "name": "Tail Attack",
    "entries": [
      "The dragon makes a tail attack."
    ]
  },
  {
    "name": "Wing Attack (Costs 2 Actions)",
    "entries": [
      "The dragon beats its wings. Each creature within 10 feet of the dragon must succeed on a {@dc 19} Dexterity saving throw or take 13 ({@damage 2d6 + 6}) bludgeoning damage and be knocked {@condition prone}. The dragon can then fly up to half its flying speed."
    ]
  }
]

// With header text
{
  "legendaryHeader": [
    "The dragon can take 3 legendary actions, choosing from the options below. Only one legendary action option can be used at a time and only at the end of another creature's turn. The dragon regains spent legendary actions at the start of its turn."
  ],
  "legendary": [...]
}
```

## Mythic Actions

The `mythic` field defines mythic encounter mechanics:

```json
{
  "mythicHeader": [
    "If the creature's mythic trait is active, it can use the options below as legendary actions."
  ],
  "mythic": [
    {
      "name": "Mythic Action",
      "entries": ["Description of mythic action"]
    }
  ]
}
```

## Lair Actions

The `legendaryGroup` field links to lair actions and regional effects:

```json
{
  "legendaryGroup": {
    "name": "Ancient Red Dragon",
    "source": "MM"
  }
}
```

## Variants

The `variant` field defines creature variants:

```json
[
  {
    "type": "variant",
    "name": "Devil's Sight",
    "entries": [
      "Magical darkness doesn't impede the imp's darkvision."
    ]
  },
  {
    "type": "variant",
    "name": "Familiar",
    "entries": [
      "The imp can serve another creature as a familiar, forming a telepathic bond with its willing master..."
    ]
  }
]
```

## Alternate Versions

The `altArt` field references alternate artwork:

```json
[
  {
    "name": "Goblin (b)",
    "source": "MM"
  }
]
```

## Environment

The `environment` field lists suitable environments:

```json
["forest", "swamp", "underdark"]

// All environments
["arctic", "coastal", "desert", "forest", "grassland", "hill", "mountain", "swamp", "underdark", "underwater", "urban"]
```

## Sound Clips

The `soundClip` field links to audio:

```json
{
  "type": "external",
  "url": "https://example.com/roar.mp3"
}
```

## Token Information

The `token` field provides VTT token data:

```json
{
  "name": "Goblin",
  "source": "MM"
}
```

## Formatting Tags in Text

Monster descriptions use 5etools formatting tags:

- `{@atk mw}` - Melee weapon attack
- `{@atk rw}` - Ranged weapon attack  
- `{@atk ms}` - Melee spell attack
- `{@atk rs}` - Ranged spell attack
- `{@hit 5}` - Attack bonus
- `{@h}` - Hit text
- `{@damage 2d6 + 3}` - Damage rolls
- `{@dc 15}` - Difficulty class
- `{@recharge 5}` - Recharge notation
- `{@condition poisoned}` - Conditions
- `{@spell fireball}` - Spell references
- `{@creature goblin}` - Creature references
- `{@item longsword}` - Item references
- `{@dice 1d20}` - Dice rolls
- `{@chance 25}` - Percentage chance

## Complete Monster Example

```json
{
  "name": "Goblin",
  "source": "MM",
  "page": 166,
  "size": ["S"],
  "type": {
    "type": "humanoid",
    "tags": ["goblinoid"]
  },
  "alignment": ["N", "E"],
  "ac": [
    {
      "ac": 15,
      "from": ["leather armor", "shield"]
    }
  ],
  "hp": {
    "average": 7,
    "formula": "2d6"
  },
  "speed": {
    "walk": 30
  },
  "str": 8,
  "dex": 14,
  "con": 10,
  "int": 10,
  "wis": 8,
  "cha": 8,
  "skill": {
    "stealth": "+6"
  },
  "senses": [
    "darkvision 60 ft.",
    "passive Perception 9"
  ],
  "languages": ["Common", "Goblin"],
  "cr": "1/4",
  "trait": [
    {
      "name": "Nimble Escape",
      "entries": [
        "The goblin can take the Disengage or Hide action as a bonus action on each of its turns."
      ]
    }
  ],
  "action": [
    {
      "name": "Scimitar",
      "entries": [
        "{@atk mw} {@hit 4} to hit, reach 5 ft., one target. {@h}5 ({@damage 1d6 + 2}) slashing damage."
      ]
    },
    {
      "name": "Shortbow",
      "entries": [
        "{@atk rw} {@hit 4} to hit, range 80/320 ft., one target. {@h}5 ({@damage 1d6 + 2}) piercing damage."
      ]
    }
  ],
  "environment": ["forest", "grassland", "hill", "underdark"],
  "soundClip": {
    "type": "external",
    "url": "https://5e.tools/audio/bestiary/goblin.mp3"
  },
  "hasFluff": true,
  "hasFluffImages": true,
  "token": {
    "name": "Goblin",
    "source": "MM"
  }
}
```

## Special Monster Types

### NPCs
NPCs often have additional fields:
- `isNpc`: true - Marks as an NPC
- `group`: ["Knights of Solamnia"] - Organization affiliations

### Named NPCs
Named creatures may have:
- `isNamedCreature`: true - Marks as unique individual
- `familiar`: true - Can serve as a familiar

### Swarms
Swarm creatures include swarmSize in their type object

### Summoned Creatures
Summoned creatures may have:
- `summonedBySpell`: true - Created by spell effects
- `summonedByClass`: "Druid" - Class that can summon

## Notes

- All numeric values should be treated as either integers or floats
- Text fields support D&D formatting tags (see formatting section)
- Monster references use the format `"creature name|source"` for cross-linking
- Boolean fields default to `false` when omitted
- Arrays default to empty `[]` when omitted
- The `page` field may be a number or object with `start` and `end` for multi-page entries