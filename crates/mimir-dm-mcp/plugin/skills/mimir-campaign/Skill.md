---
name: Mimir Campaign
description: D&D 5e campaign authoring with Mimir. Create modules, NPCs, documents, and manage character inventory using MCP tools.
---

# Mimir Campaign Authoring

Use this skill when helping users create and manage D&D 5e campaigns in Mimir. This skill provides tools for campaign management, module creation, NPC authoring, and document writing.

## Getting Started

**Always start by selecting a campaign:**

```
1. list_campaigns - See available campaigns
2. set_active_campaign(campaign_id) - Set the working campaign
```

All subsequent tools operate within the active campaign context.

## Available Tools

### Campaign Management
| Tool | Purpose |
|------|---------|
| `list_campaigns` | List all campaigns (use `include_archived: true` to see archived) |
| `set_active_campaign` | Set active campaign by ID (required before other operations) |

### Module Management
| Tool | Purpose |
|------|---------|
| `create_module` | Create a new module with auto-generated documents |
| `list_modules` | List modules (optional `status` filter: planning, active, completed) |
| `get_module_details` | Get module info with documents and NPCs |
| `add_monster_to_module` | Add a monster from catalog to module |
| `add_item_to_module` | Add an item from catalog to module |

### Document Authoring
| Tool | Purpose |
|------|---------|
| `list_documents` | List documents (filter by `level`: campaign/module, `module_id`) |
| `read_document` | Read document content by ID |
| `edit_document` | Update document content (search/replace within document) |

### Character & NPC Management
| Tool | Purpose |
|------|---------|
| `list_characters` | List characters (filter by `character_type`: pc/npc) |
| `get_character` | Get full character details |
| `create_npc` | Create an NPC with name, race, role, notes |
| `assign_npc_to_module` | Link NPC to module with role and optional encounter_tag |
| `add_item_to_character` | Add item to character inventory |
| `update_character_currency` | Update character's gold/silver/copper |

### Catalog Search
| Tool | Purpose |
|------|---------|
| `search_monsters` | Find monsters by name, type, CR range, source |
| `search_items` | Find items by name, type, rarity, source |
| `search_traps` | Find traps/hazards by name, category (Trap/Hazard), source |

## Common Workflows

### Creating a New Module

```
1. set_active_campaign(campaign_id)
2. create_module(name: "The Haunted Manor", module_type: "mystery")
   - Creates module with auto-generated documents (overview, prep notes, play notes)
3. list_documents(module_id: <new_module_id>)
4. edit_document to flesh out the overview
```

### Populating a Module with Encounters

```
1. search_monsters(creature_type: "undead", max_cr: 5)
2. add_monster_to_module(module_id, monster_name, source, quantity: 3, notes: "Guards the entrance")
3. search_traps(category: "Trap")
4. search_items(rarity: "uncommon") for treasure
5. add_item_to_module(module_id, item_name, source, quantity: 1, notes: "Hidden in chest")
```

### Creating NPCs for a Module

```
1. create_npc(name: "Garrett the Innkeeper", race: "Human", role: "quest_giver",
              location: "The Rusty Tankard", notes: "Knows about the haunted manor")
2. assign_npc_to_module(npc_id, module_id, role: "quest_giver")
```

### Giving Loot to Characters

```
1. list_characters(character_type: "pc")
2. search_items(name: "sword")
3. add_item_to_character(character_id, item_name, source, quantity: 1, notes: "Found in dungeon")
4. update_character_currency(character_id, gold: 50, silver: 30)
```

## Module Types

When creating modules, use these types for appropriate document templates:

- `mystery` - Investigation-focused adventures
- `dungeon` - Classic dungeon crawls
- `heist` - Stealth and planning adventures
- `horror` - Dark, atmospheric scenarios
- `political` - Intrigue and social encounters

## NPC Roles

When assigning NPCs to modules:

- `quest_giver` - Provides hooks and objectives
- `ally` - Helps the party
- `antagonist` - Opposes the party
- `neutral` - Could go either way
- `merchant` - Sells goods/services
- `informant` - Provides information

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
