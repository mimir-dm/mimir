# Your First Campaign

This tutorial walks you through creating your first campaign in Mimir. By the end, you'll have a fully configured campaign ready for adventure modules and session play.

**Time to complete:** 5-10 minutes

**What you'll learn:**
- Navigate the Mimir interface
- Create a new campaign
- Explore the Campaign Dashboard
- Understand the four dashboard tabs

## Prerequisites

- Mimir installed and running
- A folder on your computer where you want to store campaign files

## Step 1: Launch Mimir

When you launch Mimir, you'll see the home screen with the floating Mimir skull and the tagline "Your arcane companion for D&D 5e campaign management."

![Mimir Home Screen](../images/tutorials/home-screen.png)

The header bar contains:
- **Mimir logo** (skull icon) - Click to return home
- **Campaign selector** - Switch between campaigns
- **Players** - Manage your player roster
- **Characters** - Create and manage PCs and NPCs
- **Reference** - Open the D&D 5e reference library
- **Settings** (gear icon) - Configure application preferences

## Step 2: Create a New Campaign

There are two ways to create a campaign:

### Option A: Using the Campaign Selector
1. Click the **Campaign Selector** dropdown in the header
2. Click **+ New Campaign**

### Option B: Direct Navigation
1. Navigate to `/campaigns/new` in the URL

Either method opens the **Create New Campaign** form.

## Step 3: Fill in Campaign Details

The campaign creation form has three fields:

### Campaign Name (Required)
Enter a descriptive name for your campaign. This appears throughout Mimir and in exported materials.

**Example:** "Curse of Strahd", "Homebrew - The Shattered Realms", "One-Shot: Goblin Heist"

### Description (Optional)
Add notes about your campaign concept, themes, or setting. This is for your reference only.

**Example:** "Gothic horror campaign set in the domain of Barovia. Players are trapped and must defeat the vampire Strahd von Zarovich."

### Campaign Directory Location (Required)
Choose where Mimir stores your campaign files (maps, documents, exports). Click **Browse...** to select a folder.

Mimir creates a subdirectory using a kebab-case version of your campaign name. For example:
- Campaign name: "Curse of Strahd"
- Directory created: `/your-path/curse-of-strahd/`

> **Tip:** Keep campaign folders organized in a parent "Campaigns" directory for easy backup and management.

## Step 4: Create the Campaign

Click **Create Campaign**. Mimir will:
1. Create the campaign directory structure
2. Initialize the campaign database entry
3. Redirect you to the Campaign Dashboard

## Step 5: Explore the Campaign Dashboard

The Campaign Dashboard is your command center for the entire campaign. It has a header showing your campaign name and four tabs for organizing different aspects of your game.

![Campaign Dashboard](../images/tutorials/campaign-dashboard.png)

### The Dashboard Tabs

#### Campaign Tab
The world-building hub. Use this tab for:
- Campaign setting notes
- Lore and history
- Factions and organizations
- World documents

This is where you develop the broader context of your campaign world.

#### Modules Tab
Adventure modules are self-contained adventures within your campaign. This tab shows:
- All modules in the campaign
- Module status (preparation stage)
- Quick actions to edit or play modules

You'll create your first module in the [next tutorial](./02-first-module.md).

#### NPCs Tab
Non-player characters for this campaign. Track:
- Major NPCs and villains
- Recurring characters
- NPC relationships and notes

NPCs can be shared across modules or specific to one adventure.

#### PCs Tab
Player characters in this campaign. Manage:
- Character assignments
- Player-character relationships
- Quick access to character sheets

Characters are created separately (in the Characters section) and assigned to campaigns here.

## Step 6: Campaign Actions

The dashboard header includes an **Export Archive** button. This creates a backup of your entire campaign including:
- All documents and notes
- Module data
- Maps and tokens
- Character assignments

Use this regularly for backups or when moving campaigns between computers.

## What's Next?

Your campaign is ready! Here are your next steps:

1. **[Create your first module](./02-first-module.md)** - Build an adventure with maps and encounters
2. **Add characters** - Create PCs for your players
3. **Explore the Reference** - Browse monsters, spells, and items for inspiration

---

## Quick Reference

| Action | How To |
|--------|--------|
| Create campaign | Campaign Selector → + New Campaign |
| Switch campaigns | Campaign Selector dropdown |
| Access dashboard | Click campaign name in selector |
| Export backup | Dashboard → Export Archive |
| Return home | Click Mimir skull icon |

---

*Next tutorial: [Your First Module](./02-first-module.md)*
