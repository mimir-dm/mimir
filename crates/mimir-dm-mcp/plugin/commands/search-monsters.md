---
name: search-monsters
description: Search the D&D monster catalog by various criteria
args:
  - name: query
    description: Monster name, type, or CR to search for
    required: false
---

Search the D&D monster catalog using the `search_monsters` MCP tool.

## Interpreting the Query

Parse the user's query to determine search parameters:

- **Name search**: "goblin", "dragon", "beholder" -> `name: "<query>"`
- **Type search**: "undead", "fiend", "humanoid" -> `creature_type: "<query>"`
- **CR search**: "CR 5", "challenge rating 3" -> `max_cr: <number>`
- **Combined**: "undead CR 5 or less" -> `creature_type: "undead", max_cr: 5`

If no query provided, ask what kind of monsters the user is looking for.

## Display Results

Present results in a table with:
- Monster name
- Type
- Challenge Rating
- Source book

Limit to 10-15 results. If more exist, mention the user can refine their search.

## Follow-up

After showing results, offer to:
- Add a monster to the current module
- Search with different criteria
- Show more details about a specific monster
