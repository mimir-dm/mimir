# Equipment Schema Documentation

This document defines the JSON schema for equipment and item data in the D&D 5etools format.

## Overview

The equipment schema represents all physical items in D&D including weapons, armor, tools, adventuring gear, magic items, and vehicles. Items are stored in JSON files with the root key `"item"` containing an array of item objects.

## Root Structure

```json
{
  "item": [
    {
      // Item objects
    }
  ]
}
```

## Core Item Object

Every item object contains these required fields:

```json
{
  "name": "string",           // Item name
  "source": "string",         // Source book abbreviation (PHB, DMG, etc.)
  "page": "number",           // Page number in source book  
  "type": "string",           // Item type code
  "rarity": "string"          // Item rarity level
}
```

### Basic Properties

```json
{
  "value": "number",          // Cost in copper pieces
  "weight": "number",         // Weight in pounds
  "weightNote": "string",     // Additional weight information (e.g., "(full)")
  "basicRules": "boolean",    // Available in D&D Basic Rules
  "srd": "boolean"            // Available in System Reference Document
}
```

### Item Descriptions

```json
{
  "entries": [               // Main item description
    "string",                // Text entries with {@tag} formatting
    {                        // Structured entries
      "type": "entries",
      "name": "string",
      "entries": ["string"]
    },
    {                        // Tables
      "type": "table",
      "caption": "string",
      "colLabels": ["string"],
      "rows": [["string"]]
    }
  ],
  "additionalEntries": [     // Extended descriptions (tools, magic items)
    // Same structure as entries
  ]
}
```

## Item Type Codes

The `type` field categorizes items using these standardized codes:

### Weapons
- `M` - Melee weapons
- `R` - Ranged weapons  
- `A` - Ammunition

### Armor & Protection
- `LA` - Light armor
- `MA` - Medium armor
- `HA` - Heavy armor
- `S` - Shields

### Tools & Equipment
- `AT` - Artisan's tools
- `TG` - Tool sets
- `INS` - Musical instruments
- `T` - Thieves' tools
- `G` - General adventuring gear
- `SCF` - Spellcasting focus

### Consumables & Supplies
- `FD` - Food and drink
- `TAH` - Trade goods
- `GS` - Gaming sets

### Transport
- `MNT` - Mounts
- `VEH` - Vehicles  
- `$C` - Currency

### Magic Items
Magic items use compound type codes like `RD|DMG` (Rod from DMG), `WND|PHB` (Wand from PHB), etc.

## Weapon Properties

Weapons have additional fields defining their combat characteristics:

```json
{
  "weapon": "boolean",        // Marks item as a weapon
  "weaponCategory": "string", // "simple" or "martial"
  "dmg1": "string",          // Primary damage dice (e.g. "1d8")
  "dmg2": "string",          // Versatile damage dice
  "dmgType": "string",       // Damage type code
  "range": "string",         // Range in "short/long" format (e.g. "20/60")
  "property": ["string"]     // Array of weapon property codes
}
```

### Damage Type Codes
- `P` - Piercing
- `B` - Bludgeoning  
- `S` - Slashing
- `A` - Acid
- `C` - Cold
- `F` - Fire
- `O` - Force
- `L` - Lightning
- `N` - Necrotic
- `I` - Poison
- `Y` - Psychic
- `R` - Radiant
- `T` - Thunder

### Weapon Property Codes
- `2H` - Two-handed
- `V` - Versatile
- `L` - Light
- `T` - Thrown
- `H` - Heavy
- `A` - Ammunition
- `F` - Finesse
- `R` - Reach
- `LD` - Loading
- `S` - Special

Properties can include additional data using pipe notation:
- `T|20/60` - Thrown (range 20/60)
- `V|1d10` - Versatile (1d10 damage)

### Weapon Category Tags

Weapons can be tagged with boolean flags for specific weapon types:

```json
{
  "axe": "boolean",
  "sword": "boolean", 
  "bow": "boolean",
  "crossbow": "boolean",
  "hammer": "boolean",
  "mace": "boolean",
  "spear": "boolean",
  "dagger": "boolean",
  "staff": "boolean",
  "club": "boolean",
  "polearm": "boolean",
  "lance": "boolean",
  "rapier": "boolean",
  "net": "boolean"
}
```

## Armor Properties

