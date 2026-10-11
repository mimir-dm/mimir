# Vision System

How Mimir implements D&D 5e vision and lighting rules.

## Why Vision Matters

In D&D, what characters can see affects:
- Exploration and discovery
- Combat tactics
- Stealth and surprise
- Atmosphere and tension

Mimir's vision system brings these rules to your map and to the screens of your players.

## D&D 5e Vision Rules

### Light Levels

D&D defines three light levels:

**Bright Light**
Most creatures see normally. Includes:
- Daylight
- Torchlit rooms
- Magical light

**Dim Light**
Lightly obscured area. Creates:
- Disadvantage on Perception checks
- Shadows and half-light

**Darkness**
Heavily obscured area. Results in:
- Effectively blind
- Auto-fail sight-based checks
- Advantage for hidden creatures

### Darkvision

Many races and creatures can see in darkness:
- Treat darkness as dim light
- Treat dim light as bright light
- Cannot discern color (shades of gray)
- Typical range: 60 feet

## How Mimir Implements This

### Fog of War

The map is divided into:
- **Revealed** - Areas PCs can see
- **Hidden** - Areas beyond PC vision

As tokens move, fog updates automatically. There is no manual "reveal brush": the revealed area is always derived from where PC tokens currently stand and what they can see. This keeps the player view honest — fog retracts when the party retreats, and the DM never has to remember to re-cover an area.

### Vision Radius

Each PC token's visible area is derived from the light level at the token's own position. The ambient light sets a baseline, light sources can raise it locally, and the resulting level selects which of the token's vision ranges applies — bright vision, dim vision, or (in darkness) the better of darkvision and the token's own carried light. Walls then clip the result on UVTT maps. The exact values and precedence are listed in the [Vision & Lighting reference](../reference/vision-and-lighting.md).

### Light Sources

Placed light sources and the lights drawn into a UVTT map file create zones of bright and dim illumination (a torch, for example, gives bright light near it and dim light farther out). A token inside a light's radius sees by that light — light is evaluated where the creature is. In dim light or darkness, a lit area also shows when a PC has a line to its light, as a lit room shows from a dark corridor. Unlit sources have no effect.

### Wall Occlusion

UVTT maps include wall data:
- Vision stops at walls, computed as a visibility polygon by raycasting
- Rooms reveal individually
- Doors are portals in the wall data: closed doors block sight, open doors do not

Without wall data, vision is a simple circle (line of sight in all directions).

## What Creates Vision

**Does reveal fog:**
- Player Character tokens (while visible to players) — they are the only source of revealed area

**Does not reveal fog:**
- NPC tokens
- Monster tokens
- Hidden tokens
- Light sources on their own — a light shows its area only to a PC with a line to it

This prevents accidental reveals from positioned enemies: the DM can stage an entire ambush behind the fog, and nothing leaks until a PC can actually see it.

## Why Tokens Hide Even Without Fog

Mimir hides tokens that no PC can see, also when fog is off. Fog hides the map; the token rule hides the threats. With fog off, players know the layout, but an ambush still stays hidden until a PC can see it. With fog on, the map is hidden too: otherwise players would see enemy tokens floating in the dark, which is exactly what fog is meant to hide.

## One Source of Truth

The **Line of sight** switch, the **bright / dim / darkness** control, **Reveal map** and the doors are part of the map. The server keeps them, and the DM map and the player display both read them. The server builds the player view, and the display draws only that, so what the DM sets is what the players get. Only **Overlays**, a drawing aid, is on the DM's view alone.

The controls are in the [DM Map reference](../reference/ui/dm-map.md), and common tasks in the [Fog of War how-to](../how-to/play-mode/fog-of-war.md).

## See Also

- [Vision & Lighting Reference](../reference/vision-and-lighting.md) — ranges, radii, and precedence rules
- [Fog of War](../how-to/play-mode/fog-of-war.md) — task recipes for running a session
- [DM Map](../reference/ui/dm-map.md) — the controls themselves
- [Manage Light Sources](../how-to/maps/manage-light-sources.md)
