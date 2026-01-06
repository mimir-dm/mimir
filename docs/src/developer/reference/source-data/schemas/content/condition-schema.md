# Condition & Disease Schema Reference

Complete field reference for condition and disease entries in the extracted 5etools data format.

## File Location
- Conditions: `{book}/conditions/{source}.json`
- Diseases: `{book}/diseases/{source}.json`
- Fluff: `{book}/conditions/fluff-{source}.json`

## JSON Structure

### Conditions
```json
{
  "condition": [
    {
      // condition objects
    }
  ]
}
```

### Diseases
```json
{
  "disease": [
    {
      // disease objects
    }
  ]
}
```

## Condition Fields

### Core Fields
- `name` (string) - Condition name (e.g., "Blinded", "Charmed", "Exhaustion")
- `source` (string) - Source book abbreviation (PHB, DMG, etc.)
- `page` (number) - Page number in source book
- `entries` (array) - Condition effects and rules
- `srd` (boolean) - Available in System Reference Document
- `basicRules` (boolean) - Available in basic rules
- `hasFluffImages` (boolean) - Has associated artwork
- `otherSources` (array) - Additional source references

### Condition Effects
Conditions describe mechanical effects:

```json
{
  "name": "Poisoned",
  "source": "PHB",
  "page": 292,
  "srd": true,
  "basicRules": true,
  "entries": [
    {
      "type": "list",
      "items": [
        "A poisoned creature has disadvantage on attack rolls and ability checks."
      ]
    }
  ]
}
```

### Complex Conditions
Some conditions have multiple levels or complex effects:

```json
{
  "name": "Exhaustion",
  "source": "PHB",
  "page": 291,
  "srd": true,
  "basicRules": true,
  "entries": [
    "Some special abilities and environmental hazards...",
    {
      "type": "table",
      "colLabels": ["Level", "Effect"],
      "colStyles": ["col-1 text-center", "col-11"],
      "rows": [
        ["1", "Disadvantage on ability checks"],
        ["2", "Speed halved"],
        ["3", "Disadvantage on attack rolls and saving throws"],
        ["4", "Hit point maximum halved"],
        ["5", "Speed reduced to 0"],
        ["6", "Death"]
      ]
    },
    "If an already exhausted creature suffers another effect...",
    "An effect that removes exhaustion reduces its level...",
    "Finishing a long rest reduces a creature's exhaustion level by 1..."
  ]
}
```

## Disease Fields

### Core Disease Fields
- `name` (string) - Disease name
- `source` (string) - Source book
- `page` (number) - Page reference
- `entries` (array) - Disease description and effects

### Disease Structure
```json
{
  "name": "Cackle Fever",
  "source": "DMG",
  "page": 257,
  "entries": [
    "This disease targets humanoids, although gnomes are strangely immune...",
    {
      "type": "entries",
      "name": "Symptoms",
      "entries": [
        "Symptoms manifest {@dice 1d4} hours after infection...",
        "Any event that causes the infected creature great stress..."
      ]
    },
    {
      "type": "entries",
      "name": "Cure",
      "entries": [
        "At the end of each long rest, an infected creature..."
      ]
    }
  ]
}
```

## Standard Conditions List

### PHB Core Conditions
1. **Blinded** - Can't see, auto-fails sight checks
2. **Charmed** - Can't attack charmer, charmer has advantage on social checks
3. **Deafened** - Can't hear, auto-fails hearing checks
4. **Exhaustion** - Cumulative penalties (see above)
5. **Frightened** - Disadvantage while source in sight, can't move closer
6. **Grappled** - Speed becomes 0
7. **Incapacitated** - Can't take actions or reactions
8. **Invisible** - Unseen, attacks have advantage/disadvantage
9. **Paralyzed** - Incapacitated, can't move or speak, auto-fails Str/Dex saves
10. **Petrified** - Transformed to stone, weight ×10, stops aging
11. **Poisoned** - Disadvantage on attacks and checks
12. **Prone** - Disadvantage on attacks, melee attacks have advantage
13. **Restrained** - Speed 0, disadvantage on Dex saves and attacks
14. **Stunned** - Incapacitated, can't move, limited speech
15. **Unconscious** - Incapacitated, unaware, drops items, falls prone

