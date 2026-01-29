# Create NPC Workflow

This example shows how to create and equip an NPC for your campaign.

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

## Step 3: Equip the NPC

```
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

## Step 4: Verify

```
get_character(character_id: "character-id")
```

This returns the full character with inventory and equipment status.
