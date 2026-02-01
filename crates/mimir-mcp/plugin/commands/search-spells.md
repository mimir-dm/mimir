---
description: Search the D&D spell catalog by various criteria
arguments:
  - name: query
    description: Spell name to search for
    required: false
  - name: level
    description: Spell level (0 for cantrips, 1-9)
    required: false
  - name: school
    description: School of magic (e.g., evocation, necromancy)
    required: false
  - name: class
    description: Class spell list (e.g., wizard, cleric)
    required: false
allow_override: false
---

# Search Spell Catalog

Use `search_spells` to find spells matching the criteria.

Parse the arguments:
- If `query` is provided, use `name` for partial matching
- If `level` is provided, use as integer filter (0 = cantrips)
- If `school` is provided, use `school` filter
- If `class` is provided, use `class_name` filter

Display results in a table format:
| Name | Level | School | Source |

If searching within an active campaign, note that results are filtered to the campaign's enabled source books.

After showing results, offer to:
1. Get more details about a specific spell
2. Search with different criteria
3. Clone a spell as homebrew for customization
