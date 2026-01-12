# Example: Creating a Mystery Adventure

Complete workflow for setting up an investigation-style adventure.

## 1. Set Up the Module

```
set_active_campaign(campaign_id)

create_module(
  name: "Murder at Blackwood Manor",
  module_type: "mystery"
)
```

## 2. Create the Cast of NPCs

### The Victim (for backstory)
```
create_npc(
  name: "Lord Edmund Blackwood",
  race: "Human",
  role: "neutral",
  location: "Blackwood Manor (deceased)",
  notes: "Wealthy noble found dead in his study. Had many enemies."
)
```

### Suspects
```
create_npc(
  name: "Lady Isabelle Blackwood",
  race: "Human",
  role: "neutral",
  location: "Blackwood Manor",
  notes: "The wife. Stands to inherit everything. Alibi: was at a social gathering."
)

create_npc(
  name: "Victor Blackwood",
  race: "Human",
  role: "neutral",
  location: "Blackwood Manor",
  notes: "The estranged son. Recently returned asking for money. Alibi: claims he was in town."
)

create_npc(
  name: "Grimsby",
  race: "Human",
  role: "informant",
  location: "Blackwood Manor",
  notes: "The butler. Knows all the family secrets. Saw something that night."
)
```

### The Real Killer (antagonist)
```
create_npc(
  name: "Cassandra Vane",
  race: "Human",
  role: "antagonist",
  location: "Blackwood Manor",
  notes: "The governess. Secret lover of Lord Blackwood who was being cast aside. THE KILLER."
)
```

## 3. Assign NPCs to Module

```
assign_npc_to_module(npc_id: <lady_isabelle_id>, module_id: <module_id>, role: "neutral", encounter_tag: "suspect_interview")
assign_npc_to_module(npc_id: <victor_id>, module_id: <module_id>, role: "neutral", encounter_tag: "suspect_interview")
assign_npc_to_module(npc_id: <grimsby_id>, module_id: <module_id>, role: "informant", encounter_tag: "key_witness")
assign_npc_to_module(npc_id: <cassandra_id>, module_id: <module_id>, role: "antagonist", encounter_tag: "reveal")
```

## 4. Add Clue Items

```
search_items(name: "letter")

add_item_to_module(
  module_id: <module_id>,
  item_name: "Letter",
  source: "PHB",
  notes: "CLUE: Love letter from Cassandra to Edmund, hidden in desk"
)

add_item_to_module(
  module_id: <module_id>,
  item_name: "Dagger",
  source: "PHB",
  notes: "CLUE: Murder weapon. Has initials C.V. on the handle"
)
```

## 5. Document the Mystery Structure

```
list_documents(module_id: <module_id>)

edit_document(
  document_id: <overview_doc_id>,
  search: "## Overview",
  replace: "## Overview\n\nLord Edmund Blackwood has been murdered in his study. The party is hired to investigate. Everyone has motive, but only one had opportunity.\n\n## The Truth\nCassandra Vane, the governess and Edmund's secret lover, killed him when he threatened to end their affair and dismiss her without reference.\n\n## Clues\n1. Love letter (DC 15 Investigation in study)\n2. Dagger with initials (DC 12 Perception at crime scene)\n3. Grimsby's testimony (DC 14 Persuasion)\n4. Cassandra's alibi has holes (DC 16 Insight)"
)
```
