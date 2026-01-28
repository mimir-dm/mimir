---
description: Search the D&D monster catalog by various criteria
arguments:
  - name: query
    description: Monster name or type to search for
    required: false
  - name: cr
    description: Challenge rating (e.g., "1/4", "5", "10-15")
    required: false
allow_override: false
---

# Search Monster Catalog

Use `search_monsters` to find monsters matching the criteria.

Parse the arguments:
- If `query` looks like a creature type (undead, dragon, fiend, etc.), use `monster_type`
- Otherwise use `name` for partial matching
- If `cr` contains a dash, parse as cr_min and cr_max
- If `cr` is a single value, use it for both min and max

Display results in a table format:
| Name | CR | Type | Size | Source |

If searching within an active campaign, note that results are filtered to the campaign's enabled source books.

After showing results, offer to:
1. Add a monster to a module
2. Search with different criteria
3. Get detailed stats for a specific monster
