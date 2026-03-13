# The Homebrew System

How Mimir handles custom content and why it works the way it does.

## Why Homebrew?

The D&D 5e catalog covers thousands of items, monsters, and spells — but every DM eventually needs something custom. A magic sword with unique properties, a modified monster, a spell variant. Mimir's homebrew system lets you create and manage these alongside official content.

## Campaign-Scoped

Homebrew content belongs to a specific campaign. This is intentional:
- A magic sword in your "Curse of Strahd" campaign doesn't clutter "Tomb of Annihilation"
- Different campaigns can have different homebrew with the same name
- Exporting a campaign includes all its homebrew

## Three Content Types

### Items
The most flexible type. Items can be created **from scratch** with a structured form (name, type, rarity, weight, value, description) or **cloned from the catalog**. Weapon and armor types get additional fields for damage, AC, and properties.

### Monsters
Created by **cloning from the catalog only** — there's no blank monster form. This is because monster stat blocks are complex JSON structures. Clone an existing monster and modify its data to create your variant.

### Spells
Same as monsters: **clone-only**. Spell data is structured JSON, so cloning provides a valid starting point that you can modify.

## Clone from Catalog

The clone workflow is the heart of the homebrew system:

1. Search the D&D 5e catalog for a similar entry
2. Click to clone — Mimir creates a homebrew copy immediately
3. Edit the copy to make your changes
4. The homebrew entry tracks its origin ("Based on [original name]")

This approach is faster than creating from scratch and ensures the data structure is valid.

## Integration Points

Homebrew content isn't isolated — it flows into the rest of Mimir:

- **Items** appear in character inventory search results when adding equipment
- **Monsters** appear in Token Setup when placing tokens on maps
- **Spells** appear in character spell lists for spellcasting classes

All homebrew entries are tagged with an **HB** badge so you can distinguish them from catalog content at a glance.

## Data Storage

Homebrew content is stored in the campaign database alongside catalog data. Items use structured fields (name, type, rarity, etc.) while monsters and spells store their full stat blocks as JSON. This means:
- Items have a user-friendly form editor
- Monsters and spells use a JSON text editor
- All three types support the clone-from-catalog workflow

## See Also

- [Homebrew How-To Guides](../how-to/homebrew/)
- [The Two-Board System](./two-board-system.md)
