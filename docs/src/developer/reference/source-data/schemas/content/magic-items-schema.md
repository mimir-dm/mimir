# Magic Items Schema Documentation

This document defines the JSON schema for magic items and artifacts in the D&D 5etools format.

## Overview

The magic items schema represents all magical items in D&D including weapons, armor, wondrous items, artifacts, and other enchanted objects. Magic items are stored in JSON files with the root key `"item"` containing an array of item objects, alongside mundane items.

## File Location
- Path: `{book}/items/items-{source}.json`
- Fluff: `{book}/items/fluff-items-{source}.json`

## Root Structure

```json
{
  "item": [
    {
      // Magic item objects
    }
  ]
}
```

## Core Magic Item Object

Every magic item contains these base fields from the standard item schema:

```json
{
  "name": "string",           // Item name
  "source": "string",         // Source book abbreviation (DMG, XGE, etc.)
  "page": "number",           // Page number in source book
  "type": "string",           // Item type code with source suffix
  "rarity": "string",         // Rarity level (not "none" for magic items)
  "entries": ["string"],      // Item description and effects
  "reqAttune": "boolean|string" // Attunement requirement
}
```

## Magic Item Type Codes

Magic items use compound type codes combining base type with source:

### Wondrous Items
- `WND|DMG` - Wondrous item from DMG
- `WND|XGE` - Wondrous item from XGE

### Magic Weapons
- `M|DMG` - Melee weapon from DMG
- `R|DMG` - Ranged weapon from DMG

### Magic Armor
- `LA|DMG` - Light armor from DMG
- `MA|DMG` - Medium armor from DMG
- `HA|DMG` - Heavy armor from DMG
- `S|DMG` - Shield from DMG

### Other Magic Items
- `RD|DMG` - Rod
- `RG|DMG` - Ring
- `P|DMG` - Potion
- `SC|DMG` - Scroll
- `ST|DMG` - Staff
- `W|DMG` - Wand
- `A|DMG` - Ammunition
- `$|DMG` - Valuable object/treasure
- `G|DMG` - Adventuring gear
- `OTH|DMG` - Other items

## Rarity Levels

The `rarity` field for magic items:

```json
"rarity": "uncommon"      // Standard rarities
"rarity": "rare"
"rarity": "very rare"
"rarity": "legendary"
"rarity": "artifact"
"rarity": "varies"        // Items with multiple versions
"rarity": "unknown"       // Undefined rarity
"rarity": "unknown (magic)" // Magic but unspecified rarity
"rarity": "common"        // Common magic items
```

## Attunement

The `reqAttune` field defines attunement requirements:

```json
// Simple attunement
"reqAttune": true         // Requires attunement
"reqAttune": false        // No attunement needed

// Conditional attunement
"reqAttune": "by a spellcaster"
"reqAttune": "by a cleric, druid, or paladin"
"reqAttune": "by a creature of good alignment"
"reqAttune": "by a bard"
"reqAttune": "by a creature with a Strength score of 13 or higher"

// Optional attunement (for items with variable benefits)
"reqAttune": "optional"
"reqAttune": "(optional)"
```

### Structured Attunement Tags

Complex attunement requirements use `reqAttuneTags`:

```json
"reqAttuneTags": [
  {
    "class": "Wizard"        // Class requirement
  },
  {
    "alignment": ["G"]       // Alignment requirement
  },
  {
    "stat": [               // Ability score requirement
      {
        "str": 13
      }
    ]
  },
  {
    "spellcasting": true    // Must be a spellcaster
  }
]
```

## Magic Item Properties

### Tier Classification
```json
"tier": "minor"           // Minor magic item
"tier": "major"           // Major magic item
```

### Bonus Properties

Magic items can provide various bonuses:

