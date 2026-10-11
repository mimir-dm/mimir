# Campaign Dashboard

The campaign dashboard (`/campaigns/<id>`) holds everything of one campaign. Its tabs are in the address, so a link opens the same tab.

## Header

| Element | Function |
|---------|----------|
| Campaign name | The title. **Archived** shows next to it for an archived campaign. |
| **Edit campaign** | Change the **Name** and the **Description** |
| **More** | The campaign menu (below) |

### More Menu

| Item | Function |
|------|----------|
| **Source books…** | Choose the books that the catalog searches of the campaign use |
| **PDF…** | Export the campaign to PDF. See [PDF Export](../pdf-export/README.md). |
| **Export to a file** | Download the campaign archive (`.mimir-campaign.tar.gz`) |
| **Archive** | Hide the campaign from the list (active campaigns) |
| **Restore** | Make an archived campaign active again |
| **Delete…** | Delete an archived campaign, after you type its name |

## Tabs

### Campaign

The campaign documents and the campaign maps.

- **Documents**: the document list on the left and the open document on the right. **New document** adds one, from a markdown file or an image too. **Images** lists the images; a click opens one big, with **Delete image**. See [Manage Documents](../../how-to/campaigns/manage-documents.md).
- **Maps**: the campaign maps, for places that many modules use. **Upload a map** adds one (drop or choose the file). **Preview** shows a map's image, size and grid, with **Open** and **PDF…**. Click a name to open the [DM map](./dm-map.md).

### Modules

A table of the modules.

| Column | Contents |
|--------|----------|
| **#** | The module number |
| **Module** | The name. Click it to open the [module page](./module-page.md). |
| **Description** | The description |
| **Order** | Up and down arrows to change the order |

**New module** makes a module: **Name**, **Type**, **Description**.

### NPCs

A card for each NPC of the campaign, with its role, location and faction, **View** and **PDF…**. Click a name to open the sheet. **New NPC** makes one.

### PCs

A card for each player character, with its player, race and class, **View**, **PDF…** and **Player link**. Click a name to open the sheet. **New PC** opens the character wizard.

**Player link** opens the link panel of a PC:

| Element | Function |
|---------|----------|
| Status | "No link yet." or "A link is active (made …)." |
| **Make a link** / **New link** | Make a link. A new link ends the old one, after a confirmation. |
| Link and QR code | Shown once, after you make a link |
| **Copy link** | Copy the link |
| **Revoke** | End the link. The device of the player is signed out at once. |

See [Share a Player Link](../../how-to/characters/player-links.md).

### Homebrew

The homebrew items, monsters and spells of the campaign. The kinds are tabs: **Items**, **Monsters**, **Spells**. Each kind has a list on the left and the selected entry on the right.

| Element | Function |
|---------|----------|
| **New** | Make an entry from a template |
| **Copy from the catalog** | Copy a catalog entry |
| **Edit** | Change the selected entry |
| **×** | Delete the selected entry, after a check of what uses it |

See [Homebrew Content](../../how-to/homebrew/README.md).

## See Also

- [Module Page](./module-page.md)
- [Manage Documents](../../how-to/campaigns/manage-documents.md)
