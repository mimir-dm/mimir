# Upload a Map

Add battle maps and area maps to your campaign modules.

## Supported Formats

| Format | Extensions | Grid Data | Walls/Doors | Lighting |
|--------|-----------|-----------|-------------|----------|
| **UVTT** | `.dd2vtt`, `.uvtt` | Embedded | Embedded | Embedded |
| **Image** | `.png`, `.jpg`, `.webp` | Default (70px) | None | None |

## Steps

1. Open a module from the Modules tab
2. Find the **Maps** section in the module dashboard
3. Click the **+** button
4. Choose your map file
5. Enter a **Name** for the map
6. Click **Upload**

The map appears as a card in the Maps section. Click it to open Token Setup.

## UVTT Files (Recommended)

UVTT files from tools like [Dungeondraft](https://dungeondraft.net/) include grid, wall, door, and lighting data that Mimir imports automatically. This gives you line-of-sight fog of war, accurate token snapping, and dynamic lighting out of the box.

See [Map Formats](../../explanation/map-formats.md) for details on what UVTT files contain and why they're preferred.

## Image Files

Standard image files work for basic token placement but lack wall data, so fog of war is not available — the Fog and LOS controls only appear for UVTT maps. On image maps, use the Reveal Map toggle and per-token Hide from Players instead. Mimir defaults to 70 pixels per grid square. See [Configure Grid](./configure-grid.md) for details.

## Generated Maps

Maps produced by `mimir-mapgen` are `.dungeondraft_map` files. Mimir cannot open `.dungeondraft_map` files directly — map uploads accept only UVTT (`.dd2vtt`, `.uvtt`) and image files. To use a generated map in Mimir, open it in Dungeondraft (a paid tool) and export it as Universal VTT.

## Tips

- **Always prefer UVTT** when available — fog of war and dynamic lighting only work on UVTT maps
- Maps can belong to a specific module or to the campaign at large
- You can upload multiple maps per module (one per room/area is common for dungeon crawls)

## See Also

- [Configure Grid](./configure-grid.md) — Grid alignment details
- [Place Tokens](./place-tokens.md) — Adding monsters and NPCs to maps
- [Map Formats](../../explanation/map-formats.md) — Understanding UVTT and alternatives
- [Generate Maps](./generate-map.md) — Create maps procedurally
