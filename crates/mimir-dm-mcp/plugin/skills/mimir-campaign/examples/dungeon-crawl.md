# Example: Creating a Dungeon Crawl Module

Complete workflow for setting up a dungeon-style adventure.

## 1. Set Up the Module

```
set_active_campaign(campaign_id)

create_module(
  name: "The Sunken Crypt",
  module_type: "dungeon"
)
```

## 2. Populate with Monsters

Search for appropriate creatures:
```
search_monsters(creature_type: "undead", max_cr: 3)
```

Add them to the module:
```
add_monster_to_module(
  module_id: <module_id>,
  monster_name: "Skeleton",
  source: "MM",
  quantity: 6,
  notes: "Patrol the main hall in groups of 2"
)

add_monster_to_module(
  module_id: <module_id>,
  monster_name: "Ghoul",
  source: "MM",
  quantity: 2,
  notes: "Guard the treasure vault"
)

add_monster_to_module(
  module_id: <module_id>,
  monster_name: "Wight",
  source: "MM",
  quantity: 1,
  notes: "Boss encounter - former crypt guardian"
)
```

## 3. Add Hazards and Traps

```
search_traps(category: "Trap")

add_item_to_module(
  module_id: <module_id>,
  item_name: "Pit Trap",
  source: "DMG",
  notes: "Hidden in corridor before treasure room"
)
```

## 4. Place Treasure

```
search_items(rarity: "uncommon")

add_item_to_module(
  module_id: <module_id>,
  item_name: "Cloak of Protection",
  source: "DMG",
  quantity: 1,
  notes: "Worn by the Wight boss"
)

add_item_to_module(
  module_id: <module_id>,
  item_name: "Potion of Healing",
  source: "PHB",
  quantity: 3,
  notes: "Scattered in various rooms"
)
```

## 5. Add Supporting NPCs

```
create_npc(
  name: "Old Bartholomew",
  race: "Human",
  role: "quest_giver",
  location: "Village tavern",
  notes: "Lost his family heirloom in the crypt. Offers 100gp reward."
)

assign_npc_to_module(
  npc_id: <npc_id>,
  module_id: <module_id>,
  role: "quest_giver"
)
```

## 6. Flesh Out Documents

```
list_documents(module_id: <module_id>)

edit_document(
  document_id: <overview_doc_id>,
  search: "## Overview",
  replace: "## Overview\n\nThe Sunken Crypt is an ancient burial site that has become infested with undead. Local villagers report strange lights and sounds emanating from the hillside entrance."
)
```
