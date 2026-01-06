# Vision & Lighting

How D&D 5e vision rules work in Mimir's fog of war system.

## Vision Basics

Mimir calculates what players can see based on:
- Token positions
- Vision radius
- Light sources
- Wall obstructions (UVTT maps)

## Vision Types

### Normal Vision
Characters without special vision see based on ambient light:
- **Bright light** - Full visibility
- **Dim light** - Disadvantage on Perception
- **Darkness** - Cannot see

### Darkvision
Characters with darkvision can see in darkness as if it were dim light:
- Range varies by race (typically 60 ft)
- Colors appear as shades of gray
- Cannot see in magical darkness

## Ambient Light Levels

Set the base lighting for the entire map:

| Level | Description | Effect |
|-------|-------------|--------|
| Bright | Daylight, well-lit rooms | Full visibility |
| Dim | Twilight, torchlit | Reduced visibility |
| Dark | Underground, night | Darkvision or light required |

## Light Sources

Light sources create areas of illumination:

| Source | Bright Light | Dim Light |
|--------|-------------|-----------|
| Candle | 5 ft | 10 ft |
| Torch | 20 ft | 40 ft |
| Lantern | 30 ft | 60 ft |

### Light Radius Calculation
- Bright light radius at full effect
- Dim light extends to the listed range
- Beyond dim light, darkness applies

## Fog of War Modes

### Fog Mode
Map areas outside PC vision are hidden:
- Revealed as PCs move
- Creates exploration atmosphere
- Enemies hidden until seen

### Token Mode
Map is fully visible, but:
- Enemy tokens hidden outside vision
- Useful for complex maps
- Players see geography, not threats

## Wall Occlusion

UVTT maps include wall data:
- Walls block line of sight
- Vision stops at wall boundaries
- Doors can be toggled open/closed

Standard image maps use circular vision without obstructions.

## PC Vision Only

Only player character tokens reveal fog:
- NPCs do not create vision
- Monster tokens do not reveal areas
- Prevents accidental reveals from hidden enemies

## How Vision Is Calculated

1. For each PC token, calculate vision radius
2. Apply darkvision if applicable
3. Add light source bonuses
4. Check wall obstructions (UVTT)
5. Union all PC vision areas
6. Reveal combined area to players

## Tips for DMs

- Use darkness for tension
- Toggle lights for dramatic reveals
- Reveal Map bypasses all vision
- Token Mode for combat focus
- Fog Mode for exploration
