# Character Creation Workflow

This example shows how to create a fully detailed PC from scratch.

## Step 1: Search the Catalog

Find exact names for race, class, and background:

```
search_races(name: "Half-Elf")
search_classes(name: "Warlock")
search_backgrounds(name: "Charlatan")
```

Use the exact names and sources returned by the catalog.

## Step 2: Create the Character

```
create_character(
  name: "Lyra Ashvane",
  character_type: "pc",
  race_name: "Half-Elf",
  class_name: "Warlock"
)
```

## Step 3: Set Race, Background, and Sources

```
edit_character(
  character_id: "character-id",
  race_name: "Half-Elf",
  race_source: "PHB",
  background_name: "Charlatan",
  background_source: "PHB"
)
```

## Step 4: Set Ability Scores and Currency

Ability scores are `[STR, DEX, CON, INT, WIS, CHA]`. Currency is `[CP, SP, EP, GP, PP]`.

```
edit_character(
  character_id: "character-id",
  ability_scores: [8, 14, 13, 12, 10, 16],
  currency: [0, 0, 0, 15, 0]
)
```

## Step 5: Equip the Character

Search for items first, then use exact names:

```
search_items(name: "Light Crossbow")

add_item_to_character(
  character_id: "character-id",
  item_name: "Light Crossbow",
  equipped: true
)

add_item_to_character(
  character_id: "character-id",
  item_name: "Leather Armor",
  equipped: true
)

add_item_to_character(
  character_id: "character-id",
  item_name: "Component Pouch",
  equipped: true
)
```

## Step 6: Verify

```
get_character(character_id: "character-id")
```

This returns the full character with race, class, background, ability scores, currency, and inventory.
