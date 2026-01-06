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

As tokens move, fog updates automatically.

### Vision Radius

Each PC token calculates visible area:
1. Base vision from ambient light
2. Extended by darkvision (if applicable)
3. Extended by light sources
4. Blocked by walls (UVTT maps)

### Light Sources

Placed light sources create illumination:
- Torch: 20 ft bright, 40 ft dim
- Lantern: 30 ft bright, 60 ft dim
- Candle: 5 ft bright, 10 ft dim

Lit sources expand visible areas. Unlit sources have no effect.

### Wall Occlusion

UVTT maps include wall data:
- Vision stops at walls
- Rooms reveal individually
- Doors can be open or closed

Without wall data, vision is circular (line of sight to all directions).

## Vision Modes

### Fog Mode
Default exploration mode:
- Map hidden outside vision
- Reveals as PCs explore
- Maximum immersion

### Token Mode
Combat-focused mode:
- Map fully visible
- Enemy tokens hidden outside vision
- Geography known, enemies hidden

### Reveal Mode
Override mode:
- Everything visible
- Bypasses all fog
- Use for non-exploration scenes

## What Creates Vision

**Does reveal fog:**
- Player Character tokens
- Active light sources

**Does not reveal fog:**
- NPC tokens
- Monster tokens
- Hidden tokens

This prevents accidental reveals from positioned enemies.

## Practical Considerations

### Dungeon Exploration
- Use Fog mode
- Set ambient light to dim or dark
- Place torches on walls
- Reveal room by room

### Combat Encounters
- Consider Token mode
- Players see the battlefield
- Hidden reinforcements stay hidden

### Town Scenes
- Use Reveal mode
- Fog doesn't fit the fiction
- Focus on roleplay, not exploration

### Dramatic Reveals
- Use Blackout
- Position elements
- Reveal with effect

## Tips for DMs

### Before Sessions
- Check ambient light setting
- Place light sources strategically
- Test with a PC token

### During Play
- Toggle lights for effect
- Adjust ambient as needed
- Switch modes as appropriate

### For Atmosphere
- Dark + scattered torches = tension
- Bright = safety
- Sudden darkness = danger

## See Also

- [Vision & Lighting Reference](../reference/vision-and-lighting.md)
- [Manage Light Sources](../how-to/maps/manage-light-sources.md)
- [Fog of War](../how-to/play-mode/fog-of-war.md)
