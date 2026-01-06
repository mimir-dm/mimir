# Fog of War

Control what players can see on the map using fog of war and line of sight.

![Fog of War - Player View](../../images/reference/player-view-fog.png)

## How Fog of War Works

Fog of war reveals areas based on:
- PC token positions
- Vision radius (darkvision)
- Active light sources
- Wall obstructions (UVTT maps)

## Vision Modes

### Fog Mode
- Map is hidden outside PC vision
- Only revealed areas are visible to players
- Creates exploration atmosphere

### Token Mode

![Token Mode](../../images/reference/player-view-token-los.png)

- Entire map is visible
- Enemy tokens are hidden outside PC vision
- Useful for exploration without hiding geography

Toggle between modes using the **LOS** switch in the toolbar.

## Ambient Light

Set the base lighting level:

| Level | Effect |
|-------|--------|
| Bright | Full visibility within range |
| Dim | Muted colors, reduced perception |
| Dark | Only darkvision and light sources work |

Change using the ambient light dropdown.

## Reveal Map

The **Reveal Map** toggle bypasses fog of war:
- Entire map visible to players
- All tokens visible (including hidden)
- Use for area overviews or non-combat scenes

## Wall Occlusion

UVTT maps include wall data for accurate line of sight:
- Walls block vision
- Doors can be open or closed
- Creates realistic room-by-room exploration

Standard image maps don't have wall data - vision is circular.

## Tips

- Use darkness for dungeon atmosphere
- Toggle lights for dramatic effect
- Reveal Map for town maps where fog doesn't make sense
- Switch to Token mode for complex combat

## See Also

- [Manage Light Sources](../maps/manage-light-sources.md)
- [Vision & Lighting Reference](../../reference/vision-and-lighting.md)
- [Use Player Display](./use-player-display.md)
