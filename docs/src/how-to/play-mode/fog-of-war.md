# Fog of War

Recipes to control what the players see on the map. The controls are in the toolbar of the DM map. The player display and the player pages follow at once.

- Controls: [DM Map](../../reference/ui/dm-map.md)
- Rules and numbers: [Vision & Lighting](../../reference/vision-and-lighting.md)
- How it works: [Vision System](../../explanation/vision-system.md)

## Hide the Map While the Party Explores

1. Turn on **Fog**.
2. Look at your map. A shade covers what the party cannot see. The players see nothing under it.

Fog of war shows only what the PC tokens see. The **Maps** table shows **On** in the **Fog of war** column.

## Show More of the Map

There is no reveal brush. The seen area comes from the PC tokens where they stand now. To show more:

1. Drag the PC tokens forward. Their sight follows them.
2. Place or light a torch or a lantern where the map is dim or dark. See [Manage Light Sources](../maps/manage-light-sources.md). In dim light or darkness, the players also see a lit area when a PC has a line to its light.

When a PC moves back, the area that it no longer sees is covered again.

## Show the Whole Map

1. Turn on **Reveal map**.
2. The players see all of the map: no fog and no sight limits. Hidden tokens and hidden markers stay hidden.

Turn off **Reveal map** to hide the map again.

## Show the Map but Hide the Monsters

1. Turn off **Fog**.
2. Place the PC tokens.

The players see the whole map. Monster and NPC tokens show only where a PC can see them. With no PC token on the map, all tokens that players may see show.

## Hide One Token

1. Click the token.
2. Turn off **Players see it**.

The players never see this token, even where a PC can see it.

## Change the Light or the Walls

- Choose **bright**, **dim** or **darkness** to set the light level of the map.
- Turn off **Line of sight** to let the party see through walls. Turn it on again to cut their sight at walls.

The map keeps these settings, and the player display follows them. To see the walls of the map file and how far the monsters see, turn on **Overlays**. Only you see them.

To see exactly what the players see, open the player display: click **Open player display**.

## Open a Door

On a UVTT map, click a door to open or close it. An open door lets sight through. The map keeps the door state, and the player display follows it.

## Stage a Reveal

1. Click **Blackout**. The players see "Paused".
2. Move tokens, change lights or change the map.
3. Click **End blackout**.

## See Also

- [Use the Player Display](./use-player-display.md)
- [Manage Light Sources](../maps/manage-light-sources.md)
- [Vision & Lighting Reference](../../reference/vision-and-lighting.md)
- [Vision System](../../explanation/vision-system.md)
