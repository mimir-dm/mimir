# Creating Homebrew Content

This tutorial walks you through creating custom items, monsters, and spells for your campaign. By the end, you'll have homebrew content integrated into your modules and character sheets.

**Time to complete:** 10-15 minutes

**What you'll learn:**
- Create a custom magic item from scratch
- Clone a catalog monster and modify it
- Clone a catalog spell and customize it
- Use homebrew content in modules and on characters

## Prerequisites

- A campaign created ([Tutorial 1](./01-first-campaign.md))
- Catalog data imported ([Manage Campaign Sources](../how-to/campaigns/manage-sources.md)) — monsters, items, and spells should appear in Reference searches
- A module with a map ([Tutorial 2](./02-first-module.md)) — Step 5 places the homebrew monster via Token Setup
- A PC [created](../how-to/characters/create-pc.md) and [assigned to the campaign](../how-to/characters/assign-to-campaign.md) — Step 5 adds the item and spell to a character sheet; the spell step needs a spellcaster

## What is Homebrew?

In D&D, "homebrew" means custom content created by the DM. Mimir's homebrew system lets you create items, monsters, and spells that live alongside the official catalog data. Homebrew content is scoped to a campaign — your custom sword in one campaign doesn't appear in another.

## Step 1: Open the Homebrew Tab

1. Open your campaign from the Campaign Selector
2. Click the **Homebrew** tab in the dashboard

You'll see three sub-tabs: **Items**, **Monsters**, and **Spells**.

## Step 2: Create a Custom Magic Item

Let's create a unique magic sword for your campaign.

1. Click the **Items** sub-tab
2. Fill in the item form on the right panel:
   - **Name:** "Frostbrand Falchion"
   - **Item Type:** weapon
   - **Rarity:** rare
   - **Requires Attunement:** checked

3. The form expands to show weapon-specific fields:
   - **Category:** Martial
   - **Bonus:** +1
   - **Damage:** 2d4
   - **Damage Type:** Slashing
   - **Properties:** Select "Finesse" and "Light"

4. In the **Description** field, write:
   > This curved blade is forged from ice-steel mined in the Frostfell. On a hit, the target takes an additional 1d6 cold damage. While attuned, you have resistance to fire damage.

5. Click **Create Item**

Your item appears in the homebrew items list with an **HB** badge.

## Step 3: Create a Custom Monster

Monster stat blocks are complex, so Mimir uses a clone-only approach: you start from a catalog monster and modify it.

1. Click the **Monsters** sub-tab
2. Click **Clone from Catalog**
3. Search for "Goblin" — we'll create a goblin variant
4. Click "Goblin" in the results

The cloned monster appears in your list immediately. The detail view shows "Based on Goblin."

5. Select the new monster and click **Edit**
6. Change the **Name** to "Goblin Firestarter"
7. In the JSON data, you can modify:
   - Hit points
   - Armor class
   - Actions and abilities
   - Challenge rating

8. Save your changes

> **Tip:** The stat block is edited as JSON in the 5etools data format, but you don't need to understand the full structure to make simple changes — find the field you want (like `"hp"` or `"ac"`) and update its value. See [Create a Homebrew Monster](../how-to/homebrew/create-monster.md) and [The Homebrew System](../explanation/homebrew-system.md) for how this data is structured and stored.

## Step 4: Create a Custom Spell

Like monsters, spells are cloned from the catalog.

1. Click the **Spells** sub-tab
2. Click **Clone from Catalog**
3. Search for "Burning Hands"
4. Click it to create a homebrew copy
5. Select the new spell and click **Edit**
6. Change the **Name** to "Freezing Hands"
7. In the JSON data, change:
   - The damage type from fire to cold
   - Update the description text accordingly
8. Save your changes

Spell data uses the same JSON editing approach as monsters — see [Create a Homebrew Spell](../how-to/homebrew/create-spell.md) for the details.

## Step 5: Use Homebrew in Your Module

Now let's use the homebrew content you've created.

### Add the Monster to a Module

1. Go to the **Modules** tab and open a module
2. Click a map to open **Token Setup**
3. In the Monster palette, search for "Goblin Firestarter"
4. Your homebrew monster appears in the results with an **HB** badge
5. Click it and place the token on the map

### Add the Item to a Character

1. Go to the **PCs** tab and open a character sheet
2. Navigate to the **Equipment** tab
3. Click **Add Item**
4. Search for "Frostbrand Falchion"
5. Your homebrew item appears with an **HB** badge
6. Add it to the character's inventory

### Add the Spell to a Character

1. On a spellcasting character's sheet, go to the **Spells** tab
2. Click **Add Spell**
3. Search for "Freezing Hands"
4. Your homebrew spell appears alongside catalog spells
5. Add it to the character's known spells

## How It All Fits Together

```
Campaign: "The Frozen North"
├── Homebrew Items: Frostbrand Falchion
├── Homebrew Monsters: Goblin Firestarter
├── Homebrew Spells: Freezing Hands
├── Module: "The Ice Caves"
│   └── Tokens: Goblin Firestarter × 3
└── Characters
    └── "Kira the Ranger"
        ├── Inventory: Frostbrand Falchion (equipped, attuned)
        └── Spells: Freezing Hands (prepared)
```

Homebrew content is campaign-scoped, so it travels with your campaign when you export it as an archive.

## What's Next?

You've learned the three homebrew workflows. Continue exploring:

1. **[Create more items](../how-to/homebrew/create-item.md)** — Full item field reference including armor types
2. **Browse the catalog** — Search Reference for inspiration on what to clone
3. **Experiment** — Clone a high-CR monster, reduce its stats, and create a weakened variant for lower-level play

---

*Previous tutorial: [Player Display Setup](./04-player-display.md)*
