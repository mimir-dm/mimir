# Create Module Workflow

This example shows how to create a new adventure module and populate it with content.

## Step 1: Set Active Campaign

```
set_active_campaign(campaign_id: "your-campaign-id")
```

## Step 2: Create the Module

```
create_module(
  name: "The Haunted Manor",
  description: "A spooky investigation adventure for levels 3-5"
)
```

## Step 3: Add Narrative Content

```
create_document(
  module_id: "module-id",
  title: "Manor History",
  document_type: "backstory",
  content: "The Blackwood Manor has stood empty for fifty years..."
)
```

## Step 4: Add Read-Aloud Text

```
create_document(
  module_id: "module-id",
  title: "Entering the Manor",
  document_type: "read_aloud",
  content: "As you push open the creaking doors, a chill wind rushes past..."
)
```

## Step 5: Add Monsters

```
search_catalog(category: "monster", name: "ghost", cr_max: 5)
add_monster_to_module(module_id: "module-id", monster_name: "Ghost", count: 1)
add_monster_to_module(module_id: "module-id", monster_name: "Shadow", count: 4)
```

## Step 6: Add Treasure

Module-level loot tracking is not implemented (`add_item_to_module` currently
returns an error). Record intended treasure in a DM-notes document, or hand it
to a character with `add_item_to_character`.

```
search_catalog(category: "item", rarity: "uncommon", item_type: "wondrous item")
create_document(
  module_id: "module-id",
  title: "Treasure",
  document_type: "dm_notes",
  content: "- Cloak of Protection (uncommon, wondrous item)"
)
```
