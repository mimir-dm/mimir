# Vision System

How Mimir implements D&D 5e vision and lighting rules.

## Why Vision Matters

In D&D, what characters can see affects:
- Exploration and discovery
- Combat tactics
- Stealth and surprise
- Atmosphere and tension

Mimir's vision system brings these rules to your virtual tabletop.

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

Placed light sources create zones of bright and dim illumination (a torch, for example, gives bright light near it and dim light farther out). A token must stand inside a light's radius to benefit from it — light is evaluated where the creature is, not where it is looking. Lit sources expand visible areas; unlit sources have no effect.

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
- Light sources on their own — a light never reveals fog by itself; it raises the light level where a PC stands, extending how far that PC can see

This prevents accidental reveals from positioned enemies: the DM can stage an entire ambush behind the fog, and nothing leaks until a PC can actually see it.

## Why Fog Implies Token LOS

Mimir exposes map hiding (Fog) and token hiding (LOS) as separate toggles, but enabling Fog forces LOS on. The reason is information leakage: if the map were hidden while all tokens stayed visible, players would see enemy markers floating in unexplored blackness — revealing exactly what fog is meant to conceal. Hiding the map only makes sense if unseen tokens are hidden too, so the weaker mode (LOS) is folded into the stronger one (Fog). LOS alone remains useful when geography is known but threats are not.

The actual toolbar controls are documented in the [Play Mode reference](../reference/ui/play-mode.md), and common workflows in the [Fog of War how-to](../how-to/play-mode/fog-of-war.md).

## See Also

- [Vision & Lighting Reference](../reference/vision-and-lighting.md) — ranges, radii, and precedence rules
- [Fog of War](../how-to/play-mode/fog-of-war.md) — task recipes for running a session
- [Play Mode](../reference/ui/play-mode.md) — the controls themselves
- [Manage Light Sources](../how-to/maps/manage-light-sources.md)
