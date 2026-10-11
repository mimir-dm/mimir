# Vision & Lighting

The values and rules that Mimir uses for fog of war and sight.

For the controls, see the [DM Map](./ui/dm-map.md). For steps, see [Fog of War](../how-to/play-mode/fog-of-war.md). For the design, see [Vision System](../explanation/vision-system.md).

## Scale

One grid square is 5 feet. Mimir turns each range in feet into map pixels with the grid of the map.

## Token Vision Values

Each token has four vision values. Set them in the token panel of the DM map.

| Value | Field | Meaning | Notes |
|-------|-------|---------|-------|
| Bright vision | **In bright light** | Sight range in bright light | Empty = no limit |
| Dim vision | **In dim light** | Sight range in dim light | Empty = no limit |
| Dark vision | **Darkvision** | Sight range in darkness | 0 = blind in darkness; 60 ft is typical |
| Light radius | **Carried light** | Dim radius of a light the token carries | 0 = no light; the bright radius is half |

## Ambient Light Levels

| Level | Description | Range used |
|-------|-------------|------------|
| Bright | Daylight, lit rooms | Bright vision |
| Dim | Twilight | Dim vision |
| Darkness | Night, underground | The larger of dark vision and the carried light |

A new map starts at the ambient light of its UVTT file (bright when the file has none). The **bright / dim / darkness** control on the DM map changes the light level of the map; the player display follows it.

## Light Source Presets

| Source | Bright light | Dim light |
|--------|-------------|-----------|
| Torch | 20 ft | 40 ft |
| Lantern | 30 ft | 60 ft |

A token's carried light uses its **Carried light** value as the dim radius and half of it as the bright radius. A light drawn into a UVTT map file uses its range as the dim radius and half of it as the bright radius. Its glow stops at the walls.

## Light Level at a Point

The light level at a point is the best of:

- Bright, if the point is in the bright radius of a lit light
- Dim, if the point is in the dim radius of a lit light
- The ambient level

A token sees by the light where it stands. The light level at its own place picks which range applies.

In dim light or darkness, the players also see the area of a lit light when a PC has a line to the light (walls only, no range limit).

## Which Tokens Reveal

| Token | Reveals the map? |
|-------|------------------|
| PC token with **Players see it** on | Yes |
| PC token with **Players see it** off | No |
| NPC token | No |
| Monster token | No |

## What the Players See

| | Fog off | Fog on |
|-|---------|--------|
| Map in the party's sight | Visible | Visible |
| Map outside the party's sight | Visible in bright light when no PC has a sight limit; darkened otherwise | Covered |
| PC tokens | Shown | Shown |
| Other tokens in the party's sight | Shown | Shown |
| Other tokens outside the party's sight | Not shown | Not shown |
| Tokens with **Players see it** off | Never | Never |

With no PC token on the map and fog off, all tokens with **Players see it** on are shown. With no PC token and fog on, the whole map is covered.

With **Reveal map** on, the players see the whole map and every token with **Players see it** on, also under fog.

## Walls

UVTT maps have walls and doors:

- Sight is a visibility polygon cast against the walls.
- Closed doors block sight; open doors do not.
- The DM opens or closes a door on the DM map. The map keeps the door state, and the player display follows it.
- With **Line of sight** off, walls and doors do not block sight.

Image maps have no walls. Sight is a circle of its range.

## See Also

- [Fog of War](../how-to/play-mode/fog-of-war.md)
- [Vision System](../explanation/vision-system.md)
- [DM Map](./ui/dm-map.md)
