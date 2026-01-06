# Upload a Map

Add battle maps and area maps to your modules.

## Supported Formats

- **Image files** - PNG, JPG, WebP
- **UVTT files** - Universal VTT format (includes grid and wall data)

## Steps

1. Open a module from the Modules tab
2. Find the **Maps** section in the module dashboard
3. Click the **+** button
4. Choose your map file
5. Configure map settings:
   - **Name** - Display name for the map
   - **Grid Size** - Pixels per grid square
   - **Grid Offset** - X/Y offset if grid doesn't align
6. Click **Upload**

## UVTT Files

UVTT (Universal VTT) files from tools like Dungeondraft contain:
- Map image
- Grid configuration
- Wall and door data for line of sight

When you upload a UVTT file, grid settings are imported automatically.

## Image Files

For standard image files, you'll need to:
1. Determine the grid size (pixels per square)
2. Measure or estimate from the image
3. Adjust offset if the grid doesn't start at (0,0)

## Tips

- Common grid sizes: 70px, 100px, 140px per square
- Use UVTT files when available for best fog of war support
- Name maps descriptively (e.g., "Goblin Cave - Entrance", "Boss Chamber")

## See Also

- [Configure Grid](./configure-grid.md)
- [Place Tokens](./place-tokens.md)