```json
// Weapon bonuses
"bonusWeapon": "+1"       // Attack and damage bonus
"bonusWeaponAttack": "+2" // Attack roll bonus only
"bonusWeaponDamage": "+1" // Damage roll bonus only
"bonusWeaponCritDamage": "1d6" // Extra crit damage

// Armor bonuses
"bonusAc": "+1"           // AC bonus for armor/shields
"bonusAcSpecial": "+2"    // Special AC bonus

// Spellcasting bonuses
"bonusSpellAttack": "+1"  // Spell attack bonus
"bonusSpellSaveDc": "+2"  // Spell save DC bonus

// Ability check bonuses
"bonusAbilityCheck": "+1" // All ability checks
"bonusSavingThrow": "+1"  // All saving throws

// Specific bonuses
"bonusProficiencyBonus": "+1" // Proficiency bonus increase
```

## Charges and Recharging

Items with limited uses per day:

```json
{
  "charges": "7",           // Number of charges
  "recharge": "dawn",       // When it recharges
  "rechargeAmount": "1d6+1" // How many charges restored
}

// Complex recharge
{
  "charges": "50",
  "recharge": "dawn",
  "rechargeAmount": {
    "dice": "4d6",
    "plus": 2
  },
  "rechargeFull": false    // Doesn't fully recharge
}

// Special recharge conditions
"recharge": "dawn"
"recharge": "dusk"
"recharge": "midnight"
"recharge": "special"     // See item description
```

## Weapon Properties

Magic weapons inherit base weapon properties plus:

```json
{
  "weapon": true,          // Marks as weapon
  "weaponCategory": "martial",
  "baseItem": "longsword|phb", // Base weapon reference
  "dmg1": "1d8",
  "dmgType": "S",
  "property": ["V"],
  
  // Magic properties
  "bonusWeapon": "+2",
  "dmgBonus": "+1d6 fire", // Extra damage
  "critThreshold": 19      // Improved critical range
}
```

### Versatile Magic Weapons

Items that can be multiple weapon types:

```json
{
  "type": "M|DMG",
  "baseItem": "longsword|phb|shortsword|phb|greatsword|phb",
  "entries": [
    "This magic weapon can take the form of any sword..."
  ]
}
```

## Armor Properties

Magic armor extends base armor with:

```json
{
  "armor": true,
  "ac": 14,                // Base AC
  "bonusAc": "+1",         // Magic bonus
  "baseItem": "plate armor|phb",
  "stealth": false,        // Removes stealth disadvantage
  "strength": null,        // Removes strength requirement
  "resist": ["fire"],      // Damage resistance granted
  "immune": ["poison"]     // Damage immunity granted
}
```

## Wondrous Item Properties

Wondrous items have diverse properties:

```json
{
  "wondrous": true,
  "weight": 1,
  "focus": ["Wizard", "Sorcerer", "Warlock"], // Spellcasting focus
  "ability": {
    "static": {
      "str": 19           // Sets ability score
    },
    "modifier": {
      "cha": "+2"         // Modifies ability score
    }
  }
}
```

## Cursed Items

Cursed items have special properties:

```json
{
  "curse": true,           // Item is cursed
  "entries": [
    "Normal description...",
    {
      "type": "entries",
      "name": "Curse",
      "entries": [
        "This armor is cursed. While wearing it..."
      ]
    }
  ]
}
```

## Sentient Items

Sentient magic items have personality:

```json
{
  "sentient": true,
  "entries": [
    "Item description...",
    {
      "type": "entries", 
      "name": "Sentience",
      "entries": [
        "The item is a sentient weapon with the following properties:",
        {
          "type": "list",
          "items": [
            "Intelligence 14, Wisdom 12, Charisma 18",
            "Hearing and darkvision to 120 feet",
            "Communicates telepathically",
            "Alignment: Chaotic Good"
          ]
        }
      ]
    }
  ]
}
```

## Artifacts

Artifacts have additional properties:

```json
{
  "rarity": "artifact",
  "artifact": true,
  "entries": [
    "Description...",
    {
      "type": "entries",
      "name": "Random Properties",
      "entries": [
        "The artifact has the following randomly determined properties:",
        "{@dice 2} minor beneficial properties",
        "{@dice 1} major beneficial property",
        "{@dice 1} minor detrimental property",
        "{@dice 1} major detrimental property"
      ]
    },
    {
      "type": "entries",
      "name": "Destroying the Artifact",
      "entries": [
        "The artifact can only be destroyed by..."
      ]
    }
  ]
}
```

## Item Sets

