# Player Page

The player page is what a player link opens. A player sees the map, their own character and the party. They do not see the DM pages.

## Address

A player link has the form `<server>/play/<link>`. When a player opens it, the browser keeps the access and the address changes to `<server>/player`. The player comes back to `/player` later.

## Header

| Element | Function |
|---------|----------|
| Character name | The character of the link |
| Theme switch | Light, dark or hyper |
| **Sign out** | Forget the link in this browser |

## Tabs

### Display

The map that the DM shows, as on the [player display](./player-display.md). It has the same zoom, fit and full-screen buttons.

### Character

The sheet of the character. The player can:

- change the sheet (equipment, coins, spells, details)
- level up
- write, edit and delete notes in the **Notes** tab
- share the sheet with the party: **Details**, **Share this sheet with the party (read-only)**

A note written while the DM shows a map of a module is tagged with that module. The DM reads the notes. The PDF and **Source books…** buttons are for the DM only.

### Party

The player characters of the campaign.

| Element | Function |
|---------|----------|
| **You · shared** / **You · not shared** | The player's own character |
| **Shared** | The sheet of another player that they share. **View** opens it, read-only. |
| **Not shared** | A sheet that its player does not share |
| **Back to the party** | Close the sheet that you view |

A shared sheet shows without notes and without any button that changes it. When its player stops sharing, it closes and shows "… does not share their sheet now."

## When the Link Ends

When the DM revokes or replaces the link, the page signs out at once. It shows "This player link no longer works. Ask the DM for a new one."

## See Also

- [Share a Player Link](../../how-to/characters/player-links.md)
- [Characters and Character Sheet](./characters.md)
