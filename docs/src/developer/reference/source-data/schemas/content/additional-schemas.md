# Additional Content Schemas Reference

Field references for additional content types in the extracted 5etools data format.

## Optional Features

### File Location
- Path: `{book}/optionalfeatures/{source}.json`

### JSON Structure
```json
{
  "optionalfeature": [
    {
      // optional feature objects
    }
  ]
}
```

### Core Fields
- `name` (string) - Feature name (e.g., "Archery", "Agonizing Blast")
- `source` (string) - Source book abbreviation
- `page` (number) - Page number
- `featureType` (array) - Feature categories (e.g., ["FS:F"] for Fighting Style: Fighter)
- `entries` (array) - Feature description
- `prerequisite` (array) - Requirements

### Feature Types
Common feature type codes:
- `"FS:F"` - Fighting Style (Fighter)
- `"FS:R"` - Fighting Style (Ranger)
- `"FS:P"` - Fighting Style (Paladin)
- `"EI"` - Eldritch Invocation
- `"MM"` - Metamagic
- `"MV"` - Maneuver (Battle Master)
- `"AS"` - Arcane Shot
- `"PB"` - Pact Boon
- `"AI"` - Artificer Infusion

### Example Entry
```json
{
  "name": "Defense",
  "source": "PHB",
  "page": 72,
  "featureType": ["FS:F", "FS:P", "FS:R"],
  "entries": [
    "While you are wearing armor, you gain a +1 bonus to AC."
  ]
}
```

## Actions

### File Location
- Path: `{book}/actions/{source}.json`

### JSON Structure
```json
{
  "action": [
    {
      // action objects
    }
  ]
}
```

### Core Fields
- `name` (string) - Action name (e.g., "Attack", "Dash", "Dodge")
- `source` (string) - Source book
- `page` (number) - Page reference
- `entries` (array) - Action description
- `time` (array) - Action timing (action, bonus action, reaction)
- `srd` (boolean) - In SRD
- `basicRules` (boolean) - In basic rules

### Action Types
```json
{
  "name": "Grapple",
  "source": "PHB",
  "page": 195,
  "srd": true,
  "basicRules": true,
  "time": [
    "Action"
  ],
  "entries": [
    "When you want to grab a creature or wrestle with it...",
    {
      "type": "entries",
      "name": "Grappling Rules",
      "entries": [
        "The target must be no more than one size larger...",
        "Using at least one free hand, you try to seize the target..."
      ]
    }
  ]
}
```

## Deities

### File Location
- Path: `{book}/deities/{source}.json`

### JSON Structure
```json
{
  "deity": [
    {
      // deity objects
    }
  ]
}
```

### Core Fields
- `name` (string) - Deity name
- `source` (string) - Source book
- `page` (number) - Page reference
- `alignment` (array) - Deity alignment(s)
- `domains` (array) - Cleric domains
- `symbol` (string) - Holy symbol description
- `pantheon` (string) - Pantheon membership
- `title` (string) - Deity titles
- `category` (string) - Deity category

### Example Deity
```json
{
  "name": "Bahamut",
  "source": "PHB",
  "page": 293,
  "alignment": ["L", "G"],
  "title": "god of good dragons",
  "domains": ["Life", "War"],
  "symbol": "Dragon's head in profile",
  "pantheon": "Draconic"
}
```

## Languages

### File Location
- Path: `{book}/languages/{source}.json`
- Fluff: `{book}/languages/fluff-{source}.json`

### JSON Structure
```json
{
  "language": [
    {
      // language objects
    }
  ]
}
```

### Core Fields
- `name` (string) - Language name
- `source` (string) - Source book
- `page` (number) - Page reference
- `type` (string) - Language type ("standard", "exotic", "secret")
- `typicalSpeakers` (array) - Common speakers
- `script` (string) - Writing system used
- `entries` (array) - Description

### Example Language
```json
{
  "name": "Dwarvish",
  "source": "PHB",
  "page": 123,
  "type": "standard",
  "typicalSpeakers": ["Dwarves"],
  "script": "Dwarvish",
  "entries": [
    "Dwarvish is full of hard consonants and guttural sounds..."
  ]
}
```

## Tables

### File Location
- Path: `{book}/tables/{source}.json`

