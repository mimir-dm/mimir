# Your First Campaign

This tutorial shows you how to sign in to Mimir, make a campaign and import the catalog data. At the end you have a campaign that is ready for modules.

**Time:** 10 minutes

**You will learn how to:**
- Sign in and find your way around
- Make a campaign
- Use the campaign dashboard
- Import the catalog data for the next tutorials

## Prerequisites

- The address of your Mimir server, for example `https://mimir.example.ts.net`
- The DM token of the server, if it has one

## Step 1: Sign In

1. Open the server address in your browser.
2. If the server asks for a token, type the DM token in **Token**.
3. Click **Sign in**.

The browser keeps the token. The next visit opens Mimir directly. If the server has no token (open mode), Mimir opens without the sign-in page.

## Step 2: Find Your Way Around

The side navigation has five links:

- **Campaigns**: your campaigns. This is the start page.
- **Adventures**: the published and homebrew adventures. Read one, or make a campaign from it.
- **Characters**: every character, by campaign.
- **Reference**: the catalog and the source books.
- **Settings**: campaign archives, source books, the Claude integration and the version.

The header has the theme switch (light, dark, hyper) and, on a server with a token, **Sign out**.

## Step 3: Make a Campaign

1. Click **Campaigns**.
2. Click **New campaign**.
3. Type a **Name**, for example "Curse of Strahd".
4. Optionally, type a **Description**. Only you see it.
5. Click **Create**.

Mimir opens the new campaign. A new campaign starts with a set of planning documents: a campaign pitch, a world primer, safety tools and more.

## Step 4: Look at the Campaign Dashboard

The dashboard has the campaign name, a **More** menu and five tabs:

- **Campaign**: the campaign documents and the campaign maps
- **Modules**: the adventures of the campaign
- **NPCs**: the non-player characters
- **PCs**: the player characters and their player links
- **Homebrew**: your own items, monsters and spells

Click a document in the list to read it. Click **Edit** to change it. See the [Campaign Dashboard reference](../reference/ui/campaign-dashboard.md) for each tab.

## Step 5: Import Catalog Data

Mimir has no game content until you import it. Tutorial 2 searches the catalog for monsters, so import the data now.

1. Download a 5etools data archive (`.tar.gz`). See [Manage Source Books](../how-to/campaigns/manage-sources.md) for where to get one.
2. Click **Settings**, then the **Books** tab.
3. In **Import 5etools data**, choose the archive.
4. Click **Import**.
5. Wait for the message "Imported … sources". A full import takes a few minutes.

The books appear in the **Sources** table.

## What Is Next?

1. **[Make your first module](./02-first-module.md)**: an adventure with a map and monsters.
2. **[Make player characters](../how-to/characters/create-pc.md)** for your players.
3. **Look around the Reference**: search monsters, spells and items.

For more on this tutorial, see the [Campaign Dashboard reference](../reference/ui/campaign-dashboard.md) and [Create a Campaign](../how-to/campaigns/create-campaign.md).

---

*Next tutorial: [Your First Module](./02-first-module.md)*
