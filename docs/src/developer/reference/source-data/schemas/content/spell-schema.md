# Spell Schema Reference

Complete field reference for spell entries in the 5etools data format.

## File Location
- Path: `{book}/spells/spells-{source}.json`
- Fluff: `{book}/spells/fluff-spells-{source}.json`

## JSON Structure
```json
{
  "spell": [
    {
      // spell objects
    }
  ]
}
```

## Core Fields

### Required Fields
- `name` (string) - Spell name, must be unique within source
- `source` (string) - Source book abbreviation (PHB, DMG, etc.)
- `level` (number) - Spell level (0-9, where 0 = cantrip)
- `school` (string) - School of magic single-letter code
- `time` (array) - Casting time specification
- `range` (object) - Range specification  
- `components` (object) - V/S/M components
- `duration` (array) - Duration specification
- `entries` (array) - Main spell description

### Optional Core Fields
- `page` (number) - Page number in source book
- `otherSources` (array) - Additional source references
- `srd` (boolean/string) - Available in System Reference Document (SRD Name if the name is different)
- `basicRules` (boolean) - Available in basic rules

## School Codes
- `A` - Abjuration
- `C` - Conjuration  
- `D` - Divination
- `E` - Enchantment
- `V` - Evocation (note: V, not E)
- `I` - Illusion
- `N` - Necromancy
- `T` - Transmutation

## Casting Mechanics

### Casting Time (`time`)
Array of timing objects:

```json
// Standard actions
[{"number": 1, "unit": "action"}]
[{"number": 1, "unit": "bonus"}]
[{"number": 1, "unit": "reaction"}]

// Conditional reactions
[{"number": 1, "unit": "reaction", "condition": "which you take when you see a creature within 60 feet of you casting a spell"}]

// Extended casting
[{"number": 1, "unit": "minute"}]
[{"number": 10, "unit": "minute"}]
[{"number": 1, "unit": "hour"}]
[{"number": 8, "unit": "hour"}]
[{"number": 24, "unit": "hour"}]
```

### Range (`range`)
Object specifying targeting range:

```json
// Self-targeting
{"type": "point", "distance": {"type": "self"}}

// Touch spells  
{"type": "point", "distance": {"type": "touch"}}

// Ranged point target
{"type": "point", "distance": {"type": "feet", "amount": 60}}
{"type": "point", "distance": {"type": "miles", "amount": 1}}

// Area effects from caster
{"type": "radius", "distance": {"type": "feet", "amount": 20}}
{"type": "sphere", "distance": {"type": "feet", "amount": 40}}
{"type": "cone", "distance": {"type": "feet", "amount": 15}}
{"type": "line", "distance": {"type": "feet", "amount": 100}}

// Special ranges
{"type": "point", "distance": {"type": "sight"}}
{"type": "point", "distance": {"type": "unlimited"}}
```

### Components (`components`)
Object with V/S/M flags and material description:

```json
// Verbal only
{"v": true}

// Somatic only  
{"s": true}

// Verbal and somatic
{"v": true, "s": true}

// All components with material
{
  "v": true,
  "s": true, 
  "m": "a tiny ball of bat guano and sulfur"
}

// Complex material component
{
  "m": {
    "text": "a diamond worth at least 1,000 gp",
    "cost": 100000,
    "consume": true
  }
}
```

### Duration (`duration`)
Array of duration objects:

```json
// Instantaneous
[{"type": "instant"}]

// Timed durations
[{"type": "timed", "duration": {"amount": 1, "type": "round"}}]
[{"type": "timed", "duration": {"amount": 1, "type": "minute"}}]
[{"type": "timed", "duration": {"amount": 10, "type": "minute"}}]
[{"type": "timed", "duration": {"amount": 1, "type": "hour"}}]
[{"type": "timed", "duration": {"amount": 8, "type": "hour"}}]
[{"type": "timed", "duration": {"amount": 1, "type": "day"}}]

// Concentration spells
[{
  "type": "timed", 
  "duration": {"amount": 1, "type": "minute"}, 
  "concentration": true
}]

// Up to duration
[{
  "type": "timed",
  "duration": {"amount": 1, "type": "hour", "upTo": true}
}]

// Permanent effects
[{"type": "permanent", "ends": ["dispel"]}]
[{"type": "permanent", "ends": ["dispel", "trigger"]}]

// Special duration
[{"type": "special"}]
```

