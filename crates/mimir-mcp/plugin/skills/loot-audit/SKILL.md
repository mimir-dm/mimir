# Loot & Treasure Audit

This skill should be used when the user asks to "audit loot", "check treasure distribution", "review magic items", "is loot balanced", "treasure by level", "too much gold", "not enough magic items", "wealth check", or mentions "treasure placement", "loot tables", or "magic item distribution".

## Purpose

Analyze treasure and magic item distribution across the campaign to ensure appropriate wealth and power progression. Identifies modules with no loot, excessive rewards, or missing item types.

## D&D 5e Treasure Guidelines

### Gold by Level (Individual)

| Level | Low | Typical | High |
|-------|-----|---------|------|
| 1-4 | 50-100 gp | 100-500 gp | 500-1,000 gp |
| 5-10 | 500-1,000 gp | 1,000-5,000 gp | 5,000-25,000 gp |
| 11-16 | 5,000-10,000 gp | 10,000-50,000 gp | 50,000-200,000 gp |
| 17-20 | 20,000-50,000 gp | 50,000-200,000 gp | 200,000+ gp |

### Magic Items by Tier

| Tier | Levels | Expected Items |
|------|--------|----------------|
| 1 | 1-4 | Few common/uncommon consumables |
| 2 | 5-10 | Uncommon permanent, some rare |
| 3 | 11-16 | Rare permanent, some very rare |
| 4 | 17-20 | Very rare/legendary |

### Item Type Distribution

Balanced parties need:
- Weapons for martial characters
- Armor/defensive items
- Spellcasting focuses/components
- Utility items
- Consumables (potions, scrolls)

## Analysis Process

### 1. Gather All Loot Data

```
list_modules()
# For each module:
get_module_details(module_id)
# Extract: module_items (loot), monsters (for hoard context)

list_characters(character_type: "npc")
# For each NPC:
get_character(character_id)
# Extract: inventory items that might be loot
```

### 2. Catalog Items

For each item found:
```
search_items(name: item_name)
```

Extract:
- Rarity
- Type (weapon, armor, wondrous, etc.)
- Attunement required?
- Value

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

## 🚨 Loot Deserts (No Treasure)
- [Module]: No loot defined
  - Suggestion: Add [appropriate items] based on CR [X] monsters

## ⚠️ Balance Concerns
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
3. For loot deserts, offer to:
   - Search catalog for appropriate items
   - Add items to modules via `add_item_to_module`
4. For balance issues, suggest specific swaps
