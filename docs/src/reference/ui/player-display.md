# Player Display

The player display (`/display/<campaign>`) shows the players what their characters see. It is made for a shared screen: a TV, a projector or a second monitor. It has no app frame, only the map.

The same view is the **Display** tab of the [player page](./player-page.md).

## Open the Display

- Click **Open player display** on the DM map. The display opens in a new tab.
- Or open `/display/<campaign>` in a browser that is signed in with the DM token.

## States

| State | The display shows |
|-------|-------------------|
| No map shown | "Waiting for the DM to show a map." |
| A map shown | The map, as the players see it |
| Blackout | "Paused" |

## What Players See

- The map image and the grid
- PC tokens always
- Other tokens only where a PC can see them; never hidden tokens
- Lit lights, and the lights drawn into the map file
- The doors, open or closed as the DM set them
- Dead tokens, in grey
- Shown traps, with their name only
- Shown points of interest, with their name
- The turn order (round and names), when the DM turns on **Show order to players**

Tokens show their art from the imported images, or their first letter, and their name.

## What Players Do Not See

- Hidden tokens and hidden markers
- The DC, trigger and effect of a trap
- Unlit lights
- Hit points and conditions
- The map under fog of war, or the dark parts of a dim or dark map, outside the sight of the party (unless the DM turns on **Reveal map**)

The server sends the display only what the players may see.

## Controls

| Control | Function |
|---------|----------|
| **Zoom in**, **Zoom out** | Zoom the view |
| **Fit the map** | Fit the whole map in the screen |
| **Full screen** | Ask the browser for full screen |
| Drag, pinch, mouse wheel | Pan and zoom |

The view of the display is its own. It does not follow the zoom of the DM map.

## Live Updates

The display follows the DM over a live connection. A moved token, a change of fog or a new map shows at once.

## Troubleshooting

| Problem | Fix |
|---------|-----|
| "Waiting for the DM to show a map." | Click **Show to players** on the DM map. |
| "Paused" | Click **End blackout** on the DM map. |
| The map is all covered | Fog is on and no PC token is on the map. Place PC tokens. |
| Players see a token they should not | Click it on the DM map and turn off **Players see it**. |

## See Also

- [Use the Player Display](../../how-to/play-mode/use-player-display.md)
- [Fog of War](../../how-to/play-mode/fog-of-war.md)
- [Vision & Lighting](../vision-and-lighting.md)
