# Creating Homebrew Content

This tutorial shows you how to make your own items, monsters and spells for a campaign, and how to use them.

**Time:** 15 minutes

**You will learn how to:**
- Make a magic item with the item form
- Copy a catalog monster and change it
- Make a spell from the template
- Use homebrew in a module and on a character sheet

## Prerequisites

- A campaign ([Tutorial 1](./01-first-campaign.md)) with imported catalog data
- A module ([Tutorial 2](./02-first-module.md))
- A player character in the campaign ([Create a Player Character](../how-to/characters/create-pc.md))

## What Is Homebrew?

Homebrew is content that you make. In Mimir, homebrew belongs to one campaign. Your sword in one campaign does not show in another. Homebrew entries have the source **HB**.

## Step 1: Open the Homebrew Tab

1. Open your campaign.
2. Click the **Homebrew** tab.

The tab has three kinds: **Items**, **Monsters** and **Spells**. Each kind has a list at the side and the selected entry in the middle.

## Step 2: Make a Magic Item

1. Click **Items**.
2. Click **New**.
3. Fill in the form:
   - **Name**: "Frostbrand Falchion"
   - **Kind**: Weapon
   - **Rarity**: Rare
   - **Requires attunement**: on
   - **Category**: Martial
   - **Damage**: 2d4
   - **Damage type**: Slashing
   - **Weapon bonus**: +1
   - **Properties**: Finesse and Light
   - **Description**: "On a hit, the target takes an extra 1d6 cold damage."
4. Look at the **Preview**. It shows the item as the catalog shows it.
5. Click **Save**.

The item appears in the list.

The form writes 5etools JSON. Click the **JSON** tab to see it or to add fields that the form does not have. Click **Form** to go back.

## Step 3: Copy a Monster

A stat block has many fields. The easy way is to copy a catalog monster and change it.

1. Click **Monsters**.
2. Click **Copy from the catalog**.
3. In **Catalog name**, type "goblin" and click **Search**.
4. Click **Pick** next to **Goblin**.
5. In **Name**, type "Goblin Firestarter".
6. Click **Copy**.

The copy opens. It says "Copy of Goblin". To change it:

1. Click **Edit**.
2. Change the values in **Data (JSON)**, for example `"hp"` or `"ac"`. The preview updates as you type.
3. Click **Save**.

See [Create a Homebrew Monster](../how-to/homebrew/create-monster.md).

## Step 4: Make a Spell

1. Click **Spells**.
2. Click **New**. The JSON starts from a template: a 1st-level evocation.
3. Type the **Name**: "Freezing Hands".
4. Change the JSON, for example the level, the range and the text in `"entries"`.
5. Click **Save**.

You can also click **Copy from the catalog** to start from a catalog spell.

## Step 5: Use Your Homebrew

### Add the Monster to a Module

1. Open your module and click the **Dangers** tab.
2. In **Monster name**, type "firestarter" and click **Search**.
3. The result shows **Homebrew**. Click **Add**.

Now you can place the monster on the module maps from the **Place on the map** menu.

### Add the Item to a Character

1. Open the sheet of a character in the campaign.
2. Click the **Equipment** tab.
3. In **Add an item**, type "frostbrand" and click **Search**.
4. Click **Add** next to the item.

## How It Fits Together

```
Campaign: "The Frozen North"
├── Homebrew items: Frostbrand Falchion
├── Homebrew monsters: Goblin Firestarter
├── Homebrew spells: Freezing Hands
├── Module: "The Ice Caves"
│   └── Dangers: Goblin Firestarter × 3
└── Characters
    └── "Kira the Ranger"
        └── Inventory: Frostbrand Falchion (equipped, attuned)
```

Homebrew goes with the campaign when you export the campaign archive.

## What Is Next?

1. **[Create more items](../how-to/homebrew/create-item.md)**: all item fields, armor too.
2. **Browse the Reference**: find catalog entries to copy.
3. **Try a variant**: copy a strong monster and make a weaker one for low levels.

---

*Previous tutorial: [The Player Display](./04-player-display.md)*
