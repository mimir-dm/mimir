# Vision & Lighting

Reference values and rules used by Mimir's fog of war calculation.

For the toolbar controls, see [Play Mode](./ui/play-mode.md). For step-by-step usage, see [Fog of War](../how-to/play-mode/fog-of-war.md). For the design rationale, see [Vision System](../explanation/vision-system.md).

## Scale

One grid square equals 5 feet. All ranges below are in feet and converted to map pixels using the map's grid size.

## Token Vision Values

Each token carries four vision-related values:

| Value | Meaning | Notes |
|-------|---------|-------|
| Bright vision | Sight range in bright light | Unset = unlimited |
| Dim vision | Sight range in dim light | Unset = unlimited |
| Dark vision | Sight range in darkness | 0 = blind in darkness; 60 ft = typical darkvision |
| Light radius | Dim radius of light the token carries | 0 = no light; bright radius is half the dim radius |

## Ambient Light Levels

| Level | Description | Vision range used |
|-------|-------------|-------------------|
| Bright | Daylight, well-lit rooms | Bright vision (usually unlimited) |
| Dim | Twilight, torchlit edges | Dim vision; rendered as dim sight |
| Darkness | Underground, night | The larger of dark vision and the token's own light radius; darkvision counts as dim sight |

The ambient level initializes from the map's UVTT data when present and can be overridden per session from the map toolbar.

## Light Source Presets

| Source | Bright Light | Dim Light |
|--------|-------------|-----------|
| Candle | 5 ft | 10 ft |
| Torch | 20 ft | 40 ft |
| Lantern | 30 ft | 60 ft |
| Spell / Custom | 20 ft | 40 ft (defaults; editable) |

Token-carried light uses the token's light radius value as the dim radius, with the bright radius at half that distance.

## Light Level at a Point

The effective light level at a point is the best of:

- Bright, if the point is within any active source's bright radius
- Dim, if the point is within any active source's dim radius (when ambient is darkness)
- The ambient level otherwise

A token must be inside a light's radius to benefit from it: the light level at the token's own position determines which vision range applies.

## What Reveals Fog

| Token | Reveals fog? |
|-------|--------------|
| PC token, visible to players | Yes |
| PC token, individually hidden | No |
| NPC token | No |
| Monster token | No |

## Visibility States

The **Fog** and **LOS** toggles are separate buttons but not fully independent: enabling Fog forces LOS on, and the LOS button is locked until Fog is turned off. **Reveal Map** overrides both. The resulting states:

| State | Map (player view) | Tokens (player view) |
|-------|-------------------|----------------------|
| Fog off, LOS off | Fully visible | All player-visible tokens shown |
| LOS only | Fully visible | Only tokens within party line of sight |
| Fog on (LOS forced on) | Hidden outside party vision | Only tokens within party line of sight |
| Reveal Map | Fully visible | All player-visible tokens shown |

Individually hidden tokens are never shown to players, in any state. See [Play Mode](./ui/play-mode.md) for the controls themselves.

## Wall Occlusion

UVTT maps include wall and portal (door) data:

- Vision is computed as a visibility polygon by raycasting against walls
- Closed doors block vision; open doors do not
- The Fog and LOS toggles are only available on maps with UVTT data

Maps without wall data use circular (radius-only) vision with no obstructions.

## See Also

- [Fog of War](../how-to/play-mode/fog-of-war.md) — task recipes
- [Vision System](../explanation/vision-system.md) — how the D&D 5e rules map to the implementation
- [Play Mode](./ui/play-mode.md) — toolbar controls
