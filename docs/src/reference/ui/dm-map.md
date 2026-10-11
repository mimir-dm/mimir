# DM Map

The DM map (`/maps/<id>`) is where you prepare a map and run the session on it. It shows the whole map with every token, light and marker. The initiative tracker is at its side.

Open it from the **Maps** tab of a module page, or from the campaign maps on the **Campaign** tab.

## Toolbar

| Control | Function |
|---------|----------|
| **Map** | Switch to another map of the module or campaign |
| **Fog** | Fog of war on or off for this map. Saved; the players follow it. |
| **Line of sight** | On your view only: cut the sight of the party at walls |
| **bright / dim / darkness** | On your view only: the light for the sight of the party. Starts at the lighting of the map. |
| **Place on the map** (**Place…**) | Place a PC, a monster, an NPC, a torch, a lantern, a trap or a point of interest |
| **Show to players** / **Stop showing** | Show this map on the player display and the player pages, or stop |
| **Blackout** / **End blackout** | Hide the display ("Paused"), or show it again |
| **Open player display** | Open the player display in a new tab |
| **PDF…** | Export the map to PDF |
| **+**, **−**, **⤢** | **Zoom in**, **Zoom out**, **Fit the map** |

## The Map

The map shows, from bottom to top: the map image, the grid, the light of the lights, the fog shade, the markers, the doors and the tokens.

| Action | How |
|--------|-----|
| Pan | Drag the map |
| Zoom | Mouse wheel, pinch, or the zoom buttons |
| Select | Click a token, a light, a trap or a POI |
| Move a token | Drag it. It snaps to the grid. |
| Place | Choose an entry in **Place on the map**, then click squares. Click **Done** in the hint to stop. |
| Open or close a door | Click the door (UVTT maps). On your view only. |

The fog shade shows where the party cannot see when **Fog** is on, or in dim or dark light. The token of the creature whose turn it is in the combat is marked.

## Selection Panel

A click on an item opens a panel beside the map. **Close** closes it.

### Token

| Control | Function |
|---------|----------|
| Name, type and size | For example "monster · medium" |
| **Players see it** | Off: the token is hidden from the players |
| **Dead** | Marks the token on your view. Not saved. |
| **In bright light**, **In dim light** | Sight range in feet in that light. Empty: no limit. |
| **Darkvision** | Sight range in feet in darkness |
| **Carried light** | Radius in feet of a light that the token carries |
| **Save vision** | Save the four vision fields |
| **Remove token** | Remove the token, after a confirmation |

### Light

| Control | Function |
|---------|----------|
| Name and radii | For example "bright 20 ft · dim 40 ft" |
| **Lit** | Turn the light on or off |
| **Remove light** | Remove the light |

### Trap

| Control | Function |
|---------|----------|
| **DC**, **Trigger**, **Effect**, **Notes** | Shown when the trap has them |
| **Players see it** | Show the trap (its name only) to the players |
| **Triggered** | Mark the trap as triggered |
| **Remove trap** | Remove the trap, after a confirmation |

### Point of Interest

| Control | Function |
|---------|----------|
| Name and description | |
| **Players see it** | Show the POI to the players |
| **Remove** | Remove the POI, after a confirmation |

## Initiative Tracker

The tracker runs the combat of the module. **Show or hide the tracker** folds it away.

| Control | Function |
|---------|----------|
| **Start combat** | Start a combat at round 1 |
| **Turn order** | The creatures in initiative order. The current one is marked. |
| **Initiative of …** | The initiative of a creature |
| Creature name | Click to open its controls |
| **‹ Prev**, **Next turn ›** | Go to the previous or next turn |
| **Add a creature…**, **Add** | Add a campaign PC, a module monster or a module NPC |
| **Custom name**, **Add** | Add a creature by name |
| **Show order to players** | Show the round and the names on the display |
| **End combat**, **End** | End the combat, after a confirmation |
| **×** (**Remove …**) | Remove a creature |

Controls of a creature:

| Control | Function |
|---------|----------|
| **Amount**, **Damage**, **Heal** | Change the hit points |
| **Temp HP** | Temporary hit points |
| **Set HP** | Set the maximum, for a creature with no hit points |
| **Condition…**, **Rnds**, **Add** | Add one of the 15 conditions, for a number of rounds or with no end |
| Condition pill | Point at it to see the end. Click it to remove the condition. |
| **Concentrating** | Mark concentration. Damage then asks for the save (**Kept** or **Lost**). |
| **Recent HP changes** | The last changes and their round |

A creature that has a token on the map shows **⌖**.

## Live Changes

The page follows changes at once, from you on another device, from the players, or from the AI assistant. You do not need to reload.

## See Also

- [Place Tokens](../../how-to/maps/place-tokens.md)
- [Run a Combat](../../how-to/play-mode/manage-encounters.md)
- [Fog of War](../../how-to/play-mode/fog-of-war.md)
- [Player Display](./player-display.md)
- [Vision & Lighting](../vision-and-lighting.md)
