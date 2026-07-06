# Create NPC Workflow

This example shows how to create and fully equip an NPC for your campaign.

## Step 1: Create the NPC

```
create_character(
  name: "Captain Harken",
  character_type: "npc",
  race_name: "Human"
)
```

## Step 2: Set Role and Location

```
edit_character(
  character_id: "character-id",
  npc_role: "City Watch Captain",
  npc_location: "Waterdeep - Castle Ward"
)
```

## Step 3: Set Ability Scores and Currency

Set each score and coin with its own named integer parameter — `edit_character`
has no `ability_scores`/`currency` array (an array is silently ignored). Provide
only the fields you want to change.

```
edit_character(
  character_id: "character-id",
  strength: 16, dexterity: 12, constitution: 14,
  intelligence: 10, wisdom: 13, charisma: 11,
  gp: 50
)
```

## Step 4: Set Race and Background (Optional)

Search the catalog first to find exact names and sources:

```
search_catalog(category: "race", name: "Human")
search_catalog(category: "background", name: "Soldier")

edit_character(
  character_id: "character-id",
  race_name: "Human",
  race_source: "PHB",
  background_name: "Soldier",
  background_source: "PHB"
)
```

## Step 5: Equip the NPC

Search for items first, then use the exact name from results:

```
search_catalog(category: "item", name: "Longsword")

add_item_to_character(
  character_id: "character-id",
  item_name: "Longsword",
  equipped: true
)

add_item_to_character(
  character_id: "character-id",
  item_name: "Chain Mail",
  equipped: true
)

add_item_to_character(
  character_id: "character-id",
  item_name: "Shield",
  equipped: true
)
```

## Step 6: Verify

```
get_character(character_id: "character-id")
```

This returns the full character with ability scores, currency, inventory, and equipment status.
