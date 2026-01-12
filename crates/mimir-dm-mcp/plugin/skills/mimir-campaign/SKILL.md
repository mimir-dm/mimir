---
name: Mimir Campaign
version: 1.0.0
description: This skill should be used when the user asks to "create a D&D campaign", "make a new module", "add an NPC", "create an encounter", "search for monsters", "find D&D items", "list campaigns", "set up a dungeon crawl", "create a mystery adventure", "add loot to characters", "search for traps", "manage character inventory", "build an adventure", "populate a dungeon", "give items to players", "create a villain", "add treasure", or mentions "Mimir campaign", "D&D 5e authoring", or "dungeon master tools". Provides MCP tools for campaign management, module creation, NPC authoring, and catalog searching.
---

# Mimir Campaign Authoring

This skill provides tools for creating and managing D&D 5e campaigns in Mimir, including campaign management, module creation, NPC authoring, and document writing.

## Getting Started

**Always start by selecting a campaign:**

1. `list_campaigns` - See available campaigns
2. `set_active_campaign(campaign_id)` - Set the working campaign

All subsequent tools operate within the active campaign context.

## Quick Reference

### Core Tools

| Category | Key Tools |
|----------|-----------|
| Campaign | `list_campaigns`, `set_active_campaign` |
| Modules | `create_module`, `list_modules`, `get_module_details` |
| Documents | `list_documents`, `read_document`, `edit_document` |
| NPCs | `create_npc`, `assign_npc_to_module`, `list_characters` |
| Catalog | `search_monsters`, `search_items`, `search_traps` |
| Inventory | `add_item_to_character`, `update_character_currency` |

See `references/tool-reference.md` for complete tool parameters and usage.

### Module Types

- `mystery` - Investigation-focused
- `dungeon` - Classic dungeon crawls
- `heist` - Stealth and planning
- `horror` - Dark, atmospheric
- `political` - Intrigue and social

See `references/module-types.md` for detailed guidance on each type.

### NPC Roles

`quest_giver`, `ally`, `antagonist`, `neutral`, `merchant`, `informant`

See `references/npc-roles.md` for role descriptions and usage.

## Common Workflows

### Create a Module
```
set_active_campaign(campaign_id)
create_module(name: "The Haunted Manor", module_type: "mystery")
list_documents(module_id: <new_module_id>)
edit_document to flesh out the overview
```

### Add Encounters
```
search_monsters(creature_type: "undead", max_cr: 5)
add_monster_to_module(module_id, monster_name, source, quantity: 3, notes: "Guards the entrance")
search_items(rarity: "uncommon") for treasure
add_item_to_module(module_id, item_name, source, notes: "Hidden in chest")
```

### Create NPCs
```
create_npc(name: "Garrett", race: "Human", role: "quest_giver", location: "The Rusty Tankard")
assign_npc_to_module(npc_id, module_id, role: "quest_giver")
```

### Give Loot to Characters
```
list_characters(character_type: "pc")
add_item_to_character(character_id, item_name, source, quantity: 1)
update_character_currency(character_id, gold: 50)
```

See `examples/` for complete workflow walkthroughs:
- `examples/dungeon-crawl.md` - Setting up a dungeon adventure
- `examples/mystery-adventure.md` - Creating an investigation module

## Best Practices

1. **Always set campaign first** - All tools require an active campaign context
2. **Use search before adding** - Search the catalog to find exact names/sources
3. **Add notes liberally** - Notes on monsters, items, and NPCs help during play
4. **Use encounter_tag** - Tag NPCs with encounter names for easy reference
5. **Check module details** - Use `get_module_details` to see what's already assigned

## Error Handling

- "No active campaign" - Call `set_active_campaign` first
- "Campaign not found" - Use `list_campaigns` to find valid IDs
- "Module not found" - Use `list_modules` to find valid module IDs
- "Monster/Item not found" - Check exact name and source with search tools