### JSON Structure
```json
{
  "table": [
    {
      // table objects
    }
  ]
}
```

### Core Fields
- `name` (string) - Table name
- `source` (string) - Source book
- `page` (number) - Page reference
- `caption` (string) - Table caption
- `colLabels` (array) - Column headers
- `colStyles` (array) - Column CSS styles
- `rows` (array) - Table data rows
- `type` (string) - Table type

### Example Table
```json
{
  "name": "Character Advancement",
  "source": "PHB",
  "page": 15,
  "caption": "Character Advancement",
  "colLabels": [
    "Level",
    "Experience Points",
    "Proficiency Bonus"
  ],
  "colStyles": [
    "col-2 text-center",
    "col-5 text-center",
    "col-5 text-center"
  ],
  "rows": [
    ["1", "0", "+2"],
    ["2", "300", "+2"],
    ["3", "900", "+2"],
    ["4", "2,700", "+2"],
    ["5", "6,500", "+3"]
  ]
}
```

## Variant Rules

### File Location
- Path: `{book}/variantrules/{source}.json`

### JSON Structure
```json
{
  "variantrule": [
    {
      // variant rule objects
    }
  ]
}
```

### Core Fields
- `name` (string) - Rule name
- `source` (string) - Source book
- `page` (number) - Page reference
- `ruleType` (string) - Type of rule ("variant", "optional")
- `entries` (array) - Rule description

### Example Variant Rule
```json
{
  "name": "Feats",
  "source": "PHB",
  "page": 165,
  "ruleType": "variant",
  "entries": [
    "A feat represents a talent or an area of expertise...",
    {
      "type": "entries",
      "name": "Prerequisites",
      "entries": [
        "You must meet any prerequisite specified in a feat..."
      ]
    },
    {
      "type": "entries",
      "name": "Using Feats",
      "entries": [
        "At certain levels, your class gives you the Ability Score Improvement feature..."
      ]
    }
  ]
}
```

## Book Metadata

### File Location
- Path: `{book}/metadata.json`

### JSON Structure
```json
{
  "name": "Player's Handbook",
  "id": "PHB",
  "source": "PHB",
  "group": "core",
  "published": "2014-08-19",
  "author": "Wizards of the Coast",
  "contents": [
    {
      "name": "Introduction",
      "headers": ["Introduction"]
    },
    {
      "name": "Part 1: Creating a Character",
      "headers": ["Chapter 1: Step-by-Step Characters", "Chapter 2: Races", ...]
    }
  ]
}
```

## Book Content

### File Location
- Path: `{book}/book/book-{source}.json`

### JSON Structure
```json
{
  "book": [
    {
      "name": "Player's Handbook",
      "id": "PHB",
      "source": "PHB",
      "contents": [
        {
          "name": "Introduction",
          "headers": [],
          "ordinal": {
            "type": "chapter",
            "identifier": 0
          }
        }
      ]
    }
  ]
}
```

### Content Types
Book content includes:
- Chapters with headers
- Sections and subsections
- Tables of contents
- Appendices
- Indices

## Data Type Filtering

All content types support source filtering in the splitter:
- Primary source matching (`source` field)
- Additional sources (`otherSources` array)
- Copy references (`_copy` handling)
- Variant inclusion

## Cross-References

Content frequently includes cross-references:
- `{@spell spell name}` - Spell reference
- `{@item item name}` - Item reference
- `{@creature creature name}` - Creature reference
- `{@condition condition name}` - Condition reference
- `{@skill skill name}` - Skill reference
- `{@action action name}` - Action reference
- `{@background background name}` - Background reference
- `{@feat feat name}` - Feat reference
- `{@race race name}` - Race reference
- `{@class class name}` - Class reference
- `{@table table name}` - Table reference
- `{@book book id}` - Book reference

## Entry Types

Common entry types across all schemas:
- `"entries"` - Standard text entries
- `"list"` - Bulleted or numbered lists
- `"table"` - Tabular data
- `"quote"` - Quote blocks
- `"inset"` - Sidebar/inset boxes
- `"insetReadaloud"` - Read-aloud text boxes
- `"image"` - Image references
- `"gallery"` - Image galleries
- `"flowchart"` - Decision flowcharts
- `"section"` - Document sections