Items that are part of a set:

```json
{
  "itemGroup": "Instruments of the Bards",
  "entries": [
    "This item is one of the {@item Instruments of the Bards}..."
  ]
}
```

## Potions

Potions have special consumption properties:

```json
{
  "type": "P|DMG",
  "rarity": "common",
  "entries": [
    "When you drink this potion..."
  ],
  "miscTags": ["CNS"],     // Consumable tag
  "consume": true          // Destroyed when used
}
```

## Scrolls

Spell scrolls reference spells:

```json
{
  "type": "SC|DMG",
  "rarity": "varies",
  "entries": [
    "A spell scroll bears the words of a single spell..."
  ],
  "spells": ["{@spell fireball}"],
  "miscTags": ["CNS"],
  "consume": true
}
```

## Rods, Staffs, and Wands

Items that cast spells:

```json
{
  "type": "ST|DMG",        // Staff
  "charges": "10",
  "recharge": "dawn",
  "rechargeAmount": "1d6+4",
  "entries": [
    "This staff has 10 charges. While holding it, you can use an action to expend 1 or more charges to cast one of the following spells:",
    {
      "type": "list",
      "items": [
        "{@spell cure wounds} (1 charge per spell level, up to 4th)",
        "{@spell lesser restoration} (2 charges)",
        "{@spell mass cure wounds} (5 charges)"
      ]
    }
  ],
  "attachedSpells": [
    "cure wounds",
    "lesser restoration", 
    "mass cure wounds"
  ],
  "bonusSpellAttack": "+2",
  "bonusSpellSaveDc": "+2"
}
```

## Vehicles

Magic vehicles and mounts:

```json
{
  "type": "VEH|DMG",
  "speed": {
    "fly": 50
  },
  "carryingCapacity": 400,
  "entries": [
    "This wooden broom functions as a flying mount..."
  ]
}
```

## Container Items

Magic containers with special storage:

```json
{
  "type": "WND|DMG",
  "containerCapacity": {
    "weight": [500],       // Weight capacity
    "item": [64]          // Number of items
  },
  "entries": [
    "This bag has an interior space considerably larger than its outside dimensions..."
  ]
}
```

## Miscellaneous Tags

The `miscTags` field provides additional categorization:

```json
"miscTags": [
  "CNS",    // Consumable
  "CF/W",   // Creates Food/Water
  "HL",     // Healing
  "SCF",    // Spellcasting Focus
  "SUN",    // Sunlight generation
  "LGT",    // Light source
  "GNT",    // Giant-themed
  "DRG",    // Dragon-themed
  "FND",    // Fiend-themed
  "ELF",    // Elf-themed
  "DWF"     // Dwarf-themed
]
```

## Additional Entries

Extended descriptions for complex items:

```json
{
  "entries": [
    "Basic description..."
  ],
  "additionalEntries": [
    {
      "type": "entries",
      "name": "Awakened State",
      "entries": [
        "When the item reaches an awakened state..."
      ]
    },
    {
      "type": "entries",
      "name": "Exalted State",
      "entries": [
        "When the item reaches an exalted state..."
      ]
    }
  ]
}
```

## Variant Items

Items with multiple versions:

```json
{
  "variants": [
    {
      "name": "+1 Armor",
      "rarity": "rare",
      "bonusAc": "+1"
    },
    {
      "name": "+2 Armor",
      "rarity": "very rare",
      "bonusAc": "+2"
    },
    {
      "name": "+3 Armor",
      "rarity": "legendary",
      "bonusAc": "+3"
    }
  ]
}
```

## See Also References

Cross-references to related items:

```json
{
  "seeAlso": [
    "Sword of Answering|DMG",
    "Vorpal Sword|DMG"
  ],
  "seeAlsoVehicle": [
    "Carpet of Flying|DMG"
  ]
}
```

## Loot Tables

Reference to random treasure tables:

```json
{
  "lootTables": [
    "Magic Item Table A",
    "Magic Item Table F"
  ]
}
```

## Complete Magic Item Examples