## Content and Description

### Main Description (`entries`)
Array of strings and complex objects:

```json
[
  "A bright streak flashes from your pointing finger to a point you choose within range and then blossoms with a low roar into an explosion of flame.",
  "The fire spreads around corners. It ignites flammable objects in the area that aren't being worn or carried."
]
```

### Higher Level Effects (`entriesHigherLevel`)
Array describing effects when cast with higher level slots:

```json
[
  {
    "type": "entries",
    "name": "At Higher Levels",
    "entries": [
      "When you cast this spell using a spell slot of 4th level or higher, the damage increases by {@scaledamage 8d6|3-9|1d6} for each slot level above 3rd."
    ]
  }
]
```

## Mechanical Effects

### Damage Types (`damageInflict`)
Array of damage types the spell can deal:

```json
["fire"]
["acid", "cold", "fire"]
["bludgeoning", "piercing", "slashing"]
["necrotic", "radiant"]
```

**All Damage Types:**
- Physical: `bludgeoning`, `piercing`, `slashing`
- Elemental: `acid`, `cold`, `fire`, `lightning`, `thunder`
- Energy: `force`, `necrotic`, `psychic`, `radiant`
- Biological: `poison`

### Conditions Applied (`conditionInflict`)
Array of conditions the spell can inflict:

```json
["blinded"]
["charmed", "incapacitated"]
["frightened", "stunned"]
```

**All Conditions:**
`blinded`, `charmed`, `deafened`, `frightened`, `grappled`, `incapacitated`, `invisible`, `paralyzed`, `petrified`, `poisoned`, `prone`, `restrained`, `stunned`, `unconscious`

### Saving Throws (`savingThrow`)
Array of required saving throws:

```json
["dexterity"]
["wisdom", "charisma"]
```

**Ability Names:** `strength`, `dexterity`, `constitution`, `intelligence`, `wisdom`, `charisma`

### Attack Types (`spellAttack`)
Array indicating attack roll types:

```json
["M"]  // Melee spell attack
["R"]  // Ranged spell attack
["O"]  // Other/special attack
```

## Targeting and Area Effects

### Area Tags (`areaTags`)
Array describing area of effect:

```json
["ST"]  // Single target
["MT"]  // Multiple targets
["S"]   // Sphere
["C"]   // Cone  
["L"]   // Line
["Y"]   // Cylinder
["H"]   // Hemisphere
["Q"]   // Square
["R"]   // Rectangle
["N"]   // Square/Rectangle
["W"]   // Wall
```

### Misc Tags (`miscTags`)
Array of miscellaneous mechanical properties:

```json
["SCL"]  // Scaling (cantrip damage increases)
["SGT"]  // Single target
["HL"]   // Has "At Higher Levels" section
["OBJ"]  // Can affect objects
["PRM"]  // Creates permanent effects
["SMN"]  // Summons creatures
["THP"]  // Grants temporary hit points
["UBA"]  // Can be used as bonus action
["LGT"]  // Creates light
["HEL"]  // Provides healing
["FMV"]  // Forced movement
["MAC"]  // Multiple attack rolls/checks
```

## Spell Lists and Availability

### Class Lists (`classes`)
Object specifying which classes can learn/prepare the spell:

```json
{
  "fromClassList": [
    {"name": "Sorcerer", "source": "PHB"},
    {"name": "Wizard", "source": "PHB"},
    {"name": "Light Domain", "source": "PHB"}
  ]
}
```

### Subclass Lists (`subclasses`)
Additional availability through subclasses:

