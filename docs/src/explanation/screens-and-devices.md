# Screens and Devices

How the DM screen, the player display, the player pages and the reference relate when Mimir runs in the browser.

## The Problem

At the table, the DM needs full control of maps, monsters and notes. The players must see only what their characters see. One screen cannot do both. Players also want their own character in hand, on their own phone or tablet.

## One Server, Many Pages

Mimir is one server. Every screen is a page of the same web app, in any browser that reaches the server. A page is an address, so you open as many as you need, in tabs or on other devices.

| Page | Address | Who looks at it |
|------|---------|-----------------|
| DM pages (dashboard, module, characters, settings) | `/`, `/campaigns/…`, `/modules/…` | The DM |
| DM map | `/maps/<id>` | The DM, during the session |
| Player display | `/display/<campaign>` | The table, on a TV or projector |
| Player page | `/player` (from a player link) | One player, on their own device |
| Reference | `/reference`, and the detail drawer on every DM page | The DM |

### DM Pages

Campaign management, character sheets, module prep and settings. They share the side navigation.

### DM Map

The map with every token, the fog controls, the markers and the initiative tracker. The DM keeps it on a laptop or a tablet.

### Player Display

The map as the players see it. It has no app frame and no DM controls. The server sends it only the player view: hidden tokens, unlit lights, hidden markers and trap details never reach it. Anyone at the display can zoom and pan its own view.

### Player Page

A player link opens the same display for one player, together with their own character sheet and the party. It is a smaller app with no DM pages. The server checks each request against the character of the link.

### Reference

The catalog and the books are a page of their own. The detail drawer of a catalog entry also opens over any DM page, for example from the monsters of a module. The DM keeps the reference in a second tab beside the map.

## A Typical Setup

- **DM laptop**: the DM map in one tab, the module page with **Play Notes** in another.
- **TV**: the player display, in full screen, from a small computer or the browser of the TV.
- **Players' phones**: their player pages, from their player links.

For remote play, the DM shares the display tab in a video call, or each player uses their link.

## How the Pages Stay in Step

### One Database

All pages read and write through the server, and the server keeps one database. When the DM moves a token, the server saves the new place first.

### The Live Socket

Each page holds a WebSocket to the server at `/ws`. When something changes, the server tells each page what it may know:

- A DM page gets a notice and reads the changed part again.
- A player display or player page gets the new player view of the shown map.

So a moved token, a change of fog, a new map, a blackout or a change of the turn order shows on every screen at once. A player's change to their sheet shows on the DM's open sheet at once. When the DM revokes a player link, the player page signs out at once.

### Display State

Which map the display shows, the blackout and the turn order on the display live in the server's memory. A server restart clears them. Everything else is saved.

## Blackout

The DM can hide the display with one click. The display shows "Paused". This is for:

- Changing maps without spoiling the next area
- Setting up tokens before a reveal
- Moments where the players should listen, not look

## See Also

- [Tutorial: The Player Display](../tutorials/04-player-display.md)
- [Use the Player Display](../how-to/play-mode/use-player-display.md)
- [Share a Player Link](../how-to/characters/player-links.md)
- [Architecture Overview](./architecture-overview.md)
