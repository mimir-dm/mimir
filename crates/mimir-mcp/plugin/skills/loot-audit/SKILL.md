---
name: loot-audit
description: >-
  This skill should be used when the user asks to "audit loot",
  "check treasure distribution", "review magic items", "is loot balanced",
  "treasure by level", "too much gold", "not enough magic items",
  "wealth check", or mentions "treasure placement", "loot tables",
  or "magic item distribution". Audits treasure and magic item placement across
  modules and character inventories to flag wealth imbalances, rarity mismatches,
  and loot gaps.
---

# Loot & Treasure Audit

## Purpose

Analyze treasure and magic item distribution across the campaign to ensure appropriate wealth and power progression. Identify modules with no loot, excessive rewards, or missing item types.

## Analysis Process

### 1. Gather All Loot Data

```
list_modules()
# For each module:
get_module_details(module_id)
# Extract: module_items (loot), monsters (for hoard context)

# Check campaign-level documents for treasure references
list_documents()  # omit module_id for campaign-level docs

list_characters(character_type: "npc")
# For each NPC:
get_character(character_id)
get_character_inventory(character_id)
# Extract: inventory items that might be loot

list_characters(character_type: "pc")
# For each PC:
get_character_inventory(character_id)
# Extract: items already awarded to players
```

### 2. Catalog Items

For each item found:
```
search_items(name: item_name)
```

Extract rarity, type (weapon, armor, wondrous, etc.), attunement requirement, and value.

### 3. Distribution Analysis

Calculate:
- Total gold value by module
- Magic items by rarity
- Items by type
- Items requiring attunement

### 4. Identify Issues

| Issue | Description |
|-------|-------------|
| **Loot Desert** | Module has no treasure defined |
| **Gold Flood** | Far exceeds level-appropriate wealth |
| **Attunement Overload** | Too many attunement items (limit: 3) |
| **Type Gap** | No weapons, or no armor, or no caster items |
| **Rarity Mismatch** | Legendary item at level 3 |
| **Consumable Drought** | No potions/scrolls for resource recovery |

## Output Format

```markdown
# Treasure Audit: [Campaign Name]

## Summary
- Total modules: [X]
- Modules with loot: [Y]
- Total magic items: [Z]
- Estimated gold value: [N] gp

## Magic Item Distribution

### By Rarity
| Rarity | Count | Expected (Tier [X]) |
|--------|-------|---------------------|
| Common | [N] | [Expected] |
| Uncommon | [N] | [Expected] |
| Rare | [N] | [Expected] |
| Very Rare | [N] | [Expected] |
| Legendary | [N] | [Expected] |

### By Type
| Type | Count | Gap? |
|------|-------|------|
| Weapons | [N] | [Yes/No] |
| Armor | [N] | [Yes/No] |
| Wondrous | [N] | [Yes/No] |
| Consumables | [N] | [Yes/No] |

### Attunement Load
- Items requiring attunement: [N]
- If all found: [X] attunement slots needed
- Assessment: [OK / Overloaded]

## Module Breakdown

| Module | Gold | Magic Items | Issues |
|--------|------|-------------|--------|
| [Name] | [X] gp | [List] | [Issues] |

## Loot Deserts (No Treasure)
- [Module]: No loot defined
  - Suggestion: Add [appropriate items] based on CR [X] monsters

## Balance Concerns
- [Module]: [Issue description]
  - Suggestion: [Fix]

## Recommendations

### Missing Item Types
- Campaign needs more [type] items
- Suggested additions:
  - [Item] (search: `search_items(item_type: "X", rarity: "Y")`)

### Rarity Adjustments
- [Module] has [rarity] items too early
- Consider replacing with: [alternatives]
```

## Interactive Mode

1. Analyze full campaign loot
2. Present summary findings
3. For loot deserts, offer to search the catalog and add items via `add_item_to_module`
4. For balance issues, suggest specific swaps

## Reference Data

For gold-by-level guidelines and magic item tier expectations, see references/5e-treasure-guidelines.md.