```json
{
  "fromSubclass": [
    {
      "class": {"name": "Cleric", "source": "PHB"},
      "subclass": {"name": "Light Domain", "source": "PHB"}
    }
  ]
}
```

## Special Mechanics

### Ritual Casting (`meta`)
Object for special casting methods:

```json
{"ritual": true}  // Can be cast as ritual
```

### Cantrip Scaling (`scalingLevelDice`)
Object describing damage scaling for cantrips:

```json
{
  "label": "fire damage",
  "scaling": {
    "1": "1d10",
    "5": "2d10", 
    "11": "3d10",
    "17": "4d10"
  }
}
```

## Advanced Fields

### Ability Checks (`abilityCheck`)
Array of ability checks the spell might require:

```json
["strength"]
["intelligence", "wisdom"]
```

### Creature Type Interactions (`affectsCreatureType`)
Array of creature types affected by the spell:

```json
["beast", "humanoid"]
["undead"]
["aberration", "celestial", "elemental", "fey", "fiend"]
```

### Immunity Grants (`conditionImmune`, `damageImmune`, etc.)
Arrays of immunities/resistances granted:

```json
"conditionImmune": ["charmed", "frightened"]
"damageImmune": ["poison"]
"damageResist": ["bludgeoning", "piercing", "slashing"]
"damageVulnerable": ["cold"]
```

## Publication Information

### Availability Flags
- `srd` (boolean) - Available in System Reference Document
- `basicRules` (boolean) - Available in D&D Basic Rules
- `legacy` (boolean) - From legacy/outdated content

### Source References
- `otherSources` (array) - Additional publications containing this spell
- `reprintedAs` (array) - Where this spell was reprinted with changes

### Multimedia
- `hasFluffImages` (boolean) - Has associated artwork in fluff file
- `hasFluff` (boolean) - Has extended lore in fluff file

## Formatting Tags in Text

Spell descriptions use 5etools formatting tags:

- `{@damage 1d6}` - Damage rolls
- `{@condition blinded}` - Conditions  
- `{@spell fireball}` - Spell references
- `{@creature goblin}` - Creature references
- `{@item longsword}` - Item references
- `{@dice 1d20}` - Dice rolls
- `{@dc 15}` - Difficulty classes
- `{@scaledamage 1d6|1-9|1d6}` - Scaling damage
- `{@quickref Cover||3}` - Quick reference links

## Example Complete Spell

```json
{
  "name": "Fireball",
  "source": "PHB", 
  "page": 241,
  "level": 3,
  "school": "V",
  "time": [{"number": 1, "unit": "action"}],
  "range": {
    "type": "point",
    "distance": {"type": "feet", "amount": 150}
  },
  "components": {
    "v": true,
    "s": true,
    "m": "a tiny ball of bat guano and sulfur"
  },
  "duration": [{"type": "instant"}],
  "entries": [
    "A bright streak flashes from your pointing finger to a point you choose within range and then blossoms with a low roar into an explosion of flame. Each creature in a 20-foot-radius sphere centered on that point must make a Dexterity saving throw. A target takes {@damage 8d6} fire damage on a failed save, or half as much damage on a successful one.",
    "The fire spreads around corners. It ignites flammable objects in the area that aren't being worn or carried."
  ],
  "entriesHigherLevel": [
    {
      "type": "entries", 
      "name": "At Higher Levels",
      "entries": [
        "When you cast this spell using a spell slot of 4th level or higher, the damage increases by {@scaledamage 8d6|3-9|1d6} for each slot level above 3rd."
      ]
    }
  ],
  "damageInflict": ["fire"],
  "savingThrow": ["dexterity"], 
  "areaTags": ["S"],
  "miscTags": ["OBJ"],
  "classes": {
    "fromClassList": [
      {"name": "Sorcerer", "source": "PHB"},
      {"name": "Wizard", "source": "PHB"}
    ]
  },
  "srd": true,
  "basicRules": true,
  "hasFluffImages": true
}
```

This schema provides the complete specification for spell data as used in Mimir's D&D content system.