Armor items have these additional fields:

```json
{
  "armor": "boolean",         // Marks item as armor
  "ac": "number",            // Base armor class value
  "stealth": "boolean",      // True if armor causes stealth disadvantage
  "strength": "string"       // Minimum strength requirement (e.g. "13")
}
```

## Ammunition

Ammunition items link to their compatible weapons:

```json
{
  "ammoType": "string",      // Compatible weapon reference (e.g. "crossbow bolt|phb")
  "arrow": "boolean",        // Arrow ammunition
  "bolt": "boolean",         // Crossbow bolt ammunition  
  "bulletSling": "boolean",  // Sling bullet ammunition
  "needleBlowgun": "boolean" // Blowgun needle ammunition
}
```

## Spellcasting Focus

Items used as spellcasting focuses:

```json
{
  "scfType": "string",       // Focus type: "holy", "druidic", "arcane"
  "group": ["string"]        // Focus categories (e.g. ["Holy Symbol"])
}
```

## Container & Transport

Items that can carry other items or provide transportation:

```json
{
  "containerCapacity": {     // Storage capacity
    "weight": ["number"]     // Weight capacity in pounds
  },
  "carryingCapacity": "number", // Carrying capacity in pounds
  "speed": "number",           // Movement speed in feet
  "packContents": [            // Pre-defined contents for equipment packs
    "string",                  // Simple item reference
    {
      "item": "string",        // Item reference with source
      "quantity": "number"     // Number of items
    },
    {
      "special": "string"      // Special non-standard items
    }
  ]
}
```

## Light Sources

Items that produce light:

```json
{
  "light": [
    {
      "bright": "number",    // Bright light radius in feet
      "dim": "number",       // Dim light radius in feet  
      "shape": "string"      // Light shape: "cone", "sphere", etc.
    }
  ]
}
```

## Magic Item Properties

Magic items have additional fields for their magical effects:

```json
{
  "reqAttune": "string|boolean",     // Attunement requirement text or true/false
  "reqAttuneTags": [                 // Structured attunement requirements
    {
      "class": "string"              // Required class name
    }
  ],
  "tier": "string",                  // Magic item tier: "minor", "major"
  "bonusSpellAttack": "string",      // Spell attack roll bonus (e.g. "+1")
  "bonusSpellSaveDc": "string",      // Spell save DC bonus (e.g. "+1") 
  "bonusAc": "number",               // AC bonus for armor/shields
  "lootTables": ["string"]           // Associated random loot tables
}
```

## Miscellaneous Properties

Additional optional fields for special item features:

```json
{
  "miscTags": ["string"],        // Item classification tags
  "hasFluffImages": "boolean",   // Item has associated artwork
  "poison": "boolean",           // Item is poisonous
  "atomicPackContents": "boolean" // Pack contents cannot be separated
}
```

### Common Misc Tags
- `CNS` - Consumable item

## Source References

Cross-references to additional source books:

```json
{
  "additionalSources": [
    {
      "source": "string",        // Source book abbreviation
      "page": "number"           // Page number in that source
    }
  ]
}
```

## Rarity Values

The `rarity` field uses these standardized values:
- `none` - Mundane, non-magical items
- `common` - Common magic items
- `uncommon` - Uncommon magic items  
- `rare` - Rare magic items
- `very rare` - Very rare magic items
- `legendary` - Legendary magic items
- `artifact` - Artifact-level items

## Complete Item Example

```json
{
  "name": "+1 Longsword",
  "source": "DMG", 
  "page": 213,
  "type": "M",
  "rarity": "uncommon",
  "value": 150000,
  "weight": 3,
  "weapon": true,
  "weaponCategory": "martial",
  "dmg1": "1d8",
  "dmg2": "1d10", 
  "dmgType": "S",
  "property": ["V"],
  "sword": true,
  "reqAttune": false,
  "tier": "major",
  "bonusWeaponAttack": 1,
  "entries": [
    "You have a +1 bonus to attack and damage rolls made with this magic weapon."
  ]
}
```

## Notes

- All numeric values should be treated as either integers or floats
- Text fields support D&D formatting tags (see [Text Formatting](./text-formatting.md))
- Item references use the format `"item name|source"` for cross-linking
- Boolean fields default to `false` when omitted
- Arrays default to empty `[]` when omitted