## Condition Entries Format

### Simple Condition
```json
{
  "name": "Grappled",
  "source": "PHB",
  "page": 290,
  "srd": true,
  "basicRules": true,
  "entries": [
    {
      "type": "list",
      "items": [
        "A grappled creature's speed becomes 0, and it can't benefit from any bonus to its speed.",
        "The condition ends if the grappler is {@condition incapacitated}.",
        "The condition also ends if an effect removes the grappled creature from the reach of the grappler or grappling effect, such as when a creature is hurled away by the {@spell thunderwave} spell."
      ]
    }
  ]
}
```

### Complex Condition with Sub-effects
```json
{
  "name": "Paralyzed",
  "source": "PHB",
  "page": 291,
  "srd": true,
  "basicRules": true,
  "entries": [
    {
      "type": "list",
      "items": [
        "A paralyzed creature is {@condition incapacitated} and can't move or speak.",
        "The creature automatically fails Strength and Dexterity saving throws.",
        "Attack rolls against the creature have advantage.",
        "Any attack that hits the creature is a critical hit if the attacker is within 5 feet of the creature."
      ]
    }
  ]
}
```

## Disease Examples

### Simple Disease
```json
{
  "name": "Sewer Plague",
  "source": "DMG",
  "page": 257,
  "entries": [
    "Sewer plague is a generic term for a broad category of illnesses...",
    "A humanoid creature that is bitten by a creature that carries the disease...",
    "It takes {@dice 1d4} days for sewer plague's symptoms to manifest...",
    "At the end of each long rest, an infected creature must make a DC 11 Constitution saving throw..."
  ]
}
```

### Complex Disease with Stages
```json
{
  "name": "Sight Rot",
  "source": "DMG",
  "page": 257,
  "entries": [
    "This painful infection causes bleeding from the eyes...",
    "A beast or humanoid that drinks water tainted by sight rot...",
    "One day after infection, the creature's vision starts to become blurry...",
    {
      "type": "entries",
      "name": "Effects",
      "entries": [
        "The creature takes a -1 penalty to attack rolls and ability checks that rely on sight.",
        "At the end of each long rest after the symptoms appear..."
      ]
    },
    {
      "type": "entries",
      "name": "Magic",
      "entries": [
        "Sight rot can be cured using a rare flower called Eyebright...",
        "An herbalism kit can be used to turn the flower into one dose of ointment..."
      ]
    }
  ]
}
```

## Fluff Linkage

### Condition Fluff (`fluff-{source}.json`)
```json
{
  "conditionFluff": [
    {
      "name": "Blinded",
      "source": "PHB",
      "entries": [
        "The world becomes a dark and frightening place...",
        {
          "type": "entries",
          "name": "Effects in Combat",
          "entries": [
            "Blinded creatures must rely on other senses..."
          ]
        }
      ],
      "images": [
        {
          "type": "image",
          "href": {
            "type": "internal",
            "path": "conditions/PHB/Blinded.webp"
          }
        }
      ]
    }
  ]
}
```

## Cross-References
Conditions often reference each other:
- `{@condition incapacitated}` - Links to another condition
- `{@spell lesser restoration}` - Links to spells that remove conditions
- `{@creature medusa}` - Links to creatures that inflict conditions

## Example Complete Entry

```json
{
  "name": "Petrified",
  "source": "PHB",
  "page": 291,
  "srd": true,
  "basicRules": true,
  "entries": [
    {
      "type": "list",
      "items": [
        "A petrified creature is transformed, along with any nonmagical object it is wearing or carrying, into a solid inanimate substance (usually stone). Its weight increases by a factor of ten, and it ceases aging.",
        "The creature is {@condition incapacitated}, can't move or speak, and is unaware of its surroundings.",
        "Attack rolls against the creature have advantage.",
        "The creature automatically fails Strength and Dexterity saving throws.",
        "The creature has resistance to all damage.",
        "The creature is immune to poison and disease, although a poison or disease already in its system is suspended, not neutralized."
      ]
    }
  ],
  "hasFluffImages": true
}
```