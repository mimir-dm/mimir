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
The most flexible type. Make an item **from a template** with a form (name, kind, rarity, weight, value, attunement, description) or **copy one from the catalog**. Weapons and armor get more fields: damage, properties, armor class. The form writes 5etools JSON; the JSON tab shows it for fields the form does not have.

### Monsters
Copy a catalog monster, or start **from a template** (a simple humanoid). A stat block is a complex JSON structure, so the editor is the JSON with a live preview of the stat block.

### Spells
Same as monsters: copy one, or start from a template (a 1st-level evocation), then edit the JSON with a live preview.

## Copy from the Catalog

Copying is the fastest way to a good homebrew entry. Find a similar catalog entry and copy it. Mimir makes a homebrew copy at once, ready to edit, and the copy keeps its origin ("Copy of [original name]").

This is faster than starting from a template, and the data stays valid: a copied monster or spell has a well-formed stat block, because it started as one. For the steps, see [Create a Homebrew Monster](../how-to/homebrew/create-monster.md).

## Integration Points

Homebrew content isn't isolated — it flows into the rest of Mimir:

- **Items** appear in character inventory search results when adding equipment
- **Monsters** appear in the monster search of a module, and from there on its maps
- **Spells** stay in the Homebrew tab; character spell lists show catalog spells only

All homebrew entries have the source **HB**, and search results mark them **Homebrew**, so you can tell them from catalog content.

## Data Storage

Homebrew content is stored in the database of the server, per campaign. All three kinds store their data as 5etools JSON. This means:
- Items have a form editor that writes the JSON, and the JSON itself
- Monsters and spells use a JSON editor with a live preview
- All three kinds support copy from the catalog and a new entry from a template

## See Also

- [Homebrew How-To Guides](../how-to/homebrew/README.md)
- [The Two-Board System](./two-board-system.md)