### Simple Magic Weapon
```json
{
  "name": "+1 Longsword",
  "source": "DMG",
  "page": 213,
  "type": "M|DMG",
  "rarity": "uncommon",
  "reqAttune": false,
  "weight": 3,
  "value": 100000,
  "weapon": true,
  "weaponCategory": "martial",
  "baseItem": "longsword|phb",
  "dmg1": "1d8",
  "dmg2": "1d10",
  "dmgType": "S",
  "property": ["V"],
  "bonusWeapon": "+1",
  "entries": [
    "You have a +1 bonus to attack and damage rolls made with this magic weapon."
  ]
}
```

### Complex Wondrous Item
```json
{
  "name": "Bag of Holding",
  "source": "DMG",
  "page": 153,
  "type": "WND|DMG",
  "rarity": "uncommon",
  "reqAttune": false,
  "weight": 15,
  "value": 400000,
  "wondrous": true,
  "containerCapacity": {
    "weight": [500],
    "weightless": true
  },
  "entries": [
    "This bag has an interior space considerably larger than its outside dimensions, roughly 2 feet in diameter at the mouth and 4 feet deep. The bag can hold up to 500 pounds, not exceeding a volume of 64 cubic feet. The bag weighs 15 pounds, regardless of its contents. Retrieving an item from the bag requires an action.",
    "If the bag is overloaded, pierced, or torn, it ruptures and is destroyed, and its contents are scattered in the Astral Plane. If the bag is turned inside out, its contents spill forth, unharmed, but the bag must be put right before it can be used again. Breathing creatures inside the bag can survive up to a number of minutes equal to 10 divided by the number of creatures (minimum 1 minute), after which time they begin to suffocate.",
    "Placing a {@item bag of holding} inside an extradimensional space created by a {@item handy haversack}, {@item portable hole}, or similar item instantly destroys both items and opens a gate to the Astral Plane. The gate originates where the one item was placed inside the other. Any creature within 10 feet of the gate is sucked through it to a random location on the Astral Plane. The gate then closes. The gate is one-way only and can't be reopened."
  ]
}
```

### Staff with Charges
```json
{
  "name": "Staff of Healing",
  "source": "DMG",
  "page": 202,
  "type": "ST|DMG",
  "rarity": "rare",
  "reqAttune": "by a bard, cleric, or druid",
  "weight": 4,
  "value": 1300000,
  "charges": "10",
  "recharge": "dawn",
  "rechargeAmount": "1d6+4",
  "focus": ["Bard", "Cleric", "Druid"],
  "entries": [
    "This staff has 10 charges. While holding it, you can use an action to expend 1 or more of its charges to cast one of the following spells from it, using your spell save DC and spellcasting ability modifier:",
    {
      "type": "list",
      "items": [
        "{@spell cure wounds} (1 charge per spell level, up to 4th)",
        "{@spell lesser restoration} (2 charges)",
        "{@spell mass cure wounds} (5 charges)"
      ]
    },
    "The staff regains {@dice 1d6 + 4} expended charges daily at dawn. If you expend the last charge, roll a {@dice d20}. On a 1, the staff vanishes in a flash of light, lost forever."
  ],
  "attachedSpells": [
    "cure wounds",
    "lesser restoration",
    "mass cure wounds"
  ]
}
```

## Formatting Tags in Text

Magic item descriptions use 5etools formatting tags:

- `{@item bag of holding}` - Item references
- `{@spell fireball}` - Spell references
- `{@creature goblin}` - Creature references
- `{@damage 2d6}` - Damage rolls
- `{@dice 1d20}` - Dice rolls
- `{@dc 15}` - Difficulty classes
- `{@condition blinded}` - Conditions
- `{@action dash}` - Actions
- `{@sense darkvision}` - Senses
- `{@skill Perception}` - Skills
- `{@quickref Cover||3}` - Quick reference links

## Notes

- Magic items are distinguished from mundane items by having a `rarity` value other than `"none"`
- The `type` field always includes source suffix for magic items (e.g., `WND|DMG`)
- All numeric values should be treated as either integers or floats
- Boolean fields default to `false` when omitted
- Arrays default to empty `[]` when omitted
- Item references use the format `"item name|source"` for cross-linking
- Value is typically in copper pieces (100 cp = 1 gp)